use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::glm::{
    compute_univariate_glm, GlmModelType, SumOfSquaresType, UnivariateGlmOptions,
};
use statcraft_core::DataSet;

pub struct UnivariateGlmCommand;

impl UnivariateGlmCommand {
    pub fn to_sps(
        dependent: &str,
        factors: &[String],
        covariates: &[String],
        intercept: bool,
    ) -> String {
        let mut sps = format!("UNIANOVA {dependent}");
        if !factors.is_empty() {
            sps.push_str(&format!(" BY {}", factors.join(" ")));
        }
        if !covariates.is_empty() {
            sps.push_str(&format!(" WITH {}", covariates.join(" ")));
        }
        sps.push_str("\n  /METHOD = SSTYPE(3)");
        let inc_str = if intercept { "INCLUDE" } else { "EXCLUDE" };
        sps.push_str(&format!("\n  /INTERCEPT = {inc_str}"));
        sps.push_str("\n  /CRITERIA = ALPHA(0.05).");
        sps
    }
}

impl EngineCommand for UnivariateGlmCommand {
    fn id(&self) -> &'static str {
        "stat.glm.univariate"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > General Linear Model > Univariate"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependent"],
            "properties": {
                "dependent": {
                    "type": "string",
                    "description": "Dependent numeric variable name"
                },
                "fixed_factors": {
                    "type": "array",
                    "items": { "type": "string" },
                    "default": [],
                    "description": "Categorical fixed factors"
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
                },
                "descriptives": {
                    "type": "boolean",
                    "default": true,
                    "description": "Display descriptive statistics"
                },
                "homogeneity_levene": {
                    "type": "boolean",
                    "default": false,
                    "description": "Levene test of equality of error variances"
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

        let dep_name = params
            .get("dependent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'dependent' is required".to_string(),
            })?;

        let factor_names: Vec<String> = params
            .get("fixed_factors")
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
        let descriptives = params
            .get("descriptives")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let homogeneity_levene = params
            .get("homogeneity_levene")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let dep_col_idx = ds.dictionary.find_index(dep_name).ok_or_else(|| {
            CommandError::ExecutionFailed(format!("Dependent variable '{dep_name}' not found"))
        })?;

        let active_rows = ds.active_row_indices();
        let mut y_vals = Vec::with_capacity(active_rows.len());
        for &r in &active_rows {
            let s = ds.get_cell_display(r, dep_col_idx, false);
            let val = s.parse::<f64>().unwrap_or(f64::NAN);
            y_vals.push(val);
        }

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

        let options = UnivariateGlmOptions {
            dependent: dep_name.to_string(),
            fixed_factors: factor_names,
            covariates: covariate_names,
            model_type: GlmModelType::FullFactorial,
            custom_terms: Vec::new(),
            sum_of_squares: SumOfSquaresType::Type3,
            intercept,
            descriptives,
            partial_eta_squared: true,
            homogeneity_levene,
            confidence_level: 0.95,
        };

        let result = compute_univariate_glm(dep_name, &y_vals, &factors, &covariates, &options)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(result)
            .map_err(|e| CommandError::ExecutionFailed(format!("Serialization error: {e}")))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
