use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_curve_fit, CurveFitOptions, CurveModelType};
use statcraft_core::DataSet;

pub struct CurveFitCommand;

impl CurveFitCommand {
    pub fn to_sps(
        dep_var: &str,
        indep_var: &str,
        models: &[CurveModelType],
        include_constant: bool,
    ) -> String {
        let model_names: Vec<String> = models.iter().map(|m| m.name().to_uppercase()).collect();
        let const_str = if include_constant {
            "/CONSTANT"
        } else {
            "/NOCONSTANT"
        };

        format!(
            "CURVEFIT\n  /VARIABLES = {} WITH {}\n  /MODEL = {}\n  {}.",
            dep_var,
            indep_var,
            model_names.join(" "),
            const_str
        )
    }
}

impl EngineCommand for CurveFitCommand {
    fn id(&self) -> &'static str {
        "stat.regression.curve"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Regression > Curve Estimation"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependent_var", "independent_var"],
            "properties": {
                "dependent_var": {
                    "type": "string",
                    "description": "Name of the quantitative dependent variable"
                },
                "independent_var": {
                    "type": "string",
                    "description": "Name of the quantitative independent predictor variable"
                },
                "models": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": ["linear", "logarithmic", "inverse", "quadratic", "power", "exponential"]
                    },
                    "description": "List of curve models to evaluate"
                },
                "include_constant": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include intercept (constant) in equations"
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

        let indep_var = params
            .get("independent_var")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'independent_var' is required".to_string(),
            })?;

        let dep_col_idx = ds.dictionary.find_index(dep_var).ok_or_else(|| {
            CommandError::ExecutionFailed(format!(
                "Dependent variable '{dep_var}' not found in dataset"
            ))
        })?;

        let indep_col_idx = ds.dictionary.find_index(indep_var).ok_or_else(|| {
            CommandError::ExecutionFailed(format!(
                "Independent variable '{indep_var}' not found in dataset"
            ))
        })?;

        let models: Vec<CurveModelType> = params
            .get("models")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| match m.as_str().unwrap_or("").to_lowercase().as_str() {
                        "linear" => Some(CurveModelType::Linear),
                        "logarithmic" | "log" => Some(CurveModelType::Logarithmic),
                        "inverse" | "inv" => Some(CurveModelType::Inverse),
                        "quadratic" | "quad" => Some(CurveModelType::Quadratic),
                        "power" => Some(CurveModelType::Power),
                        "exponential" | "exp" => Some(CurveModelType::Exponential),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_else(CurveModelType::all);

        let include_constant = params
            .get("include_constant")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let active_rows = ds.active_row_indices();
        let mut dep_vals = Vec::with_capacity(active_rows.len());
        let mut indep_vals = Vec::with_capacity(active_rows.len());

        for &r in &active_rows {
            let val_dep = ds.get_cell_numeric(r, dep_col_idx);
            dep_vals.push(val_dep);

            let val_indep = ds.get_cell_numeric(r, indep_col_idx);
            indep_vals.push(val_indep);
        }

        let options = CurveFitOptions {
            models,
            include_constant,
            confidence_level: 0.95,
        };

        let result = compute_curve_fit(
            (dep_var, dep_vals.as_slice()),
            (indep_var, indep_vals.as_slice()),
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
