pub mod crosstabs;
pub mod frequencies;
pub mod ttest;

pub use frequencies::map_frequencies;
pub use ttest::{map_ttest, parse_ci_from_criteria};

use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;

pub fn map_stat_command(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    match cmd.name.as_str() {
        "DESCRIPTIVES" => map_descriptives(cmd, dataset),
        "FREQUENCIES" => map_frequencies(cmd, dataset),
        "EXAMINE" | "EXPLORE" => map_explore(cmd, dataset),
        "CROSSTABS" => crosstabs::map_crosstabs(cmd, dataset),
        "T-TEST" => map_ttest(cmd, dataset),
        "ONEWAY" => map_oneway(cmd, dataset),
        "CORRELATIONS" => map_correlations(cmd, dataset),
        "REGRESSION" => map_regression(cmd, dataset),
        "RELIABILITY" => map_reliability(cmd, dataset),
        "FACTOR" => map_factor(cmd, dataset),
        "MEANS" => map_means(cmd, dataset),
        _ => Ok(None),
    }
}

fn map_descriptives(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_vars = cmd
        .subcommand_tokens("VARIABLES")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    let vars = SyntaxParser::expand_variables(raw_vars, dataset)?;
    if vars.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VARIABLES".to_string(),
            reason: "Variable list cannot be empty".to_string(),
        });
    }

    let has_stats = cmd.has_subcommand("STATISTICS");
    let is_all = cmd.has_token("STATISTICS", "ALL");
    let is_default = !has_stats || cmd.has_token("STATISTICS", "DEFAULT");
    let save_standardized = cmd.has_subcommand("SAVE");
    let missing = if cmd.has_token("MISSING", "LISTWISE") {
        "Listwise"
    } else {
        "Variable"
    };

    let sort = if let Some(sort_tokens) = cmd.subcommand_tokens("SORT") {
        let has_mean = sort_tokens.iter().any(|t| t.eq_ignore_ascii_case("MEAN"));
        let has_name = sort_tokens.iter().any(|t| t.eq_ignore_ascii_case("NAME"));
        let is_desc = sort_tokens.iter().any(|t| {
            let u = t.to_ascii_uppercase();
            u == "(D)" || u == "D" || u == "DESC" || u == "DESCENDING"
        });
        if has_mean {
            if is_desc {
                "DescendingMeans"
            } else {
                "AscendingMeans"
            }
        } else if has_name {
            if is_desc {
                "AlphabeticDesc"
            } else {
                "Alphabetic"
            }
        } else {
            "VariableList"
        }
    } else {
        "VariableList"
    };

    let opts = json!({
        "mean": is_all || is_default || cmd.has_token("STATISTICS", "MEAN"),
        "std_dev": is_all || is_default || cmd.has_token("STATISTICS", "STDDEV"),
        "min": is_all || is_default || cmd.has_token("STATISTICS", "MIN"),
        "max": is_all || is_default || cmd.has_token("STATISTICS", "MAX"),
        "variance": is_all || cmd.has_token("STATISTICS", "VARIANCE"),
        "range": is_all || cmd.has_token("STATISTICS", "RANGE"),
        "sum": is_all || cmd.has_token("STATISTICS", "SUM"),
        "se_mean": is_all || cmd.has_token("STATISTICS", "SEMEAN"),
        "skewness": is_all || cmd.has_token("STATISTICS", "SKEWNESS"),
        "kurtosis": is_all || cmd.has_token("STATISTICS", "KURTOSIS"),
        "save_standardized": save_standardized,
        "sort": sort,
        "missing": missing
    });

    Ok(Some((
        "stat.descriptives",
        json!({ "vars": vars, "options": opts }),
    )))
}

fn map_oneway(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("POSITIONAL")
        .or_else(|| cmd.subcommand_tokens("VARIABLES"))
        .unwrap_or(&[]);

    let by_pos = raw_tokens
        .iter()
        .position(|t| t.eq_ignore_ascii_case("BY"))
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "ONEWAY".to_string(),
            reason: "ONEWAY requires 'dep_vars BY factor_var'".to_string(),
        })?;

    let dep_tokens = &raw_tokens[..by_pos];
    let factor_tokens = &raw_tokens[by_pos + 1..];

    let dep_vars = SyntaxParser::expand_variables(dep_tokens, dataset)?;
    let factor_var = factor_tokens.first().cloned().unwrap_or_default();

    let ci = parse_ci_from_criteria(cmd);
    let desc = cmd.has_token("STATISTICS", "DESCRIPTIVES");
    let homog = cmd.has_token("STATISTICS", "HOMOGENEITY");

    Ok(Some((
        "stat.anova.oneway",
        json!({
            "dependent_vars": dep_vars,
            "factor_var": factor_var,
            "ci": ci,
            "descriptives": desc,
            "homogeneity": homog
        }),
    )))
}

