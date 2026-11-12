use crate::command::{CommandError, EngineCommand};
use crate::commands::{
    AutoRecodeCommand, BatchReplaceCommand, CellEditCommand, ComputeVariableCommand,
    CorrelationsCommand, CountValuesCommand, CreateDummiesCommand, CreateTimeSeriesCommand,
    CrosstabsCommand, DataAddRowCommand, DataDeleteRowCommand, DateTimeWizardCommand,
    DescriptivesCommand, ExploreCommand, FactorCommand, FrequenciesCommand, HelpCommand,
    ImportCsvCommand, ImportExcelCommand, ImportSpssCommand, IndependentSamplesTTestCommand,
    KruskalWallisCommand, LinearRegressionCommand, MannWhitneyCommand, MeansCommand,
    OneSampleTTestCommand, OneWayAnovaCommand, OpenFileCommand, OptimalBinningCommand,
    PairedSamplesTTestCommand, PrepareDataModelingCommand, ProgrammabilityCommand,
    RandomSeedCommand, RankCasesCommand, RecodeIntoDifferentCommand, RecodeIntoSameCommand,
    RelatedSamplesTestCommand, ReliabilityCommand, ReplaceMissingCommand, SaveFileCommand,
    SelectCasesCommand, SetLabelCommand, SetMeasureCommand, ShiftValuesCommand, SortCasesCommand,
    SplitFileCommand, VarAddCommand, VarDeleteCommand, VarRenameCommand, VarSetAlignCommand,
    VarSetDecimalsCommand, VarSetMissingValuesCommand, VarSetTypeCommand, VarSetValueLabelsCommand,
    VarSetWidthCommand, VisualBinningCommand, WeightCasesCommand,
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
    redo_stack: Vec<CommandHistoryEntry>,
    max_history_depth: usize,
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
            redo_stack: vec![],
            max_history_depth: 100,
        }
    }

    pub fn with_defaults() -> Self {
        let mut bus = Self::new();
        bus.register(Box::new(ImportCsvCommand));
        bus.register(Box::new(ImportExcelCommand));
        bus.register(Box::new(ImportSpssCommand));
        bus.register(Box::new(SetLabelCommand::default()));
        bus.register(Box::new(SetMeasureCommand::default()));
        bus.register(Box::new(CellEditCommand));
        bus.register(Box::new(BatchReplaceCommand::default()));
        bus.register(Box::new(DescriptivesCommand));
        bus.register(Box::new(ExploreCommand));
        bus.register(Box::new(FrequenciesCommand));
        bus.register(Box::new(OneSampleTTestCommand));
        bus.register(Box::new(IndependentSamplesTTestCommand));
        bus.register(Box::new(PairedSamplesTTestCommand));
        bus.register(Box::new(OneWayAnovaCommand));
        bus.register(Box::new(MeansCommand));
        bus.register(Box::new(CrosstabsCommand));
        bus.register(Box::new(CorrelationsCommand));
        bus.register(Box::new(LinearRegressionCommand));
        bus.register(Box::new(ReliabilityCommand));
        bus.register(Box::new(FactorCommand));
        bus.register(Box::new(MannWhitneyCommand));
        bus.register(Box::new(KruskalWallisCommand));
        bus.register(Box::new(RelatedSamplesTestCommand));
        bus.register(Box::new(VarSetValueLabelsCommand::new()));
        bus.register(Box::new(VarSetMissingValuesCommand::new()));
        bus.register(Box::new(VarRenameCommand::default()));
        bus.register(Box::new(VarSetTypeCommand::default()));
        bus.register(Box::new(VarSetDecimalsCommand::default()));
        bus.register(Box::new(VarSetWidthCommand::default()));
        bus.register(Box::new(VarSetAlignCommand::default()));
        bus.register(Box::new(VarAddCommand::default()));
        bus.register(Box::new(VarDeleteCommand::default()));
        bus.register(Box::new(DataAddRowCommand::default()));
        bus.register(Box::new(DataDeleteRowCommand::default()));
        bus.register(Box::new(ComputeVariableCommand::new()));
        bus.register(Box::new(CountValuesCommand::new()));
        bus.register(Box::new(CreateDummiesCommand::new()));
        bus.register(Box::new(CreateTimeSeriesCommand::new()));
        bus.register(Box::new(DateTimeWizardCommand::new()));
        bus.register(Box::new(ShiftValuesCommand::new()));
        bus.register(Box::new(RankCasesCommand::new()));
        bus.register(Box::new(ReplaceMissingCommand::new()));
        bus.register(Box::new(RecodeIntoDifferentCommand::new()));
        bus.register(Box::new(RecodeIntoSameCommand::new()));
        bus.register(Box::new(AutoRecodeCommand::new()));
        bus.register(Box::new(VisualBinningCommand::new()));
        bus.register(Box::new(OptimalBinningCommand::new()));
        bus.register(Box::new(PrepareDataModelingCommand::new()));
        bus.register(Box::new(ProgrammabilityCommand::new()));
        bus.register(Box::new(RandomSeedCommand::new()));
        bus.register(Box::new(SortCasesCommand::new()));
        bus.register(Box::new(SelectCasesCommand::new()));
        bus.register(Box::new(SplitFileCommand::new()));
        bus.register(Box::new(WeightCasesCommand::new()));
        bus.register(Box::new(SaveFileCommand));
        bus.register(Box::new(OpenFileCommand));
        bus.register(Box::new(HelpCommand));
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

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn redo_stack(&self) -> &[CommandHistoryEntry] {
        &self.redo_stack
    }

    pub fn max_history_depth(&self) -> usize {
        self.max_history_depth
    }

    pub fn set_max_history_depth(&mut self, depth: usize) {
        self.max_history_depth = depth;
        if self.history.len() > depth {
            let overflow = self.history.len() - depth;
            self.history.drain(0..overflow);
        }
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
        self.redo_stack.clear();

        if self.history.len() > self.max_history_depth {
            let overflow = self.history.len() - self.max_history_depth;
            self.history.drain(0..overflow);
        }

        Ok(result)
    }

    pub fn execute_coalesced(
        &mut self,
        id: &str,
        dataset: &mut Option<DataSet>,
        params: Value,
    ) -> Result<Value, CommandError> {
        if id == "data.cell_edit" {
            let row = params.get("row").and_then(|v| v.as_u64());
            let col = params.get("col").and_then(|v| v.as_u64());
            if let Some(last) = self.history.last_mut() {
                if last.command_id == "data.cell_edit"
                    && last.params.get("row").and_then(|v| v.as_u64()) == row
                    && last.params.get("col").and_then(|v| v.as_u64()) == col
                {
                    let cmd = self.commands.get(id).ok_or_else(|| {
                        CommandError::ExecutionFailed(format!("Unknown command: '{}'", id))
                    })?;
                    let result = cmd.execute(dataset, params.clone())?;
                    if let Some(ds) = dataset.as_mut() {
                        if ds.delta.undo_stack.len() >= 2 {
                            let top = ds.delta.undo_stack.pop();
                            if let (Some(t), Some(prev)) = (top, ds.delta.undo_stack.last_mut()) {
                                if prev.coord == t.coord {
                                    prev.new_val = t.new_val;
                                } else {
                                    ds.delta.undo_stack.push(t);
                                }
                            }
                        }
                    }
                    last.params = params;
                    last.result = result.clone();
                    self.redo_stack.clear();
                    return Ok(result);
                }
            }
        }
        self.execute(id, dataset, params)
    }

    pub fn undo(&mut self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        if let Some(entry) = self.history.pop() {
            if let Some(cmd) = self.commands.get(entry.command_id.as_str()) {
                cmd.undo(dataset)?;
                self.redo_stack.push(entry);
            }
        }
        Ok(())
    }

    pub fn redo(&mut self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        if let Some(entry) = self.redo_stack.pop() {
            if let Some(cmd) = self.commands.get(entry.command_id.as_str()) {
                let result = cmd.execute(dataset, entry.params.clone())?;
                self.history.push(CommandHistoryEntry {
                    command_id: entry.command_id,
                    params: entry.params,
                    result,
                });
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
                "category": cmd.category(),
                "syntax_names": cmd.syntax_names(),
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

    pub fn find_by_syntax_keyword(&self, keyword: &str) -> Option<&dyn EngineCommand> {
        let kw_upper = keyword.trim().to_ascii_uppercase();
        for cmd in self.commands.values() {
            for name in cmd.syntax_names() {
                if name.eq_ignore_ascii_case(&kw_upper) {
                    return Some(cmd.as_ref());
                }
            }
        }
        None
    }

    pub fn all_syntax_keywords(&self) -> Vec<(&'static str, &'static str)> {
        let mut list = Vec::new();
        for (id, cmd) in &self.commands {
            for name in cmd.syntax_names() {
                list.push((*name, *id));
            }
        }
        list
    }

    pub fn full_catalog_metadata(&self) -> Value {
        json!({
            "version": env!("CARGO_PKG_VERSION"),
            "total_commands": self.commands.len(),
            "commands": self.list_specs()
        })
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
        assert!(!bus.is_enabled("transform.shift_values", ds.as_ref()));
    }

    #[test]
    fn test_shift_values_in_bus() {
        let bus = CommandBus::with_defaults();
        assert!(bus.get("transform.shift_values").is_some());
    }

    #[test]
    fn test_count_values_in_bus() {
        let bus = CommandBus::with_defaults();
        assert!(bus.get("transform.count_values").is_some());
    }
}
