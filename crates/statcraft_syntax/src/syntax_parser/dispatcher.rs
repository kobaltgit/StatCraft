use crate::bus::CommandBus;
use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::lexer::SyntaxLexer;
use crate::syntax_parser::mappers::{
    map_data_command, map_help_command, map_stat_command, map_variable_command,
};
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
        let (cmd_id, params) = if cmd.name == "RUN COMMAND"
            || cmd.name == "CALL"
            || cmd.name == "RUN"
        {
            parse_run_command_directive(cmd)?
        } else {
            let mapped = if let Some(pair) = map_stat_command(cmd, dataset.as_ref())? {
                Some(pair)
            } else if let Some(pair) = map_data_command(cmd, dataset.as_ref())? {
                Some(pair)
            } else if let Some(pair) = map_variable_command(cmd, dataset.as_ref())? {
                Some(pair)
            } else {
                map_help_command(cmd)?
            };

            match mapped {
                Some((id, params)) => (id.to_string(), params),
                None => {
                    // Check if command is registered in bus by syntax keyword
                    if let Some(reg_cmd) = bus.find_by_syntax_keyword(&cmd.name) {
                        (reg_cmd.id().to_string(), json!({}))
                    } else {
                        let suggestion =
                            crate::syntax_parser::mappers::help::find_closest_command(&cmd.name);
                        return Err(SyntaxError::UnknownCommand {
                            line: cmd.line_num,
                            command: cmd.name.clone(),
                            suggestion: suggestion.to_string(),
                        });
                    }
                }
            }
        };

        // Handle no-ops like comments and EXECUTE
        if cmd_id == "data.execute_noop" || cmd_id == "util.comment" {
            return Ok(json!({ "status": "ok", "command_id": cmd_id }));
        }

        // Execute via CommandBus
        bus.execute(&cmd_id, dataset, params)
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

fn parse_run_command_directive(cmd: &ParsedCommand) -> Result<(String, Value), SyntaxError> {
    let raw = cmd.raw_text.trim();
    let stripped = if let Some(rest) = raw.strip_prefix("RUN COMMAND") {
        rest
    } else if let Some(rest) = raw.strip_prefix("run command") {
        rest
    } else if let Some(rest) = raw.strip_prefix("Run Command") {
        rest
    } else if let Some(rest) = raw.strip_prefix("CALL") {
        rest
    } else if let Some(rest) = raw.strip_prefix("call") {
        rest
    } else if let Some(rest) = raw.strip_prefix("RUN") {
        rest
    } else if let Some(rest) = raw.strip_prefix("run") {
        rest
    } else {
        let upper_raw = raw.to_ascii_uppercase();
        let upper_name = cmd.name.to_ascii_uppercase();
        if upper_raw.starts_with(&upper_name) {
            &raw[upper_name.len()..]
        } else {
            raw
        }
    };

    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return Err(SyntaxError::ExecutionFailed {
            line: cmd.line_num,
            command: cmd.name.clone(),
            reason:
                "Expected command ID in quotes (e.g. RUN COMMAND \"stat.descriptives\" { ... })."
                    .to_string(),
        });
    }

    let (command_id, rest) = if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        let quote = trimmed.chars().next().unwrap_or('"');
        let after_first = &trimmed[quote.len_utf8()..];
        if let Some(end_idx) = after_first.find(quote) {
            let id = &after_first[..end_idx];
            let after_quote = &after_first[end_idx + quote.len_utf8()..];
            (id.trim().to_string(), after_quote.trim())
        } else {
            return Err(SyntaxError::ExecutionFailed {
                line: cmd.line_num,
                command: cmd.name.clone(),
                reason: "Unclosed quote in command ID".to_string(),
            });
        }
    } else {
        let end_idx = trimmed
            .find(|c: char| c.is_whitespace() || c == '{' || c == '.')
            .unwrap_or(trimmed.len());
        let id = &trimmed[..end_idx];
        let after_id = &trimmed[end_idx..];
        (id.trim().to_string(), after_id.trim())
    };

    if command_id.is_empty() {
        return Err(SyntaxError::ExecutionFailed {
            line: cmd.line_num,
            command: cmd.name.clone(),
            reason: "Empty command ID in RUN COMMAND directive".to_string(),
        });
    }

    let rest_clean = rest.trim().trim_end_matches('.').trim();
    let params = if rest_clean.is_empty() {
        json!({})
    } else {
        let json_str = if let Some(start_brace) = rest_clean.find('{') {
            &rest_clean[start_brace..]
        } else {
            rest_clean
        };
        serde_json::from_str::<Value>(json_str).map_err(|e| SyntaxError::ExecutionFailed {
            line: cmd.line_num,
            command: cmd.name.clone(),
            reason: format!(
                "Failed to parse JSON parameters for command '{}': {}",
                command_id, e
            ),
        })?
    };

    Ok((command_id, params))
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

    #[test]
    fn test_run_command_directive_execution() {
        let mut bus = CommandBus::with_defaults();
        let mut ds = Some(make_test_dataset());

        let sps = r#"RUN COMMAND "stat.descriptives" { "vars": ["salary"] }."#;
        let results = SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).unwrap();
        assert_eq!(results.len(), 1);
        let (_cmd, val) = &results[0];
        let desc: Vec<DescriptivesResult> = serde_json::from_value(val.clone()).unwrap();
        assert_eq!(desc.len(), 1);
        assert_eq!(desc[0].variable, "salary");
    }
}
