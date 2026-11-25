use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_algo::{ExactMode, NonIntegerWeights, RowOrder};
use statcraft_core::DataSet;

pub fn map_crosstabs(
    cmd: &ParsedCommand,
    dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let raw_tokens = cmd
        .subcommand_tokens("TABLES")
        .or_else(|| cmd.subcommand_tokens("POSITIONAL"))
        .unwrap_or(&[]);

    // Split by "BY" into [rows, cols, layer1, layer2, ...]
    let mut segments: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for tok in raw_tokens {
        if tok.eq_ignore_ascii_case("BY") {
            segments.push(current);
            current = Vec::new();
        } else {
            current.push(tok.clone());
        }
    }
    segments.push(current);

    if segments.len() < 2 || segments[0].is_empty() || segments[1].is_empty() {
        return Err(SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "TABLES".to_string(),
            reason: "CROSSTABS requires 'row_vars BY col_vars [BY layer_vars...]'".to_string(),
        });
    }

    let rows = SyntaxParser::expand_variables(&segments[0], dataset)?;
    let cols = SyntaxParser::expand_variables(&segments[1], dataset)?;
    let mut layers: Vec<Vec<String>> = Vec::new();
    for seg in &segments[2..] {
        if !seg.is_empty() {
            layers.push(SyntaxParser::expand_variables(seg, dataset)?);
        }
    }

    let row_order = if cmd.has_token("FORMAT", "DVALUE") {
        RowOrder::Descending
    } else {
        RowOrder::Ascending
    };
    let suppress_tables =
        cmd.subcommand_tokens("NOTABLES").is_some() || cmd.has_token("FORMAT", "NOTABLES");
    let display_clustered_bar_charts = cmd.subcommand_tokens("BARCHART").is_some();

    let stat_all = cmd.has_token("STATISTICS", "ALL");
    let chisq = stat_all || cmd.has_token("STATISTICS", "CHISQ");
    let correlations = stat_all || cmd.has_token("STATISTICS", "CORR");
    let contingency_coef = stat_all || cmd.has_token("STATISTICS", "CC");
    let phi_cramer = stat_all || cmd.has_token("STATISTICS", "PHI") || contingency_coef;
    let lambda = stat_all || cmd.has_token("STATISTICS", "LAMBDA");
    let uncertainty_coef = stat_all || cmd.has_token("STATISTICS", "UC");
    let gamma = stat_all || cmd.has_token("STATISTICS", "GAMMA");
    let somers_d = stat_all || cmd.has_token("STATISTICS", "D");
    let kendall_tau_b = stat_all || cmd.has_token("STATISTICS", "BTAU");
    let kendall_tau_c = stat_all || cmd.has_token("STATISTICS", "CTAU");
    let eta = stat_all || cmd.has_token("STATISTICS", "ETA");
    let kappa = stat_all || cmd.has_token("STATISTICS", "KAPPA");
    let risk = stat_all || cmd.has_token("STATISTICS", "RISK");
    let mcnemar = stat_all || cmd.has_token("STATISTICS", "MCNEMAR");

    let mut cochran_mantel_haenszel = stat_all;
    let mut cmh_common_odds_ratio = 1.0_f64;
    if let Some(stoks) = cmd.subcommand_tokens("STATISTICS") {
        for tok in stoks {
            let up = tok.to_ascii_uppercase();
            if up == "CMH" {
                cochran_mantel_haenszel = true;
            } else if up.starts_with("CMH(") && up.ends_with(')') {
                cochran_mantel_haenszel = true;
                if let Ok(v) = up[4..up.len() - 1].parse::<f64>() {
                    cmh_common_odds_ratio = v;
                }
            }
        }
    }

    let has_cells = cmd.subcommand_tokens("CELLS").is_some();
    let cells_all = cmd.has_token("CELLS", "ALL");
    let observed = !has_cells || cells_all || cmd.has_token("CELLS", "COUNT");
    let expected = !has_cells || cells_all || cmd.has_token("CELLS", "EXPECTED");
    let row_pct = !has_cells || cells_all || cmd.has_token("CELLS", "ROW");
    let col_pct = !has_cells || cells_all || cmd.has_token("CELLS", "COLUMN");
    let total_pct = !has_cells || cells_all || cmd.has_token("CELLS", "TOTAL");
    let resid_unstandardized = !has_cells || cells_all || cmd.has_token("CELLS", "RESID");
    let resid_standardized = cells_all || cmd.has_token("CELLS", "SRESID");
    let resid_adjusted = cells_all || cmd.has_token("CELLS", "ASRESID");
    let bonferroni_adjust = cmd.has_token("CELLS", "BPROP");
    let compare_col_proportions = bonferroni_adjust || cmd.has_token("CELLS", "PROP");

    let exact_mode = if cmd.has_token("METHOD", "MC") {
        let mut conf = 0.99_f64;
        let mut samples = 10000_usize;
        if let Some(mtoks) = cmd.subcommand_tokens("METHOD") {
            for tok in mtoks {
                let up = tok.to_ascii_uppercase();
                if up.starts_with("CIN(") && up.ends_with(')') {
                    if let Ok(v) = up[4..up.len() - 1].parse::<f64>() {
                        conf = if v > 1.0 { v / 100.0 } else { v };
                    }
                } else if up.starts_with("SAMPLES(") && up.ends_with(')') {
                    if let Ok(v) = up[8..up.len() - 1].parse::<usize>() {
                        samples = v;
                    }
                }
            }
        }
        ExactMode::MonteCarlo {
            confidence: conf,
            samples,
        }
    } else if cmd.has_token("METHOD", "EXACT") {
        let mut timer = Some(5u32);
        if let Some(mtoks) = cmd.subcommand_tokens("METHOD") {
            for tok in mtoks {
                let up = tok.to_ascii_uppercase();
                if up.starts_with("TIMER(") && up.ends_with(')') {
                    if let Ok(v) = up[6..up.len() - 1].parse::<u32>() {
                        timer = Some(v);
                    }
                }
            }
        }
        ExactMode::Exact {
            time_limit_min: timer,
        }
    } else {
        ExactMode::AsymptoticOnly
    };

    let noninteger_weights = if cmd.has_token("COUNT", "TRUNCATECELL") {
        NonIntegerWeights::TruncateCell
    } else if cmd.has_token("COUNT", "ROUNDCASE") {
        NonIntegerWeights::RoundCase
    } else if cmd.has_token("COUNT", "TRUNCATECASE") {
        NonIntegerWeights::TruncateCase
    } else if cmd.has_token("COUNT", "ASIS") {
        NonIntegerWeights::NoAdjust
    } else {
        NonIntegerWeights::RoundCell
    };

    let mut map = serde_json::Map::new();
    map.insert("rows".into(), json!(rows));
    map.insert("cols".into(), json!(cols));
    map.insert("layers".into(), json!(layers));
    map.insert("row_order".into(), json!(row_order));
    map.insert("exact_mode".into(), json!(exact_mode));
    map.insert("noninteger_weights".into(), json!(noninteger_weights));
    map.insert("cmh_common_odds_ratio".into(), json!(cmh_common_odds_ratio));
    for (k, v) in [
        ("suppress_tables", suppress_tables),
        ("display_clustered_bar_charts", display_clustered_bar_charts),
        ("chisq", chisq),
        ("correlations", correlations),
        ("contingency_coef", contingency_coef),
        ("phi_cramer", phi_cramer),
        ("lambda", lambda),
        ("uncertainty_coef", uncertainty_coef),
        ("gamma", gamma),
        ("somers_d", somers_d),
        ("kendall_tau_b", kendall_tau_b),
        ("kendall_tau_c", kendall_tau_c),
        ("eta", eta),
        ("kappa", kappa),
        ("risk", risk),
        ("mcnemar", mcnemar),
        ("cochran_mantel_haenszel", cochran_mantel_haenszel),
        ("observed", observed),
        ("expected", expected),
        ("row_pct", row_pct),
        ("col_pct", col_pct),
        ("total_pct", total_pct),
        ("residuals", resid_unstandardized),
        ("resid_unstandardized", resid_unstandardized),
        ("resid_standardized", resid_standardized),
        ("resid_adjusted", resid_adjusted),
        ("compare_col_proportions", compare_col_proportions),
        ("bonferroni_adjust", bonferroni_adjust),
    ] {
        map.insert(k.into(), Value::Bool(v));
    }

    Ok(Some(("stat.crosstabs", Value::Object(map))))
}
