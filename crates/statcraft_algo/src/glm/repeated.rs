use crate::anova_oneway::{f_distribution_sf, AlgoError};
use crate::crosstabs::regularized_incomplete_gamma_q;
use crate::factor::jacobi_eigen_symmetric;
use crate::glm::types::{
    RepeatedGlmOptions, RepeatedGlmResult, SphericityTestResult, WithinSubjectsEffectRow,
};

/// Generates normalized Helmert contrast matrix of size k x (k - 1).
/// Satisfies C^T 1 = 0 and C^T C = I_{k-1}.
#[allow(clippy::needless_range_loop)]
pub fn build_helmert_contrasts(k: usize) -> Vec<Vec<f64>> {
    if k <= 1 {
        return Vec::new();
    }
    let d = k - 1;
    let mut c = vec![vec![0.0; d]; k];

    for j in 0..d {
        let step = (j + 1) as f64;
        let denom = (step * (step + 1.0)).sqrt();
        let top_val = 1.0 / denom;
        let neg_val = -step / denom;

        for r in 0..=j {
            c[r][j] = top_val;
        }
        c[j + 1][j] = neg_val;
    }

    c
}

/// Computes Repeated Measures ANOVA with Mauchly Sphericity and Epsilon corrections.
#[allow(clippy::needless_range_loop)]
pub fn compute_repeated_glm(
    measure_vars: &[(&str, &[f64])],
    options: &RepeatedGlmOptions,
) -> Result<RepeatedGlmResult, AlgoError> {
    let k = measure_vars.len();
    if k < 2 {
        return Err(AlgoError::InvalidInput(
            "Repeated measures requires at least 2 time points / measures".to_string(),
        ));
    }
    let n_total = measure_vars[0].1.len();
    if n_total == 0 {
        return Err(AlgoError::InvalidInput("Data is empty".to_string()));
    }

    // 1. Listwise deletion filter
    let mut valid_indices = Vec::new();
    for i in 0..n_total {
        let mut has_nan = false;
        for (_, vals) in measure_vars {
            if vals[i].is_nan() {
                has_nan = true;
                break;
            }
        }
        if !has_nan {
            valid_indices.push(i);
        }
    }

    let n = valid_indices.len();
    if n < 3 {
        return Err(AlgoError::InsufficientData(
            "At least 3 valid cases required for repeated measures".to_string(),
        ));
    }

    // Y matrix: N x k
    let mut y = vec![vec![0.0; k]; n];
    for col_idx in 0..k {
        let raw = measure_vars[col_idx].1;
        for row_idx in 0..n {
            y[row_idx][col_idx] = raw[valid_indices[row_idx]];
        }
    }

    // 2. Compute grand mean, subject means, condition means
    let mut condition_means = vec![0.0; k];
    for col_idx in 0..k {
        let sum: f64 = (0..n).map(|i| y[i][col_idx]).sum();
        condition_means[col_idx] = sum / (n as f64);
    }

    let mut subject_means = vec![0.0; n];
    for row_idx in 0..n {
        let sum: f64 = (0..k).map(|j| y[row_idx][j]).sum();
        subject_means[row_idx] = sum / (k as f64);
    }

    let grand_mean: f64 = condition_means.iter().sum::<f64>() / (k as f64);

    // 3. Sum of squares
    // SS_time:
    let ss_time: f64 = condition_means
        .iter()
        .map(|&m| (m - grand_mean).powi(2))
        .sum::<f64>()
        * (n as f64);

    // SS_error (Time x Subject interaction):
    let mut ss_error = 0.0;
    for i in 0..n {
        for j in 0..k {
            let res = y[i][j] - condition_means[j] - subject_means[i] + grand_mean;
            ss_error += res * res;
        }
    }

    let df_time_base = (k - 1) as f64;
    let df_error_base = ((k - 1) * (n - 1)) as f64;

    let f_stat = if ss_error > 1e-12 && df_error_base > 0.0 {
        (ss_time / df_time_base) / (ss_error / df_error_base)
    } else {
        0.0
    };

    // 4. Mauchly Sphericity and Epsilon corrections
    let d = k - 1;
    let c_mat = build_helmert_contrasts(k);

    // Transformed variables Z = Y * C (N x d)
    let mut z = vec![vec![0.0; d]; n];
    for i in 0..n {
        for j in 0..d {
            let mut sum = 0.0;
            for m in 0..k {
                sum += y[i][m] * c_mat[m][j];
            }
            z[i][j] = sum;
        }
    }

    // Means of Z
    let mut z_means = vec![0.0; d];
    for j in 0..d {
        let s: f64 = (0..n).map(|i| z[i][j]).sum();
        z_means[j] = s / (n as f64);
    }

    // Sample covariance matrix of Z: Sigma (d x d)
    let mut sigma = vec![vec![0.0; d]; d];
    let df_n = (n - 1) as f64;
    for r in 0..d {
        for c in r..d {
            let mut sum = 0.0;
            for i in 0..n {
                sum += (z[i][r] - z_means[r]) * (z[i][c] - z_means[c]);
            }
            let cov = sum / df_n;
            sigma[r][c] = cov;
            sigma[c][r] = cov;
        }
    }

    // Eigenvalues of Sigma
    let (evals, _) = jacobi_eigen_symmetric(&sigma);
    let tr_sigma: f64 = evals.iter().sum();
    let tr_sigma_sq: f64 = evals.iter().map(|&v| v * v).sum();
    let det_sigma: f64 = evals.iter().product();

    // Mauchly's W
    let d_f = d as f64;
    let avg_eval = tr_sigma / d_f;
    let mauchly_w = if k >= 3 && avg_eval > 1e-12 && det_sigma > 0.0 {
        (det_sigma / avg_eval.powf(d_f)).clamp(0.0, 1.0)
    } else {
        1.0
    };

    // Chi-square for Mauchly
    let df_mauchly = if k >= 3 { (k * (k - 1)) / 2 - 1 } else { 0 };
    let (mauchly_chi2, mauchly_p) = if k >= 3 && mauchly_w > 1e-12 && mauchly_w < 1.0 - 1e-10 {
        let factor = (n - 1) as f64 - (2.0 * d_f * d_f + d_f + 2.0) / (6.0 * d_f);
        let chi2 = (-factor * mauchly_w.ln()).max(0.0);
        let p =
            regularized_incomplete_gamma_q((df_mauchly as f64) * 0.5, chi2 * 0.5).clamp(0.0, 1.0);
        (chi2, p)
    } else {
        (0.0, 1.0)
    };

    // Greenhouse-Geisser epsilon
    let gg_eps = if tr_sigma_sq > 1e-12 {
        let eps = (tr_sigma * tr_sigma) / (d_f * tr_sigma_sq);
        eps.clamp(1.0 / d_f, 1.0)
    } else {
        1.0
    };

    // Huynh-Feldt epsilon
    let hf_num = (n as f64) * d_f * gg_eps - 2.0;
    let hf_den = d_f * ((n - 1) as f64 - d_f * gg_eps);
    let hf_eps = if hf_den > 1e-12 {
        (hf_num / hf_den).clamp(gg_eps, 1.0)
    } else {
        gg_eps
    };

    let lb_eps = 1.0 / d_f;

    let sphericity = SphericityTestResult {
        within_subject_effect: options.factor_name.clone(),
        mauchly_w,
        approx_chi_square: mauchly_chi2,
        df: df_mauchly,
        p_value: mauchly_p,
        greenhouse_geisser: gg_eps,
        huynh_feldt: hf_eps,
        lower_bound: lb_eps,
    };

    // Helper for effect row
    let make_row = |ss: f64, is_effect: bool| -> WithinSubjectsEffectRow {
        let (df_sa, df_gg, df_hf, df_lb) = if is_effect {
            (
                df_time_base,
                df_time_base * gg_eps,
                df_time_base * hf_eps,
                1.0,
            )
        } else {
            (
                df_error_base,
                df_error_base * gg_eps,
                df_error_base * hf_eps,
                (n - 1) as f64,
            )
        };

        let (f_val, p_sa, p_gg, p_hf, p_lb) = if is_effect && ss_error > 1e-12 {
            let f = f_stat;
            let p1 = f_distribution_sf(f, df_sa, df_error_base);
            let p2 = f_distribution_sf(f, df_gg, df_error_base * gg_eps);
            let p3 = f_distribution_sf(f, df_hf, df_error_base * hf_eps);
            let p4 = f_distribution_sf(f, df_lb, (n - 1) as f64);
            (Some(f), Some(p1), Some(p2), Some(p3), Some(p4))
        } else {
            (None, None, None, None, None)
        };

        WithinSubjectsEffectRow {
            source: if is_effect {
                options.factor_name.clone()
            } else {
                format!("Error({})", options.factor_name)
            },
            sphericity_assumed_ss: ss,
            sphericity_assumed_df: df_sa,
            sphericity_assumed_ms: ss / df_sa,
            sphericity_assumed_f: f_val,
            sphericity_assumed_p: p_sa,
            greenhouse_geisser_ss: ss,
            greenhouse_geisser_df: df_gg,
            greenhouse_geisser_ms: ss / df_gg,
            greenhouse_geisser_f: f_val,
            greenhouse_geisser_p: p_gg,
            huynh_feldt_ss: ss,
            huynh_feldt_df: df_hf,
            huynh_feldt_ms: ss / df_hf,
            huynh_feldt_f: f_val,
            huynh_feldt_p: p_hf,
            lower_bound_ss: ss,
            lower_bound_df: df_lb,
            lower_bound_ms: ss / df_lb,
            lower_bound_f: f_val,
            lower_bound_p: p_lb,
        }
    };

    let effect_row = make_row(ss_time, true);
    let error_row = make_row(ss_error, false);

    let var_names = measure_vars.iter().map(|(n, _)| n.to_string()).collect();

    Ok(RepeatedGlmResult {
        factor_name: options.factor_name.clone(),
        measure_variables: var_names,
        n_valid: n,
        sphericity,
        within_subjects_tests: vec![effect_row],
        error_row,
    })
}
