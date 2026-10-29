use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::CompiledExpr;
use statcraft_core::{DataSet, DataType, Measure, VariableMetadata};
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum ComputeUndoEntry {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

pub struct ComputeVariableCommand {
    history: Mutex<Vec<ComputeUndoEntry>>,
}

impl Default for ComputeVariableCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeVariableCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(target: &str, expression: &str, label: Option<&str>) -> String {
        let mut sps = format!("COMPUTE {} = {}.\n", target, expression);
        if let Some(lbl) = label {
            let trimmed = lbl.trim();
            if !trimmed.is_empty() {
                sps.push_str(&format!("VARIABLE LABELS {} \"{}\".\n", target, trimmed));
            }
        }
        sps.push_str("EXECUTE.");
        sps
    }
}

impl EngineCommand for ComputeVariableCommand {
    fn id(&self) -> &'static str {
        "transform.compute"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Compute Variable"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["target", "expression"],
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target variable name"
                },
                "expression": {
                    "type": "string",
                    "description": "Numeric mathematical expression"
                },
                "label": {
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

        let target = params
            .get("target")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Target variable name cannot be empty".to_string(),
            })?;

        let expression_str = params
            .get("expression")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Expression cannot be empty".to_string(),
            })?;

        let label_opt = params.get("label").and_then(|v| v.as_str());

        let compiled =
            CompiledExpr::parse(expression_str).map_err(|e| CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Failed to parse expression: {}", e),
            })?;

        let row_count = ds.row_count();
        let mut computed_values: Vec<RawValue> = Vec::with_capacity(row_count);

        for row in 0..row_count {
            let lookup = |var_name: &str| -> Option<f64> {
                let col_idx = ds.dictionary.find_index(var_name)?;
                match ds.get_cell(row, col_idx)? {
                    RawValue::Number(n) => Some(*n),
                    RawValue::Text(s) => s.parse::<f64>().ok(),
                    RawValue::Null => Some(f64::NAN),
                }
            };

            let eval_res = compiled.eval(&lookup).unwrap_or(f64::NAN);
            if eval_res.is_nan() {
                computed_values.push(RawValue::Null);
            } else {
                computed_values.push(RawValue::Number(eval_res));
            }
        }

        let existing_idx = ds.dictionary.find_index(target);

        let undo_entry = if let Some(col_idx) = existing_idx {
            let old_meta =
                ds.dictionary.get(col_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Corrupt dictionary".to_string())
                })?;
            let old_values = ds.table.columns[col_idx].values.clone();

            ds.table.columns[col_idx].values = computed_values;
            if let Some(lbl) = label_opt {
                if !lbl.trim().is_empty() {
                    if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                        meta.label = lbl.trim().to_string();
                    }
                }
            }

            ComputeUndoEntry::Overwritten {
                col_idx,
                old_meta,
                old_values,
            }
        } else {
            let col_idx =
                ds.add_variable(target, DataType::Numeric { decimals: 2 }, Measure::Scale)?;
            if let Some(lbl) = label_opt {
                if !lbl.trim().is_empty() {
                    if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                        meta.label = lbl.trim().to_string();
                    }
                }
            }
            ds.table.columns[col_idx].values = computed_values;

            ComputeUndoEntry::Created { col_idx }
        };

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_entry);
        }

        Ok(json!({
            "status": "success",
            "target": target,
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
                ComputeUndoEntry::Created { col_idx, .. } => {
                    ds.remove_variable(col_idx)?;
                }
                ComputeUndoEntry::Overwritten {
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
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv = "height,weight\n180,80\n160,50\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_compute_new_variable_and_undo() {
        let cmd = ComputeVariableCommand::new();
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "target": "bmi",
            "expression": "weight / ((height / 100) ** 2)",
            "label": "Body Mass Index"
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");

        let active_ds = ds.as_ref().unwrap();
        assert_eq!(active_ds.col_count(), 3);
        let bmi_col = active_ds.dictionary.find_index("bmi").unwrap();
        assert_eq!(
            active_ds.dictionary.get(bmi_col).unwrap().label,
            "Body Mass Index"
        );

        // First row: 80 / (1.8^2) = 24.6913...
        match active_ds.get_cell(0, bmi_col).unwrap() {
            RawValue::Number(n) => assert!((*n - 24.6913).abs() < 1e-3),
            _ => panic!("Expected number"),
        }

        // Test Undo
        cmd.undo(&mut ds).unwrap();
        let active_ds = ds.as_ref().unwrap();
        assert_eq!(active_ds.col_count(), 2);
        assert!(active_ds.dictionary.find_index("bmi").is_none());
    }

    #[test]
    fn test_compute_overwrite_existing_and_undo() {
        let cmd = ComputeVariableCommand::new();
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "target": "weight",
            "expression": "weight * 2"
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");

        let active_ds = ds.as_ref().unwrap();
        assert_eq!(active_ds.col_count(), 2);
        let weight_col = active_ds.dictionary.find_index("weight").unwrap();
        match active_ds.get_cell(0, weight_col).unwrap() {
            RawValue::Number(n) => assert_eq!(*n, 160.0),
            _ => panic!("Expected number"),
        }

        // Test Undo
        cmd.undo(&mut ds).unwrap();
        let active_ds = ds.as_ref().unwrap();
        match active_ds.get_cell(0, weight_col).unwrap() {
            RawValue::Number(n) => assert_eq!(*n, 80.0),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_compute_sps_syntax() {
        let sps = ComputeVariableCommand::to_sps("diff", "post - pre", Some("Score Difference"));
        assert_eq!(
            sps,
            "COMPUTE diff = post - pre.\nVARIABLE LABELS diff \"Score Difference\".\nEXECUTE."
        );
    }
}
