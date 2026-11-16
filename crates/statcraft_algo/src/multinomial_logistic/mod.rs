pub mod diagnostics;
pub mod engine;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::*;

use crate::anova_oneway::AlgoError;
use crate::correlations_types::VariableInput;
use diagnostics::{
    compute_classification_table, compute_goodness_of_fit, compute_likelihood_ratio_tests,
    compute_model_fitting_info, compute_parameter_estimates, compute_pseudo_r_squared,
};
use engine::fit_multinomial_engine;

/// Computes multinomial logistic regression (NOMREG) for categorical/continuous predictors.
pub fn compute_multinomial_logistic(
    dependent: VariableInput<'_>,
    factors: &[VariableInput<'_>],
    covariates: &[VariableInput<'_>],
    labels_map: Option<&[(f64, String)]>,
    options: &MultinomialLogisticOptions,
) -> Result<MultinomialLogisticResult, AlgoError> {
    if factors.is_empty() && covariates.is_empty() {
        return Err(AlgoError::InvalidInput(
            "At least one factor or covariate is required for multinomial logistic regression"
                .to_string(),
        ));
    }

    let dep_name = dependent.0.to_string();
    let n_total_rows = dependent.1.len();

    // Verify lengths
    for (name, vals) in factors.iter().chain(covariates.iter()) {
        if vals.len() != n_total_rows {
            return Err(AlgoError::InvalidInput(format!(
                "Variable '{name}' length ({}) does not match dependent variable length ({n_total_rows})",
                vals.len()
            )));
        }
    }

    let factor_names: Vec<String> = factors.iter().map(|(n, _)| (*n).to_string()).collect();
    let cov_names: Vec<String> = covariates.iter().map(|(n, _)| (*n).to_string()).collect();

    if factor_names.contains(&dep_name) || cov_names.contains(&dep_name) {
        return Err(AlgoError::InvalidInput(format!(
            "Dependent variable '{dep_name}' cannot also be included as a predictor"
        )));
    }

    // 1. Listwise missing value filter
    let mut valid_mask = vec![true; n_total_rows];
    for (i, val) in dependent.1.iter().enumerate() {
        if val.is_none_or(|v| !v.is_finite()) {
            valid_mask[i] = false;
        }
    }
    for (_, vals) in factors.iter().chain(covariates.iter()) {
        for (i, val) in vals.iter().enumerate() {
            if val.is_none_or(|v| !v.is_finite()) {
                valid_mask[i] = false;
            }
        }
    }

    let mut raw_y: Vec<f64> = Vec::new();
    for (i, &val) in dependent.1.iter().enumerate() {
        if valid_mask[i] {
            if let Some(v) = val {
                raw_y.push(v);
            }
        }
    }

    let n_valid = raw_y.len();
    let n_missing = n_total_rows.saturating_sub(n_valid);

    if n_valid < 5 {
        return Err(AlgoError::InsufficientData(format!(
            "Valid sample size ({n_valid}) is insufficient for multinomial logistic regression"
        )));
    }

    // 2. Extract unique categories of dependent variable
    let mut unique_cats: Vec<f64> = Vec::new();
    for &v in &raw_y {
        if !unique_cats.iter().any(|&u| (u - v).abs() < 1e-9) {
            unique_cats.push(v);
        }
    }
    unique_cats.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if unique_cats.len() < 2 {
        return Err(AlgoError::InvalidInput(format!(
            "Dependent variable '{dep_name}' must have at least 2 distinct categories (found {})",
            unique_cats.len()
        )));
    }

    // Select reference category
    let ref_val = match options.reference_category {
        ReferenceCategory::First => unique_cats[0],
        ReferenceCategory::Last => *unique_cats.last().unwrap_or(&0.0),
        ReferenceCategory::Value(val) => {
            if unique_cats.iter().any(|&c| (c - val).abs() < 1e-9) {
                val
            } else {
                *unique_cats.last().unwrap_or(&0.0)
            }
        }
    };

    // Reorder categories so that non-reference categories are 0..M, reference is at index M
    let mut ordered_cats: Vec<f64> = unique_cats
        .iter()
        .copied()
        .filter(|&c| (c - ref_val).abs() >= 1e-9)
        .collect();
    ordered_cats.push(ref_val);

    let j_total = ordered_cats.len();
    let m = j_total - 1;
    let ref_idx = m;

    // Helper to resolve category label
    let get_label = |val: f64| -> Option<String> {
        labels_map.and_then(|m| {
            m.iter()
                .find(|(k, _)| (k - val).abs() < 1e-9)
                .map(|(_, l)| l.clone())
        })
    };

    let ordered_labels: Vec<Option<String>> = ordered_cats.iter().map(|&c| get_label(c)).collect();

    // Map raw_y to category indices (0..m non-reference, m is reference)
    let mut y_indices = Vec::with_capacity(n_valid);
    let mut cat_counts = vec![0; j_total];
    for &y_val in &raw_y {
        let idx = ordered_cats
            .iter()
            .position(|&c| (c - y_val).abs() < 1e-9)
            .unwrap_or(ref_idx);
        y_indices.push(idx);
        cat_counts[idx] += 1;
    }

    // 3. Build design matrix X with dummy coding for factors and columns for covariates
    let mut param_names = Vec::new();
    let mut effect_cols: Vec<(String, Vec<usize>)> = Vec::new();

    if options.include_constant {
        param_names.push("Intercept".to_string());
        effect_cols.push(("Intercept".to_string(), vec![0]));
    }

    // Map factors to dummy columns
    struct FactorSpec {
        non_ref_levels: Vec<f64>,
    }
    let mut factor_specs = Vec::new();
    for (f_name, vals) in factors {
        let mut f_unique: Vec<f64> = Vec::new();
        for (i, val) in vals.iter().enumerate() {
            if valid_mask[i] {
                if let Some(v) = val {
                    if !f_unique.iter().any(|&u| (u - *v).abs() < 1e-9) {
                        f_unique.push(*v);
                    }
                }
            }
        }
        f_unique.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // Last level of factor is reference level
        let non_ref_levels: Vec<f64> = if f_unique.len() > 1 {
            f_unique[..f_unique.len() - 1].to_vec()
        } else {
            f_unique
        };

        let _start_col = param_names.len();
        let mut cols = Vec::new();
        for &lvl in &non_ref_levels {
            cols.push(param_names.len());
            param_names.push(format!("[{}={}]", f_name, lvl));
        }
        effect_cols.push(((*f_name).to_string(), cols));

        factor_specs.push((FactorSpec { non_ref_levels }, *vals));
    }

    // Covariates directly
    for (c_name, _) in covariates {
        let col = param_names.len();
        param_names.push((*c_name).to_string());
        effect_cols.push(((*c_name).to_string(), vec![col]));
    }

    let p = param_names.len();

    // Construct numeric rows of X
    let mut x_matrix: Vec<Vec<f64>> = Vec::with_capacity(n_valid);
    for row_idx in 0..n_total_rows {
        if !valid_mask[row_idx] {
            continue;
        }
        let mut row_vec = Vec::with_capacity(p);
        if options.include_constant {
            row_vec.push(1.0);
        }
        for (f_spec, f_vals) in &factor_specs {
            let f_val = f_vals[row_idx].unwrap_or(0.0);
            for &lvl in &f_spec.non_ref_levels {
                let indicator = if (f_val - lvl).abs() < 1e-9 { 1.0 } else { 0.0 };
                row_vec.push(indicator);
            }
        }
        for (_, c_vals) in covariates {
            let c_val = c_vals[row_idx].unwrap_or(0.0);
            row_vec.push(c_val);
        }
        x_matrix.push(row_vec);
    }

    // 4. Fit multinomial engine
    let engine_output = fit_multinomial_engine(&x_matrix, &y_indices, &cat_counts, m, p, options)?;

    // 5. Diagnostics
    let model_fitting =
        compute_model_fitting_info(&engine_output, n_valid, m, p, options.include_constant);

    let goodness_of_fit = if options.print_goodness_of_fit {
        Some(compute_goodness_of_fit(
            &x_matrix,
            &y_indices,
            &engine_output.predicted_probs,
            j_total,
            p,
        ))
    } else {
        None
    };

    let pseudo_r_squared = compute_pseudo_r_squared(
        engine_output.minus_2ll_intercept_only,
        engine_output.minus_2ll,
        n_valid,
    );

    let likelihood_ratio_tests = if options.print_lrt {
        compute_likelihood_ratio_tests(
            &x_matrix,
            &y_indices,
            &cat_counts,
            m,
            &effect_cols,
            engine_output.minus_2ll,
            options,
        )
    } else {
        Vec::new()
    };

    let parameter_estimates = compute_parameter_estimates(
        &engine_output,
        &ordered_cats,
        &ordered_labels,
        &param_names,
        m,
        p,
        ref_idx,
        options.confidence_level,
    );

    let classification_table = if options.print_classification {
        Some(compute_classification_table(
            &y_indices,
            &engine_output.predicted_probs,
            &ordered_cats,
            &ordered_labels,
        ))
    } else {
        None
    };

    // Case Processing Summary
    let mut case_categories = Vec::with_capacity(j_total);
    for j in 0..j_total {
        let cnt = cat_counts[j];
        let pct = if n_valid > 0 {
            (cnt as f64) / (n_valid as f64) * 100.0
        } else {
            0.0
        };
        case_categories.push(CaseProcessingCategory {
            category_value: ordered_cats[j],
            category_label: ordered_labels[j].clone(),
            n: cnt,
            percent: pct,
        });
    }

    let case_processing = MultinomialCaseProcessingSummary {
        categories: case_categories,
        valid_n: n_valid,
        valid_percent: if n_total_rows > 0 {
            (n_valid as f64) / (n_total_rows as f64) * 100.0
        } else {
            0.0
        },
        missing_n: n_missing,
        total_n: n_total_rows,
    };

    Ok(MultinomialLogisticResult {
        dependent_var: dep_name,
        factors: factor_names,
        covariates: cov_names,
        reference_category_val: ref_val,
        categories: ordered_cats,
        case_processing,
        model_fitting,
        goodness_of_fit,
        pseudo_r_squared,
        likelihood_ratio_tests,
        parameter_estimates,
        classification_table,
        n_cases: n_valid,
        iterations: engine_output.iterations,
        converged: engine_output.converged,
        warnings: engine_output.warnings,
    })
}
