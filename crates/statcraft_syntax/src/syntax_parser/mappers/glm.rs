use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;

/// Maps General Linear Model (UNIANOVA and GLM) commands to CommandBus invocations.
pub fn map_glm_command(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    match cmd.name.as_str() {
        "UNIANOVA" => map_unianova(cmd, dataset),
        "GLM" | "GLM REPEATED" => map_glm(cmd, dataset),
        _ => Ok(None),
    }
}

fn map_unianova(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("POSITIONAL")
        .or_else(|| cmd.subcommand_tokens("VARIABLES"))
        .unwrap_or(&[]);

    if raw_tokens.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "POSITIONAL".to_string(),
            reason: "UNIANOVA requires a dependent variable".to_string(),
        });
    }

    // Split on BY and WITH keywords
    let by_pos = raw_tokens.iter().position(|t| t.eq_ignore_ascii_case("BY"));
    let with_pos = raw_tokens
        .iter()
        .position(|t| t.eq_ignore_ascii_case("WITH"));

    let (dep_tokens, factor_tokens, cov_tokens) = match (by_pos, with_pos) {
        (Some(b), Some(w)) if b < w => (
            &raw_tokens[..b],
            &raw_tokens[(b + 1)..w],
            &raw_tokens[(w + 1)..],
        ),
        (Some(b), None) => (&raw_tokens[..b], &raw_tokens[(b + 1)..], &[][..]),
        (None, Some(w)) => (&raw_tokens[..w], &[][..], &raw_tokens[(w + 1)..]),
        (None, None) => (raw_tokens, &[][..], &[][..]),
        _ => {
            return Err(SyntaxError::InvalidSubcommand {
                line: cmd.line_num,
                command: cmd.name.clone(),
                subcommand: "VARIABLES".to_string(),
                reason: "Invalid syntax: BY must appear before WITH in UNIANOVA".to_string(),
            });
        }
    };

    let dep_vars = SyntaxParser::expand_variables(dep_tokens, dataset)?;
    if dep_vars.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "VARIABLES".to_string(),
            reason: "Dependent variable name is required".to_string(),
        });
    }
    let dependent = dep_vars[0].clone();

    let factors = if !factor_tokens.is_empty() {
        SyntaxParser::expand_variables(factor_tokens, dataset)?
    } else {
        Vec::new()
    };

    let covariates = if !cov_tokens.is_empty() {
        SyntaxParser::expand_variables(cov_tokens, dataset)?
    } else {
        Vec::new()
    };

    let intercept = if let Some(intercept_tokens) = cmd.subcommand_tokens("INTERCEPT") {
        !intercept_tokens
            .iter()
            .any(|t| t.eq_ignore_ascii_case("EXCLUDE"))
    } else {
        true
    };

    let mut descriptives = true;
    let mut levene = false;
    if let Some(print_tokens) = cmd.subcommand_tokens("PRINT") {
        descriptives = print_tokens
            .iter()
            .any(|t| t.to_ascii_uppercase().contains("DESCRIPT"));
        levene = print_tokens.iter().any(|t| {
            t.to_ascii_uppercase().contains("HOMOGENE") || t.eq_ignore_ascii_case("HOMOGENEITY")
        });
    }

    Ok(Some((
        "stat.glm.univariate",
        json!({
            "dependent": dependent,
            "fixed_factors": factors,
            "covariates": covariates,
            "intercept": intercept,
            "descriptives": descriptives,
            "homogeneity_levene": levene,
        }),
    )))
}

fn map_glm(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let wsfactor_tokens = cmd.subcommand_tokens("WSFACTOR");
    let is_repeated = cmd.name == "GLM REPEATED" || wsfactor_tokens.is_some();

    if is_repeated {
        // Repeated measures
        let raw_tokens = cmd
            .subcommand_tokens("POSITIONAL")
            .or_else(|| cmd.subcommand_tokens("VARIABLES"))
            .unwrap_or(&[]);

        let measures = SyntaxParser::expand_variables(raw_tokens, dataset)?;
        if measures.len() < 2 {
            return Err(SyntaxError::InvalidSubcommand {
                line: cmd.line_num,
                command: cmd.name.clone(),
                subcommand: "VARIABLES".to_string(),
                reason: "Repeated measures GLM requires at least 2 measure variables".to_string(),
            });
        }

        let factor_name = if let Some(tokens) = wsfactor_tokens {
            tokens
                .first()
                .cloned()
                .unwrap_or_else(|| "time".to_string())
        } else {
            "time".to_string()
        };

        Ok(Some((
            "stat.glm.repeated",
            json!({
                "factor_name": factor_name,
                "measures": measures,
            }),
        )))
    } else {
        // Multivariate GLM
        let raw_tokens = cmd
            .subcommand_tokens("POSITIONAL")
            .or_else(|| cmd.subcommand_tokens("VARIABLES"))
            .unwrap_or(&[]);

        let by_pos = raw_tokens.iter().position(|t| t.eq_ignore_ascii_case("BY"));
        let with_pos = raw_tokens
            .iter()
            .position(|t| t.eq_ignore_ascii_case("WITH"));

        let (dep_tokens, factor_tokens, cov_tokens) = match (by_pos, with_pos) {
            (Some(b), Some(w)) if b < w => (
                &raw_tokens[..b],
                &raw_tokens[(b + 1)..w],
                &raw_tokens[(w + 1)..],
            ),
            (Some(b), None) => (&raw_tokens[..b], &raw_tokens[(b + 1)..], &[][..]),
            (None, Some(w)) => (&raw_tokens[..w], &[][..], &raw_tokens[(w + 1)..]),
            (None, None) => (raw_tokens, &[][..], &[][..]),
            _ => {
                return Err(SyntaxError::InvalidSubcommand {
                    line: cmd.line_num,
                    command: cmd.name.clone(),
                    subcommand: "VARIABLES".to_string(),
                    reason: "Invalid syntax: BY must appear before WITH in GLM".to_string(),
                });
            }
        };

        let dependents = SyntaxParser::expand_variables(dep_tokens, dataset)?;
        if dependents.len() < 2 {
            return Err(SyntaxError::InvalidSubcommand {
                line: cmd.line_num,
                command: cmd.name.clone(),
                subcommand: "VARIABLES".to_string(),
                reason: "Multivariate GLM requires at least 2 dependent variables".to_string(),
            });
        }

        let factors = if !factor_tokens.is_empty() {
            SyntaxParser::expand_variables(factor_tokens, dataset)?
        } else {
            Vec::new()
        };

        let covariates = if !cov_tokens.is_empty() {
            SyntaxParser::expand_variables(cov_tokens, dataset)?
        } else {
            Vec::new()
        };

        let intercept = if let Some(intercept_tokens) = cmd.subcommand_tokens("INTERCEPT") {
            !intercept_tokens
                .iter()
                .any(|t| t.eq_ignore_ascii_case("EXCLUDE"))
        } else {
            true
        };

        Ok(Some((
            "stat.glm.multivariate",
            json!({
                "dependents": dependents,
                "factors": factors,
                "covariates": covariates,
                "intercept": intercept,
            }),
        )))
    }
}
