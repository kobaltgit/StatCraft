use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::{ParsedCommand, SyntaxParser};
use serde_json::{json, Value};
use statcraft_core::DataSet;

/// Maps SPSS `FREQUENCIES` syntax to the `stat.frequencies` command payload.
pub fn map_frequencies(
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

    // 1. Table display and format
    let display_tables = !(cmd.has_token("FORMAT", "NOTABLE")
        || cmd.has_token("FORMAT", "NOTABLES")
        || cmd.has_token("FORMAT", "LIMIT"));

    // 2. Order
    let mut order = "analysis";
    if cmd.has_token("ORDER", "VARIABLE") {
        order = "variable";
    } else if cmd.has_token("ORDER", "ANALYSIS") {
        order = "analysis";
    }

    if cmd.has_token("FORMAT", "DVALUE") || cmd.has_token("ORDER", "DVALUE") {
        order = "dvalue";
    } else if cmd.has_token("FORMAT", "AFREQ") || cmd.has_token("ORDER", "AFREQ") {
        order = "afreq";
    } else if cmd.has_token("FORMAT", "DFREQ") || cmd.has_token("ORDER", "DFREQ") {
        order = "dfreq";
    } else if cmd.has_token("FORMAT", "AVALUE") || cmd.has_token("ORDER", "AVALUE") {
        order = "avalue";
    }

    // 3. Statistics
    let statistics = parse_spss_statistics(cmd);

    // 4. Percentiles and NTILES
    let percentiles = parse_spss_percentiles(cmd);

    // 5. Group midpoints
    let group_midpoints = cmd.has_token("STATISTICS", "GROUPED") || cmd.has_subcommand("GROUPED");

    let mut result = json!({
        "vars": vars,
        "display_tables": display_tables,
        "order": order,
    });

    if let Some(stats) = statistics {
        result["statistics"] = stats;
    }
    if let Some(pcts) = percentiles {
        result["percentiles"] = pcts;
    }
    if group_midpoints {
        result["group_midpoints"] = json!(true);
    }

    Ok(Some(("stat.frequencies", result)))
}

fn parse_spss_statistics(cmd: &ParsedCommand) -> Option<Value> {
    if !cmd.has_subcommand("STATISTICS") {
        return None;
    }

    if cmd.has_token("STATISTICS", "NONE") {
        return Some(json!({
            "mean": false,
            "se_mean": false,
            "median": false,
            "mode": false,
            "sum": false,
            "std_dev": false,
            "variance": false,
            "range": false,
            "min": false,
            "max": false,
            "skewness": false,
            "kurtosis": false,
        }));
    }

    let is_all = cmd.has_token("STATISTICS", "ALL");
    let is_default = cmd.has_token("STATISTICS", "DEFAULT")
        || cmd
            .subcommand("STATISTICS")
            .map(|s| s.tokens.is_empty())
            .unwrap_or(false);

    let mean = is_all || is_default || cmd.has_token("STATISTICS", "MEAN");
    let std_dev = is_all
        || is_default
        || cmd.has_token("STATISTICS", "STDDEV")
        || cmd.has_token("STATISTICS", "STD");
    let min = is_all
        || is_default
        || cmd.has_token("STATISTICS", "MINIMUM")
        || cmd.has_token("STATISTICS", "MIN");
    let max = is_all
        || is_default
        || cmd.has_token("STATISTICS", "MAXIMUM")
        || cmd.has_token("STATISTICS", "MAX");

    let se_mean =
        is_all || cmd.has_token("STATISTICS", "SEMEAN") || cmd.has_token("STATISTICS", "SEM");
    let median =
        is_all || cmd.has_token("STATISTICS", "MEDIAN") || cmd.has_token("STATISTICS", "MED");
    let mode = is_all || cmd.has_token("STATISTICS", "MODE");
    let sum = is_all || cmd.has_token("STATISTICS", "SUM");
    let variance =
        is_all || cmd.has_token("STATISTICS", "VARIANCE") || cmd.has_token("STATISTICS", "VAR");
    let range = is_all || cmd.has_token("STATISTICS", "RANGE");
    let skewness = is_all
        || cmd.has_token("STATISTICS", "SKEWNESS")
        || cmd.has_token("STATISTICS", "SKEW")
        || cmd.has_token("STATISTICS", "SESKEW");
    let kurtosis = is_all
        || cmd.has_token("STATISTICS", "KURTOSIS")
        || cmd.has_token("STATISTICS", "KURT")
        || cmd.has_token("STATISTICS", "SEKURT");

    Some(json!({
        "mean": mean,
        "se_mean": se_mean,
        "median": median,
        "mode": mode,
        "sum": sum,
        "std_dev": std_dev,
        "variance": variance,
        "range": range,
        "min": min,
        "max": max,
        "skewness": skewness,
        "kurtosis": kurtosis,
    }))
}

