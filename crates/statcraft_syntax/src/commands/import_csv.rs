use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};

pub struct ImportCsvCommand;

impl EngineCommand for ImportCsvCommand {
    fn id(&self) -> &'static str {
        "data.import_csv"
    }

    fn menu_path(&self) -> &'static str {
        "File > Open > Data (CSV)"
    }

    fn default_shortcut(&self) -> &'static str {
        "Ctrl+O"
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "csv_data": {
                    "type": "string",
                    "description": "Raw CSV or TSV text content"
                },
                "dataset_name": {
                    "type": "string",
                    "description": "Optional name of the dataset",
                    "default": "DataSet1"
                }
            },
            "required": ["csv_data"]
        })
    }

    fn enabled(&self, _dataset: Option<&DataSet>) -> bool {
        true
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let csv_data = params
            .get("csv_data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'csv_data' string field".to_string(),
            })?;

        let dataset_name = params
            .get("dataset_name")
            .and_then(|v| v.as_str())
            .unwrap_or("DataSet1");

        let raw_table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default())?;
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
    fn test_import_csv_command_execution() {
        let cmd = ImportCsvCommand;
        let mut ds = None;
        let params = json!({
            "csv_data": "id,score\n1,85\n2,92\n",
            "dataset_name": "TestScores"
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(res["rows"], 2);
        assert_eq!(res["cols"], 2);

        let active_ds = ds.as_ref().unwrap();
        assert_eq!(active_ds.row_count(), 2);
        assert_eq!(active_ds.col_count(), 2);

        // Test undo
        cmd.undo(&mut ds).unwrap();
        assert!(ds.is_none());
    }

    #[test]
    fn test_import_csv_missing_params() {
        let cmd = ImportCsvCommand;
        let mut ds = None;
        let bad_params = json!({});
        assert!(matches!(
            cmd.execute(&mut ds, bad_params),
            Err(CommandError::InvalidParams { .. })
        ));
    }
}
