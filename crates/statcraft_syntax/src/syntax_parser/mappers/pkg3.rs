use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;

/// Maps Package 3 commands (QUICK CLUSTER, CODEBOOK, SUMMARIZE, KM, SURVIVAL, COXREG) to CommandBus.
pub fn map_pkg3_command(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    match cmd.name.as_str() {
        "QUICK CLUSTER" | "KMEANS" => map_quick_cluster(cmd, dataset),
        "CODEBOOK" => map_codebook(cmd, dataset),
        "SUMMARIZE" => map_summarize(cmd, dataset),
        "KM" | "KAPLAN-MEIER" => map_km(cmd, dataset),
        "SURVIVAL" => map_survival(cmd, dataset),
        "COXREG" | "COX REGRESSION" => map_coxreg(cmd, dataset),
        _ => Ok(None),
    }
}

fn map_quick_cluster(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("POSITIONAL")
        .or_else(|| cmd.subcommand_tokens("VARIABLES"))
        .unwrap_or(&[]);

    let variables = SyntaxParser::expand_variables(raw_tokens, dataset)?;
    if variables.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "POSITIONAL".to_string(),
            reason: "QUICK CLUSTER requires at least one variable".to_string(),
        });
    }

    let mut k = 3usize;
    let mut max_iter = 10usize;
    let mut converge = 0.0001f64;

    if let Some(crit_tokens) = cmd.subcommand_tokens("CRITERIA") {
        for token in crit_tokens {
            let upper = token.to_uppercase();
            if upper.starts_with("CLUSTERS(") && upper.ends_with(')') {
                let inner = &upper[9..upper.len() - 1];
                if let Ok(num) = inner.parse::<usize>() {
                    k = num.max(2);
                }
            } else if upper.starts_with("MXITER(") && upper.ends_with(')') {
                let inner = &upper[7..upper.len() - 1];
                if let Ok(num) = inner.parse::<usize>() {
                    max_iter = num.max(1);
                }
            } else if upper.starts_with("CONVERGE(") && upper.ends_with(')') {
                let inner = &upper[9..upper.len() - 1];
                if let Ok(num) = inner.parse::<f64>() {
                    converge = num.max(0.0);
                }
            }
        }
    }

    let mut save_cluster = false;
    let mut save_distance = false;

    if let Some(save_tokens) = cmd.subcommand_tokens("SAVE") {
        for token in save_tokens {
            let upper = token.to_uppercase();
            if upper == "CLUSTER" {
                save_cluster = true;
            } else if upper == "DISTANCE" {
                save_distance = true;
            }
        }
    }

    Ok(Some((
        "stat.cluster.kmeans",
        json!({
            "variables": variables,
            "clusters": k,
            "max_iterations": max_iter,
            "convergence_criterion": converge,
            "save_cluster": save_cluster,
            "save_distance": save_distance,
        }),
    )))
}

fn map_codebook(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("POSITIONAL")
        .or_else(|| cmd.subcommand_tokens("VARIABLES"))
        .unwrap_or(&[]);

    let variables = if raw_tokens.iter().any(|t| t.eq_ignore_ascii_case("ALL")) {
        Vec::new()
    } else {
        SyntaxParser::expand_variables(raw_tokens, dataset)?
    };

    let mut max_cats = 200usize;
    if let Some(opt_tokens) = cmd.subcommand_tokens("OPTIONS") {
        for token in opt_tokens {
            let upper = token.to_uppercase();
            if upper.starts_with("MAXCATS(") && upper.ends_with(')') {
                let inner = &upper[8..upper.len() - 1];
                if let Ok(num) = inner.parse::<usize>() {
                    max_cats = num;
                }
            }
        }
    }

    Ok(Some((
        "stat.reports.codebook",
        json!({
            "variables": variables,
            "max_categories": max_cats,
            "include_counts": true,
            "include_stats": true,
        }),
    )))
}

fn map_summarize(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("TABLES")
        .or_else(|| cmd.subcommand_tokens("VARIABLES"))
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    if raw_tokens.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "TABLES".to_string(),
            reason: "SUMMARIZE requires variables to list".to_string(),
        });
    }

    let by_pos = raw_tokens.iter().position(|t| t.eq_ignore_ascii_case("BY"));
    let (var_tokens, grp_tokens) = match by_pos {
        Some(pos) => (&raw_tokens[..pos], &raw_tokens[(pos + 1)..]),
        None => (raw_tokens, &[][..]),
    };

    let variables = SyntaxParser::expand_variables(var_tokens, dataset)?;
    let group_by = SyntaxParser::expand_variables(grp_tokens, dataset)?;

    let mut limit_cases = Some(100usize);
    if let Some(fmt_tokens) = cmd.subcommand_tokens("FORMAT") {
        for token in fmt_tokens {
            let upper = token.to_uppercase();
            if upper.starts_with("LIMIT(") && upper.ends_with(')') {
                let inner = &upper[6..upper.len() - 1];
                if let Ok(num) = inner.parse::<usize>() {
                    limit_cases = Some(num);
                }
            } else if upper == "NOLIMIT" {
                limit_cases = None;
            }
        }
    }

    Ok(Some((
        "stat.reports.summarize",
        json!({
            "variables": variables,
            "group_by": group_by,
            "limit_cases": limit_cases,
            "display_cases": true,
        }),
    )))
}

