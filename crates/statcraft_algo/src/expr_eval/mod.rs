pub mod eval;
pub mod lexer;
pub mod parser;
#[cfg(test)]
mod tests;
pub mod types;

use eval::eval_function;
use lexer::tokenize;
use parser::to_rpn;
use std::collections::HashMap;
pub use types::{EvalError, EvalNode, Op, Token};

#[derive(Debug, Clone)]
pub struct CompiledExpr {
    rpn: Vec<EvalNode>,
}

impl CompiledExpr {
    pub fn parse(input: &str) -> Result<Self, EvalError> {
        let tokens = tokenize(input)?;
        if tokens.is_empty() {
            return Err(EvalError::EmptyExpression);
        }
        let rpn = to_rpn(&tokens)?;
        Ok(Self { rpn })
    }

    pub fn eval<F>(&self, var_lookup: &F) -> Result<f64, EvalError>
    where
        F: Fn(&str) -> Option<f64>,
    {
        let mut stack: Vec<f64> = Vec::new();

        for node in &self.rpn {
            match node {
                EvalNode::Number(n) => stack.push(*n),
                EvalNode::Variable(name) => {
                    let val = var_lookup(name).unwrap_or(f64::NAN);
                    stack.push(val);
                }
                EvalNode::UnaryPlus => {
                    let val = stack.pop().ok_or_else(|| {
                        EvalError::SyntaxError("Missing operand for unary +".into())
                    })?;
                    stack.push(val);
                }
                EvalNode::UnaryMinus => {
                    let val = stack.pop().ok_or_else(|| {
                        EvalError::SyntaxError("Missing operand for unary -".into())
                    })?;
                    stack.push(if val.is_nan() { f64::NAN } else { -val });
                }
                EvalNode::BinaryOp(op) => {
                    let b = stack
                        .pop()
                        .ok_or_else(|| EvalError::SyntaxError("Missing second operand".into()))?;
                    let a = stack
                        .pop()
                        .ok_or_else(|| EvalError::SyntaxError("Missing first operand".into()))?;

                    if a.is_nan() || b.is_nan() {
                        stack.push(f64::NAN);
                        continue;
                    }

                    let res = match op {
                        Op::Add => a + b,
                        Op::Sub => a - b,
                        Op::Mul => a * b,
                        Op::Div => {
                            if b == 0.0 {
                                f64::NAN
                            } else {
                                a / b
                            }
                        }
                        Op::Pow => {
                            let p = a.powf(b);
                            if p.is_nan() || p.is_infinite() {
                                f64::NAN
                            } else {
                                p
                            }
                        }
                    };
                    stack.push(if res.is_nan() || res.is_infinite() {
                        f64::NAN
                    } else {
                        res
                    });
                }
                EvalNode::Function { name, arg_count } => {
                    let mut args = Vec::with_capacity(*arg_count);
                    for _ in 0..*arg_count {
                        let arg = stack.pop().ok_or_else(|| {
                            EvalError::SyntaxError(format!("Missing arguments for {}", name))
                        })?;
                        args.push(arg);
                    }
                    args.reverse();

                    let res = eval_function(name, &args)?;
                    stack.push(res);
                }
            }
        }

        if stack.len() != 1 {
            return Err(EvalError::SyntaxError("Invalid expression layout".into()));
        }

        Ok(stack.pop().unwrap_or(f64::NAN))
    }

    pub fn eval_row(&self, row_values: &HashMap<String, f64>) -> Result<f64, EvalError> {
        self.eval(&|name| {
            row_values
                .get(name)
                .or_else(|| row_values.get(&name.to_ascii_lowercase()))
                .or_else(|| row_values.get(&name.to_ascii_uppercase()))
                .copied()
        })
    }
}
