use crate::command::{CommandError, EngineCommand};
use crate::syntax_parser::mappers::help::generate_help;
use serde_json::{json, Value};
use statcraft_core::DataSet;

pub struct HelpCommand;

impl EngineCommand for HelpCommand {
    fn id(&self) -> &'static str {
        "util.help"
    }

    fn menu_path(&self) -> &'static str {
        "Help > SPSS Syntax Reference"
    }

    fn default_shortcut(&self) -> &'static str {
        "F1"
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "topic": {
                    "type": "string",
                    "description": "Optional command or procedure name (e.g. ONEWAY, T-TEST, DESCRIPTIVES)"
                }
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
        let topic = params.get("topic").and_then(|v| v.as_str());
        let help_text = if let Some(out) = params.get("output").and_then(|v| v.as_str()) {
            out.to_string()
        } else {
            generate_help(topic)
        };

        Ok(json!({
            "status": "ok",
            "topic": topic,
            "output": help_text
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
