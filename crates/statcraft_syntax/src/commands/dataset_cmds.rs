use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::{DataSet, DataType, Measure};

pub struct VarAddCommand;

impl EngineCommand for VarAddCommand {
    fn id(&self) -> &'static str {
        "var.add"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Insert Variable"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "type": { "type": "string", "default": "numeric" },
                "measure": { "type": "string", "default": "scale" }
            },
            "required": ["name"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.is_some()
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'name'".to_string(),
            }
        })?;

        let col_idx = ds.add_variable(name, DataType::Numeric { decimals: 2 }, Measure::Scale)?;

        Ok(json!({
            "status": "success",
            "col_index": col_idx,
            "name": name
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

pub struct VarDeleteCommand;

impl EngineCommand for VarDeleteCommand {
    fn id(&self) -> &'static str {
        "var.delete"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Delete Variable"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 }
            },
            "required": ["col_index"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|d| d.col_count() > 0).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let col_index = params
            .get("col_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'col_index'".to_string(),
            })? as usize;

        ds.remove_variable(col_index)?;

        Ok(json!({
            "status": "success",
            "col_index": col_index
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

pub struct DataAddRowCommand;

impl EngineCommand for DataAddRowCommand {
    fn id(&self) -> &'static str {
        "data.add_row"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Insert Case"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({ "type": "object" })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.is_some()
    }

    fn execute(
        &self,
        dataset: &mut Option<DataSet>,
        _params: Value,
    ) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        ds.add_empty_row();
        let new_row_count = ds.row_count();

        Ok(json!({
            "status": "success",
            "row_count": new_row_count
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

pub struct DataDeleteRowCommand;

impl EngineCommand for DataDeleteRowCommand {
    fn id(&self) -> &'static str {
        "data.delete_row"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Delete Case"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "row_index": { "type": "integer", "minimum": 0 }
            },
            "required": ["row_index"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|d| d.row_count() > 0).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let row_index = params
            .get("row_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'row_index'".to_string(),
            })? as usize;

        ds.remove_row(row_index)?;

        Ok(json!({
            "status": "success",
            "row_index": row_index,
            "row_count": ds.row_count()
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_add_and_delete_command() {
        let mut ds = Some(DataSet::blank(5, 1).unwrap());
        let add_cmd = VarAddCommand;
        let res = add_cmd
            .execute(&mut ds, json!({ "name": "salary" }))
            .unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(ds.as_ref().unwrap().col_count(), 2);

        let del_cmd = VarDeleteCommand;
        del_cmd.execute(&mut ds, json!({ "col_index": 0 })).unwrap();
        assert_eq!(ds.as_ref().unwrap().col_count(), 1);
        assert_eq!(
            ds.as_ref().unwrap().dictionary.get(0).unwrap().name,
            "salary"
        );
    }

    #[test]
    fn test_data_add_and_delete_row_command() {
        let mut ds = Some(DataSet::blank(2, 1).unwrap());
        let add_row_cmd = DataAddRowCommand;
        add_row_cmd.execute(&mut ds, json!({})).unwrap();
        assert_eq!(ds.as_ref().unwrap().row_count(), 3);

        let del_row_cmd = DataDeleteRowCommand;
        del_row_cmd
            .execute(&mut ds, json!({ "row_index": 0 }))
            .unwrap();
        assert_eq!(ds.as_ref().unwrap().row_count(), 2);
    }
}
