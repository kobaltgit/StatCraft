use crate::command::{CommandError, EngineCommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use statcraft_algo::{impute_missing_values, ImputationMethod};
use statcraft_core::{DataSet, DataType, RawValue, VariableMetadata};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceMissingItemSpec {
    pub source_var: String,
    pub target_var: String,
    pub method: String,
    #[serde(default = "default_span")]
    pub span: usize,
}

fn default_span() -> usize {
    2
}

#[derive(Debug, Clone)]
enum ReplaceMissingUndoItem {
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
pub struct ReplaceMissingUndoEntry {
    items: Vec<ReplaceMissingUndoItem>,
}

pub struct ReplaceMissingCommand {
    history: Mutex<Vec<ReplaceMissingUndoEntry>>,
}

impl Default for ReplaceMissingCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplaceMissingCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn parse_method(method_str: &str, span: usize) -> Result<ImputationMethod, String> {
        match method_str.trim().to_ascii_lowercase().as_str() {
            "series_mean" | "smean" => Ok(ImputationMethod::SeriesMean),
            "mean_nearby" | "mean" => Ok(ImputationMethod::MeanOfNearby(span.max(1))),
            "median_nearby" | "median" => Ok(ImputationMethod::MedianOfNearby(span.max(1))),
            "linear" | "linear_interpolation" => Ok(ImputationMethod::LinearInterpolation),
            "trend" | "linear_trend" => Ok(ImputationMethod::LinearTrendAtPoint),
            other => Err(format!("Unknown imputation method: '{}'", other)),
        }
    }

    pub fn to_sps(items: &[ReplaceMissingItemSpec]) -> String {
        let mut sps = String::new();
        for item in items {
            let func_str = match item.method.trim().to_ascii_lowercase().as_str() {
                "series_mean" | "smean" => format!("SMEAN({})", item.source_var),
                "mean_nearby" | "mean" => {
                    format!("MEAN({}, {})", item.source_var, item.span.max(1))
                }
                "median_nearby" | "median" => {
                    format!("MEDIAN({}, {})", item.source_var, item.span.max(1))
                }
                "linear" | "linear_interpolation" => format!("LINEAR({})", item.source_var),
                "trend" | "linear_trend" => format!("TREND({})", item.source_var),
                _ => format!("SMEAN({})", item.source_var),
            };
            sps.push_str(&format!("RMV {} = {}.\n", item.target_var, func_str));
        }
        sps.push_str("EXECUTE.");
        sps
    }
}

impl EngineCommand for ReplaceMissingCommand {
    fn id(&self) -> &'static str {
        "transform.replace_missing"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Replace Missing Values"
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
                        "required": ["source_var", "target_var"],
                        "properties": {
                            "source_var": { "type": "string" },
                            "target_var": { "type": "string" },
                            "method": {
                                "type": "string",
                                "enum": ["series_mean", "mean_nearby", "median_nearby", "linear", "trend"],
                                "default": "series_mean"
                            },
                            "span": { "type": "integer", "minimum": 1, "default": 2 }
                        }
                    }
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|d| d.col_count() > 0).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let vars_val = params
            .get("variables")
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'variables' array parameter".to_string(),
            })?;

        let items: Vec<ReplaceMissingItemSpec> =
            serde_json::from_value(vars_val.clone()).map_err(|e| CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Failed to parse variables: {}", e),
            })?;

        if items.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Variables array cannot be empty".to_string(),
            });
        }

        let mut undo_items = Vec::new();

        for item in &items {
            let src_name = item.source_var.trim();
            let tgt_name = item.target_var.trim();

            if src_name.is_empty() || tgt_name.is_empty() {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: "Variable names cannot be empty".to_string(),
                });
            }

            let src_idx =
                ds.dictionary
                    .find_index(src_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Source variable '{}' not found", src_name),
                    })?;

            let src_meta =
                ds.dictionary.get(src_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Corrupt dictionary".to_string())
                })?;

            let values = ds
                .table
                .columns
                .get(src_idx)
                .map(|c| c.values.as_slice())
                .ok_or_else(|| {
                    CommandError::ExecutionFailed("Column values missing".to_string())
                })?;

            let method = Self::parse_method(&item.method, item.span).map_err(|r| {
                CommandError::InvalidParams {
                    id: self.id(),
                    reason: r,
                }
            })?;

            let imputed = impute_missing_values(values, method);

            let existing_idx = ds.dictionary.find_index(tgt_name);
            let undo_action = if let Some(target_idx) = existing_idx {
                let old_meta = ds.dictionary.get(target_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Corrupt dictionary".to_string())
                })?;
                let old_values = ds
                    .table
                    .columns
                    .get(target_idx)
                    .map(|c| c.values.clone())
                    .ok_or_else(|| {
                        CommandError::ExecutionFailed("Target column values missing".to_string())
                    })?;

                if let Some(col) = ds.table.columns.get_mut(target_idx) {
                    col.values = imputed;
                }

                ReplaceMissingUndoItem::Overwritten {
                    col_idx: target_idx,
                    old_meta,
                    old_values,
                }
            } else {
                let target_idx = ds.add_variable(
                    tgt_name,
                    DataType::Numeric { decimals: 2 },
                    src_meta.measure,
                )?;

                if let Some(col) = ds.table.columns.get_mut(target_idx) {
                    col.values = imputed;
                }

                ReplaceMissingUndoItem::Created {
                    col_idx: target_idx,
                }
            };

            undo_items.push(undo_action);
        }

        let processed = items.len();
        let rows = ds.row_count();

        if let Ok(mut hist) = self.history.lock() {
            hist.push(ReplaceMissingUndoEntry { items: undo_items });
        }

        Ok(json!({
            "status": "success",
            "variables_processed": processed,
            "rows": rows
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let entry = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        if let Some(entry) = entry {
            for item in entry.items.into_iter().rev() {
                match item {
                    ReplaceMissingUndoItem::Created { col_idx } => {
                        ds.remove_variable(col_idx)?;
                    }
                    ReplaceMissingUndoItem::Overwritten {
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
    use statcraft_core::{DataType, Measure};

    fn make_test_dataset() -> DataSet {
        let mut ds = DataSet::empty();
        let c0 = ds
            .add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();

        for v in [Some(100.0), None, Some(300.0), None, Some(500.0)] {
            ds.add_empty_row();
            let row = ds.row_count() - 1;
            let val = match v {
                Some(num) => RawValue::Number(num),
                None => RawValue::Null,
            };
            let _ = ds.set_cell(row, c0, val);
        }
        ds
    }

    #[test]
    fn test_replace_missing_series_mean_and_undo() {
        let mut ds = Some(make_test_dataset());
        let cmd = ReplaceMissingCommand::new();

        let params = json!({
            "variables": [
                {
                    "source_var": "salary",
                    "target_var": "new_salary",
                    "method": "series_mean"
                }
            ]
        });

        let res = cmd.execute(&mut ds, params).expect("execute failed");
        assert_eq!(res["status"], "success");
        assert_eq!(res["variables_processed"], 1);

        let d = ds.as_ref().unwrap();
        assert_eq!(d.col_count(), 2);
        let c1 = d.table.columns.get(1).unwrap();
        // Mean of (100 + 300 + 500) / 3 = 300.0
        assert_eq!(c1.values[0], RawValue::Number(100.0));
        assert_eq!(c1.values[1], RawValue::Number(300.0));
        assert_eq!(c1.values[2], RawValue::Number(300.0));
        assert_eq!(c1.values[3], RawValue::Number(300.0));
        assert_eq!(c1.values[4], RawValue::Number(500.0));

        // Test undo
        cmd.undo(&mut ds).expect("undo failed");
        let d_undone = ds.as_ref().unwrap();
        assert_eq!(d_undone.col_count(), 1);
        assert!(d_undone.dictionary.find_index("new_salary").is_none());
    }

    #[test]
    fn test_replace_missing_linear_interpolation() {
        let mut ds = Some(make_test_dataset());
        let cmd = ReplaceMissingCommand::new();

        let params = json!({
            "variables": [
                {
                    "source_var": "salary",
                    "target_var": "linear_salary",
                    "method": "linear"
                }
            ]
        });

        let res = cmd.execute(&mut ds, params).expect("execute failed");
        assert_eq!(res["status"], "success");

        let d = ds.as_ref().unwrap();
        let c1 = d.table.columns.get(1).unwrap();
        assert_eq!(c1.values[1], RawValue::Number(200.0)); // interpolated between 100 and 300
        assert_eq!(c1.values[3], RawValue::Number(400.0)); // interpolated between 300 and 500
    }

    #[test]
    fn test_to_sps() {
        let specs = vec![
            ReplaceMissingItemSpec {
                source_var: "salary".to_string(),
                target_var: "new_salary".to_string(),
                method: "series_mean".to_string(),
                span: 2,
            },
            ReplaceMissingItemSpec {
                source_var: "sales".to_string(),
                target_var: "new_sales".to_string(),
                method: "median_nearby".to_string(),
                span: 3,
            },
        ];

        let sps = ReplaceMissingCommand::to_sps(&specs);
        assert!(sps.contains("RMV new_salary = SMEAN(salary)."));
        assert!(sps.contains("RMV new_sales = MEDIAN(sales, 3)."));
        assert!(sps.contains("EXECUTE."));
    }
}
