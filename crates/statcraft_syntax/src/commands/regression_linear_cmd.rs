use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_linear_regression, LinearRegressionOptions, LinearRegressionResult};
use statcraft_core::DataSet;

pub struct LinearRegressionCommand;

impl LinearRegressionCommand {
    pub fn to_sps(dep_var: &str, indep_vars: &[String]) -> String {
        format!(
            "REGRESSION\n  /MISSING LISTWISE\n  /STATISTICS COEFF OUTS R ANOVA COLLIN\n  /CRITERIA=PIN(.05) POUT(.10)\n  /NOORIGIN\n  /DEPENDENT {}\n  /METHOD=ENTER {}.",
            dep_var,
            indep_vars.join(" ")
        )
    }
}

impl EngineCommand for LinearRegressionCommand {
    fn id(&self) -> &'static str {
        "stat.regression.linear"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Regression > Linear"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependent_var", "independent_vars"],
            "properties": {
                "dependent_var": {
                    "type": "string",
                    "description": "Name of the quantitative dependent variable"
                },
                "independent_vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of quantitative predictor variable names"
                },
                "ci": {
                    "type": "number",
                    "default": 0.95,
                    "description": "Confidence level for coefficient confidence intervals (e.g. 0.95)"
                },
                "collinearity": {
                    "type": "boolean",
                    "default": true,
                    "description": "Calculate collinearity statistics (Tolerance and VIF)"
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

        let dep_var = params
            .get("dependent_var")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'dependent_var' is required".to_string(),
            })?;

        let indep_vars: Vec<String> = params
            .get("independent_vars")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if indep_vars.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'independent_vars' must contain at least one predictor"
                    .to_string(),
            });
        }

        let dep_col_idx = ds.dictionary.find_index(dep_var).ok_or_else(|| {
            CommandError::ExecutionFailed(format!(
                "Dependent variable '{}' not found in dataset",
                dep_var
            ))
        })?;

        let mut indep_col_indices = Vec::with_capacity(indep_vars.len());
        for name in &indep_vars {
            let col_idx = ds.dictionary.find_index(name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!(
                    "Predictor variable '{}' not found in dataset",
                    name
                ))
            })?;
            indep_col_indices.push(col_idx);
        }

        let ci = params.get("ci").and_then(|v| v.as_f64()).unwrap_or(0.95);
        let collinearity = params
            .get("collinearity")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let options = LinearRegressionOptions {
            ci_level: ci,
            collinearity,
        };

        // Extract active rows considering Select Cases and Missing Data
        let active_rows = ds.active_row_indices();
        let mut dep_vals = Vec::with_capacity(active_rows.len());
        let mut indep_vals: Vec<Vec<Option<f64>>> =
            vec![Vec::with_capacity(active_rows.len()); indep_vars.len()];
        let mut weights = Vec::with_capacity(active_rows.len());

        for &r in &active_rows {
            let dep_str = ds.get_cell_display(r, dep_col_idx, false);
            let dep_val = if dep_str.trim().is_empty() || dep_str == "." || dep_str == "NA" {
                None
            } else {
                dep_str.parse::<f64>().ok()
            };
            dep_vals.push(dep_val);

            for (p_idx, &c_idx) in indep_col_indices.iter().enumerate() {
                let x_str = ds.get_cell_display(r, c_idx, false);
                let x_val = if x_str.trim().is_empty() || x_str == "." || x_str == "NA" {
                    None
                } else {
                    x_str.parse::<f64>().ok()
                };
                indep_vals[p_idx].push(x_val);
            }

            weights.push(ds.row_weight(r));
        }

        let result: LinearRegressionResult = compute_linear_regression(
            dep_var,
            &dep_vals,
            &indep_vars,
            &indep_vals,
            &weights,
            &options,
        )
        .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn create_test_dataset() -> DataSet {
        let csv_data = "salary,educ,prevexp\n12.0,1.0,2.5\n15.0,2.0,3.0\n18.0,3.0,4.2\n20.0,4.0,5.1\n24.0,5.0,5.8\n28.0,6.0,7.2\n30.0,7.0,8.0\n35.0,8.0,8.5\n38.0,9.0,9.8\n42.0,10.0,10.5\n";
        let table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_linear_regression_command_execution() {
        let cmd = LinearRegressionCommand;
        let mut ds = Some(create_test_dataset());

        let params = json!({
            "dependent_var": "salary",
            "independent_vars": ["educ", "prevexp"],
            "ci": 0.95,
            "collinearity": true
        });

        let res_val = cmd
            .execute(&mut ds, params)
            .expect("Execution should succeed");
        let res: LinearRegressionResult = serde_json::from_value(res_val).unwrap();

        assert_eq!(res.dep_var, "salary");
        assert_eq!(res.indep_vars, vec!["educ", "prevexp"]);
        assert!(res.r_squared > 0.99);
        assert_eq!(res.coefficients.len(), 3);
    }

    #[test]
    fn test_linear_regression_sps_syntax() {
        let sps =
            LinearRegressionCommand::to_sps("salary", &["educ".to_string(), "prevexp".to_string()]);
        assert!(sps.contains("REGRESSION"));
        assert!(sps.contains("/DEPENDENT salary"));
        assert!(sps.contains("/METHOD=ENTER educ prevexp."));
    }

    #[test]
    fn test_linear_regression_missing_variable() {
        let cmd = LinearRegressionCommand;
        let mut ds = Some(create_test_dataset());

        let params = json!({
            "dependent_var": "nonexistent",
            "independent_vars": ["educ"]
        });

        let err = cmd.execute(&mut ds, params).expect_err("Should fail");
        assert!(err.to_string().contains("not found"));
    }
}
