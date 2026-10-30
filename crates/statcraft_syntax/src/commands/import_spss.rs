use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::DataSet;
use std::path::Path;

pub struct ImportSpssCommand;

impl ImportSpssCommand {
    pub fn to_sps(path: &str) -> String {
        format!("GET FILE='{}'.", path)
    }
}

impl EngineCommand for ImportSpssCommand {
    fn id(&self) -> &'static str {
        "data.import_spss"
    }

    fn menu_path(&self) -> &'static str {
        "File > Import Data > SPSS Data..."
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
                    "description": "Path to SPSS .sav or .zsav file"
                },
                "dataset_name": {
                    "type": "string",
                    "description": "Optional name of the dataset",
                    "default": "DataSet1"
                }
            },
            "required": ["file_path"]
        })
    }

    fn enabled(&self, _dataset: Option<&DataSet>) -> bool {
        true
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let file_path = params
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'file_path' string parameter".to_string(),
            })?;

        let dataset_name = params
            .get("dataset_name")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                Path::new(file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("DataSet1")
            });

        let ds = DataSet::load_spss_from_path(file_path)?;
        let rows = ds.row_count();
        let cols = ds.col_count();

        *dataset = Some(ds);

        Ok(json!({
            "status": "success",
            "name": dataset_name,
            "rows": rows,
            "cols": cols,
            "syntax": Self::to_sps(file_path)
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        *dataset = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_spss_syntax_generation() {
        assert_eq!(
            ImportSpssCommand::to_sps("data/survey.sav"),
            "GET FILE='data/survey.sav'."
        );
    }

    #[test]
    fn test_import_spss_missing_param_error() {
        let cmd = ImportSpssCommand;
        let mut ds = None;
        assert!(matches!(
            cmd.execute(&mut ds, json!({})),
            Err(CommandError::InvalidParams { .. })
        ));
    }
}
