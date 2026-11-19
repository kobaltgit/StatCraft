use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::merge::{add_variables, load_dataset_from_file, AddVarsConfig, MergeMethod};
use statcraft_core::{DataSet, RawTable, VariableDictionary};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct MergeVarsCommand {
    history: Mutex<Vec<(RawTable, VariableDictionary)>>,
}

impl Default for MergeVarsCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeVarsCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(
        file_path: &str,
        method: MergeMethod,
        keys: &[String],
        renames: &HashMap<String, String>,
        drop_second: &[String],
        in_second: Option<&str>,
    ) -> String {
        let file_clause = if method == MergeMethod::OneToManyTable {
            format!("/TABLE='{}'", file_path)
        } else {
            format!("/FILE='{}'", file_path)
        };

        let mut parts = vec!["MATCH FILES /FILE=*".to_string(), file_clause];

        if !keys.is_empty() {
            parts.push(format!("/BY {}", keys.join(" ")));
        }

        if !renames.is_empty() {
            let mut r_parts = Vec::new();
            for (old_n, new_n) in renames {
                r_parts.push(format!("({}={})", old_n, new_n));
            }
            parts.push(format!("/RENAME {}", r_parts.join(" ")));
        }

        if !drop_second.is_empty() {
            parts.push(format!("/DROP={}", drop_second.join(" ")));
        }

        if let Some(in_v) = in_second {
            let trimmed = in_v.trim();
            if !trimmed.is_empty() {
                parts.push(format!("/IN={}", trimmed));
            }
        }

        format!("{}.\nEXECUTE.", parts.join("\n  "))
    }

    pub fn parse_config(params: &Value) -> Result<(String, AddVarsConfig), CommandError> {
        let file_path = params
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: "data.merge_vars",
                reason: "Missing 'file_path' parameter".to_string(),
            })?
            .to_string();

        let method_str = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("OneToOneWithKeys");

        let method = match method_str {
            "OneToManyTable" => MergeMethod::OneToManyTable,
            "Sequential" => MergeMethod::Sequential,
            _ => MergeMethod::OneToOneWithKeys,
        };

        let keys = params
            .get("keys")
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

        let drop_second = params
            .get("drop_second")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let in_active_variable = params
            .get("in_active_variable")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let in_second_variable = params
            .get("in_second_variable")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let config = AddVarsConfig {
            method,
            keys,
            renames_second,
            drop_second,
            in_active_variable,
            in_second_variable,
        };

        Ok((file_path, config))
    }
}

impl EngineCommand for MergeVarsCommand {
    fn id(&self) -> &'static str {
        "data.merge_vars"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Merge Files > Add Variables..."
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
                "method": {
                    "type": "string",
                    "enum": ["OneToOneWithKeys", "OneToManyTable", "Sequential"],
                    "description": "Matching algorithm"
                },
                "keys": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Key variables for matching (/BY)"
                },
                "renames": {
                    "type": "object",
                    "description": "Variable renames for secondary columns"
                },
                "drop_second": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Variables from secondary file to exclude (/DROP)"
                },
                "in_second_variable": {
                    "type": "string",
                    "description": "Optional indicator variable for secondary match presence (/IN)"
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
            config.method,
            &config.keys,
            &config.renames_second,
            &config.drop_second,
            config.in_second_variable.as_deref(),
        );

        let (old_table, old_dict) =
            add_variables(ds, &second_ds.table, &second_ds.dictionary, &config).map_err(|e| {
                CommandError::ExecutionFailed(format!("Add variables failed: {}", e))
            })?;

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
    fn test_merge_vars_sps_generation() {
        let mut renames = HashMap::new();
        renames.insert("salary".to_string(), "salary_ext".to_string());
        let sps = MergeVarsCommand::to_sps(
            "data/demographics.sav",
            MergeMethod::OneToOneWithKeys,
            &["id".to_string()],
            &renames,
            &["temp".to_string()],
            Some("in_demo"),
        );
        assert!(sps.contains("MATCH FILES /FILE=*"));
        assert!(sps.contains("/FILE='data/demographics.sav'"));
        assert!(sps.contains("/BY id"));
        assert!(sps.contains("/DROP=temp"));
        assert!(sps.contains("/IN=in_demo"));
    }
}
