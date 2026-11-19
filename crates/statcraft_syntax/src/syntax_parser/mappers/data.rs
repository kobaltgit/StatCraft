use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;

pub fn map_data_command(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    match cmd.name.as_str() {
        "DATASET NAME" => map_dataset_name(cmd),
        "DATASET ACTIVATE" => map_dataset_activate(cmd),
        "DATASET CLOSE" => map_dataset_close(cmd),
        "DATASET DISPLAY" => Ok(Some(("dataset.display", json!({})))),
        "SORT CASES" => map_sort_cases(cmd, dataset),
        "SORT VARIABLES" => map_sort_variables(cmd),
        "ADD FILES" => map_add_files(cmd),
        "MATCH FILES" => map_match_files(cmd),
        "FLIP" => map_flip(cmd, dataset),
        "WEIGHT" => map_weight_cases(cmd),
        "FILTER" => map_filter_cases(cmd),
        "SPLIT FILE" => map_split_file(cmd, dataset),
        "COMPUTE" => map_compute(cmd),
        "GET" => map_get_file(cmd),
        "SAVE" => map_save_file(cmd),
        "EXECUTE" => Ok(Some(("data.execute_noop", json!({ "status": "executed" })))),
        "COMMENT" => Ok(Some(("util.comment", json!({ "status": "comment" })))),
        _ => Ok(None),
    }
}

fn map_sort_cases(
    cmd: &ParsedCommand,
    _dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd.subcommand_tokens("BY").unwrap_or(&[]);
    if raw_tokens.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "BY".to_string(),
            reason: "SORT CASES requires at least one sort variable in BY clause".to_string(),
        });
    }

    let mut keys = Vec::new();
    let mut i = 0;
    while i < raw_tokens.len() {
        let token = &raw_tokens[i];
        if token.starts_with('(') && token.ends_with(')') {
            i += 1;
            continue;
        }

        let var_name = token.clone();
        let mut ascending = true;
        if i + 1 < raw_tokens.len() {
            let next = &raw_tokens[i + 1];
            if next.eq_ignore_ascii_case("(D)") || next.eq_ignore_ascii_case("(DESCENDING)") {
                ascending = false;
                i += 1;
            } else if next.eq_ignore_ascii_case("(A)") || next.eq_ignore_ascii_case("(ASCENDING)") {
                ascending = true;
                i += 1;
            }
        }

        keys.push(json!({
            "column_name": var_name,
            "direction": if ascending { "Ascending" } else { "Descending" }
        }));
        i += 1;
    }

    Ok(Some(("data.sort_cases", json!({ "keys": keys }))))
}

fn map_weight_cases(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let tokens = cmd
        .subcommand_tokens("BY")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    if tokens.iter().any(|t| t.eq_ignore_ascii_case("OFF")) {
        return Ok(Some((
            "data.weight_cases",
            json!({ "variable": Value::Null }),
        )));
    }

    let var_name = tokens
        .first()
        .cloned()
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: "WEIGHT".to_string(),
            subcommand: "BY".to_string(),
            reason: "Missing weight variable name".to_string(),
        })?;

    Ok(Some(("data.weight_cases", json!({ "variable": var_name }))))
}

fn map_filter_cases(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let tokens = cmd
        .subcommand_tokens("BY")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    if tokens.iter().any(|t| t.eq_ignore_ascii_case("OFF")) {
        return Ok(Some(("data.select_cases", json!({ "mode": "all" }))));
    }

    let var_name = tokens
        .first()
        .cloned()
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: "FILTER".to_string(),
            subcommand: "BY".to_string(),
            reason: "Missing filter variable name".to_string(),
        })?;

    Ok(Some((
        "data.select_cases",
        json!({
            "mode": "variable",
            "filter_var": var_name,
            "action": "filter"
        }),
    )))
}

