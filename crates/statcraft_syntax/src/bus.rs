use crate::command::{CommandError, EngineCommand};
use crate::commands::{
    CellEditCommand, ComputeVariableCommand, CorrelationsCommand, CrosstabsCommand,
    DataAddRowCommand, DataDeleteRowCommand, DescriptivesCommand, FrequenciesCommand,
    ImportCsvCommand, IndependentSamplesTTestCommand, KruskalWallisCommand, MannWhitneyCommand,
    OneSampleTTestCommand, OneWayAnovaCommand, PairedSamplesTTestCommand,
    RelatedSamplesTestCommand, SetLabelCommand, SetMeasureCommand, VarAddCommand, VarDeleteCommand,
    VarRenameCommand, VarSetAlignCommand, VarSetDecimalsCommand, VarSetMissingValuesCommand,
    VarSetTypeCommand, VarSetValueLabelsCommand, VarSetWidthCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use statcraft_core::DataSet;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryEntry {
    pub command_id: String,
    pub params: Value,
    pub result: Value,
}

pub struct CommandBus {
    commands: HashMap<&'static str, Box<dyn EngineCommand>>,
    history: Vec<CommandHistoryEntry>,
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBus {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            history: vec![],
        }
    }

    pub fn with_defaults() -> Self {
        let mut bus = Self::new();
        bus.register(Box::new(ImportCsvCommand));
        bus.register(Box::new(SetLabelCommand));
        bus.register(Box::new(SetMeasureCommand));
        bus.register(Box::new(CellEditCommand));
        bus.register(Box::new(DescriptivesCommand));
        bus.register(Box::new(FrequenciesCommand));
        bus.register(Box::new(OneSampleTTestCommand));
        bus.register(Box::new(IndependentSamplesTTestCommand));
        bus.register(Box::new(PairedSamplesTTestCommand));
        bus.register(Box::new(OneWayAnovaCommand));
        bus.register(Box::new(CrosstabsCommand));
        bus.register(Box::new(CorrelationsCommand));
        bus.register(Box::new(MannWhitneyCommand));
        bus.register(Box::new(KruskalWallisCommand));
        bus.register(Box::new(RelatedSamplesTestCommand));
        bus.register(Box::new(VarSetValueLabelsCommand::new()));
        bus.register(Box::new(VarSetMissingValuesCommand::new()));
        bus.register(Box::new(VarRenameCommand));
        bus.register(Box::new(VarSetTypeCommand));
        bus.register(Box::new(VarSetDecimalsCommand));
        bus.register(Box::new(VarSetWidthCommand));
        bus.register(Box::new(VarSetAlignCommand));
        bus.register(Box::new(VarAddCommand));
        bus.register(Box::new(VarDeleteCommand));
        bus.register(Box::new(DataAddRowCommand));
        bus.register(Box::new(DataDeleteRowCommand));
        bus.register(Box::new(ComputeVariableCommand::new()));
        bus
    }

    pub fn register(&mut self, cmd: Box<dyn EngineCommand>) {
        self.commands.insert(cmd.id(), cmd);
    }

    pub fn get(&self, id: &str) -> Option<&dyn EngineCommand> {
        self.commands.get(id).map(|b| b.as_ref())
    }

    pub fn is_enabled(&self, id: &str, dataset: Option<&DataSet>) -> bool {
        self.commands
            .get(id)
            .map(|cmd| cmd.enabled(dataset))
            .unwrap_or(false)
    }

    pub fn execute(
        &mut self,
        id: &str,
        dataset: &mut Option<DataSet>,
        params: Value,
    ) -> Result<Value, CommandError> {
        let cmd = self
            .commands
            .get(id)
            .ok_or_else(|| CommandError::ExecutionFailed(format!("Unknown command: '{}'", id)))?;

        if !cmd.enabled(dataset.as_ref()) {
            return Err(CommandError::Disabled(id.to_string()));
        }

        let result = cmd.execute(dataset, params.clone())?;

        self.history.push(CommandHistoryEntry {
            command_id: id.to_string(),
            params,
            result: result.clone(),
        });

        Ok(result)
    }

    pub fn undo(&mut self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        if let Some(entry) = self.history.pop() {
            if let Some(cmd) = self.commands.get(entry.command_id.as_str()) {
                cmd.undo(dataset)?;
            }
        }
        Ok(())
    }

    pub fn list_specs(&self) -> Vec<Value> {
        let mut specs = vec![];
        for (id, cmd) in &self.commands {
            specs.push(json!({
                "id": id,
                "menu_path": cmd.menu_path(),
                "shortcut": cmd.default_shortcut(),
                "schema": cmd.specs()
            }));
        }
        specs.sort_by(|a, b| {
            a["id"]
                .as_str()
                .unwrap_or("")
                .cmp(b["id"].as_str().unwrap_or(""))
        });
        specs
    }

    pub fn history(&self) -> &[CommandHistoryEntry] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_bus_execute_and_undo() {
        let mut bus = CommandBus::with_defaults();
        let mut ds = None;

        let import_params = json!({
            "csv_data": "col1\n100\n",
            "dataset_name": "Test"
        });

        let res = bus
            .execute("data.import_csv", &mut ds, import_params)
            .unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(bus.history().len(), 1);

        // Undo
        bus.undo(&mut ds).unwrap();
        assert!(ds.is_none());
    }

    #[test]
    fn test_command_specs_json_validity() {
        let bus = CommandBus::with_defaults();
        let specs = bus.list_specs();
        assert!(specs.len() >= 4);

        let csv_spec = specs.iter().find(|s| s["id"] == "data.import_csv").unwrap();
        assert!(csv_spec["schema"]["properties"]["csv_data"].is_object());
    }

    #[test]
    fn test_command_enabled_predicate() {
        let bus = CommandBus::with_defaults();
        let ds = None;

        // var.set_label should be disabled when ds is None
        assert!(!bus.is_enabled("var.set_label", ds.as_ref()));
        // data.import_csv is enabled without dataset
        assert!(bus.is_enabled("data.import_csv", ds.as_ref()));
    }
}
