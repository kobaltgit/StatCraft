use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_sort_permutation, invert_permutation, SortDirection, SortKeySpec};
use statcraft_core::{DataSet, SplitFileConfig, SplitFileMode};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct SplitFileUndoEntry {
    pub previous_split_config: Option<SplitFileConfig>,
    pub previous_permutation: Option<Vec<usize>>,
}

pub struct SplitFileCommand {
    history: Mutex<Vec<SplitFileUndoEntry>>,
}

impl Default for SplitFileCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SplitFileCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(mode: SplitFileMode, vars: &[String], sort: bool) -> String {
        let mut sps = String::new();
        if sort && !vars.is_empty() && mode != SplitFileMode::Off {
            let sort_keys: Vec<String> = vars.iter().map(|v| format!("{} (A)", v)).collect();
            sps.push_str(&format!("SORT CASES BY {}.\n", sort_keys.join(" ")));
        }

        match mode {
            SplitFileMode::Off => {
                sps.push_str("SPLIT FILE OFF.\nEXECUTE.");
            }
            SplitFileMode::CompareGroups => {
                sps.push_str(&format!(
                    "SPLIT FILE LAYERED BY {}.\nEXECUTE.",
                    vars.join(" ")
                ));
            }
            SplitFileMode::OrganizeOutput => {
                sps.push_str(&format!(
                    "SPLIT FILE SEPARATE BY {}.\nEXECUTE.",
                    vars.join(" ")
                ));
            }
        }
        sps
    }
}

impl EngineCommand for SplitFileCommand {
    fn id(&self) -> &'static str {
        "data.split_file"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Split File"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "enum": ["off", "compare", "separate"]
                },
                "variables": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "sort": {
                    "type": "boolean"
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

        let mode_str = params.get("mode").and_then(|v| v.as_str()).unwrap_or("off");

        let vars: Vec<String> = params
            .get("variables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let should_sort = params
            .get("sort")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mode = match mode_str.to_lowercase().as_str() {
            "compare" | "layered" => SplitFileMode::CompareGroups,
            "separate" | "organize" => SplitFileMode::OrganizeOutput,
            _ => SplitFileMode::Off,
        };

        if mode != SplitFileMode::Off && vars.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "At least one variable must be specified for split file".to_string(),
            });
        }

        for var_name in &vars {
            if ds.dictionary.find_index(var_name).is_none() {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Variable '{}' not found in dataset", var_name),
                });
            }
        }

        let mut previous_perm: Option<Vec<usize>> = None;

        if should_sort && mode != SplitFileMode::Off && !vars.is_empty() {
            let sort_keys: Vec<SortKeySpec> = vars
                .iter()
                .map(|v| SortKeySpec::new(v.clone(), SortDirection::Ascending))
                .collect();

            let perm = compute_sort_permutation(ds, &sort_keys)
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

            let inv = invert_permutation(&perm);
            ds.reorder_cases(&perm)
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

            previous_perm = Some(inv);
        }

        let undo_entry = SplitFileUndoEntry {
            previous_split_config: ds.split_file.clone(),
            previous_permutation: previous_perm,
        };

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_entry);
        }

        let new_config = if mode == SplitFileMode::Off {
            None
        } else {
            Some(SplitFileConfig::new(mode, vars.clone()))
        };

        ds.set_split_file(new_config)?;

        Ok(json!({
            "status": "ok",
            "mode": mode_str,
            "variables": vars,
            "sorted": should_sort
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
            ds.set_split_file(entry.previous_split_config)?;
            if let Some(inv) = entry.previous_permutation {
                ds.reorder_cases(&inv)
                    .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            }
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
        ds.dictionary.get_mut(0).unwrap().name = "gender".to_string();
        ds.dictionary.get_mut(1).unwrap().name = "salary".to_string();
        ds.table.columns[0].name = "gender".to_string();
        ds.table.columns[1].name = "salary".to_string();

        // Row 0: gender=2, salary=200
        // Row 1: gender=1, salary=100
        // Row 2: gender=2, salary=400
        // Row 3: gender=1, salary=300
        ds.set_cell(0, 0, RawValue::Number(2.0)).unwrap();
        ds.set_cell(0, 1, RawValue::Number(200.0)).unwrap();

        ds.set_cell(1, 0, RawValue::Number(1.0)).unwrap();
        ds.set_cell(1, 1, RawValue::Number(100.0)).unwrap();

        ds.set_cell(2, 0, RawValue::Number(2.0)).unwrap();
        ds.set_cell(2, 1, RawValue::Number(400.0)).unwrap();

        ds.set_cell(3, 0, RawValue::Number(1.0)).unwrap();
        ds.set_cell(3, 1, RawValue::Number(300.0)).unwrap();

        ds
    }

    #[test]
    fn test_split_file_command_execution_and_undo() {
        let cmd = SplitFileCommand::new();
        let mut ds = Some(make_test_dataset());

        assert!(cmd.enabled(ds.as_ref()));

        let params = json!({
            "mode": "compare",
            "variables": ["gender"],
            "sort": true
        });

        let res = cmd
            .execute(&mut ds, params)
            .expect("Execution should succeed");
        assert_eq!(res["status"], "ok");

        let current_ds = ds.as_ref().unwrap();
        assert!(current_ds.split_file().unwrap().is_active());
        assert_eq!(
            current_ds.split_file().unwrap().mode,
            SplitFileMode::CompareGroups
        );

        // Verify rows are sorted: gender=1 should be first two rows
        assert_eq!(current_ds.get_cell(0, 0), Some(&RawValue::Number(1.0)));
        assert_eq!(current_ds.get_cell(1, 0), Some(&RawValue::Number(1.0)));
        assert_eq!(current_ds.get_cell(2, 0), Some(&RawValue::Number(2.0)));

        // Test Undo
        cmd.undo(&mut ds).expect("Undo should succeed");
        let restored_ds = ds.as_ref().unwrap();
        assert_eq!(restored_ds.split_file(), None);
        // Original row 0 had gender=2
        assert_eq!(restored_ds.get_cell(0, 0), Some(&RawValue::Number(2.0)));
    }

    #[test]
    fn test_split_file_sps_syntax() {
        let sps_off = SplitFileCommand::to_sps(SplitFileMode::Off, &[], false);
        assert_eq!(sps_off, "SPLIT FILE OFF.\nEXECUTE.");

        let sps_compare = SplitFileCommand::to_sps(
            SplitFileMode::CompareGroups,
            &["gender".to_string(), "dept".to_string()],
            true,
        );
        assert_eq!(
            sps_compare,
            "SORT CASES BY gender (A) dept (A).\nSPLIT FILE LAYERED BY gender dept.\nEXECUTE."
        );

        let sps_separate = SplitFileCommand::to_sps(
            SplitFileMode::OrganizeOutput,
            &["gender".to_string()],
            false,
        );
        assert_eq!(sps_separate, "SPLIT FILE SEPARATE BY gender.\nEXECUTE.");
    }
}
