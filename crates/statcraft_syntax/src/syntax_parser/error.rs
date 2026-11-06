use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SyntaxError {
    #[error("Syntax error at line {line}, col {col}: {message}")]
    ParsingFailed {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Unclosed string literal starting at line {line}, col {col}")]
    UnclosedString { line: usize, col: usize },

    #[error("Unknown or unsupported command '{command}' at line {line}")]
    UnknownCommand { line: usize, command: String },

    #[error("Invalid subcommand '{subcommand}' for command '{command}' at line {line}: {reason}")]
    InvalidSubcommand {
        line: usize,
        command: String,
        subcommand: String,
        reason: String,
    },

    #[error("Execution failed for command '{command}' at line {line}: {reason}")]
    ExecutionFailed {
        line: usize,
        command: String,
        reason: String,
    },
}
