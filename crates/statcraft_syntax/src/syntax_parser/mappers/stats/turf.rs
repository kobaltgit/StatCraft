use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;
use std::collections::HashMap;

fn parse_yes_no(val: &str, default: bool) -> bool {
    let u = val.trim().to_ascii_uppercase();
    if u == "YES" || u == "TRUE" || u == "1" {
        true
    } else if u == "NO" || u == "FALSE" || u == "0" {
        false
    } else {
        default
    }
}

fn split_eq_tokens(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for tok in tokens {
        if let Some((left, right)) = tok.split_once('=') {
            if !left.is_empty() {
                out.push(left.to_string());
            }
            out.push("=".to_string());
            if !right.is_empty() {
                out.push(right.to_string());
            }
        } else {
            out.push(tok.clone());
        }
    }
    out
}

fn is_options_key(u: &str) -> bool {
    matches!(
        u,
        "BESTN"
            | "NUMBERTODISPLAY"
            | "CRITERIA"
            | "POSITIVE"
            | "MINPERCENT"
            | "PLOT"
            | "REMOVALCYCLES"
            | "MAXSETS"
            | "MAXOPERATIONS"
            | "SHOWONLYMAX"
            | "ONLYMAX"
    )
}

pub fn map_turf(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_var_tokens = cmd
        .subcommand_tokens("VARIABLES")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);
    let expanded_tokens = split_eq_tokens(raw_var_tokens);

    let mut var_tokens = Vec::new();
    let mut inline_force: Option<usize> = None;
    let mut i = 0;
    while i < expanded_tokens.len() {
        let t = &expanded_tokens[i];
        if t.eq_ignore_ascii_case("FORCE") {
            let mut j = i + 1;
            if expanded_tokens.get(j).is_some_and(|s| s == "=") {
                j += 1;
            }
            if let Some(val_tok) = expanded_tokens.get(j) {
                inline_force = val_tok.parse::<usize>().ok();
                i = j + 1;
                continue;
            }
        }
        var_tokens.push(t.clone());
        i += 1;
    }

    let variables = SyntaxParser::expand_variables(&var_tokens, dataset)?;
    if variables.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VARIABLES".to_string(),
            reason: "At least one variable is required for TURF".to_string(),
        });
    }

    let force = cmd
        .subcommand_tokens("FORCE")
        .and_then(|t| t.iter().find(|s| *s != "="))
        .and_then(|s| s.parse::<usize>().ok())
        .or(inline_force)
        .unwrap_or(0);

    let mut max_combinations: Option<usize> = None;
    let mut num_to_display: Option<usize> = Some(10);
    let mut positive_values: Vec<String> = vec!["1".to_string()];
    let mut min_positive_pct: Option<f64> = None;
    let mut reach_and_frequency_chart = true;
    let mut removal_cycles: usize = 0;
    let mut max_operations: usize = 5_000_000;
    let mut show_only_max_combo = false;

    if let Some(opt_raw) = cmd.subcommand_tokens("OPTIONS") {
        let toks = split_eq_tokens(opt_raw);
        let mut idx = 0;
        while idx < toks.len() {
            let key = toks[idx].to_ascii_uppercase();
            if !is_options_key(&key) {
                idx += 1;
                continue;
            }
            idx += 1;
            if toks.get(idx).is_some_and(|s| s == "=") {
                idx += 1;
            }
            let mut vals = Vec::new();
            while idx < toks.len() && !is_options_key(&toks[idx].to_ascii_uppercase()) {
                if toks[idx] != "=" {
                    vals.push(
                        toks[idx]
                            .trim_matches(|c| c == '\'' || c == '"')
                            .to_string(),
                    );
                }
                idx += 1;
            }
            match key.as_str() {
                "BESTN" => {
                    max_combinations = vals.first().and_then(|v| v.parse::<usize>().ok());
                }
                "NUMBERTODISPLAY" => {
                    num_to_display = vals.first().and_then(|v| v.parse::<usize>().ok());
                }
                "CRITERIA" | "POSITIVE" if !vals.is_empty() => {
                    positive_values = vals;
                }
                "MINPERCENT" => {
                    if let Some(p) = vals.first().and_then(|v| v.parse::<f64>().ok()) {
                        min_positive_pct = if p > 0.0 { Some(p) } else { None };
                    }
                }
                "PLOT" => {
                    if let Some(v) = vals.first() {
                        reach_and_frequency_chart = parse_yes_no(v, true);
                    }
                }
                "REMOVALCYCLES" => {
                    removal_cycles = vals
                        .first()
                        .and_then(|v| v.parse::<usize>().ok())
                        .unwrap_or(0);
                }
                "MAXSETS" | "MAXOPERATIONS" => {
                    if let Some(m) = vals.first().and_then(|v| v.parse::<usize>().ok()) {
                        max_operations = m;
                    }
                }
                "SHOWONLYMAX" | "ONLYMAX" => {
                    if let Some(v) = vals.first() {
                        show_only_max_combo = parse_yes_no(v, false);
                    }
                }
                _ => {}
            }
        }
    }

    let mut use_advanced_selection = true;
    let mut advanced_auto = true;
    let mut advanced_size: usize = 18;
    let mut advanced_winners: usize = 2;

    if let Some(heat_raw) = cmd.subcommand_tokens("HEATS") {
        let toks = split_eq_tokens(heat_raw);
        let mut idx = 0;
        while idx < toks.len() {
            let key = toks[idx].to_ascii_uppercase();
            idx += 1;
            if toks.get(idx).is_some_and(|s| s == "=") {
                idx += 1;
            }
            let Some(val) = toks.get(idx) else {
                break;
            };
            match key.as_str() {
                "USEHEATS" => {
                    use_advanced_selection = parse_yes_no(val, true);
                    idx += 1;
                }
                "HEATSIZE" => {
                    if let Ok(s) = val.parse::<usize>() {
                        advanced_size = s.max(2);
                    }
                    idx += 1;
                }
                "NUMWINNERS" => {
                    if let Ok(w) = val.parse::<usize>() {
                        advanced_winners = w.max(1);
                    }
                    idx += 1;
                }
                "AUTO" => {
                    advanced_auto = parse_yes_no(val, true);
                    idx += 1;
                }
                _ => {}
            }
        }
    }

    let mut weights_strict = true;
    let mut importance_weights: HashMap<String, f64> = HashMap::new();
    if let Some(imp_raw) = cmd.subcommand_tokens("IMPORTANCE") {
        let toks = split_eq_tokens(imp_raw);
        let mut idx = 0;
        while idx < toks.len() {
            let cur = &toks[idx];
            if cur.eq_ignore_ascii_case("STRICT") {
                idx += 1;
                if toks.get(idx).is_some_and(|s| s == "=") {
                    idx += 1;
                }
                if let Some(val) = toks.get(idx) {
                    weights_strict = parse_yes_no(val, true);
                    idx += 1;
                }
            } else if let Some(next) = toks.get(idx + 1) {
                if let Ok(w) = next.parse::<f64>() {
                    importance_weights.insert(cur.clone(), w);
                    idx += 2;
                    continue;
                }
                idx += 1;
            } else {
                idx += 1;
            }
        }
    }

    Ok(Some((
        "stat.turf",
        json!({
            "variables": variables,
            "positive_values": positive_values,
            "max_combinations": max_combinations,
            "num_to_display": num_to_display,
            "min_positive_pct": min_positive_pct,
            "always_required": force,
            "removal_cycles": removal_cycles,
            "show_only_max_combo": show_only_max_combo,
            "limit_exceeded_abort": true,
            "max_operations": max_operations,
            "importance_weights": importance_weights,
            "weights_strict": weights_strict,
            "use_advanced_selection": use_advanced_selection,
            "advanced_auto": advanced_auto,
            "advanced_size": advanced_size,
            "advanced_winners": advanced_winners,
            "reach_and_frequency_chart": reach_and_frequency_chart
        }),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax_parser::lexer::SyntaxLexer;

    #[test]
    fn test_map_spssinc_turf_and_alias() {
        let sps = "SPSSINC TURF VARIABLES = v1 v2 v3 FORCE = 1 /OPTIONS BESTN = 2 NUMBERTODISPLAY = 5 CRITERIA = 1 2 MINPERCENT = 10 PLOT = NO REMOVALCYCLES = 1 MAXSETS = 10000 /HEATS USEHEATS = NO HEATSIZE = 12 NUMWINNERS = 3 /IMPORTANCE v1 2.0 v2 1.5 STRICT = NO.";
        let blocks = SyntaxLexer::split_commands(sps).expect("lex ok");
        let cmd = SyntaxParser::parse_command(&blocks[0]).expect("parse ok");
        let (id, val) = map_turf(&cmd, None).expect("map ok").expect("some");
        assert_eq!(id, "stat.turf");
        assert_eq!(val["variables"], json!(["v1", "v2", "v3"]));
        assert_eq!(val["always_required"], 1);
        assert_eq!(val["max_combinations"], 2);
        assert_eq!(val["num_to_display"], 5);
        assert_eq!(val["positive_values"], json!(["1", "2"]));
        assert_eq!(val["min_positive_pct"], 10.0);
        assert_eq!(val["reach_and_frequency_chart"], false);
        assert_eq!(val["removal_cycles"], 1);
        assert_eq!(val["max_operations"], 10000);
        assert_eq!(val["use_advanced_selection"], false);
        assert_eq!(val["advanced_size"], 12);
        assert_eq!(val["advanced_winners"], 3);
        assert_eq!(val["weights_strict"], false);
        assert_eq!(val["importance_weights"]["v1"], 2.0);
        assert_eq!(val["importance_weights"]["v2"], 1.5);

        let short_sps = "TURF VARIABLES = a b /OPTIONS BESTN = 1.";
        let blocks2 = SyntaxLexer::split_commands(short_sps).expect("lex ok");
        let cmd2 = SyntaxParser::parse_command(&blocks2[0]).expect("parse ok");
        let (id2, val2) = map_turf(&cmd2, None).expect("map ok").expect("some");
        assert_eq!(id2, "stat.turf");
        assert_eq!(val2["variables"], json!(["a", "b"]));
        assert_eq!(val2["max_combinations"], 1);
    }
}
