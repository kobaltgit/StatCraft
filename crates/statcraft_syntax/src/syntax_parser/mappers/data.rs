use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;

pub fn map_data_command(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    match cmd.name.as_str() {
        "SORT CASES" => map_sort_cases(cmd, dataset),
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
