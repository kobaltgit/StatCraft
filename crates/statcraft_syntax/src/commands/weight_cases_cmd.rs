use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::DataSet;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct WeightCasesUndoEntry {
    pub previous_weight_var: Option<String>,
}

pub struct WeightCasesCommand {
    history: Mutex<Vec<WeightCasesUndoEntry>>,
}

impl Default for WeightCasesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl WeightCasesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(weight_var: Option<&str>) -> String {
        match weight_var {
            Some(var) if !var.trim().is_empty() => {
                format!("WEIGHT BY {}.\nEXECUTE.", var.trim())
            }
            _ => "WEIGHT OFF.\nEXECUTE.".to_string(),
        }
    }
}

impl EngineCommand for WeightCasesCommand {
    fn id(&self) -> &'static str {
        "data.weight_cases"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Weight Cases"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "variable": {
                    "type": ["string", "null"],
                    "description": "Name of the numeric weight variable, or null to disable weighting"
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        match dataset {
            Some(ds) => ds.col_count() > 0 && ds.row_count() > 0,
            None => false,
        }
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let var_name = params
            .get("variable")
            .and_then(|v| {
                if v.is_null() {
                    None
                } else {
                    v.as_str().map(|s| s.trim().to_string())
                }
            })
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("off"));

        if let Some(ref name) = var_name {
            let col_idx =
                ds.dictionary
                    .find_index(name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dataset", name),
                    })?;

            let meta = ds
                .dictionary
                .get(col_idx)
                .ok_or_else(|| CommandError::ExecutionFailed("Metadata missing".to_string()))?;

            if !meta.var_type.is_numeric() {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Weight variable '{}' must be numeric", name),
                });
            }
        }

        let undo_entry = WeightCasesUndoEntry {
            previous_weight_var: ds.weight_var().map(|s| s.to_string()),
        };

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_entry);
        }

        let target_var = var_name.clone();
        ds.set_weight_variable(target_var)?;

        Ok(json!({
            "status": "ok",
            "weight_var": var_name
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let entry = {
            let mut hist = self
                .history
                .lock()
                .map_err(|_| CommandError::ExecutionFailed("Mutex poison error".to_string()))?;
            hist.pop()
        };

        if let Some(entry) = entry {
            ds.set_weight_variable(entry.previous_weight_var)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::{DataType, Measure, RawValue};

    fn make_test_dataset() -> DataSet {
        let mut ds = DataSet::blank(4, 2).unwrap();
        ds.dictionary.get_mut(0).unwrap().name = "salary".to_string();
        ds.dictionary.get_mut(1).unwrap().name = "w_cases".to_string();
        ds.table.columns[0].name = "salary".to_string();
        ds.table.columns[1].name = "w_cases".to_string();

        ds.set_cell(0, 1, RawValue::Number(2.0)).unwrap();
        ds.set_cell(1, 1, RawValue::Number(1.0)).unwrap();
        ds.set_cell(2, 1, RawValue::Number(3.0)).unwrap();
        ds.set_cell(3, 1, RawValue::Number(0.0)).unwrap();
        ds
    }

    #[test]
    fn test_weight_cases_sps_syntax() {
        assert_eq!(
            WeightCasesCommand::to_sps(Some("w_cases")),
            "WEIGHT BY w_cases.\nEXECUTE."
        );
        assert_eq!(WeightCasesCommand::to_sps(None), "WEIGHT OFF.\nEXECUTE.");
        assert_eq!(
            WeightCasesCommand::to_sps(Some("")),
            "WEIGHT OFF.\nEXECUTE."
        );
    }

    #[test]
    fn test_weight_cases_command_execution_and_undo() {
        let cmd = WeightCasesCommand::new();
        let mut ds = Some(make_test_dataset());

        assert!(!ds.as_ref().unwrap().is_weighted());

        let res = cmd.execute(&mut ds, json!({ "variable": "w_cases" }));
        assert!(res.is_ok());
        assert!(ds.as_ref().unwrap().is_weighted());
        assert_eq!(ds.as_ref().unwrap().weight_var(), Some("w_cases"));
        assert_eq!(ds.as_ref().unwrap().row_weight(0), 2.0);

        let undo_res = cmd.undo(&mut ds);
        assert!(undo_res.is_ok());
        assert!(!ds.as_ref().unwrap().is_weighted());
        assert_eq!(ds.as_ref().unwrap().weight_var(), None);
        assert_eq!(ds.as_ref().unwrap().row_weight(0), 1.0);
    }

    #[test]
    fn test_weight_cases_invalid_var() {
        let cmd = WeightCasesCommand::new();
        let mut ds = Some(make_test_dataset());

        let res_missing = cmd.execute(&mut ds, json!({ "variable": "unknown_var" }));
        assert!(matches!(
            res_missing,
            Err(CommandError::InvalidParams { .. })
        ));

        ds.as_mut()
            .unwrap()
            .add_variable(
                "text_col",
                DataType::String { max_len: 20 },
                Measure::Nominal,
            )
            .unwrap();

        let res_str = cmd.execute(&mut ds, json!({ "variable": "text_col" }));
        assert!(matches!(res_str, Err(CommandError::InvalidParams { .. })));
    }
}
