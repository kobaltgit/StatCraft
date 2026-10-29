use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum EvalError {
    #[error("Empty expression")]
    EmptyExpression,

    #[error("Syntax error: {0}")]
    SyntaxError(String),

    #[error("Mismatched parentheses")]
    MismatchedParentheses,

    #[error("Unknown function: '{0}'")]
    UnknownFunction(String),

    #[error("Invalid argument count for '{name}': expected {expected}, found {got}")]
    InvalidArgCount {
        name: String,
        expected: usize,
        got: usize,
    },

    #[error("Unexpected token: '{0}'")]
    UnexpectedToken(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Ident(String),
    BinaryOp(Op),
    UnaryMinus,
    UnaryPlus,
    LParen,
    RParen,
    Comma,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvalNode {
    Number(f64),
    Variable(String),
    UnaryMinus,
    UnaryPlus,
    BinaryOp(Op),
    Function { name: String, arg_count: usize },
}
