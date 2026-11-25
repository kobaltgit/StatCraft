use crate::command::CommandError;
use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::ParsedCommand;
use serde_json::{json, Value};
use statcraft_algo::{BootstrapCiMethod, BootstrapOptions, BootstrapSamplingMethod};
use statcraft_core::DataSet;
use statcraft_io::RawValue;

pub fn parse_bootstrap_options(
    params: &Value,
    cmd_id: &'static str,
) -> Result<Option<BootstrapOptions>, CommandError> {
    let boot_opt = params
        .get("bootstrap")
        .and_then(|v| serde_json::from_value::<BootstrapOptions>(v.clone()).ok())
        .filter(|b| b.enabled);

    if let Some(ref boot) = boot_opt {
        boot.validate().map_err(|e| CommandError::InvalidParams {
            id: cmd_id,
            reason: e.to_string(),
        })?;
    }
    Ok(boot_opt)
}

pub fn build_strata_keys(
    ds: &DataSet,
    active_rows: &[usize],
    strata_vars: &[String],
    cmd_id: &'static str,
) -> Result<Vec<String>, CommandError> {
    let mut col_indices = Vec::with_capacity(strata_vars.len());
    for sv in strata_vars {
        let idx = ds
            .dictionary
            .find_index(sv)
            .ok_or_else(|| CommandError::InvalidParams {
                id: cmd_id,
                reason: format!("Strata variable '{}' not found in dataset", sv),
            })?;
        col_indices.push(idx);
    }

    let mut keys = Vec::with_capacity(active_rows.len());
    for &r in active_rows {
        let mut parts = Vec::with_capacity(col_indices.len());
        for &c in &col_indices {
            let s = match ds.get_cell(r, c) {
                Some(RawValue::Number(n)) => {
                    if n.fract() == 0.0 {
                        format!("{:.0}", n)
                    } else {
                        n.to_string()
                    }
                }
                Some(RawValue::Text(t)) => t.trim().to_string(),
                Some(RawValue::Null) | None => String::new(),
            };
            parts.push(s);
        }
        keys.push(parts.join("\x1F"));
    }
    Ok(keys)
}

pub fn extract_strata_keys(
    ds: &DataSet,
    active_rows: &[usize],
    boot_options: Option<&BootstrapOptions>,
    cmd_id: &'static str,
) -> Result<Option<Vec<String>>, CommandError> {
    match boot_options {
        Some(boot) => match &boot.sampling_method {
            BootstrapSamplingMethod::Stratified { strata_vars } => Ok(Some(build_strata_keys(
                ds,
                active_rows,
                strata_vars,
                cmd_id,
            )?)),
            BootstrapSamplingMethod::Simple => Ok(None),
        },
        None => Ok(None),
    }
}

/// Returns the reusable JSON Schema for the `bootstrap` parameter in `EngineCommand::specs()`.
pub fn bootstrap_param_schema() -> Value {
    json!({
        "type": "object",
        "description": "Optional bootstrap resampling configuration",
        "properties": {
            "enabled": {
                "type": "boolean",
                "default": false,
                "description": "Whether bootstrap resampling is enabled"
            },
            "samples": {
                "type": "integer",
                "minimum": 2,
                "default": 1000,
                "description": "Number of bootstrap replicates"
            },
            "seed": {
                "type": ["integer", "null"],
                "minimum": 1,
                "maximum": 2_000_000_000,
                "default": null,
                "description": "Optional fixed MT19937 seed (1..=2,000,000,000)"
            },
            "ci_level": {
                "type": "number",
                "exclusiveMinimum": 0.0,
                "exclusiveMaximum": 100.0,
                "default": 95.0,
                "description": "Confidence interval level in percent"
            },
            "ci_method": {
                "type": "string",
                "enum": ["percentile", "bca"],
                "default": "percentile",
                "description": "Bootstrap confidence interval method"
            },
            "sampling_method": {
                "type": "object",
                "default": { "method": "simple" },
                "properties": {
                    "method": {
                        "type": "string",
                        "enum": ["simple", "stratified"]
                    },
                    "strata_vars": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                }
            }
        }
    })
}