fn map_correlations(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_vars = cmd.subcommand_tokens("VARIABLES").unwrap_or(&[]);
    let vars = SyntaxParser::expand_variables(raw_vars, dataset)?;

    let two_tailed = !cmd.has_token("PRINT", "ONETAIL");
    let flag_sig = !cmd.has_token("PRINT", "NOSIG");
    let desc = cmd.has_token("STATISTICS", "DESCRIPTIVES");
    let xprod = cmd.has_token("STATISTICS", "XPROD");

    Ok(Some((
        "stat.correlations",
        json!({
            "vars": vars,
            "pearson": true,
            "spearman": false,
            "kendall": false,
            "two_tailed": two_tailed,
            "flag_significant": flag_sig,
            "descriptives": desc,
            "cross_prod_cov": xprod,
            "missing": "pairwise"
        }),
    )))
}

fn map_regression(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let dep_var = cmd
        .subcommand_tokens("DEPENDENT")
        .and_then(|t| t.first())
        .cloned()
        .unwrap_or_default();

    let method_tokens = cmd.subcommand_tokens("METHOD").unwrap_or(&[]);
    let indep_tokens =
        if !method_tokens.is_empty() && method_tokens[0].eq_ignore_ascii_case("ENTER") {
            &method_tokens[1..]
        } else {
            method_tokens
        };

    let indep_vars = SyntaxParser::expand_variables(indep_tokens, dataset)?;

    Ok(Some((
        "stat.regression.linear",
        json!({
            "dependent_var": dep_var,
            "independent_vars": indep_vars,
            "ci": 0.95,
            "collinearity": true
        }),
    )))
}

fn map_reliability(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_vars = cmd.subcommand_tokens("VARIABLES").unwrap_or(&[]);
    let vars = SyntaxParser::expand_variables(raw_vars, dataset)?;

    Ok(Some((
        "stat.scale.reliability",
        json!({ "variables": vars }),
    )))
}

fn map_factor(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_vars = cmd.subcommand_tokens("VARIABLES").unwrap_or(&[]);
    let vars = SyntaxParser::expand_variables(raw_vars, dataset)?;

    Ok(Some(("stat.factor", json!({ "variables": vars }))))
}

fn map_means(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("TABLES")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    let by_pos = raw_tokens
        .iter()
        .position(|t| t.eq_ignore_ascii_case("BY"))
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "TABLES".to_string(),
            reason: "MEANS requires 'dep_vars BY factor_vars'".to_string(),
        })?;

    let dep_tokens = &raw_tokens[..by_pos];
    let factor_tokens = &raw_tokens[by_pos + 1..];

    let dep_vars = SyntaxParser::expand_variables(dep_tokens, dataset)?;
    let factor_vars = SyntaxParser::expand_variables(factor_tokens, dataset)?;

    let anova = cmd.has_token("STATISTICS", "ANOVA");

    Ok(Some((
        "stat.means",
        json!({
            "dependent_vars": dep_vars, "factor_vars": factor_vars,
            "cell_mean": true, "cell_count": true, "cell_stddev": true,
            "cell_variance": false, "cell_median": false, "cell_min": false,
            "cell_max": false, "anova": anova
        }),
    )))
}

fn parse_explore_cinterval(cmd: &ParsedCommand) -> f64 {
    if let Some(tokens) = cmd.subcommand_tokens("CINTERVAL") {
        for tok in tokens {
            let clean = tok.trim_matches(|c| c == '(' || c == ')' || c == '=');
            if let Ok(val) = clean.parse::<f64>() {
                if val.is_finite() && val > 0.0 {
                    return if val > 1.0 { val / 100.0 } else { val };
                }
            }
        }
    }
    parse_ci_from_criteria(cmd)
}

