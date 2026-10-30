use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::DataSet;
use statcraft_io::load_excel_from_path;

pub struct ImportExcelCommand;

impl EngineCommand for ImportExcelCommand {
    fn id(&self) -> &'static str {
        "data.import_excel"
    }

    fn menu_path(&self) -> &'static str {
        "File > Import Data > Excel"
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
                    "description": "Path to Excel (.xlsx, .xls, .ods) file"
                },
                "sheet_name": {
                    "type": "string",
                    "description": "Optional sheet name"
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
                reason: "Missing 'file_path' string field".to_string(),
            })?;

        let sheet_name = params.get("sheet_name").and_then(|v| v.as_str());

        let dataset_name = params
            .get("dataset_name")
            .and_then(|v| v.as_str())
            .unwrap_or("DataSet1");

        let raw_table = load_excel_from_path(file_path, sheet_name)?;
        let rows = raw_table.row_count;
        let cols = raw_table.col_count();

        let ds = DataSet::from_raw_table(raw_table)?;
        *dataset = Some(ds);

        Ok(json!({
            "status": "success",
            "name": dataset_name,
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
    fn test_import_excel_missing_params() {
        let cmd = ImportExcelCommand;
        let mut ds = None;
        let bad_params = json!({});
        assert!(matches!(
            cmd.execute(&mut ds, bad_params),
            Err(CommandError::InvalidParams { .. })
        ));
    }

    #[test]
    fn test_import_excel_nonexistent_file() {
        let cmd = ImportExcelCommand;
        let mut ds = None;
        let params = json!({
            "file_path": "nonexistent_file_xyz_12345.xlsx"
        });
        assert!(matches!(
            cmd.execute(&mut ds, params),
            Err(CommandError::Io(_))
        ));
    }

    #[test]
    fn test_import_excel_undo() {
        let cmd = ImportExcelCommand;
        let mut ds = DataSet::blank(10, 2).ok();
        assert!(ds.is_some());
        cmd.undo(&mut ds).unwrap();
        assert!(ds.is_none());
    }
}
