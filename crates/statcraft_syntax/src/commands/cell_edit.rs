use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::DataSet;
use statcraft_io::RawValue;

pub struct CellEditCommand;

impl EngineCommand for CellEditCommand {
    fn id(&self) -> &'static str {
        "data.cell_edit"
    }

    fn menu_path(&self) -> &'static str {
        ""
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "row": { "type": "integer", "minimum": 0 },
                "col": { "type": "integer", "minimum": 0 },
                "value": { "type": "string" }
            },
            "required": ["row", "col", "value"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset
            .map(|d| d.row_count() > 0 && d.col_count() > 0)
            .unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let row = params.get("row").and_then(|v| v.as_u64()).ok_or_else(|| {
            CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'row'".to_string(),
            }
        })? as usize;

        let col = params.get("col").and_then(|v| v.as_u64()).ok_or_else(|| {
            CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'col'".to_string(),
            }
        })? as usize;

        let val_str = params
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'value'".to_string(),
            })?;

        let raw_val = if val_str.trim().is_empty() || val_str == "." {
            RawValue::Null
        } else if let Ok(n) = val_str.parse::<f64>() {
            RawValue::Number(n)
        } else {
            RawValue::Text(val_str.to_string())
        };

        ds.set_cell(row, col, raw_val)?;

        Ok(json!({
            "status": "success",
            "row": row,
            "col": col
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        if let Some(ds) = dataset.as_mut() {
            ds.undo();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_io::{RawColumn, RawDataType, RawTable};

    fn make_test_ds() -> Option<DataSet> {
        let col = RawColumn {
            name: "score".to_string(),
            data_type: RawDataType::Numeric,
            values: vec![RawValue::Number(10.0)],
        };
        let table = RawTable {
            columns: vec![col],
            row_count: 1,
        };
        Some(DataSet::from_raw_table(table).unwrap())
    }

    #[test]
    fn test_cell_edit_command_and_undo() {
        let mut ds = make_test_ds();
        let cmd = CellEditCommand;

        let params = json!({
            "row": 0,
            "col": 0,
            "value": "99.5"
        });

        cmd.execute(&mut ds, params).unwrap();
        assert_eq!(
            ds.as_ref().unwrap().get_cell(0, 0),
            Some(&RawValue::Number(99.5))
        );

        cmd.undo(&mut ds).unwrap();
        assert_eq!(
            ds.as_ref().unwrap().get_cell(0, 0),
            Some(&RawValue::Number(10.0))
        );
    }
}
