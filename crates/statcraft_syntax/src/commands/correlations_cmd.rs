use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_bivariate_correlations, BivariateCorrelationsOptions, MissingHandling, SignificanceTest,
};
use statcraft_core::DataSet;

pub struct CorrelationsCommand;

impl EngineCommand for CorrelationsCommand {
    fn id(&self) -> &'static str {
        "stat.correlations"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Correlate > Bivariate"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["vars"],
            "properties": {
                "vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of variable names to correlate (at least 2)"
                },
                "pearson": { "type": "boolean", "default": true },
                "spearman": { "type": "boolean", "default": false },
                "kendall": { "type": "boolean", "default": false },
                "two_tailed": { "type": "boolean", "default": true },
                "flag_significant": { "type": "boolean", "default": true },
                "descriptives": { "type": "boolean", "default": false },
                "cross_prod_cov": { "type": "boolean", "default": false },
                "missing": {
                    "type": "string",
                    "enum": ["pairwise", "listwise"],
                    "default": "pairwise"
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        match dataset {
            Some(ds) => ds.row_count() > 0 && ds.col_count() >= 2,
            None => false,
        }
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let var_names: Vec<String> = params
            .get("vars")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if var_names.len() < 2 {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'vars' must contain at least 2 variables".to_string(),
            });
        }

        let pearson = params
            .get("pearson")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let spearman = params
            .get("spearman")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let kendall = params
            .get("kendall")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let two_tailed = params
            .get("two_tailed")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let flag_significant = params
            .get("flag_significant")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let descriptives = params
            .get("descriptives")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let cross_prod_cov = params
            .get("cross_prod_cov")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let missing_str = params
            .get("missing")
            .and_then(|v| v.as_str())
            .unwrap_or("pairwise");

        let missing = match missing_str {
            "listwise" => MissingHandling::Listwise,
            _ => MissingHandling::Pairwise,
        };

        let significance = if two_tailed {
            SignificanceTest::TwoTailed
        } else {
            SignificanceTest::OneTailed
        };

        let options = BivariateCorrelationsOptions {
            pearson,
            spearman,
            kendall,
            significance,
            flag_significant,
            descriptives,
            cross_prod_cov,
            missing,
        };

        let total_rows = ds.row_count();
        let mut var_data: Vec<(&str, Vec<Option<f64>>)> = Vec::with_capacity(var_names.len());

        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;

            let mut vals: Vec<Option<f64>> = Vec::with_capacity(total_rows);
            for r in 0..total_rows {
                let cell_str = ds.get_cell_display(r, col_idx, false);
                let v = if cell_str.trim().is_empty() || cell_str == "." || cell_str == "NA" {
                    None
                } else {
                    cell_str.parse::<f64>().ok()
                };
                vals.push(v);
            }
            var_data.push((var_name.as_str(), vals));
        }

        let input: Vec<(&str, &[Option<f64>])> = var_data
            .iter()
            .map(|(name, vals)| (*name, vals.as_slice()))
            .collect();

        let result = compute_bivariate_correlations(&input, &options)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl CorrelationsCommand {
    pub fn to_sps(vars: &[String], options: &BivariateCorrelationsOptions) -> String {
        let sig_part = match options.significance {
            SignificanceTest::TwoTailed => "TWOTAIL",
            SignificanceTest::OneTailed => "ONETAIL",
        };
        let flag_part = if options.flag_significant {
            "SIG"
        } else {
            "NOSIG"
        };
        let miss_part = match options.missing {
            MissingHandling::Pairwise => "PAIRWISE",
            MissingHandling::Listwise => "LISTWISE",
        };

        let mut commands = Vec::new();

        if options.pearson {
            let mut stats = Vec::new();
            if options.descriptives {
                stats.push("DESCRIPTIVES");
            }
            if options.cross_prod_cov {
                stats.push("XPROD");
            }
            let stat_clause = if !stats.is_empty() {
                format!("\n  /STATISTICS {}", stats.join(" "))
            } else {
                String::new()
            };

            commands.push(format!(
                "CORRELATIONS\n  /VARIABLES={}\n  /PRINT={} {}{}\n  /MISSING={}.",
                vars.join(" "),
                sig_part,
                flag_part,
                stat_clause,
                miss_part
            ));
        }

        if options.spearman || options.kendall {
            let mut types = Vec::new();
            if options.spearman {
                types.push("SPEARMAN");
            }
            if options.kendall {
                types.push("KENDALL");
            }
            commands.push(format!(
                "NONPAR CORR\n  /VARIABLES={}\n  /PRINT={} {} {}\n  /MISSING={}.",
                vars.join(" "),
                types.join(" "),
                sig_part,
                flag_part,
                miss_part
            ));
        }

        if commands.is_empty() {
            "* No correlation coefficients selected.".to_string()
        } else {
            commands.join("\n\n")
        }
    }

    pub fn to_sps_syntax(&self, vars: &[String], options: &BivariateCorrelationsOptions) -> String {
        Self::to_sps(vars, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::{DataType, Measure};
    use statcraft_io::RawValue;

    #[test]
    fn test_correlations_sps_syntax() {
        let vars = vec!["salary".to_string(), "salbegin".to_string()];
        let opts = BivariateCorrelationsOptions {
            descriptives: true,
            cross_prod_cov: true,
            spearman: true,
            ..Default::default()
        };

        let sps = CorrelationsCommand::to_sps(&vars, &opts);
        assert!(sps.contains("CORRELATIONS\n  /VARIABLES=salary salbegin"));
        assert!(sps.contains("/STATISTICS DESCRIPTIVES XPROD"));
        assert!(sps
            .contains("NONPAR CORR\n  /VARIABLES=salary salbegin\n  /PRINT=SPEARMAN TWOTAIL SIG"));
    }

    #[test]
    fn test_correlations_command_execution() {
        let mut ds = DataSet::empty();
        let c1 = ds
            .add_variable("x", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
        let c2 = ds
            .add_variable("y", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();

        for i in 1..=5 {
            ds.add_empty_row();
            ds.set_cell(i - 1, c1, RawValue::Number(i as f64)).unwrap();
            ds.set_cell(i - 1, c2, RawValue::Number((i * 2) as f64))
                .unwrap();
        }

        let mut opt_ds = Some(ds);
        let cmd = CorrelationsCommand;
        let params = json!({
            "vars": ["x", "y"],
            "pearson": true,
            "descriptives": true
        });

        let val = cmd
            .execute(&mut opt_ds, params)
            .expect("execution succeeds");
        assert!(val.get("matrices").is_some());
        assert!(val.get("descriptives").is_some());
    }
}
