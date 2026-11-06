use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::lexer::RawCommandBlock;
use statcraft_core::DataSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Subcommand {
    pub name: String,
    pub raw_value: String,
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCommand {
    pub name: String,
    pub subcommands: Vec<Subcommand>,
    pub line_num: usize,
    pub raw_text: String,
}

impl ParsedCommand {
    pub fn subcommand(&self, name: &str) -> Option<&Subcommand> {
        self.subcommands
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    pub fn subcommand_value(&self, name: &str) -> Option<&str> {
        self.subcommand(name).map(|s| s.raw_value.as_str())
    }

    pub fn subcommand_tokens(&self, name: &str) -> Option<&[String]> {
        self.subcommand(name).map(|s| s.tokens.as_slice())
    }

    pub fn has_subcommand(&self, name: &str) -> bool {
        self.subcommand(name).is_some()
    }

    pub fn has_token(&self, subcmd_name: &str, token: &str) -> bool {
        self.subcommand(subcmd_name)
            .map(|s| s.tokens.iter().any(|t| t.eq_ignore_ascii_case(token)))
            .unwrap_or(false)
    }
}

pub struct SyntaxParser;

impl SyntaxParser {
    pub fn parse_command(block: &RawCommandBlock) -> Result<ParsedCommand, SyntaxError> {
        let text = block.raw_text.trim().trim_end_matches('.').trim();

        // 1. Identify full command name (including two-word commands)
        let (cmd_name, rest) = Self::extract_command_name(text);

        // 2. Parse subcommands
        let subcommands = Self::parse_subcommands(rest)?;

        Ok(ParsedCommand {
            name: cmd_name,
            subcommands,
            line_num: block.line_num,
            raw_text: block.raw_text.clone(),
        })
    }

    fn extract_command_name(text: &str) -> (String, &str) {
        let mut words = text.split_whitespace();
        let first = match words.next() {
            Some(w) => w,
            None => return (String::new(), ""),
        };

        let second = words.next();
        let two_word = match (first.to_ascii_uppercase().as_str(), second) {
            ("SORT", Some(s)) if s.eq_ignore_ascii_case("CASES") => Some("SORT CASES"),
            ("SPLIT", Some(s)) if s.eq_ignore_ascii_case("FILE") => Some("SPLIT FILE"),
            _ => None,
        };

        if let Some(compound) = two_word {
            let offset = first.len() + 1 + second.map(|s| s.len()).unwrap_or(0);
            let rest = if offset < text.len() {
                text[offset..].trim_start()
            } else {
                ""
            };
            (compound.to_string(), rest)
        } else {
            let rest = if first.len() < text.len() {
                text[first.len()..].trim_start()
            } else {
                ""
            };
            (first.to_ascii_uppercase(), rest)
        }
    }

    fn parse_subcommands(text: &str) -> Result<Vec<Subcommand>, SyntaxError> {
        let mut subcommands = Vec::new();
        if text.trim().is_empty() {
            return Ok(subcommands);
        }

        // Split by `/` while respecting quotes
        let parts = Self::split_by_slash(text)?;

        for (idx, part) in parts.into_iter().enumerate() {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            let (name, val) = if idx == 0 && !text.trim_start().starts_with('/') {
                if trimmed.contains('=') {
                    Self::split_subcommand_name_value(trimmed)
                } else {
                    let first_word = trimmed
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_ascii_uppercase();
                    if first_word == "BY"
                        || first_word == "VARIABLES"
                        || first_word == "TABLES"
                        || first_word == "GROUPS"
                        || first_word == "PAIRS"
                    {
                        Self::split_subcommand_name_value(trimmed)
                    } else {
                        ("POSITIONAL".to_string(), trimmed)
                    }
                }
            } else {
                Self::split_subcommand_name_value(trimmed)
            };

            let subcmd = Self::parse_subcommand_content(&name, val);
            subcommands.push(subcmd);
        }

        Ok(subcommands)
    }

    fn split_by_slash(text: &str) -> Result<Vec<String>, SyntaxError> {
        let mut parts = Vec::new();
        let mut cur = String::new();
        let mut in_quote = false;
        let mut quote_char = ' ';

        for ch in text.chars() {
            if in_quote {
                cur.push(ch);
                if ch == quote_char {
                    in_quote = false;
                }
            } else if ch == '\'' || ch == '"' {
                in_quote = true;
                quote_char = ch;
                cur.push(ch);
            } else if ch == '/' {
                parts.push(cur.clone());
                cur.clear();
            } else {
                cur.push(ch);
            }
        }
        parts.push(cur);
        Ok(parts)
    }

    fn split_subcommand_name_value(trimmed: &str) -> (String, &str) {
        if let Some(eq_pos) = trimmed.find('=') {
            let name = trimmed[..eq_pos].trim().to_ascii_uppercase();
            let val = trimmed[eq_pos + 1..].trim();
            (name, val)
        } else {
            // Check for space separating name from value (e.g., `CRITERIA CI(0.95)` or `BY var`)
            let mut words = trimmed.split_whitespace();
            if let Some(first) = words.next() {
                let name = first.to_ascii_uppercase();
                let rest = if first.len() < trimmed.len() {
                    trimmed[first.len()..].trim()
                } else {
                    ""
                };
                (name, rest)
            } else {
                (trimmed.to_ascii_uppercase(), "")
            }
        }
    }

    fn parse_subcommand_content(name: &str, raw_value: &str) -> Subcommand {
        let tokens = Self::tokenize_values(raw_value);
        let normalized_name = if name == "POSITIONAL" {
            // If the positional part was `VARIABLES=x y`, normalize name to VARIABLES
            if let Some((n, v)) = raw_value.split_once('=') {
                return Subcommand {
                    name: n.trim().to_ascii_uppercase(),
                    raw_value: v.trim().to_string(),
                    tokens: Self::tokenize_values(v.trim()),
                };
            }
            name.to_string()
        } else {
            name.to_string()
        };

        Subcommand {
            name: normalized_name,
            raw_value: raw_value.to_string(),
            tokens,
        }
    }

    pub fn tokenize_values(input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut cur = String::new();
        let mut in_single = false;
        let mut in_double = false;
        let mut paren_depth = 0;

        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let ch = chars[i];
            if in_single {
                if ch == '\'' {
                    if i + 1 < len && chars[i + 1] == '\'' {
                        cur.push('\'');
                        i += 2;
                        continue;
                    }
                    in_single = false;
                } else {
                    cur.push(ch);
                }
            } else if in_double {
                if ch == '"' {
                    if i + 1 < len && chars[i + 1] == '"' {
                        cur.push('"');
                        i += 2;
                        continue;
                    }
                    in_double = false;
                } else {
                    cur.push(ch);
                }
            } else if ch == '\'' {
                in_single = true;
            } else if ch == '"' {
                in_double = true;
            } else if ch == '(' {
                paren_depth += 1;
                cur.push(ch);
            } else if ch == ')' {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
                cur.push(ch);
            } else if ch.is_whitespace() || ch == ',' {
                if paren_depth > 0 {
                    cur.push(ch);
                } else if !cur.is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
            } else {
                cur.push(ch);
            }
            i += 1;
        }

        if !cur.is_empty() {
            tokens.push(cur);
        }

        tokens
    }

    /// Expands variable lists containing ranges `VAR1 TO VAR5` and keyword `ALL`
    /// against an optional active dataset.
    pub fn expand_variables(
        tokens: &[String],
        dataset: Option<&DataSet>,
    ) -> Result<Vec<String>, SyntaxError> {
        let mut expanded = Vec::new();
        let ds_vars: Vec<String> = dataset
            .map(|ds| {
                ds.dictionary
                    .variables
                    .iter()
                    .map(|v| v.name.clone())
                    .collect()
            })
            .unwrap_or_default();

        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];

            if token.eq_ignore_ascii_case("ALL") {
                if !ds_vars.is_empty() {
                    expanded.extend(ds_vars.clone());
                } else {
                    expanded.push(token.clone());
                }
                i += 1;
                continue;
            }

            if i + 2 < tokens.len() && tokens[i + 1].eq_ignore_ascii_case("TO") {
                let start_var = token;
                let end_var = &tokens[i + 2];

                let start_idx = ds_vars
                    .iter()
                    .position(|v| v.eq_ignore_ascii_case(start_var));
                let end_idx = ds_vars.iter().position(|v| v.eq_ignore_ascii_case(end_var));

                match (start_idx, end_idx) {
                    (Some(s), Some(e)) if s <= e => {
                        for v in &ds_vars[s..=e] {
                            expanded.push(v.clone());
                        }
                    }
                    _ => {
                        // Fallback if dataset not available or indices inverted
                        expanded.push(start_var.clone());
                        expanded.push("TO".to_string());
                        expanded.push(end_var.clone());
                    }
                }
                i += 3;
                continue;
            }

            expanded.push(token.clone());
            i += 1;
        }

        Ok(expanded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_descriptives_command() {
        let block = RawCommandBlock {
            command_name: "DESCRIPTIVES".to_string(),
            raw_text: "DESCRIPTIVES VARIABLES=salary age\n  /STATISTICS=MEAN STDDEV MIN MAX."
                .to_string(),
            line_num: 1,
            is_comment: false,
        };

        let cmd = SyntaxParser::parse_command(&block).unwrap();
        assert_eq!(cmd.name, "DESCRIPTIVES");
        assert_eq!(cmd.subcommands.len(), 2);
        assert_eq!(
            cmd.subcommand("VARIABLES").unwrap().tokens,
            vec!["salary", "age"]
        );
        assert!(cmd.has_token("STATISTICS", "MEAN"));
        assert!(cmd.has_token("STATISTICS", "MAX"));
    }

    #[test]
    fn test_parse_compound_command_sort_cases() {
        let block = RawCommandBlock {
            command_name: "SORT".to_string(),
            raw_text: "SORT CASES BY salary (A) age (D).".to_string(),
            line_num: 5,
            is_comment: false,
        };

        let cmd = SyntaxParser::parse_command(&block).unwrap();
        assert_eq!(cmd.name, "SORT CASES");
        assert_eq!(
            cmd.subcommand("BY").unwrap().tokens,
            vec!["salary", "(A)", "age", "(D)"]
        );
    }

    #[test]
    fn test_expand_variables_all_and_range() {
        use statcraft_io::{load_csv_from_reader, CsvOptions};

        let csv = "q1,q2,q3,q4,q5\n1,2,3,4,5\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        let ds = DataSet::from_raw_table(table).unwrap();

        let tokens = vec!["q2".to_string(), "TO".to_string(), "q4".to_string()];
        let expanded = SyntaxParser::expand_variables(&tokens, Some(&ds)).unwrap();
        assert_eq!(expanded, vec!["q2", "q3", "q4"]);

        let all_tokens = vec!["ALL".to_string()];
        let expanded_all = SyntaxParser::expand_variables(&all_tokens, Some(&ds)).unwrap();
        assert_eq!(expanded_all, vec!["q1", "q2", "q3", "q4", "q5"]);
    }
}
