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
        "CROSSTABS" => map_crosstabs(cmd, dataset),
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
    let opts = if !has_stats || cmd.has_token("STATISTICS", "DEFAULT") {
        json!({
            "mean": true,
            "std_dev": true,
            "min": true,
            "max": true,
            "variance": cmd.has_token("STATISTICS", "VARIANCE"),
            "range": cmd.has_token("STATISTICS", "RANGE"),
            "se_mean": cmd.has_token("STATISTICS", "SEMEAN"),
            "skewness": cmd.has_token("STATISTICS", "SKEWNESS"),
            "kurtosis": cmd.has_token("STATISTICS", "KURTOSIS")
        })
    } else {
        json!({
            "mean": cmd.has_token("STATISTICS", "MEAN"),
            "std_dev": cmd.has_token("STATISTICS", "STDDEV"),
            "variance": cmd.has_token("STATISTICS", "VARIANCE"),
            "min": cmd.has_token("STATISTICS", "MIN"),
            "max": cmd.has_token("STATISTICS", "MAX"),
            "range": cmd.has_token("STATISTICS", "RANGE"),
            "se_mean": cmd.has_token("STATISTICS", "SEMEAN"),
            "skewness": cmd.has_token("STATISTICS", "SKEWNESS"),
            "kurtosis": cmd.has_token("STATISTICS", "KURTOSIS")
        })
    };

    Ok(Some((
        "stat.descriptives",
        json!({ "vars": vars, "options": opts }),
    )))
}

fn map_frequencies(
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

    Ok(Some(("stat.frequencies", json!({ "vars": vars }))))
}

fn map_crosstabs(
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
            reason: "CROSSTABS requires 'row_vars BY col_vars'".to_string(),
        })?;

    let row_tokens = &raw_tokens[..by_pos];
    let col_tokens = &raw_tokens[by_pos + 1..];

    let rows = SyntaxParser::expand_variables(row_tokens, dataset)?;
    let cols = SyntaxParser::expand_variables(col_tokens, dataset)?;

    let chisq = cmd.has_token("STATISTICS", "CHISQ");
    let phi = cmd.has_token("STATISTICS", "PHI") || cmd.has_token("STATISTICS", "CC");

    Ok(Some((
        "stat.crosstabs",
        json!({
            "rows": rows,
            "cols": cols,
            "chisq": chisq,
            "phi_cramer": phi,
            "expected": true,
            "row_pct": true,
            "col_pct": true,
            "total_pct": true,
            "residuals": true
        }),
    )))
}

fn map_ttest(
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
                "vars": vars,
                "grouping_var": grp_var,
                "group1": v1,
                "group2": v2,
                "ci": ci
            }),
        )))
    } else if let Some(pairs_sub) = cmd.subcommand("PAIRS") {
        // Paired samples: PAIRS=v1 WITH v2 (PAIRED)
        let pairs = parse_pairs_subcommand(pairs_sub, cmd.line_num)?;
        Ok(Some((
            "stat.ttest.paired",
            json!({
                "pairs": pairs,
                "ci": ci
            }),
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
            json!({
                "vars": vars,
                "test_value": testval,
                "ci": ci
            }),
        )))
    }
}

fn parse_ci_from_criteria(cmd: &ParsedCommand) -> f64 {
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

fn parse_groups_subcommand(
    sub: &crate::syntax_parser::parser::Subcommand,
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

fn parse_pairs_subcommand(
    sub: &crate::syntax_parser::parser::Subcommand,
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
            "dependent_vars": dep_vars,
            "factor_vars": factor_vars,
            "cell_mean": true,
            "cell_count": true,
            "cell_stddev": true,
            "cell_variance": false,
            "cell_median": false,
            "cell_min": false,
            "cell_max": false,
            "anova": anova
        }),
    )))
}
