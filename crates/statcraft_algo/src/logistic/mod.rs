pub mod diagnostics;
pub mod engine;
pub mod math;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::*;

use crate::anova_oneway::AlgoError;
use crate::correlations_types::VariableInput;
use diagnostics::{compute_classification_table, compute_hosmer_lemeshow};
use engine::fit_logistic_engine;

/// Computes binary logistic regression for a dependent variable and one or more covariates.
pub fn compute_logistic_regression(
    dependent: VariableInput<'_>,
    covariates: &[VariableInput<'_>],
    options: &LogisticRegressionOptions,
) -> Result<LogisticRegressionResult, AlgoError> {
    if covariates.is_empty() {
        return Err(AlgoError::InvalidInput(
            "At least one covariate is required for logistic regression".to_string(),
        ));
    }

    let dep_name = dependent.0.to_string();
    let n_rows = dependent.1.len();

    // Verify lengths
    for (name, vals) in covariates {
        if vals.len() != n_rows {
            return Err(AlgoError::InvalidInput(format!(
                "Covariate '{name}' length ({}) does not match dependent variable length ({n_rows})",
                vals.len()
            )));
        }
    }

    let cov_names: Vec<String> = covariates.iter().map(|(n, _)| (*n).to_string()).collect();

    // Check for duplicate names
    if cov_names.contains(&dep_name) {
        return Err(AlgoError::InvalidInput(format!(
            "Dependent variable '{dep_name}' cannot also be included as a covariate"
        )));
    }

    // 1. Listwise filter: both dependent and all covariates must be finite
    let mut valid_mask = vec![true; n_rows];
    for (i, val) in dependent.1.iter().enumerate() {
        if val.is_none_or(|v| !v.is_finite()) {
            valid_mask[i] = false;
        }
    }
    for (_, vals) in covariates {
        for (i, val) in vals.iter().enumerate() {
            if val.is_none_or(|v| !v.is_finite()) {
                valid_mask[i] = false;
            }
        }
    }

    // Collect filtered dependent values
    let mut raw_y: Vec<f64> = Vec::new();
    for (i, &val) in dependent.1.iter().enumerate() {
        if valid_mask[i] {
            if let Some(v) = val {
                raw_y.push(v);
            }
        }
    }

    let n_valid = raw_y.len();
    let min_required = cov_names.len() + if options.include_constant { 2 } else { 1 };
    if n_valid < min_required {
        return Err(AlgoError::InsufficientData(format!(
            "Valid sample size ({n_valid}) is insufficient for {} parameters",
            cov_names.len() + usize::from(options.include_constant)
        )));
    }

    // 2. Identify two distinct levels of the dependent variable
    let mut unique_levels: Vec<f64> = Vec::new();
    for &val in &raw_y {
        if !unique_levels.iter().any(|&u| (u - val).abs() < 1e-9) {
            unique_levels.push(val);
        }
    }

    if unique_levels.len() != 2 {
        return Err(AlgoError::InvalidInput(format!(
            "Dependent variable '{dep_name}' must have exactly two distinct values (found {})",
            unique_levels.len()
        )));
    }

    unique_levels.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let _val_zero = unique_levels[0];
    let val_one = unique_levels[1];

    let y_binary: Vec<f64> = raw_y
        .iter()
        .map(|&v| if (v - val_one).abs() < 1e-9 { 1.0 } else { 0.0 })
        .collect();

    // 3. Build design matrix X
    let mut x_matrix: Vec<Vec<f64>> = Vec::with_capacity(n_valid);
    for row_idx in 0..n_rows {
        if !valid_mask[row_idx] {
            continue;
        }
        let mut row_vec = Vec::new();
        if options.include_constant {
            row_vec.push(1.0);
        }
        for (_, vals) in covariates {
            let val = vals[row_idx].unwrap_or(0.0);
            row_vec.push(val);
        }
        x_matrix.push(row_vec);
    }

    let mut param_names = Vec::new();
    if options.include_constant {
        param_names.push("(Constant)".to_string());
    }
    param_names.extend(cov_names.clone());

    // 4. Fit logistic model via IRLS
    let engine_output = fit_logistic_engine(&x_matrix, &y_binary, &param_names, options)?;

    // 5. Diagnostics
    let classification_table = compute_classification_table(
        &y_binary,
        &engine_output.predicted_probs,
        options.classification_cutoff,
    );

    let hosmer_lemeshow = if options.hosmer_lemeshow {
        compute_hosmer_lemeshow(&y_binary, &engine_output.predicted_probs)
    } else {
        None
    };

    Ok(LogisticRegressionResult {
        dependent_var: dep_name,
        covariates: cov_names,
        coefficients: engine_output.coefficients,
        omnibus: engine_output.omnibus,
        model_summary: engine_output.model_summary,
        classification_table,
        hosmer_lemeshow,
        n_cases: n_valid,
        iterations: engine_output.iterations,
        converged: engine_output.converged,
        warnings: engine_output.warnings,
    })
}
