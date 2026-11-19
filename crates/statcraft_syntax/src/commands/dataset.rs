use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::DataSet;

/// SPSS `DATASET NAME <name> [WINDOW=ASIS|FRONT].`
pub struct DatasetNameCommand;

impl EngineCommand for DatasetNameCommand {
    fn id(&self) -> &'static str {
        "dataset.name"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Dataset Name"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "New dataset identifier" },
                "window": { "type": "string", "enum": ["asis", "front"], "default": "asis" }
            },
            "required": ["name"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.is_some()
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let _ = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'name' parameter".to_string(),
            }
        })?;

        let window = params
            .get("window")
            .and_then(|v| v.as_str())
            .unwrap_or("asis");

        Ok(json!({
            "status": "success",
            "action": "name",
            "dataset_name": name,
            "window": window
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["DATASET NAME"]
    }

    fn category(&self) -> &'static str {
        "Data"
    }
}

/// SPSS `DATASET ACTIVATE <name> [WINDOW=ASIS|FRONT].`
pub struct DatasetActivateCommand;

impl EngineCommand for DatasetActivateCommand {
    fn id(&self) -> &'static str {
        "dataset.activate"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Dataset Activate"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Dataset identifier to activate" },
                "window": { "type": "string", "enum": ["asis", "front"], "default": "asis" }
            },
            "required": ["name"]
        })
    }

    fn enabled(&self, _dataset: Option<&DataSet>) -> bool {
        true
    }

    fn execute(
        &self,
        _dataset: &mut Option<DataSet>,
        params: Value,
    ) -> Result<Value, CommandError> {
        let name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'name' parameter".to_string(),
            }
        })?;

        let window = params
            .get("window")
            .and_then(|v| v.as_str())
            .unwrap_or("asis");

        Ok(json!({
            "status": "success",
            "action": "activate",
            "dataset_name": name,
            "window": window
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["DATASET ACTIVATE"]
    }

    fn category(&self) -> &'static str {
        "Data"
    }
}

/// SPSS `DATASET CLOSE <name | *>.`
pub struct DatasetCloseCommand;

impl EngineCommand for DatasetCloseCommand {
    fn id(&self) -> &'static str {
        "dataset.close"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Dataset Close"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "default": "*", "description": "Dataset identifier or '*' for active" }
            }
        })
    }

    fn enabled(&self, _dataset: Option<&DataSet>) -> bool {
        true
    }

    fn execute(
        &self,
        _dataset: &mut Option<DataSet>,
        params: Value,
    ) -> Result<Value, CommandError> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("*");

        Ok(json!({
            "status": "success",
            "action": "close",
            "dataset_name": name
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["DATASET CLOSE"]
    }

    fn category(&self) -> &'static str {
        "Data"
    }
}

/// SPSS `DATASET DISPLAY.`
pub struct DatasetDisplayCommand;

impl EngineCommand for DatasetDisplayCommand {
    fn id(&self) -> &'static str {
        "dataset.display"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Dataset Display"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object"
        })
    }

    fn enabled(&self, _dataset: Option<&DataSet>) -> bool {
        true
    }

    fn execute(
        &self,
        _dataset: &mut Option<DataSet>,
        _params: Value,
    ) -> Result<Value, CommandError> {
        Ok(json!({
            "status": "success",
            "action": "display"
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["DATASET DISPLAY"]
    }

    fn category(&self) -> &'static str {
        "Data"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_commands_execution() {
        let mut ds = Some(DataSet::blank(5, 0).unwrap());

        let name_cmd = DatasetNameCommand;
        assert!(name_cmd.enabled(ds.as_ref()));
        let res_name = name_cmd
            .execute(&mut ds, json!({ "name": "SurveyData" }))
            .unwrap();
        assert_eq!(res_name["action"], "name");
        assert_eq!(res_name["dataset_name"], "SurveyData");

        let act_cmd = DatasetActivateCommand;
        let res_act = act_cmd
            .execute(&mut ds, json!({ "name": "ClientsData" }))
            .unwrap();
        assert_eq!(res_act["action"], "activate");
        assert_eq!(res_act["dataset_name"], "ClientsData");

        let close_cmd = DatasetCloseCommand;
        let res_close = close_cmd.execute(&mut ds, json!({ "name": "*" })).unwrap();
        assert_eq!(res_close["action"], "close");
        assert_eq!(res_close["dataset_name"], "*");

        let disp_cmd = DatasetDisplayCommand;
        let res_disp = disp_cmd.execute(&mut ds, json!({})).unwrap();
        assert_eq!(res_disp["action"], "display");
    }
}