fn parse_spss_percentiles(cmd: &ParsedCommand) -> Option<Value> {
    let quartiles = cmd.has_token("STATISTICS", "QUARTILES")
        || cmd.has_subcommand("QUARTILES")
        || cmd.has_token("PERCENTILES", "QUARTILES");

    let ntiles = cmd.subcommand("NTILES").and_then(|sub| {
        sub.tokens
            .iter()
            .find_map(|t| t.trim_matches(',').parse::<usize>().ok())
    });

    let mut custom = Vec::new();
    if let Some(sub) = cmd.subcommand("PERCENTILES") {
        for t in &sub.tokens {
            let clean = t.trim_matches(',');
            if let Ok(v) = clean.parse::<f64>() {
                if (0.0..=100.0).contains(&v) {
                    custom.push(v);
                }
            }
        }
    }

    if quartiles || ntiles.is_some() || !custom.is_empty() {
        Some(json!({
            "quartiles": quartiles,
            "ntiles": ntiles,
            "custom": custom,
        }))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax_parser::lexer::SyntaxLexer;

    #[test]
    fn test_map_frequencies_basic() {
        let blocks = SyntaxLexer::split_commands("FREQUENCIES VARIABLES=salary educ.").unwrap();
        let cmd = SyntaxParser::parse_command(&blocks[0]).unwrap();
        let (id, val) = map_frequencies(&cmd, None).unwrap().unwrap();
        assert_eq!(id, "stat.frequencies");
        assert_eq!(val["vars"], json!(["salary", "educ"]));
        assert_eq!(val["display_tables"], true);
        assert_eq!(val["order"], "analysis");
        assert!(val.get("statistics").is_none());
    }

    #[test]
    fn test_map_frequencies_statistics_default_and_custom() {
        let blocks = SyntaxLexer::split_commands(
            "FREQUENCIES VARIABLES=salary\n  /STATISTICS=DEFAULT MEDIAN MODE\n  /FORMAT=NOTABLE DFREQ.",
        )
        .unwrap();
        let cmd = SyntaxParser::parse_command(&blocks[0]).unwrap();
        let (id, val) = map_frequencies(&cmd, None).unwrap().unwrap();
        assert_eq!(id, "stat.frequencies");
        assert_eq!(val["display_tables"], false);
        assert_eq!(val["order"], "dfreq");

        let stats = &val["statistics"];
        assert_eq!(stats["mean"], true);
        assert_eq!(stats["std_dev"], true);
        assert_eq!(stats["min"], true);
        assert_eq!(stats["max"], true);
        assert_eq!(stats["median"], true);
        assert_eq!(stats["mode"], true);
        assert_eq!(stats["skewness"], false);
    }

    #[test]
    fn test_map_frequencies_all_and_percentiles() {
        let blocks = SyntaxLexer::split_commands(
            "FREQUENCIES VARIABLES=score\n  /STATISTICS=ALL\n  /PERCENTILES=10 25 75 90\n  /NTILES=10.",
        )
        .unwrap();
        let cmd = SyntaxParser::parse_command(&blocks[0]).unwrap();
        let (_, val) = map_frequencies(&cmd, None).unwrap().unwrap();

        let stats = &val["statistics"];
        assert_eq!(stats["mean"], true);
        assert_eq!(stats["se_mean"], true);
        assert_eq!(stats["skewness"], true);
        assert_eq!(stats["kurtosis"], true);

        let pcts = &val["percentiles"];
        assert_eq!(pcts["ntiles"], 10);
        assert_eq!(pcts["custom"], json!([10.0, 25.0, 75.0, 90.0]));
    }
}
