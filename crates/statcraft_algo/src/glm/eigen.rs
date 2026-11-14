use crate::anova_oneway::{f_distribution_sf, AlgoError};
use crate::factor::jacobi_eigen_symmetric;
use crate::glm::types::{MultivariateEffectRow, MultivariateStatName};

/// Computes generalized eigenvalues of E^-1 H for symmetric E and H.
/// Uses symmetric transformation A = E^{-1/2} H E^{-1/2}.
#[allow(clippy::needless_range_loop)]
pub fn generalized_eigenvalues(
    e_mat: &[Vec<f64>],
    h_mat: &[Vec<f64>],
) -> Result<Vec<f64>, AlgoError> {
    let p = e_mat.len();
    if p == 0 || h_mat.len() != p {
        return Err(AlgoError::InvalidInput(
            "Invalid matrix dimensions in generalized_eigenvalues".to_string(),
        ));
    }

    // 1. Eigen-decomposition of E
    let (evals_e, evecs_e) = jacobi_eigen_symmetric(e_mat);
    let max_eval_e = evals_e.first().copied().unwrap_or(0.0).abs().max(1e-12);
    let threshold_e = 1e-10 * max_eval_e;

    // 2. Compute E^{-1/2} = sum( (1/sqrt(lambda_i)) * v_i * v_i^T )
    let mut e_inv_sqrt = vec![vec![0.0; p]; p];
    for idx in 0..p {
        let val = evals_e[idx];
        if val > threshold_e {
            let inv_sqrt = 1.0 / val.sqrt();
            for r in 0..p {
                for c in 0..p {
                    e_inv_sqrt[r][c] += evecs_e[r][idx] * evecs_e[c][idx] * inv_sqrt;
                }
            }
        }
    }

    // 3. Compute A = E^{-1/2} * H * E^{-1/2}
    // First temp = H * E^{-1/2}
    let mut temp = vec![vec![0.0; p]; p];
    for r in 0..p {
        for c in 0..p {
            let mut sum = 0.0;
            for k in 0..p {
                sum += h_mat[r][k] * e_inv_sqrt[k][c];
            }
            temp[r][c] = sum;
        }
    }

    // A = E^{-1/2} * temp
    let mut a_mat = vec![vec![0.0; p]; p];
    for r in 0..p {
        for c in 0..p {
            let mut sum = 0.0;
            for k in 0..p {
                sum += e_inv_sqrt[r][k] * temp[k][c];
            }
            a_mat[r][c] = sum;
        }
    }
    // Symmetrize A to cancel floating point roundoff
    for r in 0..p {
        for c in (r + 1)..p {
            let avg = (a_mat[r][c] + a_mat[c][r]) * 0.5;
            a_mat[r][c] = avg;
            a_mat[c][r] = avg;
        }
    }

    // 4. Eigen-decomposition of symmetric A
    let (evals_a, _) = jacobi_eigen_symmetric(&a_mat);

    // Filter negative eigenvalues due to numerical noise
    let clean_evals: Vec<f64> = evals_a
        .into_iter()
        .map(|v| if v > 0.0 { v } else { 0.0 })
        .collect();

    Ok(clean_evals)
}

