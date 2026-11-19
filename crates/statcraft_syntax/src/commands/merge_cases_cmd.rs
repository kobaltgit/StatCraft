use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::merge::{add_cases, load_dataset_from_file, AddCasesConfig};
use statcraft_core::{DataSet, RawTable, VariableDictionary};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct MergeCasesCommand {
    history: Mutex<Vec<(RawTable, VariableDictionary)>>,
}

impl Default for MergeCasesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeCasesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(
        file_path: &str,
        in_variable: Option<&str>,
        renames: &HashMap<String, String>,
        interleave_by: Option<&[String]>,
    ) -> String {
        let mut parts = vec![
            "ADD FILES /FILE=*".to_string(),
            format!("/FILE='{}'", file_path),
        ];

        if !renames.is_empty() {
            let mut r_parts = Vec::new();
            for (old_n, new_n) in renames {
                r_parts.push(format!("({}={})", old_n, new_n));
            }
            parts.push(format!("/RENAME {}", r_parts.join(" ")));
        }

        if let Some(in_v) = in_variable {
            let trimmed = in_v.trim();
            if !trimmed.is_empty() {
                parts.push(format!("/IN={}", trimmed));
            }
        }

        if let Some(keys) = interleave_by {
            if !keys.is_empty() {
                parts.push(format!("/BY {}", keys.join(" ")));
            }
        }

        format!("{}.\nEXECUTE.", parts.join("\n  "))
    }

    pub fn parse_config(params: &Value) -> Result<(String, AddCasesConfig), CommandError> {
        let file_path = params
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: "data.merge_cases",
                reason: "Missing 'file_path' parameter".to_string(),
            })?
            .to_string();

        let included_variables = params
            .get("included_variables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let mut renames_second = HashMap::new();
        if let Some(renames_obj) = params.get("renames").and_then(|v| v.as_object()) {
            for (k, v) in renames_obj {
                if let Some(new_name) = v.as_str() {
                    renames_second.insert(k.clone(), new_name.trim().to_string());
                }
            }
        }

        let in_variable = params
            .get("in_variable")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let interleave_by = params
            .get("interleave_by")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            });

        let config = AddCasesConfig {
            included_variables,
            renames_second,
            in_variable,
            interleave_by,
        };

        Ok((file_path, config))
    }
}

impl EngineCommand for MergeCasesCommand {
    fn id(&self) -> &'static str {
        "data.merge_cases"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Merge Files > Add Cases..."
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the secondary dataset file (.sav, .zsav, .csv, .stc, .xlsx)"
                },
                "included_variables": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional list of variables to retain in the merged active dataset"
                },
                "renames": {
                    "type": "object",
                    "description": "Optional variable renames for the secondary dataset (old_name -> new_name)"
                },
                "in_variable": {
                    "type": "string",
                    "description": "Optional source indicator variable name (0 for active, 1 for secondary)"
                },
                "interleave_by": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional key variables for interleaved sorting (/BY)"
                }
            },
            "required": ["file_path"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|d| d.col_count() > 0).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let (file_path, config) = Self::parse_config(&params)?;

        let second_ds = load_dataset_from_file(&file_path).map_err(|e| {
            CommandError::ExecutionFailed(format!("Failed to load secondary file: {}", e))
        })?;

        let sps = Self::to_sps(
            &file_path,
            config.in_variable.as_deref(),
            &config.renames_second,
            config.interleave_by.as_deref(),
        );

        let (old_table, old_dict) = add_cases(ds, &second_ds.table, &second_ds.dictionary, &config)
            .map_err(|e| CommandError::ExecutionFailed(format!("Add cases failed: {}", e)))?;

        if let Ok(mut hist) = self.history.lock() {
            hist.push((old_table, old_dict));
        }

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

    #[test]
    fn test_merge_cases_sps_generation() {
        let mut renames = HashMap::new();
        renames.insert("v1".to_string(), "age".to_string());
        let sps = MergeCasesCommand::to_sps(
            "data/part2.sav",
            Some("source01"),
            &renames,
            Some(&["id".to_string()]),
        );
        assert!(sps.contains("ADD FILES /FILE=*"));
        assert!(sps.contains("/FILE='data/part2.sav'"));
        assert!(sps.contains("/IN=source01"));
        assert!(sps.contains("/BY id"));
    }
}