/// Generates canonical SPSS `BOOTSTRAP` syntax (with optional `SET MTINDEX=...`)
/// from a `BootstrapOptions` struct. Returns an empty string if `enabled == false`.
pub fn bootstrap_to_sps(opts: &BootstrapOptions) -> String {
    if !opts.enabled {
        return String::new();
    }

    let mut lines = Vec::new();
    if let Some(seed) = opts.seed {
        lines.push(format!("SET MTINDEX={seed}."));
    }

    let sampling_clause = match &opts.sampling_method {
        BootstrapSamplingMethod::Simple => "/SAMPLING METHOD=SIMPLE".to_string(),
        BootstrapSamplingMethod::Stratified { strata_vars } => {
            if strata_vars.is_empty() {
                "/SAMPLING METHOD=SIMPLE".to_string()
            } else {
                format!(
                    "/SAMPLING METHOD=STRATIFIED(STRATA={})",
                    strata_vars.join(" ")
                )
            }
        }
    };

    let ci_type = match opts.ci_method {
        BootstrapCiMethod::Percentile => "PERCENTILE",
        BootstrapCiMethod::Bca => "BCA",
    };

    let ci_level_str = if (opts.ci_level - opts.ci_level.round()).abs() < 1e-9 {
        format!("{}", opts.ci_level.round() as i64)
    } else {
        format!("{}", opts.ci_level)
    };

    lines.push("BOOTSTRAP".to_string());
    lines.push(format!("  {sampling_clause}"));
    lines.push(format!(
        "  /CRITERIA CILEVEL={ci_level_str} CITYPE={ci_type} NSAMPLES={}",
        opts.samples
    ));
    lines.push("  /MISSING USERMISSING=EXCLUDE.".to_string());

    lines.join("\n")
}

