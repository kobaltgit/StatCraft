use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_logistic_regression, LogisticRegressionOptions};
use statcraft_core::DataSet;

pub struct LogisticRegressionCommand;

impl LogisticRegressionCommand {
    pub fn to_sps(
        dependent_var: &str,
        covariates: &[String],
        cutoff: f64,
        ci: f64,
        hosmer_lemeshow: bool,
    ) -> String {
        let ci_pct = (ci * 100.0).round() as usize;
        let mut print_sub = format!("CI({ci_pct})");
        if hosmer_lemeshow {
            print_sub.push_str(" GOODFIT");
        }

        format!(
            "LOGISTIC REGRESSION VARIABLES {}\n  /METHOD = ENTER {}\n  /CRITERIA = CUT({:.2}) CI({})\n  /PRINT = {}.",
            dependent_var,
            covariates.join(" "),
            cutoff,
            ci_pct,
            print_sub
        )
    }
}

impl EngineCommand for LogisticRegressionCommand {
    fn id(&self) -> &'static str {
        "stat.regression.logistic"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Regression > Binary Logistic"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependent_var", "covariates"],
            "properties": {
                "dependent_var": {
                    "type": "string",
                    "description": "Binary dependent variable name (0/1)"
                },
                "covariates": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of covariate (predictor) variable names"
                },
                "cutoff": {
                    "type": "number",
                    "default": 0.5,
                    "description": "Classification table probability cutoff"
                },
                "ci": {
                    "type": "number",
                    "default": 0.95,
                    "description": "Confidence level for Exp(B) (e.g. 0.95)"
                },
                "hosmer_lemeshow": {
                    "type": "boolean",
                    "default": true,
                    "description": "Compute Hosmer-Lemeshow goodness-of-fit test"
                },
                "include_constant": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include constant in the equation"
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

        let covariates: Vec<String> = params
            .get("covariates")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if covariates.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'covariates' must contain at least one predictor".to_string(),
            });
        }

        let dep_col_idx = ds.dictionary.find_index(dep_var).ok_or_else(|| {
            CommandError::ExecutionFailed(format!(
                "Dependent variable '{dep_var}' not found in dataset"
            ))
        })?;

        let mut cov_col_indices = Vec::with_capacity(covariates.len());
        for name in &covariates {
            let col_idx = ds.dictionary.find_index(name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!(
                    "Covariate variable '{name}' not found in dataset"
                ))
            })?;
            cov_col_indices.push(col_idx);
        }

        let cutoff = params.get("cutoff").and_then(|v| v.as_f64()).unwrap_or(0.5);
        let ci = params.get("ci").and_then(|v| v.as_f64()).unwrap_or(0.95);
        let hosmer_lemeshow = params
            .get("hosmer_lemeshow")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let include_constant = params
            .get("include_constant")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let active_rows = ds.active_row_indices();
        let n_active = active_rows.len();

        let mut dep_vals = Vec::with_capacity(n_active);
        for &r in &active_rows {
            let val = ds.get_cell_numeric(r, dep_col_idx);
            dep_vals.push(val);
        }

        let mut cov_vals: Vec<Vec<Option<f64>>> =
            vec![Vec::with_capacity(n_active); covariates.len()];
        for &r in &active_rows {
            for (c_idx, &col_idx) in cov_col_indices.iter().enumerate() {
                let val = ds.get_cell_numeric(r, col_idx);
                cov_vals[c_idx].push(val);
            }
        }

        let cov_inputs: Vec<(&str, &[Option<f64>])> = covariates
            .iter()
            .zip(cov_vals.iter())
            .map(|(name, vals)| (name.as_str(), vals.as_slice()))
            .collect();

        let options = LogisticRegressionOptions {
            include_constant,
            classification_cutoff: cutoff,
            confidence_level: ci,
            hosmer_lemeshow,
            max_iterations: 25,
            convergence_tol: 1e-6,
        };

        let result =
            compute_logistic_regression((dep_var, dep_vals.as_slice()), &cov_inputs, &options)
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(result)
            .map_err(|e| CommandError::ExecutionFailed(format!("Serialization error: {e}")))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
