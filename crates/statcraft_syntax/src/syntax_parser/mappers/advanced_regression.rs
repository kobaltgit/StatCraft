use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;

/// Maps advanced regression and partial correlation commands to CommandBus invocations.
pub fn map_advanced_regression_command(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    match cmd.name.as_str() {
        "PARTIAL CORR" | "PARTIAL CORRELATIONS" => map_partial_corr(cmd, dataset),
        "LOGISTIC REGRESSION" | "LOGISTIC" => map_logistic_regression(cmd, dataset),
        "CURVEFIT" => map_curvefit(cmd, dataset),
        _ => Ok(None),
    }
}

fn map_partial_corr(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("VARIABLES")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    // Split on WITH keyword
    let with_pos = raw_tokens
        .iter()
        .position(|t| t.eq_ignore_ascii_case("WITH"));

    let (vars_tokens, controls_tokens) = match with_pos {
        Some(pos) => (&raw_tokens[..pos], &raw_tokens[(pos + 1)..]),
        None => {
            return Err(SyntaxError::InvalidSubcommand {
                line: cmd.line_num,
                command: cmd.name.clone(),
                subcommand: "VARIABLES".to_string(),
                reason:
                    "PARTIAL CORR requires 'WITH' keyword between analyzed and control variables"
                        .to_string(),
            });
        }
    };

    let vars = SyntaxParser::expand_variables(vars_tokens, dataset)?;
    let controls = SyntaxParser::expand_variables(controls_tokens, dataset)?;

    if vars.len() < 2 {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VARIABLES".to_string(),
            reason: "At least two analyzed variables required before WITH".to_string(),
        });
    }

    if controls.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VARIABLES".to_string(),
            reason: "At least one control variable required after WITH".to_string(),
        });
    }

    let two_tailed = !cmd.has_token("SIGNIFICANCE", "ONETAIL");
    let missing = if cmd.has_token("MISSING", "PAIRWISE") {
        "pairwise"
    } else {
        "listwise"
    };

    let descriptives = cmd.has_token("STATISTICS", "DESCRIPTIVES");
    let zero_order = cmd.has_token("STATISTICS", "CORR") || cmd.has_token("STATISTICS", "ALL");

    Ok(Some((
        "stat.correlations.partial",
        json!({
            "vars": vars,
            "controls": controls,
            "two_tailed": two_tailed,
            "missing": missing,
            "descriptives": descriptives,
            "zero_order": zero_order
        }),
    )))
}

fn map_logistic_regression(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let dep_tokens = cmd
        .subcommand_tokens("VARIABLES")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    let dep_vars = SyntaxParser::expand_variables(dep_tokens, dataset)?;
    if dep_vars.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VARIABLES".to_string(),
            reason: "Dependent variable is required".to_string(),
        });
    }
    let dep_var = dep_vars[0].clone();

    // Covariates from METHOD=ENTER
    let method_tokens = cmd.subcommand_tokens("METHOD").unwrap_or(&[]);
    let cov_tokens = if !method_tokens.is_empty() && method_tokens[0].eq_ignore_ascii_case("ENTER")
    {
        &method_tokens[1..]
    } else {
        method_tokens
    };

    let covariates = SyntaxParser::expand_variables(cov_tokens, dataset)?;
    if covariates.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "METHOD".to_string(),
            reason: "At least one covariate predictor required in /METHOD=ENTER".to_string(),
        });
    }

    let mut cutoff = 0.5;
    let mut ci = 0.95;

    // Parse CRITERIA for CUT and CI
    if let Some(crit_tokens) = cmd.subcommand_tokens("CRITERIA") {
        for t in crit_tokens {
            let upper = t.to_uppercase();
            if upper.starts_with("CUT(") && upper.ends_with(')') {
                let inner = &upper[4..upper.len() - 1];
                if let Ok(c) = inner.parse::<f64>() {
                    cutoff = c;
                }
            } else if upper.starts_with("CI(") && upper.ends_with(')') {
                let inner = &upper[3..upper.len() - 1];
                if let Ok(c) = inner.parse::<f64>() {
                    ci = if c > 1.0 { c / 100.0 } else { c };
                }
            }
        }
    }

    let hosmer_lemeshow = cmd.has_token("PRINT", "GOODFIT");
    let include_constant = !cmd.has_subcommand("NOORIGIN") && !cmd.has_token("CRITERIA", "NOCONST");

    Ok(Some((
        "stat.regression.logistic",
        json!({
            "dependent_var": dep_var,
            "covariates": covariates,
            "cutoff": cutoff,
            "ci": ci,
            "hosmer_lemeshow": hosmer_lemeshow,
            "include_constant": include_constant
        }),
    )))
}

fn map_curvefit(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("VARIABLES")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    let with_pos = raw_tokens
        .iter()
        .position(|t| t.eq_ignore_ascii_case("WITH"));

    let (dep_tokens, indep_tokens) = match with_pos {
        Some(pos) => (&raw_tokens[..pos], &raw_tokens[(pos + 1)..]),
        None => {
            return Err(SyntaxError::InvalidSubcommand {
                line: cmd.line_num,
                command: cmd.name.clone(),
                subcommand: "VARIABLES".to_string(),
                reason: "CURVEFIT requires 'WITH' between dependent and independent variable"
                    .to_string(),
            });
        }
    };

    let dep_vars = SyntaxParser::expand_variables(dep_tokens, dataset)?;
    let indep_vars = SyntaxParser::expand_variables(indep_tokens, dataset)?;

    if dep_vars.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VARIABLES".to_string(),
            reason: "Dependent variable is required before WITH".to_string(),
        });
    }

    if indep_vars.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VARIABLES".to_string(),
            reason: "Independent variable is required after WITH".to_string(),
        });
    }

    // Models
    let model_tokens = cmd.subcommand_tokens("MODEL").unwrap_or(&[]);
    let mut models = Vec::new();
    if model_tokens.is_empty() || cmd.has_token("MODEL", "ALL") {
        models = vec![
            "linear",
            "logarithmic",
            "inverse",
            "quadratic",
            "power",
            "exponential",
        ];
    } else {
        for t in model_tokens {
            match t.to_uppercase().as_str() {
                "LINEAR" => models.push("linear"),
                "LOGARITHMIC" | "LOG" => models.push("logarithmic"),
                "INVERSE" | "INV" => models.push("inverse"),
                "QUADRATIC" | "QUAD" => models.push("quadratic"),
                "POWER" => models.push("power"),
                "EXPONENTIAL" | "EXP" => models.push("exponential"),
                _ => {}
            }
        }
    }

    if models.is_empty() {
        models = vec!["linear"];
    }

    let include_constant = !cmd.has_subcommand("NOCONSTANT");

    Ok(Some((
        "stat.regression.curve",
        json!({
            "dependent_var": dep_vars[0],
            "independent_var": indep_vars[0],
            "models": models,
            "include_constant": include_constant
        }),
    )))
}
