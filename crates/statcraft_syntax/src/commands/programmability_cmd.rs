use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{execute_programmability, ProgrammabilityScript};
use statcraft_core::{DataSet, VariableMetadata};
use statcraft_io::RawValue;
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct OverwrittenVarSnapshot {
    pub name: String,
    pub meta: VariableMetadata,
    pub values: Vec<RawValue>,
}

#[derive(Debug, Clone)]
pub struct ProgrammabilityUndoEntry {
    pub created_vars: Vec<String>,
    pub overwritten_vars: Vec<OverwrittenVarSnapshot>,
}

pub struct ProgrammabilityCommand {
    history: Mutex<Vec<ProgrammabilityUndoEntry>>,
}

impl Default for ProgrammabilityCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgrammabilityCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(script: &str) -> String {
        let mut sps =
            String::from("BEGIN PROGRAM.\n# StatCraft Programmability Transformation Script\n");
        for line in script.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                sps.push_str(trimmed);
                sps.push('\n');
            }
        }
        sps.push_str("END PROGRAM.\nEXECUTE.");
        sps
    }
}

impl EngineCommand for ProgrammabilityCommand {
    fn id(&self) -> &'static str {
        "transform.programmability"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Programmability Transformations"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["script"],
            "properties": {
                "script": {
                    "type": "string",
                    "description": "Multi-line transformation script"
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

        let script_str = params
            .get("script")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Script cannot be empty".to_string(),
            })?;

        let parsed_script =
            ProgrammabilityScript::parse(script_str).map_err(|e| CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Failed to parse script: {}", e),
            })?;

        let mut overwritten_vars = Vec::new();
        let mut seen_targets = HashSet::new();

        for rule in &parsed_script.rules {
            if seen_targets.insert(rule.target_var.clone()) {
                if let Some(col_idx) = ds.dictionary.find_index(&rule.target_var) {
                    if let Some(meta) = ds.dictionary.get(col_idx).cloned() {
                        let values = ds
                            .table
                            .columns
                            .get(col_idx)
                            .map(|col| col.values.clone())
                            .unwrap_or_default();
                        overwritten_vars.push(OverwrittenVarSnapshot {
                            name: rule.target_var.clone(),
                            meta,
                            values,
                        });
                    }
                }
            }
        }

        let exec_result = execute_programmability(ds, &parsed_script).map_err(|e| {
            CommandError::ExecutionFailed(format!("Programmability execution failed: {}", e))
        })?;

        let undo_entry = ProgrammabilityUndoEntry {
            created_vars: exec_result.created_vars.clone(),
            overwritten_vars,
        };

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_entry);
        }

        Ok(json!({
            "status": "success",
            "created": exec_result.created_vars,
            "modified": exec_result.modified_vars,
            "rows_affected": exec_result.rows_affected,
            "statements": exec_result.statements_count,
            "logs": exec_result.logs,
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
                .ok_or_else(|| CommandError::ExecutionFailed("Nothing to undo".to_string()))?
        };

        // 1. Delete created variables
        for var_name in &entry.created_vars {
            if let Some(idx) = ds.dictionary.find_index(var_name) {
                let _ = ds.remove_variable(idx);
            }
        }

        // 2. Restore overwritten variables
        for snap in entry.overwritten_vars {
            if let Some(col_idx) = ds.dictionary.find_index(&snap.name) {
                if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                    *meta = snap.meta;
                }
                if let Some(col) = ds.table.columns.get_mut(col_idx) {
                    col.values = snap.values;
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
        let csv = "salary,educ\n10000,12\n20000,14\n50000,16\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_programmability_to_sps() {
        let script = "target = salary * 2\nIF educ >= 16 THEN flag = 1 ELSE 0";
        let sps = ProgrammabilityCommand::to_sps(script);
        assert!(sps.starts_with("BEGIN PROGRAM."));
        assert!(sps.contains("target = salary * 2"));
        assert!(sps.contains("IF educ >= 16 THEN flag = 1 ELSE 0"));
        assert!(sps.ends_with("END PROGRAM.\nEXECUTE."));
    }

    #[test]
    fn test_programmability_execute_and_undo() {
        let cmd = ProgrammabilityCommand::new();
        let mut ds = Some(make_test_dataset());

        let script = "bonus = salary * 0.1\nIF educ >= 16 THEN high_educ = 1 ELSE 0";
        let params = json!({ "script": script });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(res["statements"], 2);

        let active_ds = ds.as_ref().unwrap();
        assert_eq!(active_ds.col_count(), 4);
        let bonus_idx = active_ds.dictionary.find_index("bonus").unwrap();
        assert_eq!(
            active_ds.get_cell(0, bonus_idx),
            Some(&RawValue::Number(1000.0))
        );

        // Test Undo
        cmd.undo(&mut ds).unwrap();
        let rolled_back_ds = ds.as_ref().unwrap();
        assert_eq!(rolled_back_ds.col_count(), 2);
        assert!(rolled_back_ds.dictionary.find_index("bonus").is_none());
        assert!(rolled_back_ds.dictionary.find_index("high_educ").is_none());
    }
}
