use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::{DataSet, RawTable, VariableDictionary};
use std::sync::Mutex;

pub struct TransposeCommand {
    history: Mutex<Vec<(RawTable, VariableDictionary)>>,
}

impl Default for TransposeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl TransposeCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(variables: Option<&[String]>, name_var: Option<&str>) -> String {
        let mut parts = Vec::new();
        if let Some(vars) = variables {
            if !vars.is_empty() {
                parts.push(format!("VARIABLES={}", vars.join(" ")));
            }
        }
        if let Some(nv) = name_var {
            let trimmed = nv.trim();
            if !trimmed.is_empty() {
                parts.push(format!("/NEWNAMES={}", trimmed));
            }
        }
        if parts.is_empty() {
            "FLIP.".to_string()
        } else {
            format!("FLIP {}.", parts.join(" "))
        }
    }

    pub fn parse_params(params: &Value) -> (Option<Vec<String>>, Option<String>) {
        let vars = params
            .get("variables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            });

        let name_var = params
            .get("name_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        (vars, name_var)
    }
}

impl EngineCommand for TransposeCommand {
    fn id(&self) -> &'static str {
        "data.transpose"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Transpose"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "variables": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional subset of variables to transpose. If omitted, all variables except name_var are transposed."
                },
                "name_var": {
                    "type": "string",
                    "description": "Optional variable name whose row values will become the new column names."
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

        let (vars, name_var) = Self::parse_params(&params);

        let (old_table, old_dict) = ds
            .transpose(vars.as_deref(), name_var.as_deref())
            .map_err(|e| CommandError::ExecutionFailed(format!("Transpose failed: {}", e)))?;

        if let Ok(mut hist) = self.history.lock() {
            hist.push((old_table, old_dict));
        }

        let sps = Self::to_sps(vars.as_deref(), name_var.as_deref());

        Ok(json!({
            "status": "success",
            "rows": ds.row_count(),
            "cols": ds.col_count(),
            "sps": sps
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let previous_state = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        if let Some((old_table, old_dict)) = previous_state {
            ds.restore_structure(old_table, old_dict);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::RawValue;

    #[test]
    fn test_transpose_command_lifecycle() {
        let cmd = TransposeCommand::new();
        let mut ds = DataSet::blank(2, 2).unwrap();
        ds.set_cell(0, 0, RawValue::Number(10.0)).unwrap();
        ds.set_cell(0, 1, RawValue::Number(20.0)).unwrap();
        ds.set_cell(1, 0, RawValue::Number(30.0)).unwrap();
        ds.set_cell(1, 1, RawValue::Number(40.0)).unwrap();

        let mut opt_ds = Some(ds);
        let params = json!({});

        let res = cmd.execute(&mut opt_ds, params).unwrap();
        assert_eq!(res["status"], "success");

        let current = opt_ds.as_ref().unwrap();
        assert_eq!(current.col_count(), 3); // CASE_LBL + VAR001 + VAR002
        assert_eq!(current.row_count(), 2);

        // Undo
        cmd.undo(&mut opt_ds).unwrap();
        let undone = opt_ds.as_ref().unwrap();
        assert_eq!(undone.col_count(), 2);
        assert_eq!(undone.row_count(), 2);
    }
}
