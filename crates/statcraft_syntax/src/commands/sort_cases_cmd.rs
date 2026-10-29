use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_sort_permutation, invert_permutation, SortDirection, SortKeySpec};
use statcraft_core::DataSet;
use std::sync::Mutex;

pub struct SortCasesCommand {
    history: Mutex<Vec<Vec<usize>>>,
}

impl Default for SortCasesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SortCasesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(keys: &[SortKeySpec]) -> String {
        let mut parts = Vec::new();
        for key in keys {
            let dir = match key.direction {
                SortDirection::Ascending => "(A)",
                SortDirection::Descending => "(D)",
            };
            parts.push(format!("{} {}", key.column_name, dir));
        }
        format!("SORT CASES BY {}.", parts.join(" "))
    }

    pub fn to_sps_syntax(&self, keys: &[SortKeySpec]) -> String {
        Self::to_sps(keys)
    }

    pub fn parse_keys(
        params: &Value,
        cmd_id: &'static str,
    ) -> Result<Vec<SortKeySpec>, CommandError> {
        let keys_arr = params
            .get("keys")
            .and_then(|v| v.as_array())
            .ok_or_else(|| CommandError::InvalidParams {
                id: cmd_id,
                reason: "Missing 'keys' array".to_string(),
            })?;

        if keys_arr.is_empty() {
            return Err(CommandError::InvalidParams {
                id: cmd_id,
                reason: "Sort keys list cannot be empty".to_string(),
            });
        }

        let mut specs = Vec::with_capacity(keys_arr.len());
        for item in keys_arr {
            let col_name = item
                .get("column_name")
                .and_then(|v| v.as_str())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| CommandError::InvalidParams {
                    id: cmd_id,
                    reason: "Missing or invalid 'column_name'".to_string(),
                })?;

            let direction = match item.get("direction").and_then(|v| v.as_str()) {
                Some("descending") | Some("Descending") | Some("D") | Some("d") => {
                    SortDirection::Descending
                }
                _ => SortDirection::Ascending,
            };

            specs.push(SortKeySpec::new(col_name, direction));
        }

        Ok(specs)
    }
}

impl EngineCommand for SortCasesCommand {
    fn id(&self) -> &'static str {
        "data.sort_cases"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Sort Cases"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["keys"],
            "properties": {
                "keys": {
                    "type": "array",
                    "description": "List of variables to sort cases by, in priority order",
                    "items": {
                        "type": "object",
                        "required": ["column_name"],
                        "properties": {
                            "column_name": { "type": "string" },
                            "direction": {
                                "type": "string",
                                "enum": ["ascending", "descending", "A", "D"],
                                "default": "ascending"
                            }
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

        let keys = Self::parse_keys(&params, self.id())?;

        let permutation = compute_sort_permutation(ds, &keys)
            .map_err(|e| CommandError::ExecutionFailed(format!("Sort cases failed: {}", e)))?;

        let inv_perm = invert_permutation(&permutation);

        ds.reorder_cases(&permutation)
            .map_err(|e| CommandError::ExecutionFailed(format!("Reorder cases failed: {}", e)))?;

        if let Ok(mut hist) = self.history.lock() {
            hist.push(inv_perm);
        }

        let sps = Self::to_sps(&keys);

        Ok(json!({
            "status": "success",
            "keys": keys,
            "rows": ds.row_count(),
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
            ds.reorder_cases(&perm).map_err(|e| {
                CommandError::ExecutionFailed(format!("Undo sort cases failed: {}", e))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::RawValue;

    fn make_test_dataset() -> DataSet {
        let mut ds = DataSet::blank(4, 2).unwrap();
        ds.dictionary.get_mut(0).unwrap().name = "dept".to_string();
        ds.dictionary.get_mut(1).unwrap().name = "salary".to_string();
        ds.table.columns[0].name = "dept".to_string();
        ds.table.columns[1].name = "salary".to_string();

        ds.set_cell(0, 0, RawValue::Number(2.0)).unwrap();
        ds.set_cell(0, 1, RawValue::Number(100.0)).unwrap();

        ds.set_cell(1, 0, RawValue::Number(1.0)).unwrap();
        ds.set_cell(1, 1, RawValue::Number(300.0)).unwrap();

        ds.set_cell(2, 0, RawValue::Number(2.0)).unwrap();
        ds.set_cell(2, 1, RawValue::Number(200.0)).unwrap();

        ds.set_cell(3, 0, RawValue::Number(1.0)).unwrap();
        ds.set_cell(3, 1, RawValue::Number(150.0)).unwrap();

        ds
    }

    #[test]
    fn test_sort_cases_sps_syntax() {
        let keys = vec![
            SortKeySpec::ascending("dept"),
            SortKeySpec::descending("salary"),
        ];
        let sps = SortCasesCommand::to_sps(&keys);
        assert_eq!(sps, "SORT CASES BY dept (A) salary (D).");
    }

    #[test]
    fn test_sort_cases_command_execution_and_undo() {
        let cmd = SortCasesCommand::new();
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "keys": [
                { "column_name": "dept", "direction": "ascending" },
                { "column_name": "salary", "direction": "descending" }
            ]
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(res["rows"], 4);

        let d = ds.as_ref().unwrap();
        // dept 1: salary 300 (old row 1), then 150 (old row 3)
        // dept 2: salary 200 (old row 2), then 100 (old row 0)
        assert_eq!(d.get_cell(0, 0), Some(&RawValue::Number(1.0)));
        assert_eq!(d.get_cell(0, 1), Some(&RawValue::Number(300.0)));
        assert_eq!(d.get_cell(1, 0), Some(&RawValue::Number(1.0)));
        assert_eq!(d.get_cell(1, 1), Some(&RawValue::Number(150.0)));
        assert_eq!(d.get_cell(2, 0), Some(&RawValue::Number(2.0)));
        assert_eq!(d.get_cell(2, 1), Some(&RawValue::Number(200.0)));
        assert_eq!(d.get_cell(3, 0), Some(&RawValue::Number(2.0)));
        assert_eq!(d.get_cell(3, 1), Some(&RawValue::Number(100.0)));

        // Test Undo
        cmd.undo(&mut ds).unwrap();
        let d = ds.as_ref().unwrap();
        // Restores original rows
        assert_eq!(d.get_cell(0, 0), Some(&RawValue::Number(2.0)));
        assert_eq!(d.get_cell(0, 1), Some(&RawValue::Number(100.0)));
        assert_eq!(d.get_cell(1, 0), Some(&RawValue::Number(1.0)));
        assert_eq!(d.get_cell(1, 1), Some(&RawValue::Number(300.0)));
    }

    #[test]
    fn test_sort_cases_empty_keys_error() {
        let cmd = SortCasesCommand::new();
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "keys": []
        });

        let res = cmd.execute(&mut ds, params);
        assert!(res.is_err());
    }
}
