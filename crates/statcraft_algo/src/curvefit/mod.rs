pub mod engine;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::*;

use crate::anova_oneway::AlgoError;
use crate::correlations_types::VariableInput;
use engine::fit_single_curve_model;

/// Fits selected regression curve models for a dependent and independent variable.
pub fn compute_curve_fit(
    dependent: VariableInput<'_>,
    independent: VariableInput<'_>,
    options: &CurveFitOptions,
) -> Result<CurveFitResult, AlgoError> {
    let dep_name = dependent.0.to_string();
    let indep_name = independent.0.to_string();

    if dep_name == indep_name {
        return Err(AlgoError::InvalidInput(
            "Dependent and independent variables cannot be the same".to_string(),
        ));
    }

    let n_rows = dependent.1.len();
    if independent.1.len() != n_rows {
        return Err(AlgoError::InvalidInput(format!(
            "Independent variable length ({}) does not match dependent variable length ({n_rows})",
            independent.1.len()
        )));
    }

    if options.models.is_empty() {
        return Err(AlgoError::InvalidInput(
            "At least one curve model must be selected".to_string(),
        ));
    }

    // Listwise filtering
    let mut valid_x: Vec<f64> = Vec::with_capacity(n_rows);
    let mut valid_y: Vec<f64> = Vec::with_capacity(n_rows);

    for i in 0..n_rows {
        if let (Some(y_val), Some(x_val)) = (dependent.1[i], independent.1[i]) {
            if y_val.is_finite() && x_val.is_finite() {
                valid_y.push(y_val);
                valid_x.push(x_val);
            }
        }
    }

    if valid_x.len() < 3 {
        return Err(AlgoError::InsufficientData(format!(
            "At least 3 valid pairs are required for curve estimation (found {})",
            valid_x.len()
        )));
    }

    let mut model_summaries = Vec::with_capacity(options.models.len());
    let mut warnings = Vec::new();

    for &model in &options.models {
        match fit_single_curve_model(model, &valid_x, &valid_y, options.include_constant) {
            Ok(summary) => model_summaries.push(summary),
            Err(e) => {
                warnings.push(format!("Model {}: {}", model.name(), e));
            }
        }
    }

    if model_summaries.is_empty() {
        return Err(AlgoError::ComputationError(
            "None of the selected curve models could be fitted to the data".to_string(),
        ));
    }

    Ok(CurveFitResult {
        dependent_var: dep_name,
        independent_var: indep_name,
        models: model_summaries,
        warnings,
    })
}