fn map_split_file(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let tokens = cmd
        .subcommand_tokens("POSITIONAL")
        .or_else(|| cmd.subcommand_tokens("BY"))
        .unwrap_or(&[]);

    if tokens.iter().any(|t| t.eq_ignore_ascii_case("OFF")) {
        return Ok(Some((
            "data.split_file",
            json!({
                "mode": "off",
                "variables": [],
                "sort": false
            }),
        )));
    }

    let is_separate = tokens.iter().any(|t| t.eq_ignore_ascii_case("SEPARATE"));
    let mode = if is_separate { "separate" } else { "layered" };

    let var_tokens = if let Some(by_sub) = cmd.subcommand("BY") {
        &by_sub.tokens[..]
    } else if let Some(pos) = tokens.iter().position(|t| t.eq_ignore_ascii_case("BY")) {
        &tokens[pos + 1..]
    } else {
        tokens
    };

    let vars = SyntaxParser::expand_variables(var_tokens, dataset)?;

    Ok(Some((
        "data.split_file",
        json!({
            "mode": mode,
            "variables": vars,
            "sort": false
        }),
    )))
}

fn map_compute(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw = &cmd.raw_text;
    let stripped = raw
        .trim_start()
        .strip_prefix("COMPUTE")
        .unwrap_or(raw)
        .trim()
        .trim_end_matches('.')
        .trim();

    let (target, expr) = stripped
        .split_once('=')
        .ok_or_else(|| SyntaxError::ParsingFailed {
            line: cmd.line_num,
            col: 1,
            message: "COMPUTE requires 'target_var = expression'".to_string(),
        })?;

    Ok(Some((
        "transform.compute",
        json!({
            "target": target.trim(),
            "expression": expr.trim()
        }),
    )))
}

fn map_get_file(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let path = cmd
        .subcommand_value("FILE")
        .or_else(|| cmd.subcommand_value("POSITIONAL"))
        .map(|s| s.trim_matches('\'').trim_matches('"'))
        .unwrap_or("");

    Ok(Some(("data.open_file", json!({ "path": path }))))
}

fn map_save_file(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let path = cmd
        .subcommand_value("OUTFILE")
        .or_else(|| cmd.subcommand_value("FILE"))
        .map(|s| s.trim_matches('\'').trim_matches('"'))
        .unwrap_or("");

    Ok(Some(("data.save_file", json!({ "path": path }))))
}

fn map_add_files(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let mut file_path = String::new();
    for sub in &cmd.subcommands {
        if sub.name.eq_ignore_ascii_case("FILE") {
            let val = sub
                .raw_value
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .trim();
            if val != "*" && !val.is_empty() {
                file_path = val.to_string();
            }
        }
    }

    if file_path.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "FILE".to_string(),
            reason: "ADD FILES requires a secondary file path (/FILE='...')".to_string(),
        });
    }

    let in_variable = cmd.subcommand_value("IN").map(|s| s.trim().to_string());
    let interleave_by: Option<Vec<String>> = cmd.subcommand_tokens("BY").map(|toks| toks.to_vec());

    let mut renames = serde_json::Map::new();
    if let Some(r_val) = cmd.subcommand_value("RENAME") {
        for part in r_val.split('(') {
            let clean = part.trim_end_matches(')').trim();
            if let Some((old_n, new_n)) = clean.split_once('=') {
                renames.insert(old_n.trim().to_string(), json!(new_n.trim()));
            }
        }
    }

    Ok(Some((
        "data.merge_cases",
        json!({
            "file_path": file_path,
            "in_variable": in_variable,
            "renames": renames,
            "interleave_by": interleave_by,
        }),
    )))
}

