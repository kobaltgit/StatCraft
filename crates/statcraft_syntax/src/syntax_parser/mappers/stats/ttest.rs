use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, Subcommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;

pub fn map_ttest(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let ci = parse_ci_from_criteria(cmd);

    if let Some(grp_sub) = cmd.subcommand("GROUPS") {
        // Independent samples: GROUPS=var(v1 v2) /VARIABLES=vars
        let (grp_var, v1, v2) = parse_groups_subcommand(grp_sub, cmd.line_num)?;
        let raw_vars = cmd.subcommand_tokens("VARIABLES").unwrap_or(&[]);
        let vars = SyntaxParser::expand_variables(raw_vars, dataset)?;

        Ok(Some((
            "stat.ttest.indep",
            json!({
                "vars": vars, "grouping_var": grp_var,
                "group1": v1, "group2": v2, "ci": ci
            }),
        )))
    } else if let Some(pairs_sub) = cmd.subcommand("PAIRS") {
        // Paired samples: PAIRS=v1 WITH v2 (PAIRED)
        let pairs = parse_pairs_subcommand(pairs_sub, cmd.line_num)?;
        Ok(Some((
            "stat.ttest.paired",
            json!({ "pairs": pairs, "ci": ci }),
        )))
    } else {
        // One sample: /TESTVAL=val /VARIABLES=vars
        let testval = cmd
            .subcommand_value("TESTVAL")
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or(0.0);

        let raw_vars = cmd.subcommand_tokens("VARIABLES").unwrap_or(&[]);
        let vars = SyntaxParser::expand_variables(raw_vars, dataset)?;

        Ok(Some((
            "stat.ttest.onesample",
            json!({ "vars": vars, "test_value": testval, "ci": ci }),
        )))
    }
}

pub fn parse_ci_from_criteria(cmd: &ParsedCommand) -> f64 {
    cmd.subcommand("CRITERIA")
        .and_then(|sub| {
            for token in &sub.tokens {
                if let Some(inner) = token.strip_prefix("CI(").and_then(|s| s.strip_suffix(')')) {
                    if let Ok(v) = inner.parse::<f64>() {
                        return Some(v);
                    }
                }
            }
            None
        })
        .unwrap_or(0.95)
}

pub fn parse_groups_subcommand(
    sub: &Subcommand,
    line: usize,
) -> Result<(String, String, String), SyntaxError> {
    let raw = &sub.raw_value;
    let paren_start = raw
        .find('(')
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line,
            command: "T-TEST".to_string(),
            subcommand: "GROUPS".to_string(),
            reason: "Expected GROUPS=variable(val1 val2)".to_string(),
        })?;
    let paren_end = raw
        .find(')')
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line,
            command: "T-TEST".to_string(),
            subcommand: "GROUPS".to_string(),
            reason: "Missing closing parenthesis in GROUPS".to_string(),
        })?;

    let var_name = raw[..paren_start].trim().to_string();
    let inside = &raw[paren_start + 1..paren_end];
    let vals: Vec<&str> = inside.split_whitespace().collect();
    if vals.len() < 2 {
        return Err(SyntaxError::InvalidSubcommand {
            line,
            command: "T-TEST".to_string(),
            subcommand: "GROUPS".to_string(),
            reason: "GROUPS requires two group values inside parentheses".to_string(),
        });
    }

    Ok((var_name, vals[0].to_string(), vals[1].to_string()))
}

pub fn parse_pairs_subcommand(
    sub: &Subcommand,
    _line: usize,
) -> Result<Vec<(String, String)>, SyntaxError> {
    let tokens: Vec<String> = sub
        .tokens
        .iter()
        .filter(|t| !t.eq_ignore_ascii_case("(PAIRED)"))
        .cloned()
        .collect();

    let mut pairs = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if i + 2 < tokens.len() && tokens[i + 1].eq_ignore_ascii_case("WITH") {
            pairs.push((tokens[i].clone(), tokens[i + 2].clone()));
            i += 3;
        } else {
            i += 1;
        }
    }

    Ok(pairs)
}
