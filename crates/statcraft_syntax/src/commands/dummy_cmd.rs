use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::generate_dummies_for_column;
use statcraft_core::{DataSet, DataType, Measure};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct DummyUndoEntry {
    pub created_cols: Vec<usize>,
}

pub struct CreateDummiesCommand {
    history: Mutex<Vec<DummyUndoEntry>>,
}

pub type DummyCommand = CreateDummiesCommand;

impl Default for CreateDummiesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateDummiesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(variables: &[&str], root_prefix: Option<&str>, omit_reference: bool) -> String {
        let prefix = root_prefix.unwrap_or("d_");
        let roots: Vec<String> = variables
            .iter()
            .map(|v| format!("{}{}", prefix, v))
            .collect();

        let omit_clause = if omit_reference {
            "/OMITFIRST=YES"
        } else {
            "/OMITFIRST=NO"
        };

        format!(
            "SPSSINC CREATE DUMMIES VARIABLES={}\n  /ROOTNAMES={}\n  {}.\nEXECUTE.",
            variables.join(" "),
            roots.join(" "),
            omit_clause
        )
    }
}

impl EngineCommand for CreateDummiesCommand {
    fn id(&self) -> &'static str {
        "transform.create_dummies"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Create Dummy Variables"
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
                    "items": { "type": "string" }
                },
                "root_prefix": { "type": "string", "default": "d_" },
                "omit_reference": { "type": "boolean", "default": false }
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

        let var_names: Vec<String> = params
            .get("variables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        if var_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Select at least one variable".to_string(),
            });
        }

        let root_prefix = params
            .get("root_prefix")
            .and_then(|v| v.as_str())
            .unwrap_or("d_")
            .trim()
            .to_string();

        let omit_reference = params
            .get("omit_reference")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Validate all variables exist before performing any mutations
        for var in &var_names {
            if ds.dictionary.find_index(var).is_none() {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Variable '{}' not found in dataset", var),
                });
            }
        }

        let mut batch_created_cols: Vec<usize> = Vec::new();
        let mut created_names: Vec<String> = Vec::new();

        for var in &var_names {
            let col_idx = match ds.dictionary.find_index(var) {
                Some(idx) => idx,
                None => continue,
            };

            let src_meta = match ds.dictionary.get(col_idx) {
                Some(m) => m.clone(),
                None => continue,
            };

            let col_values = match ds.table.columns.get(col_idx) {
                Some(c) => c.values.clone(),
                None => continue,
            };

            let root = format!("{}{}", root_prefix, var);
            let dummies = generate_dummies_for_column(
                var,
                Some(&root),
                &col_values,
                Some(&src_meta.values),
                omit_reference,
            );

            for gen_col in dummies {
                let mut target_name = gen_col.col_name.clone();
                let mut counter = 1;
                while ds.dictionary.find_index(&target_name).is_some() {
                    target_name = format!("{}_{}", gen_col.col_name, counter);
                    counter += 1;
                }

                let new_idx = ds.add_variable(
                    &target_name,
                    DataType::Numeric { decimals: 0 },
                    Measure::Nominal,
                )?;

                if let Some(m) = ds.dictionary.get_mut(new_idx) {
                    m.label = gen_col.label;
                }

                if let Some(col) = ds.table.columns.get_mut(new_idx) {
                    col.values = gen_col.values;
                }

                batch_created_cols.push(new_idx);
                created_names.push(target_name);
            }
        }

        let count = created_names.len();

        if let Ok(mut hist) = self.history.lock() {
            hist.push(DummyUndoEntry {
                created_cols: batch_created_cols,
            });
        }

        Ok(json!({
            "status": "success",
            "created_variables": created_names,
            "count": count
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

        if let Some(mut undo_entry) = entry {
            // Sort indices in descending order so that removing higher indices does not invalidate lower ones
            undo_entry.created_cols.sort_unstable_by(|a, b| b.cmp(a));
            for col_idx in undo_entry.created_cols {
                ds.remove_variable(col_idx)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::RawValue;

    fn build_test_dataset() -> DataSet {
        let mut ds = DataSet::empty();
        let idx = ds
            .add_variable(
                "jobcat",
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )
            .unwrap();

        if let Some(meta) = ds.dictionary.get_mut(idx) {
            meta.values.insert("1", "Clerical");
            meta.values.insert("2", "Custodial");
            meta.values.insert("3", "Manager");
        }

        if let Some(col) = ds.table.columns.get_mut(idx) {
            col.values = vec![
                RawValue::Number(1.0),
                RawValue::Number(2.0),
                RawValue::Number(3.0),
                RawValue::Number(1.0),
                RawValue::Null,
            ];
        }

        ds
    }

    #[test]
    fn test_create_dummies_command_execute_and_undo() {
        let cmd = CreateDummiesCommand::new();
        let mut ds = Some(build_test_dataset());

        let params = json!({
            "variables": ["jobcat"],
            "root_prefix": "d_",
            "omit_reference": false
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(res["count"], 3);

        let d = ds.as_ref().unwrap();
        assert_eq!(d.col_count(), 4); // jobcat + 3 dummies
        assert_eq!(d.dictionary.get(1).unwrap().name, "d_jobcat_1");
        assert_eq!(d.dictionary.get(2).unwrap().name, "d_jobcat_2");
        assert_eq!(d.dictionary.get(3).unwrap().name, "d_jobcat_3");

        // Verify values
        assert_eq!(d.get_cell(0, 1), Some(&RawValue::Number(1.0)));
        assert_eq!(d.get_cell(1, 1), Some(&RawValue::Number(0.0)));
        assert_eq!(d.get_cell(4, 1), Some(&RawValue::Null));

        // Undo
        cmd.undo(&mut ds).unwrap();
        let d_undone = ds.as_ref().unwrap();
        assert_eq!(d_undone.col_count(), 1);
        assert_eq!(d_undone.dictionary.get(0).unwrap().name, "jobcat");
    }

    #[test]
    fn test_create_dummies_command_omit_reference() {
        let cmd = CreateDummiesCommand::new();
        let mut ds = Some(build_test_dataset());

        let params = json!({
            "variables": ["jobcat"],
            "root_prefix": "cat_",
            "omit_reference": true
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(res["count"], 2);

        let d = ds.as_ref().unwrap();
        assert_eq!(d.col_count(), 3); // jobcat + 2 dummies
        assert_eq!(d.dictionary.get(1).unwrap().name, "cat_jobcat_2");
        assert_eq!(d.dictionary.get(2).unwrap().name, "cat_jobcat_3");
    }

    #[test]
    fn test_create_dummies_sps() {
        let sps = CreateDummiesCommand::to_sps(&["gender", "jobcat"], Some("d_"), false);
        assert!(sps.contains("SPSSINC CREATE DUMMIES VARIABLES=gender jobcat"));
        assert!(sps.contains("/ROOTNAMES=d_gender d_jobcat"));
        assert!(sps.contains("/OMITFIRST=NO"));
    }
}