/// Computes the 4 canonical SPSS multivariate tests for an effect.
pub fn compute_multivariate_tests(
    effect_name: &str,
    eigenvalues: &[f64],
    p_dep: usize,
    df_h: usize,
    df_e: usize,
) -> Vec<MultivariateEffectRow> {
    if p_dep == 0 || df_h == 0 || df_e == 0 {
        return Vec::new();
    }

    let p = p_dep as f64;
    let s_h = df_h as f64;
    let s_e = df_e as f64;
    let s = (p_dep.min(df_h)) as f64;

    let m = ((p - s_h).abs() - 1.0) * 0.5;
    let n = (s_e - p - 1.0) * 0.5;

    let s_int = s as usize;
    let valid_evals: Vec<f64> = eigenvalues.iter().take(s_int).copied().collect();

    let mut rows = Vec::with_capacity(4);

    // 1. Pillai's Trace
    let pillai_val: f64 = valid_evals
        .iter()
        .map(|&lambda| lambda / (1.0 + lambda))
        .sum();
    let pillai_df1 = s * (2.0 * m + s + 1.0);
    let pillai_df2 = s * (2.0 * n + s + 1.0);
    let (pillai_f, pillai_p) =
        if pillai_df1 > 0.0 && pillai_df2 > 0.0 && (s - pillai_val).abs() > 1e-12 {
            let f = (pillai_df2 * pillai_val) / (pillai_df1 * (s - pillai_val));
            let p = f_distribution_sf(f, pillai_df1, pillai_df2);
            (f, p)
        } else {
            (0.0, 1.0)
        };
    let pillai_eta = if s > 0.0 {
        Some((pillai_val / s).clamp(0.0, 1.0))
    } else {
        None
    };

    rows.push(MultivariateEffectRow {
        effect: effect_name.to_string(),
        stat_name: MultivariateStatName::PillaiTrace,
        value: pillai_val,
        f_stat: pillai_f,
        hypothesis_df: pillai_df1,
        error_df: pillai_df2,
        p_value: pillai_p,
        partial_eta_squared: pillai_eta,
    });

    // 2. Wilks' Lambda
    let wilks_val: f64 = valid_evals
        .iter()
        .map(|&lambda| 1.0 / (1.0 + lambda))
        .product();
    let w = s_e + s_h - (p + s_h + 1.0) * 0.5;
    let term_denom = p * p + s_h * s_h - 5.0;
    let t = if term_denom > 0.0 {
        ((p * p * s_h * s_h - 4.0) / term_denom).max(0.0).sqrt()
    } else {
        1.0
    };
    let wilks_df1 = p * s_h;
    let wilks_df2 = w * t - (p * s_h - 2.0) * 0.5;

    let (wilks_f, wilks_p) = if wilks_df1 > 0.0 && wilks_df2 > 0.0 && wilks_val > 1e-15 {
        let root = wilks_val.powf(1.0 / t);
        let f = ((1.0 - root) / root) * (wilks_df2 / wilks_df1);
        let p = f_distribution_sf(f, wilks_df1, wilks_df2);
        (f, p)
    } else {
        (0.0, 1.0)
    };
    let wilks_eta = if s > 0.0 && wilks_val > 0.0 {
        Some((1.0 - wilks_val.powf(1.0 / s)).clamp(0.0, 1.0))
    } else {
        None
    };

    rows.push(MultivariateEffectRow {
        effect: effect_name.to_string(),
        stat_name: MultivariateStatName::WilksLambda,
        value: wilks_val,
        f_stat: wilks_f,
        hypothesis_df: wilks_df1,
        error_df: wilks_df2,
        p_value: wilks_p,
        partial_eta_squared: wilks_eta,
    });

    // 3. Hotelling-Lawley Trace
    let hotelling_val: f64 = valid_evals.iter().sum();
    let hotelling_df1 = p * s_h;
    let hotelling_df2 = 2.0 * (s * n + 1.0);
    let (hotelling_f, hotelling_p) = if hotelling_df1 > 0.0 && hotelling_df2 > 0.0 {
        let f = (hotelling_df2 * hotelling_val) / (s * hotelling_df1);
        let p = f_distribution_sf(f, hotelling_df1, hotelling_df2);
        (f, p)
    } else {
        (0.0, 1.0)
    };
    let hotelling_eta = if s > 0.0 {
        let ratio = hotelling_val / s;
        Some((ratio / (1.0 + ratio)).clamp(0.0, 1.0))
    } else {
        None
    };

    rows.push(MultivariateEffectRow {
        effect: effect_name.to_string(),
        stat_name: MultivariateStatName::HotellingTrace,
        value: hotelling_val,
        f_stat: hotelling_f,
        hypothesis_df: hotelling_df1,
        error_df: hotelling_df2,
        p_value: hotelling_p,
        partial_eta_squared: hotelling_eta,
    });

    // 4. Roy's Largest Root
    let roy_lambda = valid_evals.first().copied().unwrap_or(0.0);
    let v = p.max(s_h);
    let roy_df1 = v;
    let roy_df2 = s_e - v + s_h;
    let (roy_f, roy_p) = if roy_df1 > 0.0 && roy_df2 > 0.0 {
        let f = roy_lambda * (roy_df2 / roy_df1);
        let p = f_distribution_sf(f, roy_df1, roy_df2);
        (f, p)
    } else {
        (0.0, 1.0)
    };
    let roy_eta = Some((roy_lambda / (1.0 + roy_lambda)).clamp(0.0, 1.0));

    rows.push(MultivariateEffectRow {
        effect: effect_name.to_string(),
        stat_name: MultivariateStatName::RoyLargestRoot,
        value: roy_lambda,
        f_stat: roy_f,
        hypothesis_df: roy_df1,
        error_df: roy_df2,
        p_value: roy_p,
        partial_eta_squared: roy_eta,
    });

    rows
}
