use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_oneway_anova, OneWayAnovaOptions, OneWayAnovaResult};
use statcraft_core::DataSet;

pub struct OneWayAnovaCommand;

impl EngineCommand for OneWayAnovaCommand {
    fn id(&self) -> &'static str {
        "stat.anova.oneway"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Compare Means > One-Way ANOVA"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependent_vars", "factor_var"],
            "properties": {
                "dependent_vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of quantitative dependent variable names to analyze"
                },
                "factor_var": {
                    "type": "string",
                    "description": "Name of the categorical grouping factor variable"
                },
                "descriptives": {
                    "type": "boolean",
                    "default": true,
                    "description": "Compute group descriptives"
                },
                "homogeneity": {
                    "type": "boolean",
                    "default": true,
                    "description": "Compute Levene test of homogeneity of variances"
                },
                "ci": {
                    "type": "number",
                    "default": 0.95,
                    "description": "Confidence level for descriptives mean CI"
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

        let dep_var_names: Vec<String> = params
            .get("dependent_vars")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if dep_var_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'dependent_vars' must contain at least one variable name"
                    .to_string(),
            });
        }

        let factor_var = params
            .get("factor_var")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'factor_var' is required".to_string(),
            })?;

        let descriptives = params
            .get("descriptives")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let homogeneity = params
            .get("homogeneity")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let ci = params.get("ci").and_then(|v| v.as_f64()).unwrap_or(0.95);

        let factor_col_idx =
            ds.dictionary
                .find_index(factor_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Factor variable '{}' not found in dataset", factor_var),
                })?;

        let options = OneWayAnovaOptions {
            descriptives,
            homogeneity,
            confidence_level: ci,
        };

        let total_rows = ds.row_count();
        let mut factor_values: Vec<Option<String>> = Vec::with_capacity(total_rows);
        for r in 0..total_rows {
            let cell = ds.get_cell_display(r, factor_col_idx, false);
            let val = if cell.trim().is_empty() || cell == "." || cell == "NA" {
                None
            } else {
                Some(cell)
            };
            factor_values.push(val);
        }

        let mut results: Vec<OneWayAnovaResult> = Vec::new();

        for dep_name in &dep_var_names {
            let col_idx =
                ds.dictionary
                    .find_index(dep_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dataset", dep_name),
                    })?;

            let mut dep_values: Vec<Option<f64>> = Vec::with_capacity(total_rows);
            for r in 0..total_rows {
                let cell_str = ds.get_cell_display(r, col_idx, false);
                let val = if cell_str.trim().is_empty() || cell_str == "." || cell_str == "NA" {
                    None
                } else {
                    cell_str.parse::<f64>().ok()
                };
                dep_values.push(val);
            }

            let res =
                compute_oneway_anova(dep_name, factor_var, &dep_values, &factor_values, &options)
                    .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            results.push(res);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl OneWayAnovaCommand {
    pub fn to_sps(
        dependent_vars: &[String],
        factor_var: &str,
        descriptives: bool,
        homogeneity: bool,
    ) -> String {
        let mut stats = Vec::new();
        if descriptives {
            stats.push("DESCRIPTIVES");
        }
        if homogeneity {
            stats.push("HOMOGENEITY");
        }
        let stat_clause = if !stats.is_empty() {
            format!("\n  /STATISTICS {}", stats.join(" "))
        } else {
            String::new()
        };
        format!(
            "ONEWAY {} BY {}{}\n  /MISSING ANALYSIS.",
            dependent_vars.join(" "),
            factor_var,
            stat_clause
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv_data = "group,score\n1,10\n1,12\n1,11\n1,13\n1,9\n2,15\n2,16\n2,14\n2,15\n2,17\n3,20\n3,22\n3,19\n3,21\n3,23\n";
        let table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_oneway_anova_command_execution() {
        let cmd = OneWayAnovaCommand;
        let mut dataset = Some(make_test_dataset());

        let params = json!({
            "dependent_vars": ["score"],
            "factor_var": "group",
            "descriptives": true,
            "homogeneity": true,
            "ci": 0.95
        });

        let res = cmd.execute(&mut dataset, params).expect("Should succeed");
        let results: Vec<OneWayAnovaResult> =
            serde_json::from_value(res).expect("Should deserialize");

        assert_eq!(results.len(), 1);
        let anova = &results[0];
        assert_eq!(anova.dep_var, "score");
        assert_eq!(anova.factor_var, "group");
        assert_eq!(anova.between_groups.df, 2);
        assert!((anova.between_groups.sum_of_squares - 251.2).abs() < 1e-6);
        assert!((anova.between_groups.f_stat.unwrap_or(0.0) - 59.8095238).abs() < 1e-6);
    }

    #[test]
    fn test_oneway_anova_sps_syntax() {
        let sps = OneWayAnovaCommand::to_sps(
            &["salary".to_string(), "bonus".to_string()],
            "jobcat",
            true,
            true,
        );
        assert_eq!(
            sps,
            "ONEWAY salary bonus BY jobcat\n  /STATISTICS DESCRIPTIVES HOMOGENEITY\n  /MISSING ANALYSIS."
        );

        let sps_minimal =
            OneWayAnovaCommand::to_sps(&["salary".to_string()], "jobcat", false, false);
        assert_eq!(sps_minimal, "ONEWAY salary BY jobcat\n  /MISSING ANALYSIS.");
    }

    #[test]
    fn test_oneway_anova_command_missing_vars() {
        let cmd = OneWayAnovaCommand;
        let mut dataset = Some(make_test_dataset());

        let params = json!({
            "dependent_vars": [],
            "factor_var": "group"
        });

        let err = cmd.execute(&mut dataset, params).unwrap_err();
        match err {
            CommandError::InvalidParams { reason, .. } => {
                assert!(reason.contains("at least one variable"));
            }
            _ => panic!("Expected InvalidParams"),
        }
    }
}