fn map_km(
    cmd: &ParsedCommand,
    _dataset: Option<&DataSet>,
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
            reason: "KM requires a time variable".to_string(),
        });
    }

    let by_pos = raw_tokens.iter().position(|t| t.eq_ignore_ascii_case("BY"));
    let (time_tokens, factor_tokens) = match by_pos {
        Some(pos) => (&raw_tokens[..pos], &raw_tokens[(pos + 1)..]),
        None => (raw_tokens, &[][..]),
    };

    let time_var = time_tokens.first().cloned().unwrap_or_default();
    let factor_var = factor_tokens.first().cloned();

    let mut status_var = "status".to_string();
    let mut event_values = vec![1.0];

    if let Some(status_tokens) = cmd.subcommand_tokens("STATUS") {
        if let Some(first) = status_tokens.first() {
            let upper = first.to_uppercase();
            if let Some(open_p) = upper.find('(') {
                if let Some(close_p) = upper.find(')') {
                    status_var = first[..open_p].to_string();
                    let inner = &first[open_p + 1..close_p];
                    if let Ok(val) = inner.parse::<f64>() {
                        event_values = vec![val];
                    }
                }
            } else {
                status_var = first.clone();
            }
        }
    }

    Ok(Some((
        "stat.survival.km",
        json!({
            "time_var": time_var,
            "status_var": status_var,
            "factor_var": factor_var,
            "event_values": event_values,
        }),
    )))
}

fn map_survival(
    cmd: &ParsedCommand,
    _dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let time_var = cmd
        .subcommand_tokens("TABLE")
        .or_else(|| cmd.subcommand_tokens("TABLES"))
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .and_then(|tokens| tokens.first().cloned())
        .unwrap_or_default();

    let mut status_var = "status".to_string();
    let mut event_values = vec![1.0];

    if let Some(status_tokens) = cmd.subcommand_tokens("STATUS") {
        if let Some(first) = status_tokens.first() {
            let upper = first.to_uppercase();
            if let Some(open_p) = upper.find('(') {
                if let Some(close_p) = upper.find(')') {
                    status_var = first[..open_p].to_string();
                    let inner = &first[open_p + 1..close_p];
                    if let Ok(val) = inner.parse::<f64>() {
                        event_values = vec![val];
                    }
                }
            } else {
                status_var = first.clone();
            }
        }
    }

    let mut end = 60.0;
    let mut width = 10.0;

    if let Some(int_tokens) = cmd.subcommand_tokens("INTERVAL") {
        let mut thru_seen = false;
        let mut by_seen = false;
        for token in int_tokens {
            let upper = token.to_uppercase();
            if upper == "THRU" {
                thru_seen = true;
                continue;
            }
            if upper == "BY" {
                by_seen = true;
                continue;
            }
            if thru_seen && !by_seen {
                if let Ok(val) = token.parse::<f64>() {
                    end = val;
                }
            } else if by_seen {
                if let Ok(val) = token.parse::<f64>() {
                    width = val;
                }
            }
        }
    }

    Ok(Some((
        "stat.survival.lifetables",
        json!({
            "time_var": time_var,
            "status_var": status_var,
            "event_values": event_values,
            "interval_start": 0.0,
            "interval_end": end,
            "interval_width": width,
        }),
    )))
}

fn map_coxreg(
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
            reason: "COXREG requires a time variable".to_string(),
        });
    }

    let time_var = raw_tokens.first().cloned().unwrap_or_default();

    let mut status_var = "status".to_string();
    let mut event_values = vec![1.0];

    if let Some(status_tokens) = cmd.subcommand_tokens("STATUS") {
        if let Some(first) = status_tokens.first() {
            let upper = first.to_uppercase();
            if let Some(open_p) = upper.find('(') {
                if let Some(close_p) = upper.find(')') {
                    status_var = first[..open_p].to_string();
                    let inner = &first[open_p + 1..close_p];
                    if let Ok(val) = inner.parse::<f64>() {
                        event_values = vec![val];
                    }
                }
            } else {
                status_var = first.clone();
            }
        }
    }

    // Covariates from METHOD = ENTER var1 var2 or PATTERN = BY var1 var2 or VARIABLES
    let mut cov_tokens: Vec<String> = Vec::new();
    if let Some(method_tokens) = cmd.subcommand_tokens("METHOD") {
        let enter_pos = method_tokens
            .iter()
            .position(|t| t.eq_ignore_ascii_case("ENTER"));
        let slice = match enter_pos {
            Some(p) => &method_tokens[(p + 1)..],
            None => method_tokens,
        };
        cov_tokens.extend_from_slice(slice);
    } else if let Some(pattern_tokens) = cmd.subcommand_tokens("PATTERN") {
        let by_pos = pattern_tokens
            .iter()
            .position(|t| t.eq_ignore_ascii_case("BY"));
        let slice = match by_pos {
            Some(p) => &pattern_tokens[(p + 1)..],
            None => pattern_tokens,
        };
        cov_tokens.extend_from_slice(slice);
    } else if raw_tokens.len() > 1 {
        cov_tokens.extend_from_slice(&raw_tokens[1..]);
    }

    let covariates = SyntaxParser::expand_variables(&cov_tokens, dataset)?;
    if covariates.is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "METHOD".to_string(),
            reason: "COXREG requires at least one covariate".to_string(),
        });
    }

    let strata_var = cmd
        .subcommand_tokens("STRATA")
        .and_then(|t| t.first().cloned());

    let mut tie_method = "Breslow".to_string();
    if let Some(crit_tokens) = cmd.subcommand_tokens("CRITERIA") {
        for token in crit_tokens {
            let upper = token.to_uppercase();
            if upper.contains("TIES(EFRON)") {
                tie_method = "Efron".to_string();
            } else if upper.contains("TIES(BRESLOW)") {
                tie_method = "Breslow".to_string();
            }
        }
    }

    Ok(Some((
        "stat.survival.cox",
        json!({
            "time_var": time_var,
            "status_var": status_var,
            "covariates": covariates,
            "strata_var": strata_var,
            "event_values": event_values,
            "tie_method": tie_method,
        }),
    )))
}
