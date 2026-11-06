use crate::bus::CommandBus;
use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::lexer::SyntaxLexer;
use crate::syntax_parser::mappers::{map_data_command, map_stat_command};
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;

pub struct SyntaxDispatcher;

impl SyntaxDispatcher {
    /// Maps and dispatches a single parsed command to the CommandBus.
    pub fn dispatch(
        cmd: &ParsedCommand,
        dataset: &mut Option<DataSet>,
        bus: &mut CommandBus,
    ) -> Result<Value, SyntaxError> {
        let mapped = match map_stat_command(cmd, dataset.as_ref())? {
            Some(pair) => Some(pair),
            None => map_data_command(cmd, dataset.as_ref())?,
        };

        let (cmd_id, params) = match mapped {
            Some(pair) => pair,
            None => {
                return Err(SyntaxError::UnknownCommand {
                    line: cmd.line_num,
                    command: cmd.name.clone(),
                });
            }
        };

        // Handle no-ops like comments and EXECUTE
        if cmd_id == "data.execute_noop" || cmd_id == "util.comment" {
            return Ok(json!({ "status": "ok", "command_id": cmd_id }));
        }

        // Execute via CommandBus
        bus.execute(cmd_id, dataset, params)
            .map_err(|err| SyntaxError::ExecutionFailed {
                line: cmd.line_num,
                command: cmd.name.clone(),
                reason: err.to_string(),
            })
    }

    /// Executes a full SPSS syntax batch script against the CommandBus.
    /// Iterates sequentially through command blocks, returning all executed commands
    /// and their JSON results, or the first SyntaxError if validation or execution fails.
    pub fn execute_batch(
        sps_script: &str,
        dataset: &mut Option<DataSet>,
        bus: &mut CommandBus,
    ) -> Result<Vec<(ParsedCommand, Value)>, SyntaxError> {
        let blocks = SyntaxLexer::split_commands(sps_script)?;
        let mut executed = Vec::with_capacity(blocks.len());

        for block in &blocks {
            if block.is_comment {
                let parsed = ParsedCommand {
                    name: "COMMENT".to_string(),
                    subcommands: vec![],
                    line_num: block.line_num,
                    raw_text: block.raw_text.clone(),
                };
                executed.push((parsed, json!({ "status": "comment" })));
                continue;
            }

            let parsed = SyntaxParser::parse_command(block)?;
            let result = Self::dispatch(&parsed, dataset, bus)?;
            executed.push((parsed, result));
        }

        Ok(executed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_algo::DescriptivesResult;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv = "age,salary\n25,50000\n30,60000\n35,70000\n40,80000\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_dispatch_descriptives_command() {
        let mut bus = CommandBus::with_defaults();
        let mut ds = Some(make_test_dataset());

        let sps = "DESCRIPTIVES VARIABLES=salary age\n  /STATISTICS=MEAN STDDEV MIN MAX.";
        let results = SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).unwrap();

        assert_eq!(results.len(), 1);
        let (cmd, val) = &results[0];
        assert_eq!(cmd.name, "DESCRIPTIVES");

        let desc: Vec<DescriptivesResult> = serde_json::from_value(val.clone()).unwrap();
        assert_eq!(desc.len(), 2);
        assert_eq!(desc[0].variable, "salary");
        assert_eq!(desc[0].mean, Some(65000.0));
        assert_eq!(desc[1].variable, "age");
        assert_eq!(desc[1].mean, Some(32.5));
    }

    #[test]
    fn test_dispatch_batch_with_sort_and_freq() {
        let mut bus = CommandBus::with_defaults();
        let mut ds = Some(make_test_dataset());

        let sps = r#"
        * Perform data sort.
        SORT CASES BY age (D).
        FREQUENCIES VARIABLES=salary.
        "#;

        let results = SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0.name, "COMMENT");
        assert_eq!(results[1].0.name, "SORT CASES");
        assert_eq!(results[2].0.name, "FREQUENCIES");
    }

    #[test]
    fn test_unknown_command_returns_typed_error() {
        let mut bus = CommandBus::with_defaults();
        let mut ds = Some(make_test_dataset());

        let sps = "UNSUPPORTED_FOOBAR_CMD VAR=salary.";
        let err = SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).unwrap_err();

        match err {
            SyntaxError::UnknownCommand { command, .. } => {
                assert_eq!(command, "UNSUPPORTED_FOOBAR_CMD");
            }
            _ => panic!("Expected UnknownCommand error"),
        }
    }
}
