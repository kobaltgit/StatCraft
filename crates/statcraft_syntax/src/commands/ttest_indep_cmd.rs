use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_indep_ttest, IndepTTestOptions, IndepTTestResult};
use statcraft_core::DataSet;

pub struct IndependentSamplesTTestCommand;

impl EngineCommand for IndependentSamplesTTestCommand {
    fn id(&self) -> &'static str {
        "stat.ttest.indep"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Compare Means > Independent-Samples T Test"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["vars", "grouping_var", "group1", "group2"],
            "properties": {
                "vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of quantitative test variable names to analyze"
                },
                "grouping_var": {
                    "type": "string",
                    "description": "Name of the binary grouping variable"
                },
                "group1": {
                    "type": "string",
                    "description": "Value defining the first group"
                },
                "group2": {
                    "type": "string",
                    "description": "Value defining the second group"
                },
                "ci": {
                    "type": "number",
                    "default": 0.95,
                    "description": "Confidence interval level (e.g. 0.95 for 95%)"
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

        if var_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'vars' must contain at least one test variable name".to_string(),
            });
        }

        let grouping_var = params
            .get("grouping_var")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'grouping_var' is required".to_string(),
            })?;

        let group1 = params.get("group1").and_then(|v| v.as_str()).unwrap_or("1");

        let group2 = params.get("group2").and_then(|v| v.as_str()).unwrap_or("2");

        let ci = params.get("ci").and_then(|v| v.as_f64()).unwrap_or(0.95);

        let group_col_idx =
            ds.dictionary
                .find_index(grouping_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Grouping variable '{}' not found in dataset", grouping_var),
                })?;

        let options = IndepTTestOptions {
            group1_value: group1.to_string(),
            group2_value: group2.to_string(),
            confidence_level: ci,
        };

        let active_rows = ds.active_row_indices();
        let mut group_values: Vec<Option<String>> = Vec::with_capacity(active_rows.len());
        for &r in &active_rows {
            let cell = ds.get_cell_display(r, group_col_idx, false);
            let val = if cell.trim().is_empty() || cell == "." || cell == "NA" {
                None
            } else {
                Some(cell)
            };
            group_values.push(val);
        }

        let mut results: Vec<IndepTTestResult> = Vec::new();

        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;

            let mut var_values: Vec<Option<f64>> = Vec::with_capacity(active_rows.len());
            for &r in &active_rows {
                let cell_str = ds.get_cell_display(r, col_idx, false);
                let val = if cell_str.trim().is_empty() || cell_str == "." || cell_str == "NA" {
                    None
                } else {
                    cell_str.parse::<f64>().ok()
                };
                var_values.push(val);
            }

            let res =
                compute_indep_ttest(var_name, grouping_var, &var_values, &group_values, &options);
            results.push(res);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl IndependentSamplesTTestCommand {
    pub fn to_sps(
        vars: &[String],
        grouping_var: &str,
        group1: &str,
        group2: &str,
        ci: f64,
    ) -> String {
        format!(
            "T-TEST GROUPS={}({} {})\n  /CRITERIA=CI({:.2})\n  /VARIABLES={}.",
            grouping_var,
            group1,
            group2,
            ci,
            vars.join(" ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv = "gender,salary\n1,14.0\n1,15.2\n1,16.1\n1,14.8\n1,17.0\n2,18.0\n2,19.5\n2,17.8\n2,20.1\n2,18.9\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_ttest_indep_command_execution() {
        let cmd = IndependentSamplesTTestCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "vars": ["salary"],
            "grouping_var": "gender",
            "group1": "1",
            "group2": "2",
            "ci": 0.95
        });

        assert!(cmd.enabled(ds.as_ref()));

        let res = cmd
            .execute(&mut ds, params)
            .expect("Execution should succeed");
        let results: Vec<IndepTTestResult> = serde_json::from_value(res).expect("Valid JSON");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].variable, "salary");
        assert_eq!(results[0].grouping_var, "gender");
        assert_eq!(results[0].group1_stats.n, 5);
        assert_eq!(results[0].group2_stats.n, 5);
        assert!(results[0].equal_variances_assumed.t_stat.is_some());
        assert!(results[0].equal_variances_not_assumed.t_stat.is_some());
        assert!(results[0].levene.f_stat.is_some());
    }

    #[test]
    fn test_ttest_indep_command_missing_grouping_var() {
        let cmd = IndependentSamplesTTestCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "vars": ["salary"],
            "grouping_var": "unknown_var",
            "group1": "1",
            "group2": "2"
        });

        let err = cmd.execute(&mut ds, params).unwrap_err();
        match err {
            CommandError::InvalidParams { id, .. } => assert_eq!(id, "stat.ttest.indep"),
            _ => panic!("Expected InvalidParams error"),
        }
    }

    #[test]
    fn test_ttest_indep_sps_syntax() {
        let vars = vec!["salary".to_string(), "bonus".to_string()];
        let sps = IndependentSamplesTTestCommand::to_sps(&vars, "gender", "1", "2", 0.95);
        assert_eq!(
            sps,
            "T-TEST GROUPS=gender(1 2)\n  /CRITERIA=CI(0.95)\n  /VARIABLES=salary bonus."
        );
    }
}
