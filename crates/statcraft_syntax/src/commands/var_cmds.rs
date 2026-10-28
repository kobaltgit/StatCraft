use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::{DataSet, DataType};

pub use super::var_format_cmds::{SetLabelCommand, SetMeasureCommand, VarSetAlignCommand};

pub struct VarRenameCommand;

impl EngineCommand for VarRenameCommand {
    fn id(&self) -> &'static str {
        "var.rename"
    }

    fn menu_path(&self) -> &'static str {
        "Variable > Rename"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 },
                "new_name": { "type": "string" }
            },
            "required": ["col_index", "new_name"]
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

        let new_name = params
            .get("new_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'new_name'".to_string(),
            })?;

        ds.rename_variable(col_index, new_name)?;

        Ok(json!({
            "status": "success",
            "col_index": col_index,
            "new_name": new_name
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

pub struct VarSetTypeCommand;

impl EngineCommand for VarSetTypeCommand {
    fn id(&self) -> &'static str {
        "var.set_type"
    }

    fn menu_path(&self) -> &'static str {
        "Variable > Set Type"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 },
                "type": { "type": "string" },
                "width": { "type": "integer", "default": 8 },
                "decimals": { "type": "integer", "default": 2 }
            },
            "required": ["col_index", "type"]
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

        let type_str = params.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
            CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'type'".to_string(),
            }
        })?;

        let width = params.get("width").and_then(|v| v.as_u64()).unwrap_or(8) as usize;
        let decimals = params.get("decimals").and_then(|v| v.as_u64()).unwrap_or(2) as u8;

        let var_type = match type_str.to_lowercase().as_str() {
            "numeric" => DataType::Numeric { decimals },
            "comma" => DataType::Comma { decimals },
            "dot" => DataType::Dot { decimals },
            "scientific" => DataType::Scientific { decimals },
            "date" => DataType::Date {
                format: "dd-mmm-yyyy".to_string(),
            },
            "dollar" => DataType::Dollar { decimals },
            "customcurrency" | "custom_currency" => DataType::CustomCurrency {
                format_code: "CCA".to_string(),
                decimals,
            },
            "string" => DataType::String { max_len: width },
            "restrictednumeric" | "restricted_numeric" => DataType::RestrictedNumeric { width },
            _ => {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Unknown variable type: '{}'", type_str),
                });
            }
        };

        ds.set_variable_type(col_index, var_type, width, decimals)?;

        Ok(json!({
            "status": "success",
            "col_index": col_index,
            "type": type_str,
            "width": width,
            "decimals": decimals
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

pub struct VarSetDecimalsCommand;

impl EngineCommand for VarSetDecimalsCommand {
    fn id(&self) -> &'static str {
        "var.set_decimals"
    }

    fn menu_path(&self) -> &'static str {
        "Variable > Set Decimals"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 },
                "decimals": { "type": "integer", "minimum": 0, "maximum": 16 }
            },
            "required": ["col_index", "decimals"]
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

        let decimals = params
            .get("decimals")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'decimals'".to_string(),
            })? as u8;

        ds.set_variable_decimals(col_index, decimals)?;

        Ok(json!({
            "status": "success",
            "col_index": col_index,
            "decimals": decimals
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

pub struct VarSetWidthCommand;

impl EngineCommand for VarSetWidthCommand {
    fn id(&self) -> &'static str {
        "var.set_width"
    }

    fn menu_path(&self) -> &'static str {
        "Variable > Set Width"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 },
                "width": { "type": "integer", "minimum": 1 }
            },
            "required": ["col_index", "width"]
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

        let width = params
            .get("width")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'width'".to_string(),
            })? as usize;

        ds.set_variable_width(col_index, width)?;

        Ok(json!({
            "status": "success",
            "col_index": col_index,
            "width": width
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
