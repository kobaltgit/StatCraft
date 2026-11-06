pub mod dispatcher;
pub mod error;
pub mod lexer;
pub mod mappers;
pub mod parser;

pub use dispatcher::SyntaxDispatcher;
pub use error::SyntaxError;
pub use lexer::{RawCommandBlock, SyntaxLexer};
pub use parser::{ParsedCommand, Subcommand, SyntaxParser};
