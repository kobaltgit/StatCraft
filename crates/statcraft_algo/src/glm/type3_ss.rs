use crate::anova_oneway::{f_distribution_sf, AlgoError};
use crate::glm::design_matrix::{extract_submatrix, solve_ols_svd, DesignMatrix, TermType};
use crate::glm::types::{BetweenSubjectsRow, UnivariateGlmResult};

/// Computes Type III Sum of Squares for each term in the design matrix via model reduction.
pub fn compute_type3_ss(
    dependent_name: &str,
    full: &DesignMatrix,
    y: &[f64],
    intercept_included: bool,
) -> Result<UnivariateGlmResult, AlgoError> {
    let n = full.n_rows;
    if n == 0 || y.len() != n {
        return Err(AlgoError::InvalidInput(
            "Data length mismatch in compute_type3_ss".to_string(),
        ));
    }

    // 1. Solve full model OLS
    let (ss_full, rank_full, _) = solve_ols_svd(&full.matrix, y)?;

    let df_error = n.saturating_sub(rank_full);
    let ms_error = if df_error > 0 {
        ss_full / (df_error as f64)
    } else {
        0.0
    };

    let mut rows: Vec<BetweenSubjectsRow> = Vec::new();

    // 2. Intercept-only model for Corrected Model calculation
    let non_intercept_cols: Vec<usize> = full
        .terms
        .iter()
        .filter(|t| t.term_type != TermType::Intercept)
        .flat_map(|t| t.column_indices.clone())
        .collect();

    let (ss_intercept_only, rank_intercept_only) = if intercept_included {
        let sub = extract_submatrix(full, &non_intercept_cols);
        let (ss, r, _) = solve_ols_svd(&sub, y)?;
        (ss, r)
    } else {
        let sum_sq: f64 = y.iter().map(|v| v * v).sum();
        (sum_sq, 0)
    };

    let ss_corrected_model = (ss_intercept_only - ss_full).max(0.0);
    let df_corrected_model = rank_full.saturating_sub(rank_intercept_only);

    let ms_corrected_model = if df_corrected_model > 0 {
        Some(ss_corrected_model / (df_corrected_model as f64))
    } else {
        None
    };

    let (f_model, p_model) = match (ms_corrected_model, ms_error > 1e-12, df_error > 0) {
        (Some(ms_m), true, true) => {
            let f = ms_m / ms_error;
            let p = f_distribution_sf(f, df_corrected_model as f64, df_error as f64);
            (Some(f), Some(p))
        }
        _ => (None, None),
    };

    let eta_sq_model = if ss_corrected_model + ss_full > 1e-12 {
        Some(ss_corrected_model / (ss_corrected_model + ss_full))
    } else {
        None
    };

    rows.push(BetweenSubjectsRow {
        source: "Corrected Model".to_string(),
        type3_ss: ss_corrected_model,
        df: df_corrected_model,
        mean_square: ms_corrected_model,
        f_stat: f_model,
        p_value: p_model,
        partial_eta_squared: eta_sq_model,
    });

    // 3. Marginal model reduction for each individual term
    for term in &full.terms {
        let sub = extract_submatrix(full, &term.column_indices);
        let (ss_reduced, rank_reduced, _) = solve_ols_svd(&sub, y)?;

        let ss_term = (ss_reduced - ss_full).max(0.0);
        let df_term = rank_full.saturating_sub(rank_reduced);

        let ms_term = if df_term > 0 {
            Some(ss_term / (df_term as f64))
        } else {
            None
        };

        let (f_stat, p_val) = match (ms_term, ms_error > 1e-12, df_error > 0) {
            (Some(ms_t), true, true) => {
                let f = ms_t / ms_error;
                let p = f_distribution_sf(f, df_term as f64, df_error as f64);
                (Some(f), Some(p))
            }
            _ => (None, None),
        };

        let partial_eta = if ss_term + ss_full > 1e-12 {
            Some(ss_term / (ss_term + ss_full))
        } else {
            None
        };

        rows.push(BetweenSubjectsRow {
            source: term.name.clone(),
            type3_ss: ss_term,
            df: df_term,
            mean_square: ms_term,
            f_stat,
            p_value: p_val,
            partial_eta_squared: partial_eta,
        });
    }

    // 4. Error row
    rows.push(BetweenSubjectsRow {
        source: "Error".to_string(),
        type3_ss: ss_full,
        df: df_error,
        mean_square: if df_error > 0 { Some(ms_error) } else { None },
        f_stat: None,
        p_value: None,
        partial_eta_squared: None,
    });

    // 5. Total and Corrected Total
    let ss_total: f64 = y.iter().map(|v| v * v).sum();
    let mean_y = if n > 0 {
        y.iter().sum::<f64>() / (n as f64)
    } else {
        0.0
    };
    let ss_corrected_total: f64 = y.iter().map(|v| (v - mean_y).powi(2)).sum();

    rows.push(BetweenSubjectsRow {
        source: "Total".to_string(),
        type3_ss: ss_total,
        df: n,
        mean_square: None,
        f_stat: None,
        p_value: None,
        partial_eta_squared: None,
    });

    let df_corrected_total = if n > 0 { n - 1 } else { 0 };
    rows.push(BetweenSubjectsRow {
        source: "Corrected Total".to_string(),
        type3_ss: ss_corrected_total,
        df: df_corrected_total,
        mean_square: None,
        f_stat: None,
        p_value: None,
        partial_eta_squared: None,
    });

    // R-squared and Adjusted R-squared
    let r_squared = if ss_corrected_total > 1e-12 {
        (ss_corrected_model / ss_corrected_total).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let adjusted_r_squared = if df_error > 0 && df_corrected_total > 0 && ss_corrected_total > 1e-12
    {
        let term = (1.0 - r_squared) * (df_corrected_total as f64) / (df_error as f64);
        1.0 - term
    } else {
        r_squared
    };

    Ok(UnivariateGlmResult {
        dependent: dependent_name.to_string(),
        n_valid: n,
        r_squared,
        adjusted_r_squared,
        tests: rows,
        levene_test: None,
        group_descriptives: Vec::new(),
    })
}