/// Parses a `SET MTINDEX=<seed|RANDOM>.` command if it is an `MTINDEX` directive.
/// Returns `Ok(Some(seed_opt))` when `SET MTINDEX=...` is recognized, or `Ok(None)` otherwise.
pub fn parse_set_mtindex(cmd: &ParsedCommand) -> Result<Option<Option<u64>>, SyntaxError> {
    if !cmd.name.eq_ignore_ascii_case("SET") {
        return Ok(None);
    }
    let body = cmd
        .raw_text
        .trim()
        .trim_end_matches('.')
        .trim()
        .strip_prefix(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(cmd.raw_text.trim());
    let _ = body;

    let upper = cmd.raw_text.to_ascii_uppercase();
    let Some(idx) = upper.find("MTINDEX") else {
        return Ok(None);
    };
    let after = cmd.raw_text[idx + "MTINDEX".len()..]
        .trim()
        .trim_start_matches('=')
        .trim()
        .trim_end_matches('.')
        .trim();

    let token = after.split_whitespace().next().unwrap_or("");
    if token.eq_ignore_ascii_case("RANDOM") {
        return Ok(Some(None));
    }

    let seed = token
        .parse::<u64>()
        .map_err(|_| SyntaxError::ExecutionFailed {
            line: cmd.line_num,
            command: cmd.name.clone(),
            reason: format!("Invalid MTINDEX seed value: '{token}'"),
        })?;
    if !(1..=2_000_000_000).contains(&seed) {
        return Err(SyntaxError::ExecutionFailed {
            line: cmd.line_num,
            command: cmd.name.clone(),
            reason: format!("MTINDEX seed must be between 1 and 2,000,000,000, got {seed}"),
        });
    }
    Ok(Some(Some(seed)))
}

/// Parses a `BOOTSTRAP` command into `BootstrapOptions`.
pub fn parse_bootstrap_command(cmd: &ParsedCommand) -> Result<BootstrapOptions, SyntaxError> {
    let mut opts = BootstrapOptions {
        enabled: true,
        ..BootstrapOptions::default()
    };

    if let Some(sampling) = cmd.subcommand("SAMPLING") {
        let raw_upper = sampling.raw_value.to_ascii_uppercase();
        if raw_upper.contains("STRATIFIED") {
            let strata_vars = extract_strata_vars(&sampling.raw_value);
            if strata_vars.is_empty() {
                return Err(SyntaxError::ExecutionFailed {
                    line: cmd.line_num,
                    command: cmd.name.clone(),
                    reason: "STRATIFIED sampling requires STRATA=<var1 var2 ...>".to_string(),
                });
            }
            opts.sampling_method = BootstrapSamplingMethod::Stratified { strata_vars };
        } else if raw_upper.contains("SIMPLE") {
            opts.sampling_method = BootstrapSamplingMethod::Simple;
        }
    }

    if let Some(criteria) = cmd.subcommand("CRITERIA") {
        let raw = &criteria.raw_value;
        if let Some(val) = extract_kv_value(raw, "CILEVEL") {
            let level = val
                .parse::<f64>()
                .map_err(|_| SyntaxError::ExecutionFailed {
                    line: cmd.line_num,
                    command: cmd.name.clone(),
                    reason: format!("Invalid CILEVEL value: '{val}'"),
                })?;
            opts.ci_level = level;
        }
        if let Some(val) = extract_kv_value(raw, "CITYPE") {
            if val.eq_ignore_ascii_case("BCA") {
                opts.ci_method = BootstrapCiMethod::Bca;
            } else if val.eq_ignore_ascii_case("PERCENTILE") {
                opts.ci_method = BootstrapCiMethod::Percentile;
            } else {
                return Err(SyntaxError::ExecutionFailed {
                    line: cmd.line_num,
                    command: cmd.name.clone(),
                    reason: format!("Unsupported CITYPE: '{val}' (expected PERCENTILE or BCA)"),
                });
            }
        }
        if let Some(val) = extract_kv_value(raw, "NSAMPLES") {
            let samples = val
                .parse::<usize>()
                .map_err(|_| SyntaxError::ExecutionFailed {
                    line: cmd.line_num,
                    command: cmd.name.clone(),
                    reason: format!("Invalid NSAMPLES value: '{val}'"),
                })?;
            opts.samples = samples;
        }
    }

    opts.validate().map_err(|e| SyntaxError::ExecutionFailed {
        line: cmd.line_num,
        command: cmd.name.clone(),
        reason: e.to_string(),
    })?;

    Ok(opts)
}

fn extract_kv_value<'a>(raw: &'a str, key: &str) -> Option<&'a str> {
    let upper = raw.to_ascii_uppercase();
    let key_upper = key.to_ascii_uppercase();
    let mut search_start = 0;
    while let Some(pos) = upper[search_start..].find(&key_upper) {
        let abs_pos = search_start + pos;
        let before_ok = abs_pos == 0
            || !raw[..abs_pos]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
        let after_pos = abs_pos + key_upper.len();
        if before_ok {
            let rest = raw[after_pos..].trim_start();
            if let Some(after_eq) = rest.strip_prefix('=') {
                let val = after_eq.split_whitespace().next()?;
                return Some(val.trim_end_matches([',', ';', ')']));
            }
        }
        search_start = after_pos;
    }
    None
}

fn extract_strata_vars(raw: &str) -> Vec<String> {
    let upper = raw.to_ascii_uppercase();
    let Some(pos) = upper.find("STRATA") else {
        return Vec::new();
    };
    let rest = raw[pos + "STRATA".len()..].trim_start();
    let Some(after_eq) = rest.strip_prefix('=') else {
        return Vec::new();
    };
    let inner = match after_eq.find(')') {
        Some(end) => &after_eq[..end],
        None => after_eq,
    };
    inner
        .split(|c: char| c.is_whitespace() || c == ',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}
