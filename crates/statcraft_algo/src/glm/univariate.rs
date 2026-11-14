use crate::anova_oneway::{compute_oneway_anova_from_slices, AlgoError, OneWayAnovaOptions};
use crate::glm::design_matrix::build_design_matrix;
use crate::glm::type3_ss::compute_type3_ss;
use crate::glm::types::{GlmModelType, UnivariateGlmOptions, UnivariateGlmResult};

/// Solves Univariate General Linear Model (UNIANOVA) with Type III Sum of Squares.
#[allow(clippy::needless_range_loop)]
pub fn compute_univariate_glm(
    dependent_name: &str,
    y_raw: &[f64],
    factors: &[(String, Vec<String>)],
    covariates: &[(String, Vec<f64>)],
    options: &UnivariateGlmOptions,
) -> Result<UnivariateGlmResult, AlgoError> {
    let n_total = y_raw.len();
    if n_total == 0 {
        return Err(AlgoError::InvalidInput(
            "Dependent variable is empty".to_string(),
        ));
    }

    // 1. Validate lengths
    for (f_name, f_vals) in factors {
        if f_vals.len() != n_total {
            return Err(AlgoError::InvalidInput(format!(
                "Factor {} length mismatch",
                f_name
            )));
        }
    }
    for (c_name, c_vals) in covariates {
        if c_vals.len() != n_total {
            return Err(AlgoError::InvalidInput(format!(
                "Covariate {} length mismatch",
                c_name
            )));
        }
    }

    // 2. Listwise deletion filter
    let mut valid_indices = Vec::new();
    for i in 0..n_total {
        if y_raw[i].is_nan() {
            continue;
        }
        let mut has_nan = false;
        for (_, c_vals) in covariates {
            if c_vals[i].is_nan() {
                has_nan = true;
                break;
            }
        }
        if has_nan {
            continue;
        }
        valid_indices.push(i);
    }

    let n_valid = valid_indices.len();
    if n_valid < 3 {
        return Err(AlgoError::InsufficientData(
            "At least 3 valid observations required for GLM".to_string(),
        ));
    }

    let y_filtered: Vec<f64> = valid_indices.iter().map(|&i| y_raw[i]).collect();

    // 3. Extract unique sorted levels for factors
    let mut factors_with_levels = Vec::new();
    for (f_name, f_vals) in factors {
        let filtered_f: Vec<String> = valid_indices.iter().map(|&i| f_vals[i].clone()).collect();
        let mut unique_levels: Vec<String> = filtered_f.clone();
        unique_levels.sort();
        unique_levels.dedup();
        factors_with_levels.push((f_name.clone(), filtered_f, unique_levels));
    }

    let covariates_filtered: Vec<(String, Vec<f64>)> = covariates
        .iter()
        .map(|(name, vals)| {
            let filtered = valid_indices.iter().map(|&i| vals[i]).collect();
            (name.clone(), filtered)
        })
        .collect();

    // 4. Generate terms
    let interactions = match options.model_type {
        GlmModelType::FullFactorial => {
            // All combinations of fixed factors
            let f_names: Vec<String> = factors.iter().map(|(n, _)| n.clone()).collect();
            let mut inters = Vec::new();
            if f_names.len() >= 2 {
                // 2-way interactions
                for i in 0..f_names.len() {
                    for j in (i + 1)..f_names.len() {
                        inters.push(vec![f_names[i].clone(), f_names[j].clone()]);
                    }
                }
                // 3-way interactions if 3 factors
                if f_names.len() >= 3 {
                    for i in 0..f_names.len() {
                        for j in (i + 1)..f_names.len() {
                            for k in (j + 1)..f_names.len() {
                                inters.push(vec![
                                    f_names[i].clone(),
                                    f_names[j].clone(),
                                    f_names[k].clone(),
                                ]);
                            }
                        }
                    }
                }
            }
            inters
        }
        GlmModelType::Custom => {
            // Custom interaction terms
            options
                .custom_terms
                .iter()
                .filter(|term| term.len() >= 2)
                .cloned()
                .collect()
        }
    };

    // 5. Build design matrix
    let design = build_design_matrix(
        n_valid,
        options.intercept,
        &factors_with_levels,
        &interactions,
        &covariates_filtered,
    )?;

    // 6. Compute Type III SS
    let mut result = compute_type3_ss(dependent_name, &design, &y_filtered, options.intercept)?;

    // 7. Optional Levene test and Group Descriptives
    if (options.homogeneity_levene || options.descriptives) && !factors.is_empty() {
        // Construct composite cell group strings (e.g. "Factor1_val:Factor2_val")
        let mut cell_labels = Vec::with_capacity(n_valid);
        for row_idx in 0..n_valid {
            let label = factors_with_levels
                .iter()
                .map(|(_, vals, _)| vals[row_idx].as_str())
                .collect::<Vec<_>>()
                .join(" / ");
            cell_labels.push(label);
        }

        let oneway_opts = OneWayAnovaOptions {
            descriptives: options.descriptives,
            homogeneity: options.homogeneity_levene,
            confidence_level: options.confidence_level,
        };

        if let Ok(oneway_res) = compute_oneway_anova_from_slices(
            dependent_name,
            &y_filtered,
            "Cells",
            &cell_labels,
            &oneway_opts,
        ) {
            result.levene_test = oneway_res.homogeneity;
            result.group_descriptives = oneway_res.descriptives;
        }
    }

    Ok(result)
}
