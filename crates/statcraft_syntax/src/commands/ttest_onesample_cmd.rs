use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_onesample_ttest, OneSampleTTestOptions, OneSampleTTestResult};
use statcraft_core::DataSet;

pub struct OneSampleTTestCommand;

impl EngineCommand for OneSampleTTestCommand {
    fn id(&self) -> &'static str {
        "stat.ttest.onesample"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Compare Means > One-Sample T Test"
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
                    "description": "List of variable names to analyze"
                },
                "test_value": {
                    "type": "number",
                    "default": 0.0,
                    "description": "Hypothesized population mean (test value)"
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
            Some(ds) => ds.row_count() > 0 && ds.col_count() > 0,
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
                reason: "Parameter 'vars' must contain at least one variable name".to_string(),
            });
        }

        let test_value = params
            .get("test_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let ci = params.get("ci").and_then(|v| v.as_f64()).unwrap_or(0.95);

        let options = OneSampleTTestOptions {
            test_value,
            confidence_level: ci,
        };

        let mut results: Vec<OneSampleTTestResult> = Vec::new();
        let total_rows = ds.row_count();

        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: "stat.ttest.onesample",
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;

            let mut values: Vec<Option<f64>> = Vec::with_capacity(total_rows);
            for r in 0..total_rows {
                let cell_str = ds.get_cell_display(r, col_idx, false);
                let val = if cell_str.trim().is_empty() || cell_str == "." || cell_str == "NA" {
                    None
                } else {
                    cell_str.parse::<f64>().ok()
                };
                values.push(val);
            }

            let res = compute_onesample_ttest(var_name, &values, &options);
            results.push(res);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl OneSampleTTestCommand {
    pub fn to_sps(vars: &[String], test_value: f64, ci: f64) -> String {
        format!(
            "T-TEST\n  /TESTVAL={}\n  /CRITERIA=CI({:.2})\n  /VARIABLES={}.",
            test_value,
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
        let csv = "score,hours\n12,2\n15,3\n18,4\n20,5\n22,6\n25,7\n28,8\n30,9\n35,10\n40,11\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_ttest_onesample_command_execution() {
        let cmd = OneSampleTTestCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "vars": ["score"],
            "test_value": 20.0,
            "ci": 0.95
        });

        assert!(cmd.enabled(ds.as_ref()));

        let res = cmd
            .execute(&mut ds, params)
            .expect("Execution should succeed");
        let results: Vec<OneSampleTTestResult> = serde_json::from_value(res).expect("Valid JSON");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].variable, "score");
        assert_eq!(results[0].n_valid, 10);
        assert_eq!(results[0].test_value, 20.0);
        assert!((results[0].mean.unwrap() - 24.5).abs() < 1e-5);
        assert!((results[0].t_stat.unwrap() - 1.6038504).abs() < 1e-5);
        assert_eq!(results[0].df, Some(9.0));
    }

    #[test]
    fn test_ttest_onesample_command_missing_variable() {
        let cmd = OneSampleTTestCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "vars": ["non_existent_var"],
            "test_value": 0.0
        });

        let err = cmd.execute(&mut ds, params).unwrap_err();
        match err {
            CommandError::InvalidParams { id, .. } => assert_eq!(id, "stat.ttest.onesample"),
            _ => panic!("Expected InvalidParams error"),
        }
    }

    #[test]
    fn test_ttest_onesample_sps_syntax() {
        let vars = vec!["score".to_string(), "hours".to_string()];
        let sps = OneSampleTTestCommand::to_sps(&vars, 50.0, 0.95);
        assert_eq!(
            sps,
            "T-TEST\n  /TESTVAL=50\n  /CRITERIA=CI(0.95)\n  /VARIABLES=score hours."
        );
    }
}
