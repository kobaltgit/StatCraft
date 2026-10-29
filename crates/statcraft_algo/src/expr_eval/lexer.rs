use super::types::{EvalError, Op, Token};

pub fn tokenize(input: &str) -> Result<Vec<Token>, EvalError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let mut can_be_unary = true;

    while i < len {
        let ch = chars[i];

        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        if ch.is_ascii_digit() || (ch == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let start = i;
            let mut has_dot = ch == '.';
            i += 1;
            while i < len && (chars[i].is_ascii_digit() || (chars[i] == '.' && !has_dot)) {
                if chars[i] == '.' {
                    has_dot = true;
                }
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            let num = s
                .parse::<f64>()
                .map_err(|_| EvalError::SyntaxError(format!("Invalid number: {}", s)))?;
            tokens.push(Token::Number(num));
            can_be_unary = false;
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            i += 1;
            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();
            tokens.push(Token::Ident(ident));
            can_be_unary = false;
            continue;
        }

        match ch {
            '+' => {
                if can_be_unary {
                    tokens.push(Token::UnaryPlus);
                } else {
                    tokens.push(Token::BinaryOp(Op::Add));
                }
                can_be_unary = true;
            }
            '-' => {
                if can_be_unary {
                    tokens.push(Token::UnaryMinus);
                } else {
                    tokens.push(Token::BinaryOp(Op::Sub));
                }
                can_be_unary = true;
            }
            '*' => {
                if i + 1 < len && chars[i + 1] == '*' {
                    tokens.push(Token::BinaryOp(Op::Pow));
                    i += 1;
                } else {
                    tokens.push(Token::BinaryOp(Op::Mul));
                }
                can_be_unary = true;
            }
            '/' => {
                tokens.push(Token::BinaryOp(Op::Div));
                can_be_unary = true;
            }
            '^' => {
                tokens.push(Token::BinaryOp(Op::Pow));
                can_be_unary = true;
            }
            '(' => {
                tokens.push(Token::LParen);
                can_be_unary = true;
            }
            ')' => {
                tokens.push(Token::RParen);
                can_be_unary = false;
            }
            ',' => {
                tokens.push(Token::Comma);
                can_be_unary = true;
            }
            _ => return Err(EvalError::UnexpectedToken(ch.to_string())),
        }
        i += 1;
    }

    Ok(tokens)
}
