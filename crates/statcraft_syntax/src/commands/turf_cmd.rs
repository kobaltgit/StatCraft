use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_turf, TurfOptions};
use statcraft_core::DataSet;

pub struct TurfCommand;

impl TurfCommand {
    pub fn to_sps(opts: &TurfOptions) -> String {
        let mut lines = vec![format!(
            "SPSSINC TURF VARIABLES = {}",
            opts.variables.join(" ")
        )];
        lines.push(format!("  FORCE = {}", opts.always_required));

        let mut opt_parts = Vec::new();
        let bestn = opts.max_combinations.unwrap_or(10);
        opt_parts.push(format!("BESTN = {bestn}"));
        if let Some(num_disp) = opts.num_to_display {
            opt_parts.push(format!("NUMBERTODISPLAY = {num_disp}"));
        }
        if !opts.positive_values.is_empty() {
            let vals: Vec<String> = opts
                .positive_values
                .iter()
                .map(|v| {
                    if v.parse::<f64>().is_ok() {
                        v.clone()
                    } else {
                        format!("'{v}'")
                    }
                })
                .collect();
            opt_parts.push(format!("CRITERIA = {}", vals.join(" ")));
        }
        if let Some(min_pct) = opts.min_positive_pct {
            opt_parts.push(format!("MINPERCENT = {min_pct}"));
        }
        opt_parts.push(format!(
            "PLOT = {}",
            if opts.reach_and_frequency_chart {
                "YES"
            } else {
                "NO"
            }
        ));
        opt_parts.push(format!("REMOVALCYCLES = {}", opts.removal_cycles));
        opt_parts.push(format!("MAXSETS = {}", opts.max_operations));
        lines.push(format!("  /OPTIONS {}", opt_parts.join(" ")));

        if opts.advanced_auto {
            lines.push(format!(
                "  /HEATS USEHEATS = {}",
                if opts.use_advanced_selection {
                    "YES"
                } else {
                    "NO"
                }
            ));
        } else {
            lines.push(format!(
                "  /HEATS USEHEATS = {} HEATSIZE = {} NUMWINNERS = {}",
                if opts.use_advanced_selection {
                    "YES"
                } else {
                    "NO"
                },
                opts.advanced_size,
                opts.advanced_winners
            ));
        }

        let mut imp_parts = Vec::new();
        if !opts.importance_weights.is_empty() {
            let mut pairs: Vec<(&String, &f64)> = opts.importance_weights.iter().collect();
            pairs.sort_by(|a, b| a.0.cmp(b.0));
            for (k, v) in pairs {
                imp_parts.push(format!("{k} {v}"));
            }
        }
        imp_parts.push(format!(
            "STRICT = {}",
            if opts.weights_strict { "YES" } else { "NO" }
        ));
        lines.push(format!("  /IMPORTANCE {}", imp_parts.join(" ")));

        let last_idx = lines.len() - 1;
        lines[last_idx].push('.');
        lines.join("\n")
    }
}

impl EngineCommand for TurfCommand {
    fn id(&self) -> &'static str {
        "stat.turf"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Descriptive Statistics > TURF Analysis"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["variables"],
            "properties": {
                "variables": { "type": "array", "items": { "type": "string" } },
                "positive_values": { "type": "array", "items": { "type": "string" }, "default": ["1"] },
                "max_combinations": { "type": ["integer", "null"], "minimum": 1 },
                "num_to_display": { "type": ["integer", "null"], "default": 10, "minimum": 1 },
                "min_positive_pct": { "type": ["number", "null"], "minimum": 0.0, "maximum": 100.0 },
                "always_required": { "type": "integer", "default": 0, "minimum": 0 },
                "removal_cycles": { "type": "integer", "default": 0, "minimum": 0 },
                "show_only_max_combo": { "type": "boolean", "default": false },
                "limit_exceeded_abort": { "type": "boolean", "default": true },
                "max_operations": { "type": "integer", "default": 5000000, "minimum": 1 },
                "importance_weights": { "type": "object", "additionalProperties": { "type": "number" } },
                "weights_strict": { "type": "boolean", "default": true },
                "use_advanced_selection": { "type": "boolean", "default": true },
                "advanced_auto": { "type": "boolean", "default": true },
                "advanced_size": { "type": "integer", "default": 18, "minimum": 2 },
                "advanced_winners": { "type": "integer", "default": 2, "minimum": 1 },
                "reach_and_frequency_chart": { "type": "boolean", "default": true }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        match dataset {
            Some(ds) => ds.row_count() > 0 && ds.col_count() >= 1,
            None => false,
        }
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let opts: TurfOptions =
            serde_json::from_value(params).map_err(|e| CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Invalid TURF parameters: {e}"),
            })?;

        if opts.variables.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'variables' must contain at least one variable".to_string(),
            });
        }

        let res =
            compute_turf(ds, &opts).map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
        serde_json::to_value(&res).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["SPSSINC TURF", "TURF"]
    }

    fn category(&self) -> &'static str {
        "Descriptive Statistics"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax_parser::lexer::SyntaxLexer;
    use crate::syntax_parser::mappers::map_stat_command;
    use crate::syntax_parser::SyntaxParser;
    use statcraft_core::{DataType, Measure, RawValue};

    #[test]
    fn test_turf_command_execute_and_to_sps_roundtrip() {
        let mut ds = DataSet::empty();
        let _ = ds.add_variable("v1", DataType::Numeric { decimals: 0 }, Measure::Nominal);
        let _ = ds.add_variable("v2", DataType::Numeric { decimals: 0 }, Measure::Nominal);
        ds.add_empty_row();
        let _ = ds.set_cell(0, 0, RawValue::Number(1.0));
        let _ = ds.set_cell(0, 1, RawValue::Number(0.0));
        ds.add_empty_row();
        let _ = ds.set_cell(1, 0, RawValue::Number(0.0));
        let _ = ds.set_cell(1, 1, RawValue::Number(1.0));

        let opts = TurfOptions {
            variables: vec!["v1".into(), "v2".into()],
            max_combinations: Some(2),
            ..Default::default()
        };
        let sps = TurfCommand::to_sps(&opts);
        assert!(sps.starts_with("SPSSINC TURF VARIABLES = v1 v2"));

        let blocks = SyntaxLexer::split_commands(&sps).expect("lex ok");
        let parsed = SyntaxParser::parse_command(&blocks[0]).expect("parse ok");
        let (id, val) = map_stat_command(&parsed, Some(&ds))
            .expect("map ok")
            .expect("some");
        assert_eq!(id, "stat.turf");

        let mut ds_opt = Some(ds);
        let out = TurfCommand.execute(&mut ds_opt, val).expect("execute ok");
        assert_eq!(out["total_cases_unweighted"], 2);
    }
}
