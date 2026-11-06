use crate::syntax_parser::error::SyntaxError;

#[derive(Debug, Clone, PartialEq)]
pub struct RawCommandBlock {
    pub command_name: String,
    pub raw_text: String,
    pub line_num: usize,
    pub is_comment: bool,
}

pub struct SyntaxLexer;

impl SyntaxLexer {
    /// Splits an SPSS syntax script into individual raw command blocks.
    /// Handles comments (`*` or `COMMENT`), string literals with escaped quotes,
    /// and command-terminating periods.
    pub fn split_commands(input: &str) -> Result<Vec<RawCommandBlock>, SyntaxError> {
        let mut blocks = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();

        let mut idx = 0;
        let mut line = 1;
        let mut col = 1;

        while idx < len {
            // Skip leading whitespace / newlines
            while idx < len && (chars[idx].is_whitespace()) {
                if chars[idx] == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                idx += 1;
            }

            if idx >= len {
                break;
            }

            let start_line = line;
            let mut current_block = String::new();
            let mut in_single_quote = false;
            let mut in_double_quote = false;
            let mut quote_start_line = line;
            let mut quote_start_col = col;

            while idx < len {
                let ch = chars[idx];
                let next_ch = if idx + 1 < len {
                    Some(chars[idx + 1])
                } else {
                    None
                };

                // Track string literal boundaries and quote escaping
                if in_single_quote {
                    current_block.push(ch);
                    if ch == '\'' {
                        if next_ch == Some('\'') {
                            // Escaped quote ''
                            current_block.push('\'');
                            idx += 2;
                            col += 2;
                            continue;
                        } else {
                            in_single_quote = false;
                        }
                    } else if ch == '\n' {
                        line += 1;
                        col = 0;
                    }
                    idx += 1;
                    col += 1;
                    continue;
                }

                if in_double_quote {
                    current_block.push(ch);
                    if ch == '"' {
                        if next_ch == Some('"') {
                            // Escaped quote ""
                            current_block.push('"');
                            idx += 2;
                            col += 2;
                            continue;
                        } else {
                            in_double_quote = false;
                        }
                    } else if ch == '\n' {
                        line += 1;
                        col = 0;
                    }
                    idx += 1;
                    col += 1;
                    continue;
                }

                // Not currently in string
                if ch == '\'' {
                    in_single_quote = true;
                    quote_start_line = line;
                    quote_start_col = col;
                    current_block.push(ch);
                    idx += 1;
                    col += 1;
                    continue;
                } else if ch == '"' {
                    in_double_quote = true;
                    quote_start_line = line;
                    quote_start_col = col;
                    current_block.push(ch);
                    idx += 1;
                    col += 1;
                    continue;
                }

                // Check for terminating period '.'
                if ch == '.' {
                    let is_float_decimal = match next_ch {
                        Some(c) if c.is_ascii_digit() => {
                            // E.g. .05 or 3.14 -> decimal point, not command terminator
                            true
                        }
                        _ => false,
                    };

                    if !is_float_decimal {
                        // In SPSS, a period followed by whitespace, newline, or EOF terminates the command.
                        let is_terminator = match next_ch {
                            None => true,
                            Some(c) if c.is_whitespace() || c == '/' || c == '*' => true,
                            _ => false,
                        };

                        if is_terminator {
                            current_block.push(ch);
                            idx += 1;
                            col += 1;
                            break;
                        }
                    }
                }

                current_block.push(ch);
                if ch == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                idx += 1;
            }

            if in_single_quote || in_double_quote {
                return Err(SyntaxError::UnclosedString {
                    line: quote_start_line,
                    col: quote_start_col,
                });
            }

            let trimmed = current_block.trim();
            if !trimmed.is_empty() {
                // Determine command name and whether it's a comment
                let first_word = trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('.');

                let is_comment =
                    first_word.starts_with('*') || first_word.eq_ignore_ascii_case("COMMENT");

                let cmd_name = if is_comment {
                    "COMMENT".to_string()
                } else {
                    first_word.to_ascii_uppercase()
                };

                blocks.push(RawCommandBlock {
                    command_name: cmd_name,
                    raw_text: trimmed.to_string(),
                    line_num: start_line,
                    is_comment,
                });
            }
        }

        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_command_with_period() {
        let sps = "DESCRIPTIVES VARIABLES=salary.";
        let blocks = SyntaxLexer::split_commands(sps).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].command_name, "DESCRIPTIVES");
        assert_eq!(blocks[0].line_num, 1);
        assert!(!blocks[0].is_comment);
    }

    #[test]
    fn test_multiple_commands_and_comments() {
        let sps = r#"
        * Initial descriptive check.
        DESCRIPTIVES VARIABLES=salary age
          /STATISTICS=MEAN STDDEV.

        COMMENT Frequency count follows.
        FREQUENCIES VARIABLES=gender.
        "#;
        let blocks = SyntaxLexer::split_commands(sps).unwrap();
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].command_name, "COMMENT");
        assert!(blocks[0].is_comment);
        assert_eq!(blocks[1].command_name, "DESCRIPTIVES");
        assert!(!blocks[1].is_comment);
        assert_eq!(blocks[2].command_name, "COMMENT");
        assert!(blocks[2].is_comment);
        assert_eq!(blocks[3].command_name, "FREQUENCIES");
        assert!(!blocks[3].is_comment);
    }

    #[test]
    fn test_periods_inside_quotes_and_floats() {
        let sps = "GET FILE='C:\\my.data.sav' /CRITERIA=CI(0.95).";
        let blocks = SyntaxLexer::split_commands(sps).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].command_name, "GET");
        assert!(blocks[0].raw_text.contains("C:\\my.data.sav"));
    }

    #[test]
    fn test_escaped_quotes() {
        let sps = "COMPUTE label='Don''t stop'.";
        let blocks = SyntaxLexer::split_commands(sps).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].raw_text, "COMPUTE label='Don''t stop'.");
    }

    #[test]
    fn test_unclosed_string_error() {
        let sps = "GET FILE='unfinished_path.sav";
        let err = SyntaxLexer::split_commands(sps).unwrap_err();
        match err {
            SyntaxError::UnclosedString { line, col } => {
                assert_eq!(line, 1);
                assert_eq!(col, 10);
            }
            _ => panic!("Expected UnclosedString error"),
        }
    }

    #[test]
    fn test_command_without_trailing_period_at_eof() {
        let sps = "DESCRIPTIVES VARIABLES=salary";
        let blocks = SyntaxLexer::split_commands(sps).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].command_name, "DESCRIPTIVES");
    }
}
