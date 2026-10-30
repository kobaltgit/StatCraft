use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_shift_raw, ShiftDirection};
use statcraft_core::{DataSet, VariableMetadata};
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum ShiftUndoEntry {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

pub struct ShiftValuesCommand {
    history: Mutex<Vec<ShiftUndoEntry>>,
}

impl Default for ShiftValuesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ShiftValuesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(
        source_var: &str,
        target_var: &str,
        direction: ShiftDirection,
        step: usize,
        target_label: Option<&str>,
    ) -> String {
        let dir_clause = match direction {
            ShiftDirection::Lag => format!("/LAG={}", step),
            ShiftDirection::Lead => format!("/LEAD={}", step),
        };
        let mut sps = format!(
            "SHIFT VALUES /VARIABLE={} /RESULT={} {}.\n",
            source_var, target_var, dir_clause
        );
        if let Some(lbl) = target_label {
            let trimmed = lbl.trim();
            if !trimmed.is_empty() {
                sps.push_str(&format!(
                    "VARIABLE LABELS {} \"{}\".\n",
                    target_var, trimmed
                ));
            }
        }
        sps.push_str("EXECUTE.");
        sps
    }
}

impl EngineCommand for ShiftValuesCommand {
    fn id(&self) -> &'static str {
        "transform.shift_values"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Shift Values"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["source_var", "target_var", "direction", "step"],
            "properties": {
                "source_var": {
                    "type": "string",
                    "description": "Source variable name"
                },
                "target_var": {
                    "type": "string",
                    "description": "Target variable name"
                },
                "direction": {
                    "type": "string",
                    "enum": ["lag", "lead"],
                    "description": "Shift direction"
                },
                "step": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 1,
                    "description": "Number of shift steps"
                },
                "target_label": {
                    "type": "string",
                    "description": "Optional label for target variable"
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

        let source_var = params
            .get("source_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Source variable name cannot be empty".to_string(),
            })?;

