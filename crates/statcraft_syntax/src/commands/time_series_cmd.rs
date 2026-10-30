use crate::command::{CommandError, EngineCommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use statcraft_algo::{compute_time_series_transform, TimeSeriesFunction};
use statcraft_core::{DataSet, DataType, Measure, RawValue, VariableMetadata};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesVarSpec {
    pub source_var: String,
    pub target_var: String,
    pub function: String,
    #[serde(default = "default_order")]
    pub order: usize,
    pub target_label: Option<String>,
}

fn default_order() -> usize {
    1
}

#[derive(Debug, Clone)]
enum SingleVarUndoEntry {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

#[derive(Debug, Clone)]
struct TimeSeriesUndoBatch {
    entries: Vec<SingleVarUndoEntry>,
}

pub struct CreateTimeSeriesCommand {
    history: Mutex<Vec<TimeSeriesUndoBatch>>,
}

impl Default for CreateTimeSeriesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateTimeSeriesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn parse_function(func_str: &str, order: usize) -> Result<TimeSeriesFunction, String> {
        let ord = if order == 0 { 1 } else { order };
        match func_str.to_ascii_lowercase().as_str() {
            "diff" => Ok(TimeSeriesFunction::Diff(ord)),
            "sdiff" => Ok(TimeSeriesFunction::SeasonalDiff(ord)),
            "sma" => Ok(TimeSeriesFunction::SimpleMovingAverage(ord)),
            "csum" => Ok(TimeSeriesFunction::CumulativeSum),
            other => Err(format!("Unknown time series function: '{}'", other)),
        }
    }

    pub fn to_sps(variables: &[TimeSeriesVarSpec]) -> String {
        if variables.is_empty() {
            return String::new();
        }

        let mut sps = String::new();
        for (i, spec) in variables.iter().enumerate() {
            let fn_call = match spec.function.to_ascii_lowercase().as_str() {
                "diff" => format!("DIFF({}, {})", spec.source_var, spec.order.max(1)),
                "sdiff" => format!("SDIFF({}, {})", spec.source_var, spec.order.max(1)),
                "sma" => format!("SMA({}, {})", spec.source_var, spec.order.max(1)),
                "csum" => format!("CSUM({})", spec.source_var),
                _ => format!(
                    "{}({})",
                    spec.function.to_ascii_uppercase(),
                    spec.source_var
                ),
            };

            if i == 0 {
                sps.push_str(&format!("CREATE {} = {}\n", spec.target_var, fn_call));
            } else {
                sps.push_str(&format!("  /{} = {}\n", spec.target_var, fn_call));
            }
        }

        if !sps.is_empty() {
            if let Some(pos) = sps.rfind('\n') {
                sps.replace_range(pos..pos + 1, ".\n");
            }
        }

        for spec in variables {
            if let Some(ref lbl) = spec.target_label {
                let trimmed = lbl.trim();
                if !trimmed.is_empty() {
                    sps.push_str(&format!(
                        "VARIABLE LABELS {} \"{}\".\n",
                        spec.target_var, trimmed
                    ));
                }
            }
        }

        sps.push_str("EXECUTE.");
        sps
    }
}

impl EngineCommand for CreateTimeSeriesCommand {
    fn id(&self) -> &'static str {
        "transform.create_time_series"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Create Time Series"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["variables"],
            "properties": {
                "variables": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["source_var", "target_var", "function"],
                        "properties": {
                            "source_var": { "type": "string" },
                            "target_var": { "type": "string" },
                            "function": {
                                "type": "string",
                                "enum": ["diff", "sdiff", "sma", "csum"]
                            },
                            "order": { "type": "integer", "minimum": 1, "default": 1 },
                            "target_label": { "type": "string" }
                        }
                    }
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset
            .map(|d| d.col_count() > 0 && d.row_count() > 0)
            .unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let var_specs: Vec<TimeSeriesVarSpec> =
            if let Some(arr) = params.get("variables").and_then(|v| v.as_array()) {
                serde_json::from_value(Value::Array(arr.clone())).map_err(|e| {
                    CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Failed to parse variables array: {}", e),
                    }
                })?
            } else if params.get("source_var").is_some() {
                let single: TimeSeriesVarSpec =
                    serde_json::from_value(params.clone()).map_err(|e| {
                        CommandError::InvalidParams {
                            id: self.id(),
                            reason: format!("Failed to parse single variable spec: {}", e),
                        }
                    })?;
                vec![single]
            } else {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: "Expected 'variables' array or single variable parameters".to_string(),
                });
            };

        if var_specs.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "No variables specified for time series creation".to_string(),
            });
        }

        let mut batch_entries = Vec::new();
        let row_count = ds.row_count();

        for spec in &var_specs {
            let source_name = spec.source_var.trim();
            let target_name = spec.target_var.trim();
            if source_name.is_empty() || target_name.is_empty() {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: "Source and target variable names cannot be empty".to_string(),
                });
            }

            let ts_fn = Self::parse_function(&spec.function, spec.order).map_err(|err| {
                CommandError::InvalidParams {
                    id: self.id(),
                    reason: err,
                }
            })?;

            let src_idx = ds.dictionary.find_index(source_name).ok_or_else(|| {
                CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Source variable '{}' not found", source_name),
                }
            })?;

            let values = ds
                .table
                .columns
                .get(src_idx)
                .map(|c| c.values.as_slice())
                .ok_or_else(|| {
                    CommandError::ExecutionFailed("Column values missing".to_string())
                })?;

            let transformed = compute_time_series_transform(values, ts_fn);

            let existing_idx = ds.dictionary.find_index(target_name);
            let undo_entry = if let Some(col_idx) = existing_idx {
                let old_meta = ds.dictionary.get(col_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Corrupt dictionary".to_string())
                })?;
                let old_values = ds
                    .table
                    .columns
                    .get(col_idx)
                    .map(|c| c.values.clone())
                    .ok_or_else(|| {
                        CommandError::ExecutionFailed("Target column missing".to_string())
                    })?;

                if let Some(col) = ds.table.columns.get_mut(col_idx) {
                    col.values = transformed;
                }
                if let Some(ref lbl) = spec.target_label {
                    let trimmed = lbl.trim();
                    if !trimmed.is_empty() {
                        if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                            meta.label = trimmed.to_string();
                        }
                    }
                }

                SingleVarUndoEntry::Overwritten {
                    col_idx,
                    old_meta,
                    old_values,
                }
            } else {
                let col_idx = ds.add_variable(
                    target_name,
                    DataType::Numeric { decimals: 2 },
                    Measure::Scale,
                )?;
                if let Some(ref lbl) = spec.target_label {
                    let trimmed = lbl.trim();
                    if !trimmed.is_empty() {
                        if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                            meta.label = trimmed.to_string();
                        }
                    }
                }
                if let Some(col) = ds.table.columns.get_mut(col_idx) {
                    col.values = transformed;
                }

                SingleVarUndoEntry::Created { col_idx }
            };

            batch_entries.push(undo_entry);
        }

        let count = batch_entries.len();
        if let Ok(mut hist) = self.history.lock() {
            hist.push(TimeSeriesUndoBatch {
                entries: batch_entries,
            });
        }

        Ok(json!({
            "status": "success",
            "count": count,
            "rows": row_count
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let batch = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        if let Some(batch) = batch {
            for entry in batch.entries.into_iter().rev() {
                match entry {
                    SingleVarUndoEntry::Created { col_idx } => {
                        ds.remove_variable(col_idx)?;
                    }
                    SingleVarUndoEntry::Overwritten {
                        col_idx,
                        old_meta,
                        old_values,
                    } => {
                        if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                            *meta = old_meta;
                        }
                        if let Some(col) = ds.table.columns.get_mut(col_idx) {
                            col.values = old_values;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;

    fn create_test_dataset() -> DataSet {
        let mut ds = DataSet::empty();
        let col = ds
            .add_variable("sales", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
        ds.add_empty_row();
        ds.set_cell(0, col, RawValue::Number(10.0)).unwrap();
        ds.add_empty_row();
        ds.set_cell(1, col, RawValue::Number(20.0)).unwrap();
        ds.add_empty_row();
        ds.set_cell(2, col, RawValue::Number(30.0)).unwrap();
        ds.add_empty_row();
        ds.set_cell(3, col, RawValue::Number(40.0)).unwrap();
        ds
    }

    #[test]
    fn test_create_time_series_execute_and_undo() {
        let mut ds = Some(create_test_dataset());
        let cmd = CreateTimeSeriesCommand::new();

        let params = json!({
            "variables": [
                {
                    "source_var": "sales",
                    "target_var": "diff_sales",
                    "function": "diff",
                    "order": 1,
                    "target_label": "First Difference of Sales"
                },
                {
                    "source_var": "sales",
                    "target_var": "csum_sales",
                    "function": "csum"
                }
            ]
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(res["count"], 2);

        let d = ds.as_ref().unwrap();
        assert_eq!(d.col_count(), 3);

        let diff_idx = d.dictionary.find_index("diff_sales").unwrap();
        let csum_idx = d.dictionary.find_index("csum_sales").unwrap();

        assert_eq!(d.get_cell(0, diff_idx), Some(&RawValue::Null));
        assert_eq!(d.get_cell(1, diff_idx), Some(&RawValue::Number(10.0)));
        assert_eq!(d.get_cell(2, diff_idx), Some(&RawValue::Number(10.0)));
        assert_eq!(d.get_cell(3, diff_idx), Some(&RawValue::Number(10.0)));

        assert_eq!(d.get_cell(0, csum_idx), Some(&RawValue::Number(10.0)));
        assert_eq!(d.get_cell(1, csum_idx), Some(&RawValue::Number(30.0)));
        assert_eq!(d.get_cell(2, csum_idx), Some(&RawValue::Number(60.0)));
        assert_eq!(d.get_cell(3, csum_idx), Some(&RawValue::Number(100.0)));

        assert_eq!(
            d.dictionary.get(diff_idx).unwrap().label,
            "First Difference of Sales"
        );

        // Undo
        cmd.undo(&mut ds).unwrap();
        let d = ds.as_ref().unwrap();
        assert_eq!(d.col_count(), 1);
        assert!(d.dictionary.find_index("diff_sales").is_none());
        assert!(d.dictionary.find_index("csum_sales").is_none());
    }

    #[test]
    fn test_to_sps() {
        let specs = vec![
            TimeSeriesVarSpec {
                source_var: "sales".to_string(),
                target_var: "diff_sales".to_string(),
                function: "diff".to_string(),
                order: 1,
                target_label: Some("Diff Label".to_string()),
            },
            TimeSeriesVarSpec {
                source_var: "sales".to_string(),
                target_var: "sma_sales".to_string(),
                function: "sma".to_string(),
                order: 3,
                target_label: None,
            },
            TimeSeriesVarSpec {
                source_var: "sales".to_string(),
                target_var: "csum_sales".to_string(),
                function: "csum".to_string(),
                order: 1,
                target_label: None,
            },
        ];

        let sps = CreateTimeSeriesCommand::to_sps(&specs);
        assert!(sps.contains("CREATE diff_sales = DIFF(sales, 1)"));
        assert!(sps.contains("/sma_sales = SMA(sales, 3)"));
        assert!(sps.contains("/csum_sales = CSUM(sales)."));
        assert!(sps.contains("VARIABLE LABELS diff_sales \"Diff Label\"."));
        assert!(sps.ends_with("EXECUTE."));
    }
}
