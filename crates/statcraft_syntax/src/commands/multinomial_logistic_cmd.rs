use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_multinomial_logistic, MultinomialLogisticOptions, ReferenceCategory};
use statcraft_core::DataSet;

pub struct MultinomialLogisticCommand;

impl MultinomialLogisticCommand {
    pub fn to_sps(
        dependent_var: &str,
        factors: &[String],
        covariates: &[String],
        ref_cat_str: &str,
        ci: f64,
    ) -> String {
        let ci_pct = (ci * 100.0).round() as usize;
        let by_part = if factors.is_empty() {
            String::new()
        } else {
            format!(" BY {}", factors.join(" "))
        };
        let with_part = if covariates.is_empty() {
            String::new()
        } else {
            format!(" WITH {}", covariates.join(" "))
        };
        let mut model_vars = Vec::new();
        model_vars.extend_from_slice(factors);
        model_vars.extend_from_slice(covariates);

        format!(
            "NOMREG {} (BASE={}){}{}\n  /CRITERIA = CIN({})\n  /MODEL = {}\n  /PRINT = PARAMETER SUMMARY CLASSTBL FIT LRT.",
            dependent_var,
            ref_cat_str,
            by_part,
            with_part,
            ci_pct,
            model_vars.join(" ")
        )
    }
}

