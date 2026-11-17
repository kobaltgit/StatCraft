use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_variable_sort_permutation, invert_permutation, SortDirection, VariableSortAttribute,
};
use statcraft_core::DataSet;
use std::sync::Mutex;

pub struct SortVariablesCommand {
    history: Mutex<Vec<Vec<usize>>>,
}

impl Default for SortVariablesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SortVariablesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(attribute: VariableSortAttribute, direction: SortDirection) -> String {
        let attr_name = match attribute {
            VariableSortAttribute::Name => "name",
            VariableSortAttribute::Type => "type",
            VariableSortAttribute::Label => "label",
            VariableSortAttribute::Measure => "measure",
            VariableSortAttribute::Role => "role",
            VariableSortAttribute::Alignment => "align",
            VariableSortAttribute::Width => "width",
            VariableSortAttribute::Decimals => "decimals",
        };
        let dir = match direction {
            SortDirection::Ascending => "(A)",
            SortDirection::Descending => "(D)",
        };
        format!("SORT VARIABLES BY {} {}.", attr_name, dir)
    }

    pub fn parse_params(
        params: &Value,
        cmd_id: &'static str,
    ) -> Result<(VariableSortAttribute, SortDirection), CommandError> {
        let attr_str = params
            .get("attribute")
            .and_then(|v| v.as_str())
            .unwrap_or("name");

        let attribute = match attr_str.to_lowercase().as_str() {
            "name" => VariableSortAttribute::Name,
            "type" => VariableSortAttribute::Type,
            "label" => VariableSortAttribute::Label,
            "measure" => VariableSortAttribute::Measure,
            "role" => VariableSortAttribute::Role,
            "alignment" | "align" => VariableSortAttribute::Alignment,
            "width" => VariableSortAttribute::Width,
            "decimals" => VariableSortAttribute::Decimals,
            other => {
                return Err(CommandError::InvalidParams {
                    id: cmd_id,
                    reason: format!("Unknown sort attribute: '{}'", other),
                })
            }
        };

        let direction = match params.get("direction").and_then(|v| v.as_str()) {
            Some("descending") | Some("Descending") | Some("D") | Some("d") => {
                SortDirection::Descending
            }
            _ => SortDirection::Ascending,
        };

        Ok((attribute, direction))
    }
}

impl EngineCommand for SortVariablesCommand {
    fn id(&self) -> &'static str {
        "data.sort_variables"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Sort Variables"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "attribute": {
                    "type": "string",
                    "enum": ["name", "type", "label", "measure", "role", "alignment", "width", "decimals"],
                    "default": "name",
                    "description": "Dictionary attribute by which variables are sorted"
                },
                "direction": {
                    "type": "string",
                    "enum": ["ascending", "descending", "A", "D"],
                    "default": "ascending",
                    "description": "Sort order: ascending (A) or descending (D)"
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

        let (attribute, direction) = Self::parse_params(&params, self.id())?;

        let permutation = compute_variable_sort_permutation(&ds.dictionary, attribute, direction);
        let inv_perm = invert_permutation(&permutation);

        ds.reorder_variables(&permutation).map_err(|e| {
            CommandError::ExecutionFailed(format!("Reorder variables failed: {}", e))
        })?;

        if let Ok(mut hist) = self.history.lock() {
            hist.push(inv_perm);
        }

        let sps = Self::to_sps(attribute, direction);

        Ok(json!({
            "status": "success",
            "cols": ds.col_count(),
            "sps": sps
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let inv_perm = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        if let Some(perm) = inv_perm {
            ds.reorder_variables(&perm).map_err(|e| {
                CommandError::ExecutionFailed(format!("Undo sort variables failed: {}", e))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_variables_command_lifecycle() {
        let cmd = SortVariablesCommand::new();
        let mut ds = DataSet::blank(2, 3).unwrap();
        // Rename vars for clear test
        ds.dictionary.get_mut(0).unwrap().name = "c_var".to_string();
        ds.dictionary.get_mut(1).unwrap().name = "a_var".to_string();
        ds.dictionary.get_mut(2).unwrap().name = "b_var".to_string();
        ds.table.columns[0].name = "c_var".to_string();
        ds.table.columns[1].name = "a_var".to_string();
        ds.table.columns[2].name = "b_var".to_string();

        let mut opt_ds = Some(ds);
        let params = json!({
            "attribute": "name",
            "direction": "ascending"
        });

        let res = cmd.execute(&mut opt_ds, params).unwrap();
        assert_eq!(res["status"], "success");

        let current = opt_ds.as_ref().unwrap();
        assert_eq!(current.dictionary.get(0).unwrap().name, "a_var");
        assert_eq!(current.dictionary.get(1).unwrap().name, "b_var");
        assert_eq!(current.dictionary.get(2).unwrap().name, "c_var");

        // Undo
        cmd.undo(&mut opt_ds).unwrap();
        let undone = opt_ds.as_ref().unwrap();
        assert_eq!(undone.dictionary.get(0).unwrap().name, "c_var");
        assert_eq!(undone.dictionary.get(1).unwrap().name, "a_var");
        assert_eq!(undone.dictionary.get(2).unwrap().name, "b_var");
    }
}
