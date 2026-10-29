use super::recode_diff_cmd::parse_recode_plan;
use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{apply_recode, RecodePlan};
use statcraft_core::DataSet;
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone)]
struct SameUndoItem {
    col_idx: usize,
    old_values: Vec<RawValue>,
}

pub struct RecodeIntoSameCommand {
    history: Mutex<Vec<Vec<SameUndoItem>>>,
}

impl Default for RecodeIntoSameCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl RecodeIntoSameCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(vars: &[&str], plan: &RecodePlan) -> String {
        format!("RECODE {} {}.\nEXECUTE.", vars.join(" "), plan.to_sps())
    }
}

impl EngineCommand for RecodeIntoSameCommand {
    fn id(&self) -> &'static str {
        "transform.recode.same"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Recode into Same Variables"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["vars", "mappings"],
            "properties": {
                "vars": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "mappings": { "type": "array" }
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

        let plan = parse_recode_plan(&params, self.id())?;

        let vars: Vec<String> = params
            .get("vars")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        if vars.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Variable list cannot be empty".to_string(),
            });
        }

        let mut batch_undo = Vec::new();
        let row_count = ds.row_count();

        for var_name in &vars {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found", var_name),
                    })?;

            let meta = ds.dictionary.get(col_idx).cloned().ok_or_else(|| {
                CommandError::ExecutionFailed("Dictionary metadata missing".to_string())
            })?;

            let mut input_vals = Vec::with_capacity(row_count);
            for row in 0..row_count {
                let cell = ds.get_cell(row, col_idx);
                let num_val = match cell {
                    Some(RawValue::Number(n)) => {
                        if meta.missing.is_missing(&RawValue::Number(*n)) {
                            None
                        } else {
                            Some(*n)
                        }
                    }
                    Some(RawValue::Text(s)) => s.parse::<f64>().ok(),
                    _ => None,
                };
                input_vals.push(num_val);
            }

            let is_user_missing = |val: f64| meta.missing.is_missing(&RawValue::Number(val));
            let transformed = apply_recode(&input_vals, &plan, is_user_missing);

            let new_raw_vals: Vec<RawValue> = transformed
                .into_iter()
                .map(|opt| match opt {
                    Some(v) => RawValue::Number(v),
                    None => RawValue::Null,
                })
                .collect();

            let old_values = ds.table.columns[col_idx].values.clone();
            ds.table.columns[col_idx].values = new_raw_vals;

            batch_undo.push(SameUndoItem {
                col_idx,
                old_values,
            });
        }

        if let Ok(mut hist) = self.history.lock() {
            hist.push(batch_undo);
        }

        Ok(json!({
            "status": "success",
            "vars": vars,
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

        if let Some(items) = batch {
            for item in items.into_iter().rev() {
                if let Some(col) = ds.table.columns.get_mut(item.col_idx) {
                    col.values = item.old_values;
                }
            }
        }

        Ok(())
    }
}
