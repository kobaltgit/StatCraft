use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::stc_codec::{load_stc_from_path, save_csv_to_path, save_stc_to_path};
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_path, CsvOptions};
use std::path::Path;

pub struct SaveFileCommand;

impl SaveFileCommand {
    pub fn to_sps(path: &str) -> String {
        format!("SAVE OUTFILE='{}'.\nEXECUTE.", path)
    }
}

impl EngineCommand for SaveFileCommand {
    fn id(&self) -> &'static str {
        "data.save_file"
    }

    fn menu_path(&self) -> &'static str {
        "File > Save"
    }

    fn default_shortcut(&self) -> &'static str {
        "Ctrl+S"
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Destination file path"
                },
                "format": {
                    "type": "string",
                    "enum": ["stc", "csv"],
                    "description": "Target format (stc or csv)"
                }
            },
            "required": ["path"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.is_some()
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled("No dataset to save".to_string()))?;

        let path_str = params.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'path' parameter".to_string(),
            }
        })?;

        let path = Path::new(path_str);
        let format = params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                if path_str.to_lowercase().ends_with(".csv") {
                    "csv"
                } else {
                    "stc"
                }
            });

        if format.eq_ignore_ascii_case("csv") {
            save_csv_to_path(path, ds)?;
        } else {
            save_stc_to_path(path, ds)?;
        }

        Ok(json!({
            "status": "success",
            "path": path_str,
            "format": format
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

pub struct OpenFileCommand;

impl OpenFileCommand {
    pub fn to_sps(path: &str) -> String {
        format!("GET FILE='{}'.\nEXECUTE.", path)
    }
}

impl EngineCommand for OpenFileCommand {
    fn id(&self) -> &'static str {
        "data.open_file"
    }

    fn menu_path(&self) -> &'static str {
        "File > Open"
    }

    fn default_shortcut(&self) -> &'static str {
        "Ctrl+O"
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Source file path (.stc, .csv, .tsv, .txt)"
                }
            },
            "required": ["path"]
        })
    }

    fn enabled(&self, _dataset: Option<&DataSet>) -> bool {
        true
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let path_str = params.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'path' parameter".to_string(),
            }
        })?;

        let path = Path::new(path_str);
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let loaded_ds = if ext == "stc" {
            load_stc_from_path(path)?
        } else if ext == "sav" || ext == "zsav" {
            DataSet::load_spss_from_path(path)?
        } else {
            let raw_table = load_csv_from_path(path, CsvOptions::default())?;
            DataSet::from_raw_table(raw_table)?
        };

        let rows = loaded_ds.row_count();
        let cols = loaded_ds.col_count();
        *dataset = Some(loaded_ds);

        Ok(json!({
            "status": "success",
            "path": path_str,
            "rows": rows,
            "cols": cols
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
    fn test_save_and_open_file_sps_syntax() {
        assert_eq!(
            SaveFileCommand::to_sps("data/my_file.stc"),
            "SAVE OUTFILE='data/my_file.stc'.\nEXECUTE."
        );
        assert_eq!(
            OpenFileCommand::to_sps("data/my_file.stc"),
            "GET FILE='data/my_file.stc'.\nEXECUTE."
        );
    }

    #[test]
    fn test_save_and_open_stc_command_execution() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("statcraft_test_save_cmd.stc");
        let path_str = test_path.to_string_lossy().to_string();

        let mut ds = Some(DataSet::blank(5, 2).unwrap());

        let save_cmd = SaveFileCommand;
        let res = save_cmd
            .execute(&mut ds, json!({ "path": path_str }))
            .unwrap();
        assert_eq!(res["status"], "success");
        assert!(test_path.exists());

        // Now clear ds and test open
        ds = None;
        let open_cmd = OpenFileCommand;
        let res_open = open_cmd
            .execute(&mut ds, json!({ "path": path_str }))
            .unwrap();
        assert_eq!(res_open["status"], "success");
        assert_eq!(res_open["rows"], 5);
        assert_eq!(res_open["cols"], 2);

        assert!(ds.is_some());
        let active_ds = ds.as_ref().unwrap();
        assert_eq!(active_ds.row_count(), 5);
        assert_eq!(active_ds.col_count(), 2);

        let _ = std::fs::remove_file(test_path);
    }
}
