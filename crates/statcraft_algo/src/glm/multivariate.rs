use crate::anova_oneway::AlgoError;
use crate::glm::design_matrix::{build_design_matrix, extract_submatrix, solve_ols_svd};
use crate::glm::eigen::{compute_multivariate_tests, generalized_eigenvalues};
use crate::glm::type3_ss::compute_type3_ss;
use crate::glm::types::{MultivariateEffectRow, MultivariateGlmOptions, MultivariateGlmResult};

/// Computes Multivariate General Linear Model (MANOVA).
#[allow(clippy::needless_range_loop)]
pub fn compute_multivariate_glm(
    dependents: &[(&str, &[f64])],
    factors: &[(String, Vec<String>)],
    covariates: &[(String, Vec<f64>)],
    options: &MultivariateGlmOptions,
) -> Result<MultivariateGlmResult, AlgoError> {
    let p_dep = dependents.len();
    if p_dep < 2 {
        return Err(AlgoError::InvalidInput(
            "Multivariate GLM requires at least 2 dependent variables".to_string(),
        ));
    }
    let n_total = dependents[0].1.len();
    if n_total == 0 {
        return Err(AlgoError::InvalidInput(
            "Dependent variables are empty".to_string(),
        ));
    }

    // 1. Verify lengths
    for (d_name, d_vals) in dependents {
        if d_vals.len() != n_total {
            return Err(AlgoError::InvalidInput(format!(
                "Dependent variable {} length mismatch",
                d_name
            )));
        }
    }
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
        let mut has_nan = false;
        for (_, d_vals) in dependents {
            if d_vals[i].is_nan() {
                has_nan = true;
                break;
            }
        }
        if !has_nan {
            for (_, c_vals) in covariates {
                if c_vals[i].is_nan() {
                    has_nan = true;
                    break;
                }
            }
        }
        if !has_nan {
            valid_indices.push(i);
        }
    }

    let n_valid = valid_indices.len();
    if n_valid < p_dep + 2 {
        return Err(AlgoError::InsufficientData(
            "Insufficient valid observations for MANOVA".to_string(),
        ));
    }

    let y_matrices: Vec<Vec<f64>> = dependents
        .iter()
        .map(|(_, vals)| valid_indices.iter().map(|&i| vals[i]).collect())
        .collect();

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

    // 4. Generate factorial interactions
    let f_names: Vec<String> = factors.iter().map(|(n, _)| n.clone()).collect();
    let mut interactions = Vec::new();
    if f_names.len() >= 2 {
        for i in 0..f_names.len() {
            for j in (i + 1)..f_names.len() {
                interactions.push(vec![f_names[i].clone(), f_names[j].clone()]);
            }
        }
    }

    // 5. Build full design matrix
    let design = build_design_matrix(
        n_valid,
        options.intercept,
        &factors_with_levels,
        &interactions,
        &covariates_filtered,
    )?;

    // 6. Compute Full model residuals for all Ys and Error SSCP matrix E
    let mut full_residuals = vec![vec![0.0; n_valid]; p_dep];
    let mut rank_full = 0;
    for j in 0..p_dep {
        let y_j = &y_matrices[j];
        let (_, r_full, beta) = solve_ols_svd(&design.matrix, y_j)?;
        rank_full = r_full;
        for i in 0..n_valid {
            let mut y_hat = 0.0;
            for c in 0..design.matrix[0].len() {
                y_hat += design.matrix[i][c] * beta[c];
            }
            full_residuals[j][i] = y_j[i] - y_hat;
        }
    }

    let df_e = n_valid.saturating_sub(rank_full);

    // Error SSCP matrix E (p_dep x p_dep)
    let mut e_mat = vec![vec![0.0; p_dep]; p_dep];
    for r in 0..p_dep {
        for c in r..p_dep {
            let mut sum = 0.0;
            for i in 0..n_valid {
                sum += full_residuals[r][i] * full_residuals[c][i];
            }
            e_mat[r][c] = sum;
            e_mat[c][r] = sum;
        }
    }

    // 7. For each model term, compute Hypothesis SSCP matrix H = E_reduced - E
    let mut multivariate_effects: Vec<MultivariateEffectRow> = Vec::new();

    for term in &design.terms {
        let sub = extract_submatrix(&design, &term.column_indices);
        let mut reduced_residuals = vec![vec![0.0; n_valid]; p_dep];
        let mut rank_reduced = 0;

        for j in 0..p_dep {
            let y_j = &y_matrices[j];
            let (_, r_red, beta_red) = solve_ols_svd(&sub, y_j)?;
            rank_reduced = r_red;
            for i in 0..n_valid {
                let mut y_hat = 0.0;
                for c in 0..sub[0].len() {
                    y_hat += sub[i][c] * beta_red[c];
                }
                reduced_residuals[j][i] = y_j[i] - y_hat;
            }
        }

        let df_h = rank_full.saturating_sub(rank_reduced);
        if df_h == 0 {
            continue;
        }

        // H = E_reduced - E
        let mut h_mat = vec![vec![0.0; p_dep]; p_dep];
        for r in 0..p_dep {
            for c in r..p_dep {
                let mut sum_red = 0.0;
                for i in 0..n_valid {
                    sum_red += reduced_residuals[r][i] * reduced_residuals[c][i];
                }
                let diff = (sum_red - e_mat[r][c]).max(0.0);
                h_mat[r][c] = diff;
                h_mat[c][r] = diff;
            }
        }

        // Generalized eigenvalues and 4 multivariate tests
        let evals = generalized_eigenvalues(&e_mat, &h_mat)?;
        let mut test_rows = compute_multivariate_tests(&term.name, &evals, p_dep, df_h, df_e);
        multivariate_effects.append(&mut test_rows);
    }

    // 8. Compute accompanying univariate Between-Subjects tests for each dependent variable
    let mut between_subjects = Vec::with_capacity(p_dep);
    for j in 0..p_dep {
        let d_name = dependents[j].0;
        let y_j = &y_matrices[j];
        let uni_res = compute_type3_ss(d_name, &design, y_j, options.intercept)?;
        between_subjects.push(uni_res);
    }

    let dep_names = dependents
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();

    Ok(MultivariateGlmResult {
        dependents: dep_names,
        n_valid,
        effects: multivariate_effects,
        between_subjects,
    })
}