fn map_match_files(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let mut file_path = String::new();
    let mut is_table = false;

    for sub in &cmd.subcommands {
        if sub.name.eq_ignore_ascii_case("TABLE") {
            let val = sub
                .raw_value
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .trim();
            if !val.is_empty() {
                file_path = val.to_string();
                is_table = true;
            }
        } else if sub.name.eq_ignore_ascii_case("FILE") {
            let val = sub
                .raw_value
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .trim();
            if val != "*" && !val.is_empty() && file_path.is_empty() {
                file_path = val.to_string();
            }
        }
    }

    if file_path.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "FILE".to_string(),
            reason:
                "MATCH FILES requires a secondary file or table path (/FILE='...' or /TABLE='...')"
                    .to_string(),
        });
    }

    let keys = cmd
        .subcommand_tokens("BY")
        .map(|toks| toks.to_vec())
        .unwrap_or_default();
    let method = if is_table {
        "OneToManyTable"
    } else if keys.is_empty() {
        "Sequential"
    } else {
        "OneToOneWithKeys"
    };

    let drop_second = cmd
        .subcommand_tokens("DROP")
        .map(|toks| toks.to_vec())
        .unwrap_or_default();
    let in_second_variable = cmd.subcommand_value("IN").map(|s| s.trim().to_string());

    let mut renames = serde_json::Map::new();
    if let Some(r_val) = cmd.subcommand_value("RENAME") {
        for part in r_val.split('(') {
            let clean = part.trim_end_matches(')').trim();
            if let Some((old_n, new_n)) = clean.split_once('=') {
                renames.insert(old_n.trim().to_string(), json!(new_n.trim()));
            }
        }
    }

    Ok(Some((
        "data.merge_vars",
        json!({
            "file_path": file_path,
            "method": method,
            "keys": keys,
            "drop_second": drop_second,
            "renames": renames,
            "in_second_variable": in_second_variable,
        }),
    )))
}

fn map_sort_variables(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("BY")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    let mut attribute = "name";
    let mut direction = "Ascending";

    for token in raw_tokens {
        if token.eq_ignore_ascii_case("(D)") || token.eq_ignore_ascii_case("(DESCENDING)") {
            direction = "Descending";
        } else if token.eq_ignore_ascii_case("(A)") || token.eq_ignore_ascii_case("(ASCENDING)") {
            direction = "Ascending";
        } else {
            let lower = token.to_lowercase();
            match lower.as_str() {
                "name" | "type" | "label" | "measure" | "role" | "align" | "alignment"
                | "width" | "decimals" => {
                    attribute = match lower.as_str() {
                        "align" | "alignment" => "alignment",
                        _ => token.as_str(),
                    };
                }
                _ => {}
            }
        }
    }

    Ok(Some((
        "data.sort_variables",
        json!({
            "attribute": attribute,
            "direction": direction
        }),
    )))
}

fn map_flip(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let var_tokens = cmd
        .subcommand_tokens("VARIABLES")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    let vars = if var_tokens.is_empty() {
        Vec::new()
    } else {
        SyntaxParser::expand_variables(var_tokens, dataset)?
    };

    let name_var = cmd
        .subcommand_value("NEWNAMES")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(Some((
        "data.transpose",
        json!({
            "variables": if vars.is_empty() { Value::Null } else { json!(vars) },
            "name_var": name_var
        }),
    )))
}

fn map_dataset_name(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let name = cmd
        .subcommand_tokens("POSITIONAL")
        .and_then(|t| t.first())
        .or_else(|| cmd.subcommand_tokens("NAME").and_then(|t| t.first()))
        .cloned()
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "NAME".to_string(),
            reason: "Dataset name identifier is required".to_string(),
        })?;

    let window = cmd
        .subcommand_value("WINDOW")
        .map(|w| w.to_ascii_lowercase())
        .unwrap_or_else(|| "asis".to_string());

    Ok(Some((
        "dataset.name",
        json!({ "name": name, "window": window }),
    )))
}

fn map_dataset_activate(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let name = cmd
        .subcommand_tokens("POSITIONAL")
        .and_then(|t| t.first())
        .or_else(|| cmd.subcommand_tokens("NAME").and_then(|t| t.first()))
        .cloned()
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "NAME".to_string(),
            reason: "Dataset name identifier is required".to_string(),
        })?;

    let window = cmd
        .subcommand_value("WINDOW")
        .map(|w| w.to_ascii_lowercase())
        .unwrap_or_else(|| "asis".to_string());

    Ok(Some((
        "dataset.activate",
        json!({ "name": name, "window": window }),
    )))
}

fn map_dataset_close(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let name = cmd
        .subcommand_tokens("POSITIONAL")
        .and_then(|t| t.first())
        .or_else(|| cmd.subcommand_tokens("NAME").and_then(|t| t.first()))
        .cloned()
        .unwrap_or_else(|| "*".to_string());

    Ok(Some(("dataset.close", json!({ "name": name }))))
}
