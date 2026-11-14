use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::glm::{compute_multivariate_glm, MultivariateGlmOptions, SumOfSquaresType};
use statcraft_core::DataSet;

pub struct MultivariateGlmCommand;

impl MultivariateGlmCommand {
    pub fn to_sps(
        dependents: &[String],
        factors: &[String],
        covariates: &[String],
        intercept: bool,
    ) -> String {
        let mut sps = format!("GLM {}", dependents.join(" "));
        if !factors.is_empty() {
            sps.push_str(&format!(" BY {}", factors.join(" ")));
        }
        if !covariates.is_empty() {
            sps.push_str(&format!(" WITH {}", covariates.join(" ")));
        }
        sps.push_str("\n  /METHOD = SSTYPE(3)");
        let inc_str = if intercept { "INCLUDE" } else { "EXCLUDE" };
        sps.push_str(&format!("\n  /INTERCEPT = {inc_str}"));
        sps.push_str("\n  /PRINT = TEST(MULTIVAR).");
        sps
    }
}

impl EngineCommand for MultivariateGlmCommand {
    fn id(&self) -> &'static str {
        "stat.glm.multivariate"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > General Linear Model > Multivariate"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependents"],
            "properties": {
                "dependents": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of dependent numeric variables (at least 2)"
                },
                "factors": {
                    "type": "array",
                    "items": { "type": "string" },
                    "default": [],
                    "description": "Categorical factors"
                },
                "covariates": {
                    "type": "array",
                    "items": { "type": "string" },
                    "default": [],
                    "description": "Continuous covariates"
                },
                "intercept": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include intercept in model"
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        match dataset {
            Some(ds) => ds.row_count() > 0 && ds.col_count() >= 3,
            None => false,
        }
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let dep_names: Vec<String> = params
            .get("dependents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if dep_names.len() < 2 {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'dependents' must contain at least 2 variables".to_string(),
            });
        }

        let factor_names: Vec<String> = params
            .get("factors")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let covariate_names: Vec<String> = params
            .get("covariates")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let intercept = params
            .get("intercept")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let active_rows = ds.active_row_indices();
        let mut dep_vectors: Vec<Vec<f64>> = Vec::with_capacity(dep_names.len());
        for name in &dep_names {
            let col_idx = ds.dictionary.find_index(name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Dependent variable '{name}' not found"))
            })?;
            let mut vals = Vec::with_capacity(active_rows.len());
            for &r in &active_rows {
                let s = ds.get_cell_display(r, col_idx, false);
                let val = s.parse::<f64>().unwrap_or(f64::NAN);
                vals.push(val);
            }
            dep_vectors.push(vals);
        }

        let dependents_slices: Vec<(&str, &[f64])> = dep_names
            .iter()
            .enumerate()
            .map(|(idx, name)| (name.as_str(), dep_vectors[idx].as_slice()))
            .collect();

        let mut factors = Vec::with_capacity(factor_names.len());
        for f_name in &factor_names {
            let col_idx = ds.dictionary.find_index(f_name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Factor '{f_name}' not found"))
            })?;
            let mut f_vals = Vec::with_capacity(active_rows.len());
            for &r in &active_rows {
                let s = ds.get_cell_display(r, col_idx, false);
                f_vals.push(s);
            }
            factors.push((f_name.clone(), f_vals));
        }

        let mut covariates = Vec::with_capacity(covariate_names.len());
        for c_name in &covariate_names {
            let col_idx = ds.dictionary.find_index(c_name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Covariate '{c_name}' not found"))
            })?;
            let mut c_vals = Vec::with_capacity(active_rows.len());
            for &r in &active_rows {
                let s = ds.get_cell_display(r, col_idx, false);
                let val = s.parse::<f64>().unwrap_or(f64::NAN);
                c_vals.push(val);
            }
            covariates.push((c_name.clone(), c_vals));
        }

        let options = MultivariateGlmOptions {
            dependents: dep_names.clone(),
            factors: factor_names,
            covariates: covariate_names,
            intercept,
            sum_of_squares: SumOfSquaresType::Type3,
            partial_eta_squared: true,
        };

        let result = compute_multivariate_glm(&dependents_slices, &factors, &covariates, &options)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(result)
            .map_err(|e| CommandError::ExecutionFailed(format!("Serialization error: {e}")))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
