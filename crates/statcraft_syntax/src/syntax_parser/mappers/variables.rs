use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::ParsedCommand;
use serde_json::{json, Value};
use statcraft_core::DataSet;

pub fn map_variable_command(
    cmd: &ParsedCommand,
    _dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    match cmd.name.as_str() {
        "VARIABLE LEVEL" => map_variable_level(cmd),
        "VARIABLE LABELS" => map_variable_labels(cmd),
        "VALUE LABELS" => map_value_labels(cmd),
        "MISSING VALUES" => map_missing_values(cmd),
        _ => Ok(None),
    }
}

fn map_variable_level(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let mut assignments = Vec::new();

    for sub in &cmd.subcommands {
        let mut cur_vars = Vec::new();
        let sub_name = sub.name.to_ascii_uppercase();
        if sub_name != "POSITIONAL"
            && sub_name != "LEVEL"
            && sub_name != "VARIABLES"
            && sub_name != "SCALE"
            && sub_name != "ORDINAL"
            && sub_name != "NOMINAL"
        {
            cur_vars.push(sub.name.clone());
        }

        for token in &sub.tokens {
            let clean = token
                .trim()
                .trim_start_matches('(')
                .trim_end_matches(')')
                .to_ascii_uppercase();

            if clean == "SCALE" || clean == "ORDINAL" || clean == "NOMINAL" {
                if !cur_vars.is_empty() {
                    assignments.push(json!({
                        "variables": cur_vars.clone(),
                        "measure": clean.to_ascii_lowercase()
                    }));
                    cur_vars.clear();
                }
            } else {
                cur_vars.push(token.clone());
            }
        }

        // If measure was specified as the subcommand name itself (e.g. /SCALE var1 var2)
        if (sub_name == "SCALE" || sub_name == "ORDINAL" || sub_name == "NOMINAL")
            && !cur_vars.is_empty()
        {
            assignments.push(json!({
                "variables": cur_vars,
                "measure": sub_name.to_ascii_lowercase()
            }));
        }
    }

    if assignments.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "LEVEL".to_string(),
            reason: "Expected variable list followed by (SCALE), (ORDINAL), or (NOMINAL)"
                .to_string(),
        });
    }

    Ok(Some((
        "var.set_measure",
        json!({ "assignments": assignments }),
    )))
}

fn map_variable_labels(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let mut labels_map = serde_json::Map::new();

    for sub in &cmd.subcommands {
        let sub_name = sub.name.trim();
        let tokens = &sub.tokens;

        if sub_name != "POSITIONAL" && sub_name != "LABELS" && !tokens.is_empty() {
            let label = strip_quotes(tokens[0].trim());
            labels_map.insert(sub_name.to_string(), json!(label));
            let mut i = 1;
            while i + 1 < tokens.len() {
                let v = tokens[i].trim().to_string();
                let lbl = strip_quotes(tokens[i + 1].trim());
                labels_map.insert(v, json!(lbl));
                i += 2;
            }
        } else {
            let mut i = 0;
            while i + 1 < tokens.len() {
                let v = tokens[i].trim().to_string();
                let lbl = strip_quotes(tokens[i + 1].trim());
                labels_map.insert(v, json!(lbl));
                i += 2;
            }
        }
    }

    if labels_map.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "LABELS".to_string(),
            reason: "Expected variable name followed by label string".to_string(),
        });
    }

    Ok(Some(("var.set_label", json!({ "labels": labels_map }))))
}

fn map_value_labels(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let mut assignments = Vec::new();

    for sub in &cmd.subcommands {
        let sub_name = sub.name.trim();
        let tokens = &sub.tokens;
        if tokens.is_empty() && (sub_name == "POSITIONAL" || sub_name == "LABELS") {
            continue;
        }

        let (var_name, start_idx) = if sub_name != "POSITIONAL" && sub_name != "LABELS" {
            (sub_name.to_string(), 0)
        } else {
            (tokens[0].trim().to_string(), 1)
        };

        let mut labels = serde_json::Map::new();
        let mut i = start_idx;
        while i + 1 < tokens.len() {
            let code = tokens[i]
                .trim()
                .trim_start_matches('(')
                .trim_end_matches(')')
                .to_string();
            let label = strip_quotes(tokens[i + 1].trim());
            labels.insert(code, json!(label));
            i += 2;
        }

        if !labels.is_empty() {
            assignments.push(json!({
                "var_name": var_name,
                "labels": labels
            }));
        }
    }

    if assignments.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "LABELS".to_string(),
            reason:
                "Expected variable name followed by code label pairs (e.g. 1 'Male' 2 'Female')"
                    .to_string(),
        });
    }

    Ok(Some((
        "var.set_value_labels",
        json!({ "assignments": assignments }),
    )))
}

fn map_missing_values(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let mut assignments = Vec::new();

    for sub in &cmd.subcommands {
        let sub_name = sub.name.trim();
        let tokens = &sub.tokens;
        if tokens.is_empty() && (sub_name == "POSITIONAL" || sub_name == "VALUES") {
            continue;
        }

        let (var_name, raw_vals) = if sub_name != "POSITIONAL" && sub_name != "VALUES" {
            (sub_name.to_string(), sub.raw_value.clone())
        } else {
            let v_name = tokens[0].trim().to_string();
            let r_vals = if tokens.len() > 1 {
                tokens[1..].join(" ")
            } else {
                sub.raw_value.clone()
            };
            (v_name, r_vals)
        };

        let inside = raw_vals
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim();

        if inside.to_ascii_uppercase().contains("THRU") {
            let parts: Vec<&str> = inside
                .split(|c: char| c.is_whitespace() || c == ',')
                .filter(|s| !s.is_empty())
                .collect();
            let thru_pos = parts.iter().position(|s| s.eq_ignore_ascii_case("THRU"));
            if let Some(pos) = thru_pos {
                if pos > 0 && pos + 1 < parts.len() {
                    let low = parts[pos - 1].parse::<f64>().ok();
                    let high = parts[pos + 1].parse::<f64>().ok();
                    if let (Some(l), Some(h)) = (low, high) {
                        assignments.push(json!({
                            "var_name": var_name,
                            "discrete": [],
                            "range": [l, h]
                        }));
                        continue;
                    }
                }
            }
        }

        let discrete: Vec<String> = inside
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        assignments.push(json!({
            "var_name": var_name,
            "discrete": discrete,
            "range": Value::Null
        }));
    }

    if assignments.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VALUES".to_string(),
            reason: "Expected variable name followed by missing values in parentheses".to_string(),
        });
    }

    Ok(Some((
        "var.set_missing_values",
        json!({ "assignments": assignments }),
    )))
}

fn strip_quotes(s: &str) -> String {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            trimmed[1..trimmed.len() - 1].to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        trimmed.to_string()
    }
}