        let target_var = params
            .get("target_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Target variable name cannot be empty".to_string(),
            })?;

        let direction_str = params
            .get("direction")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_ascii_lowercase())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Shift direction must be specified ('lag' or 'lead')".to_string(),
            })?;

        let direction = match direction_str.as_str() {
            "lag" => ShiftDirection::Lag,
            "lead" => ShiftDirection::Lead,
            _ => {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Unknown shift direction: '{}'", direction_str),
                });
            }
        };

        let step = params
            .get("step")
            .and_then(|v| v.as_u64())
            .map(|s| s as usize)
            .unwrap_or(1);

        if step == 0 {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Shift step must be at least 1".to_string(),
            });
        }

        let target_label = params.get("target_label").and_then(|v| v.as_str());

        let src_idx =
            ds.dictionary
                .find_index(source_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Source variable '{}' not found", source_var),
                })?;

        let src_meta = ds
            .dictionary
            .get(src_idx)
            .cloned()
            .ok_or_else(|| CommandError::ExecutionFailed("Corrupt dictionary".to_string()))?;

        let values = ds
            .table
            .columns
            .get(src_idx)
            .map(|c| c.values.as_slice())
            .ok_or_else(|| CommandError::ExecutionFailed("Column values missing".to_string()))?;

        let shifted_values = compute_shift_raw(values, direction, step);
        let row_count = ds.row_count();

        let existing_idx = ds.dictionary.find_index(target_var);

        let undo_entry = if let Some(col_idx) = existing_idx {
            let old_meta =
                ds.dictionary.get(col_idx).cloned().ok_or_else(|| {
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
                col.values = shifted_values;
            }
            if let Some(lbl) = target_label {
                let trimmed = lbl.trim();
                if !trimmed.is_empty() {
                    if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                        meta.label = trimmed.to_string();
                    }
                }
            }

            ShiftUndoEntry::Overwritten {
                col_idx,
                old_meta,
                old_values,
            }
        } else {
            let col_idx =
                ds.add_variable(target_var, src_meta.var_type.clone(), src_meta.measure)?;
            if let Some(lbl) = target_label {
                let trimmed = lbl.trim();
                if !trimmed.is_empty() {
                    if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                        meta.label = trimmed.to_string();
                    }
                }
            }
            if let Some(col) = ds.table.columns.get_mut(col_idx) {
                col.values = shifted_values;
            }

            ShiftUndoEntry::Created { col_idx }
        };

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_entry);
        }

        Ok(json!({
            "status": "success",
            "source_var": source_var,
            "target_var": target_var,
            "rows": row_count
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
            match entry {
                ShiftUndoEntry::Created { col_idx } => {
                    ds.remove_variable(col_idx)?;
                }
                ShiftUndoEntry::Overwritten {
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
            .add_variable("sales", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
        let _ = ds
            .add_variable("dept", DataType::String { max_len: 10 }, Measure::Nominal)
            .unwrap();

        for i in 1..=5 {
            ds.add_empty_row();
            let row = ds.row_count() - 1;
            let _ = ds.set_cell(row, c0, RawValue::Number(i as f64 * 100.0));
        }
        ds
    }

    #[test]
    fn test_shift_values_lag_create_and_undo() {
        let mut dataset = Some(make_test_dataset());
        let cmd = ShiftValuesCommand::new();

        let params = json!({
            "source_var": "sales",
            "target_var": "sales_lag1",
            "direction": "lag",
            "step": 1,
            "target_label": "Previous Sales"
        });

        let res = cmd.execute(&mut dataset, params);
        assert!(res.is_ok());

        let ds = dataset.as_ref().unwrap();
        assert_eq!(ds.col_count(), 3);
        let lag_idx = ds.dictionary.find_index("sales_lag1").unwrap();
        let meta = ds.dictionary.get(lag_idx).unwrap();
        assert_eq!(meta.label, "Previous Sales");

        let vals: Vec<RawValue> = (0..ds.row_count())
            .map(|r| ds.get_cell(r, lag_idx).cloned().unwrap())
            .collect();
        assert_eq!(
            vals,
            vec![
                RawValue::Null,
                RawValue::Number(100.0),
                RawValue::Number(200.0),
                RawValue::Number(300.0),
                RawValue::Number(400.0),
            ]
        );

        let undo_res = cmd.undo(&mut dataset);
        assert!(undo_res.is_ok());
        let ds_after = dataset.as_ref().unwrap();
        assert_eq!(ds_after.col_count(), 2);
        assert!(ds_after.dictionary.find_index("sales_lag1").is_none());
    }

    #[test]
    fn test_shift_values_lead_overwrite_and_undo() {
        let mut dataset = Some(make_test_dataset());
        let cmd = ShiftValuesCommand::new();

        let c_extra = dataset
            .as_mut()
            .unwrap()
            .add_variable(
                "future_sales",
                DataType::Numeric { decimals: 2 },
                Measure::Scale,
            )
            .unwrap();
        for r in 0..5 {
            let _ = dataset
                .as_mut()
                .unwrap()
                .set_cell(r, c_extra, RawValue::Number(999.0));
        }

        let params = json!({
            "source_var": "sales",
            "target_var": "future_sales",
            "direction": "lead",
            "step": 2,
        });

        let res = cmd.execute(&mut dataset, params);
        assert!(res.is_ok());

        let ds = dataset.as_ref().unwrap();
        assert_eq!(ds.col_count(), 3);
        let lead_idx = ds.dictionary.find_index("future_sales").unwrap();
        let vals: Vec<RawValue> = (0..ds.row_count())
            .map(|r| ds.get_cell(r, lead_idx).cloned().unwrap())
            .collect();
        assert_eq!(
            vals,
            vec![
                RawValue::Number(300.0),
                RawValue::Number(400.0),
                RawValue::Number(500.0),
                RawValue::Null,
                RawValue::Null,
            ]
        );

        let undo_res = cmd.undo(&mut dataset);
        assert!(undo_res.is_ok());
        let ds_after = dataset.as_ref().unwrap();
        assert_eq!(ds_after.col_count(), 3);
        assert_eq!(
            ds_after.get_cell(0, lead_idx).unwrap(),
            &RawValue::Number(999.0)
        );
    }

    #[test]
    fn test_shift_values_to_sps() {
        let sps_lag =
            ShiftValuesCommand::to_sps("sales", "sales_lag", ShiftDirection::Lag, 1, Some("Lag 1"));
        assert_eq!(
            sps_lag,
            "SHIFT VALUES /VARIABLE=sales /RESULT=sales_lag /LAG=1.\nVARIABLE LABELS sales_lag \"Lag 1\".\nEXECUTE."
        );

        let sps_lead =
            ShiftValuesCommand::to_sps("sales", "sales_lead", ShiftDirection::Lead, 2, None);
        assert_eq!(
            sps_lead,
            "SHIFT VALUES /VARIABLE=sales /RESULT=sales_lead /LEAD=2.\nEXECUTE."
        );
    }
}