fn parse_explore_spread_level(cmd: &ParsedCommand) -> (&'static str, Option<f64>) {
    if cmd.has_token("PLOT", "NONE") {
        return ("none", None);
    }
    if cmd.has_token("PLOT", "ALL") {
        return ("power_estimation", None);
    }
    let Some(tokens) = cmd.subcommand_tokens("PLOT") else {
        return ("none", None);
    };
    let mut found_sl = false;
    for tok in tokens {
        let upper = tok.to_ascii_uppercase();
        if upper == "SPREADLEVEL" {
            found_sl = true;
            continue;
        }
        if upper.starts_with("SPREADLEVEL(") {
            let inner = upper
                .trim_start_matches("SPREADLEVEL(")
                .trim_end_matches(')');
            if inner == "LN" || inner == "0" {
                return ("natural_log", Some(0.0));
            }
            if let Ok(p) = inner.parse::<f64>() {
                return if p.abs() < 1e-12 {
                    ("natural_log", Some(0.0))
                } else {
                    ("power", Some(p))
                };
            }
            return ("power_estimation", None);
        }
        if found_sl {
            let clean = tok.trim_matches(|c| c == '(' || c == ')');
            if clean.eq_ignore_ascii_case("LN") {
                return ("natural_log", Some(0.0));
            }
            if let Ok(p) = clean.parse::<f64>() {
                return if p.abs() < 1e-12 {
                    ("natural_log", Some(0.0))
                } else {
                    ("power", Some(p))
                };
            }
            return ("power_estimation", None);
        }
    }
    if found_sl {
        ("power_estimation", None)
    } else {
        ("none", None)
    }
}

fn map_explore(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw = cmd
        .subcommand_tokens("VARIABLES")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    let (dep_tokens, factor_tokens) = match raw.iter().position(|t| t.eq_ignore_ascii_case("BY")) {
        Some(pos) => (&raw[..pos], &raw[pos + 1..]),
        None => (raw, &[][..]),
    };

    let dep_vars = SyntaxParser::expand_variables(dep_tokens, dataset)?;
    if dep_vars.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VARIABLES".to_string(),
            reason: "Dependent variable list cannot be empty".to_string(),
        });
    }

    let factor_vars = SyntaxParser::expand_variables(factor_tokens, dataset)?;
    let label_var = cmd
        .subcommand_tokens("ID")
        .and_then(|t| t.iter().find(|s| *s != "="))
        .cloned();

    let normality = !cmd.has_token("PLOT", "NONE")
        && (cmd.has_token("PLOT", "NPPLOT") || cmd.has_token("PLOT", "ALL"));
    let (spread_level_mode, spread_level_power) = parse_explore_spread_level(cmd);

    let (desc, ext, mest, pct) = if cmd.has_subcommand("STATISTICS") {
        if cmd.has_token("STATISTICS", "NONE") {
            (false, false, false, false)
        } else {
            let all = cmd.has_token("STATISTICS", "ALL");
            let has_ext =
                cmd.has_token("STATISTICS", "EXTREME") || cmd.has_token("STATISTICS", "EXTREMES");
            let has_mest = cmd.has_token("STATISTICS", "MESTIMATORS");
            let has_pct = cmd.has_token("STATISTICS", "PERCENTILES");
            let has_desc =
                cmd.has_token("STATISTICS", "DESCRIPTIVES") || (!has_ext && !has_mest && !has_pct);
            (
                all || has_desc,
                all || has_ext,
                all || has_mest,
                all || has_pct,
            )
        }
    } else {
        (true, true, false, false)
    };

    let missing = if cmd.has_token("MISSING", "PAIRWISE") {
        "pairwise"
    } else if cmd.has_token("MISSING", "REPORT") {
        "report"
    } else {
        "listwise"
    };

    let ci = parse_explore_cinterval(cmd);

    let mut payload = json!({
        "dependent_vars": dep_vars,
        "factor_vars": factor_vars,
        "compute_descriptives": desc,
        "compute_m_estimators": mest,
        "compute_percentiles": pct,
        "compute_normality": normality,
        "compute_extremes": ext,
        "spread_level_mode": spread_level_mode,
        "confidence_level": ci,
        "missing": missing,
    });
    if let Some(lv) = label_var {
        payload["label_var"] = json!(lv);
    }
    if let Some(p) = spread_level_power {
        payload["spread_level_power"] = json!(p);
    }

    Ok(Some(("stat.explore", payload)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax_parser::lexer::SyntaxLexer;

    #[test]
    fn test_map_explore_examine() {
        let blocks =
            SyntaxLexer::split_commands("EXAMINE VARIABLES=salary BY gender /PLOT NPPLOT.")
                .expect("lex ok");
        let cmd = SyntaxParser::parse_command(&blocks[0]).expect("parse ok");
        let (id, val) = map_explore(&cmd, None).expect("map ok").expect("some");
        assert_eq!(id, "stat.explore");
        assert_eq!(val["dependent_vars"], json!(["salary"]));
        assert_eq!(val["factor_vars"], json!(["gender"]));
        assert_eq!(val["compute_normality"], true);
    }
}
