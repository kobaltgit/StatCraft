use serde_json::Value;
use statcraft_core::{CoreError, DataSet};
use statcraft_io::IoError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CommandError {
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Command '{0}' is currently disabled")]
    Disabled(String),

    #[error("Invalid parameters for command '{id}': {reason}")]
    InvalidParams { id: &'static str, reason: String },

    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    #[error("I/O error: {0}")]
    Io(String),
}

impl From<IoError> for CommandError {
    fn from(err: IoError) -> Self {
        CommandError::Io(err.to_string())
    }
}

pub trait EngineCommand: Send + Sync {
    fn id(&self) -> &'static str;
    fn menu_path(&self) -> &'static str;
    fn default_shortcut(&self) -> &'static str;
    fn specs(&self) -> Value;
    fn enabled(&self, dataset: Option<&DataSet>) -> bool;
    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError>;
    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError>;
}