impl EngineCommand for MultinomialLogisticCommand {
    fn id(&self) -> &'static str {
        "stat.regression.multinomial"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Regression > Multinomial Logistic"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependent_var"],
            "properties": {
                "dependent_var": {
                    "type": "string",
                    "description": "Multinomial categorical dependent variable name"
                },
                "factors": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Categorical factor variable names"
                },
                "covariates": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Continuous covariate variable names"
                },
                "reference_category": {
                    "type": "string",
                    "enum": ["last", "first", "custom"],
                    "default": "last",
                    "description": "Baseline category policy (last, first, or custom)"
                },
                "reference_value": {
                    "type": "number",
                    "description": "Custom reference category value if reference_category is custom"
                },
                "confidence_level": {
                    "type": "number",
                    "default": 0.95,
                    "description": "Confidence level for Exp(B) (e.g. 0.95)"
                },
                "include_constant": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include intercept constant in the model"
                },
                "print_classification": {
                    "type": "boolean",
                    "default": true,
                    "description": "Output classification table"
                },
                "print_goodness_of_fit": {
                    "type": "boolean",
                    "default": true,
                    "description": "Output Pearson and Deviance Goodness-of-Fit tests"
                },
                "print_lrt": {
                    "type": "boolean",
                    "default": true,
                    "description": "Output Likelihood Ratio Tests"
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

        let factors: Vec<String> = params
            .get("factors")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let covariates: Vec<String> = params
            .get("covariates")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if factors.is_empty() && covariates.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "At least one factor or covariate is required".to_string(),
            });
        }

        let dep_col_idx = ds.dictionary.find_index(dep_var).ok_or_else(|| {
            CommandError::ExecutionFailed(format!(
                "Dependent variable '{dep_var}' not found in dataset"
            ))
        })?;

        let mut factor_col_indices = Vec::with_capacity(factors.len());
        for name in &factors {
            let col_idx = ds.dictionary.find_index(name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Factor '{name}' not found in dataset"))
            })?;
            factor_col_indices.push(col_idx);
        }

        let mut cov_col_indices = Vec::with_capacity(covariates.len());
        for name in &covariates {
            let col_idx = ds.dictionary.find_index(name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Covariate '{name}' not found in dataset"))
            })?;
            cov_col_indices.push(col_idx);
        }

        let ref_policy_str = params
            .get("reference_category")
            .and_then(|v| v.as_str())
            .unwrap_or("last");
        let ref_category = match ref_policy_str {
            "first" => ReferenceCategory::First,
            "custom" => {
                if let Some(val) = params.get("reference_value").and_then(|v| v.as_f64()) {
                    ReferenceCategory::Value(val)
                } else {
                    ReferenceCategory::Last
                }
            }
            _ => ReferenceCategory::Last,
        };

        let ci = params
            .get("confidence_level")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.95);
        let include_constant = params
            .get("include_constant")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let print_classification = params
            .get("print_classification")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let print_goodness_of_fit = params
            .get("print_goodness_of_fit")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let print_lrt = params
            .get("print_lrt")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let active_rows = ds.active_row_indices();
        let n_active = active_rows.len();

        let mut dep_vals = Vec::with_capacity(n_active);
        for &r in &active_rows {
            let val = ds.get_cell_numeric(r, dep_col_idx);
            dep_vals.push(val);
        }

        let mut factor_vals: Vec<Vec<Option<f64>>> =
            vec![Vec::with_capacity(n_active); factors.len()];
        for &r in &active_rows {
            for (f_idx, &col_idx) in factor_col_indices.iter().enumerate() {
                let val = ds.get_cell_numeric(r, col_idx);
                factor_vals[f_idx].push(val);
            }
        }

        let mut cov_vals: Vec<Vec<Option<f64>>> =
            vec![Vec::with_capacity(n_active); covariates.len()];
        for &r in &active_rows {
            for (c_idx, &col_idx) in cov_col_indices.iter().enumerate() {
                let val = ds.get_cell_numeric(r, col_idx);
                cov_vals[c_idx].push(val);
            }
        }

        let factor_inputs: Vec<(&str, &[Option<f64>])> = factors
            .iter()
            .zip(factor_vals.iter())
            .map(|(name, vals)| (name.as_str(), vals.as_slice()))
            .collect();

        let cov_inputs: Vec<(&str, &[Option<f64>])> = covariates
            .iter()
            .zip(cov_vals.iter())
            .map(|(name, vals)| (name.as_str(), vals.as_slice()))
            .collect();

        let labels_map: Vec<(f64, String)> = ds
            .dictionary
            .get(dep_col_idx)
            .map(|v| {
                v.values
                    .mapping
                    .iter()
                    .filter_map(|(val_str, label)| {
                        val_str.parse::<f64>().ok().map(|num| (num, label.clone()))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let options = MultinomialLogisticOptions {
            reference_category: ref_category,
            include_constant,
            confidence_level: ci,
            max_iterations: 50,
            convergence_tol: 1e-6,
            print_classification,
            print_goodness_of_fit,
            print_lrt,
        };

        let result = compute_multinomial_logistic(
            (dep_var, dep_vals.as_slice()),
            &factor_inputs,
            &cov_inputs,
            Some(&labels_map),
            &options,
        )
        .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(result)
            .map_err(|e| CommandError::ExecutionFailed(format!("Serialization error: {e}")))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::{DataType, Measure, RawValue};

    #[test]
    fn test_multinomial_logistic_to_sps() {
        let sps = MultinomialLogisticCommand::to_sps(
            "prog",
            &["ses".to_string()],
            &["write".to_string(), "math".to_string()],
            "LAST",
            0.95,
        );
        assert!(sps.contains("NOMREG prog (BASE=LAST) BY ses WITH write math"));
        assert!(sps.contains("/CRITERIA = CIN(95)"));
        assert!(sps.contains("/MODEL = ses write math"));
        assert!(sps.contains("/PRINT = PARAMETER SUMMARY CLASSTBL FIT LRT."));
    }

    #[test]
    fn test_multinomial_logistic_command_execute() {
        let mut ds = DataSet::empty();
        let c_choice = ds
            .add_variable(
                "choice",
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )
            .unwrap();
        let c_x1 = ds
            .add_variable("x1", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();

        let choice_vals = [0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
        let x1_vals = [
            10.0, 12.0, 20.0, 22.0, 30.0, 32.0, 11.0, 21.0, 31.0, 13.0, 23.0, 33.0,
        ];

        for i in 0..choice_vals.len() {
            ds.add_empty_row();
            ds.set_cell(i, c_choice, RawValue::Number(choice_vals[i]))
                .unwrap();
            ds.set_cell(i, c_x1, RawValue::Number(x1_vals[i])).unwrap();
        }

        let cmd = MultinomialLogisticCommand;
        assert!(cmd.enabled(Some(&ds)));

        let mut opt_ds = Some(ds);
        let params = json!({
            "dependent_var": "choice",
            "covariates": ["x1"],
            "reference_category": "last"
        });

        let res = cmd
            .execute(&mut opt_ds, params)
            .expect("execution should succeed");
        assert_eq!(res["dependent_var"], "choice");
        assert_eq!(res["reference_category_val"], 2.0);
        assert_eq!(res["n_cases"], 12);
        assert!(res["converged"].as_bool().unwrap_or(false));
    }
}
