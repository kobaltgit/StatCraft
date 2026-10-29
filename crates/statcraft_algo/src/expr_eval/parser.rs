use super::types::{EvalError, EvalNode, Op, Token};

#[derive(Debug)]
pub enum OpStackEntry {
    Op(Op),
    UnaryMinus,
    UnaryPlus,
    LParen,
    Func { name: String, arg_count: usize },
}

pub fn op_precedence(op: Op) -> (u8, bool) {
    match op {
        Op::Pow => (4, true), // right-associative
        Op::Mul | Op::Div => (3, false),
        Op::Add | Op::Sub => (2, false),
    }
}

pub fn to_rpn(tokens: &[Token]) -> Result<Vec<EvalNode>, EvalError> {
    let mut output: Vec<EvalNode> = Vec::new();
    let mut op_stack: Vec<OpStackEntry> = Vec::new();
    let mut arg_count_stack: Vec<usize> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Number(n) => output.push(EvalNode::Number(*n)),
            Token::Ident(name) => {
                if i + 1 < tokens.len() && tokens[i + 1] == Token::LParen {
                    op_stack.push(OpStackEntry::Func {
                        name: name.to_ascii_uppercase(),
                        arg_count: 0,
                    });
                    arg_count_stack.push(0);
                } else {
                    output.push(EvalNode::Variable(name.clone()));
                }
            }
            Token::UnaryMinus => op_stack.push(OpStackEntry::UnaryMinus),
            Token::UnaryPlus => op_stack.push(OpStackEntry::UnaryPlus),
            Token::BinaryOp(op) => {
                let (prec, right_assoc) = op_precedence(*op);
                while let Some(top) = op_stack.last() {
                    let top_prec = match top {
                        OpStackEntry::Op(top_op) => op_precedence(*top_op).0,
                        OpStackEntry::UnaryMinus | OpStackEntry::UnaryPlus => 5,
                        _ => 0,
                    };
                    if top_prec > prec || (top_prec == prec && !right_assoc) {
                        match op_stack.pop().unwrap() {
                            OpStackEntry::Op(o) => output.push(EvalNode::BinaryOp(o)),
                            OpStackEntry::UnaryMinus => output.push(EvalNode::UnaryMinus),
                            OpStackEntry::UnaryPlus => output.push(EvalNode::UnaryPlus),
                            _ => {}
                        }
                    } else {
                        break;
                    }
                }
                op_stack.push(OpStackEntry::Op(*op));
            }
            Token::LParen => {
                if let Some(top) = op_stack.last() {
                    if matches!(top, OpStackEntry::Func { .. }) {
                        // Handled when function was encountered
                    } else {
                        op_stack.push(OpStackEntry::LParen);
                    }
                } else {
                    op_stack.push(OpStackEntry::LParen);
                }
            }
            Token::Comma => {
                while let Some(top) = op_stack.last() {
                    if matches!(top, OpStackEntry::LParen | OpStackEntry::Func { .. }) {
                        break;
                    }
                    match op_stack.pop().unwrap() {
                        OpStackEntry::Op(o) => output.push(EvalNode::BinaryOp(o)),
                        OpStackEntry::UnaryMinus => output.push(EvalNode::UnaryMinus),
                        OpStackEntry::UnaryPlus => output.push(EvalNode::UnaryPlus),
                        _ => {}
                    }
                }
                if let Some(count) = arg_count_stack.last_mut() {
                    *count += 1;
                } else {
                    return Err(EvalError::SyntaxError("Misplaced comma".into()));
                }
            }
            Token::RParen => {
                while let Some(top) = op_stack.last() {
                    if matches!(top, OpStackEntry::LParen | OpStackEntry::Func { .. }) {
                        break;
                    }
                    match op_stack.pop().unwrap() {
                        OpStackEntry::Op(o) => output.push(EvalNode::BinaryOp(o)),
                        OpStackEntry::UnaryMinus => output.push(EvalNode::UnaryMinus),
                        OpStackEntry::UnaryPlus => output.push(EvalNode::UnaryPlus),
                        _ => {}
                    }
                }

                match op_stack.pop() {
                    Some(OpStackEntry::LParen) => {}
                    Some(OpStackEntry::Func {
                        name,
                        mut arg_count,
                    }) => {
                        let total_args = if let Some(last_cnt) = arg_count_stack.pop() {
                            if tokens.get(i.saturating_sub(1)) == Some(&Token::LParen) {
                                0
                            } else {
                                last_cnt + 1
                            }
                        } else {
                            arg_count
                        };
                        arg_count = total_args;
                        output.push(EvalNode::Function { name, arg_count });
                    }
                    _ => return Err(EvalError::MismatchedParentheses),
                }
            }
        }
        i += 1;
    }

    while let Some(top) = op_stack.pop() {
        match top {
            OpStackEntry::Op(o) => output.push(EvalNode::BinaryOp(o)),
            OpStackEntry::UnaryMinus => output.push(EvalNode::UnaryMinus),
            OpStackEntry::UnaryPlus => output.push(EvalNode::UnaryPlus),
            OpStackEntry::LParen | OpStackEntry::Func { .. } => {
                return Err(EvalError::MismatchedParentheses)
            }
        }
    }

    Ok(output)
}
