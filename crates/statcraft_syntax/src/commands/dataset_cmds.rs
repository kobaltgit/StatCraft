use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::{DataSet, DataType, Measure, VariableMetadata};
use statcraft_io::{RawColumn, RawValue};
use std::sync::Mutex;

pub struct VarAddCommand {
    history: Mutex<Vec<usize>>,
}

impl Default for VarAddCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl VarAddCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

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

        let col_idx = if let Some(req_idx) = params
            .get("col_index")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
        {
            ds.insert_variable(
                req_idx,
                name,
                DataType::Numeric { decimals: 2 },
                Measure::Scale,
            )?
        } else {
            ds.add_variable(name, DataType::Numeric { decimals: 2 }, Measure::Scale)?
        };

        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        hist.push(col_idx);

        Ok(json!({
            "status": "success",
            "col_index": col_idx,
            "name": name
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if let Some(col_idx) = hist.pop() {
            if let Some(ds) = dataset.as_mut() {
                ds.remove_variable(col_idx)?;
            }
        }
        Ok(())
    }
}

pub struct VarDeleteCommand {
    history: Mutex<Vec<(usize, VariableMetadata, RawColumn)>>,
}

impl Default for VarDeleteCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl VarDeleteCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

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

        let meta =
            ds.dictionary
                .get(col_index)
                .cloned()
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Column index {} out of bounds", col_index),
                })?;
        let col = ds.table.columns.get(col_index).cloned().ok_or_else(|| {
            CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Column index {} out of bounds in table", col_index),
            }
        })?;

        ds.remove_variable(col_index)?;

        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        hist.push((col_index, meta, col));

        Ok(json!({
            "status": "success",
            "col_index": col_index
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if let Some((col_index, meta, col)) = hist.pop() {
            if let Some(ds) = dataset.as_mut() {
                ds.restore_variable(col_index, meta, col)?;
            }
        }
        Ok(())
    }
}

pub struct DataAddRowCommand {
    history: Mutex<Vec<usize>>,
}

impl Default for DataAddRowCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl DataAddRowCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

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

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let row_idx = if let Some(req_idx) = params
            .get("row_index")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
        {
            let clamped = req_idx.min(ds.row_count());
            let null_row = vec![RawValue::Null; ds.col_count()];
            ds.insert_row(clamped, &null_row);
            clamped
        } else {
            ds.add_empty_row();
            ds.row_count().saturating_sub(1)
        };

        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        hist.push(row_idx);

        Ok(json!({
            "status": "success",
            "row_index": row_idx,
            "row_count": ds.row_count()
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if let Some(row_idx) = hist.pop() {
            if let Some(ds) = dataset.as_mut() {
                ds.remove_row(row_idx)?;
            }
        }
        Ok(())
    }
}

pub struct DataDeleteRowCommand {
    history: Mutex<Vec<(usize, Vec<RawValue>)>>,
}

impl Default for DataDeleteRowCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl DataDeleteRowCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

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

        if row_index >= ds.row_count() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Row index {} out of bounds", row_index),
            });
        }

        let row_values: Vec<RawValue> = ds
            .table
            .columns
            .iter()
            .map(|c| c.values.get(row_index).cloned().unwrap_or(RawValue::Null))
            .collect();

        ds.remove_row(row_index)?;

        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        hist.push((row_index, row_values));

        Ok(json!({
            "status": "success",
            "row_index": row_index,
            "row_count": ds.row_count()
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if let Some((row_index, row_values)) = hist.pop() {
            if let Some(ds) = dataset.as_mut() {
                ds.insert_row(row_index, &row_values);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_add_and_delete_command() {
        let mut ds = Some(DataSet::blank(5, 1).unwrap());
        let initial_ds = ds.clone().unwrap();
        let add_cmd = VarAddCommand::default();
        let res = add_cmd
            .execute(&mut ds, json!({ "name": "salary" }))
            .unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(ds.as_ref().unwrap().col_count(), 2);

        // Test undo of add
        add_cmd.undo(&mut ds).unwrap();
        assert_eq!(ds.as_ref().unwrap(), &initial_ds);

        // Re-execute
        add_cmd
            .execute(&mut ds, json!({ "name": "salary" }))
            .unwrap();

        let del_cmd = VarDeleteCommand::default();
        let before_del = ds.clone().unwrap();
        del_cmd.execute(&mut ds, json!({ "col_index": 0 })).unwrap();
        assert_eq!(ds.as_ref().unwrap().col_count(), 1);
        assert_eq!(
            ds.as_ref().unwrap().dictionary.get(0).unwrap().name,
            "salary"
        );

        // Test undo of delete
        del_cmd.undo(&mut ds).unwrap();
        assert_eq!(ds.as_ref().unwrap(), &before_del);
    }

    #[test]
    fn test_data_add_and_delete_row_command() {
        let mut ds = Some(DataSet::blank(2, 1).unwrap());
        let initial_ds = ds.clone().unwrap();
        let add_row_cmd = DataAddRowCommand::default();
        add_row_cmd.execute(&mut ds, json!({})).unwrap();
        assert_eq!(ds.as_ref().unwrap().row_count(), 3);

        // Test undo of add_row
        add_row_cmd.undo(&mut ds).unwrap();
        assert_eq!(ds.as_ref().unwrap(), &initial_ds);

        let del_row_cmd = DataDeleteRowCommand::default();
        let before_del = ds.clone().unwrap();
        del_row_cmd
            .execute(&mut ds, json!({ "row_index": 0 }))
            .unwrap();
        assert_eq!(ds.as_ref().unwrap().row_count(), 1);

        // Test undo of delete_row
        del_row_cmd.undo(&mut ds).unwrap();
        assert_eq!(ds.as_ref().unwrap(), &before_del);
    }
}
