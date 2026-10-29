use crate::anova_oneway::{f_distribution_sf, AlgoError};
use crate::regression_types::{
    LinearRegressionOptions, LinearRegressionResult, RegressionAnova, RegressionAnovaRow,
    RegressionCoefficient,
};
use crate::ttest_onesample::{student_t_inv_cdf, student_t_sf_2tailed};

/// Inverts a square matrix using Gauss-Jordan elimination with partial pivoting.
/// Returns AlgoError if the matrix is singular or non-invertible.
#[allow(clippy::needless_range_loop)]
pub fn invert_matrix(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, AlgoError> {
    let n = matrix.len();
    if n == 0 {
        return Err(AlgoError::InvalidInput("Matrix is empty".to_string()));
    }
    for row in matrix {
        if row.len() != n {
            return Err(AlgoError::InvalidInput("Matrix is not square".to_string()));
        }
    }

    let mut augmented = vec![vec![0.0; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            augmented[i][j] = matrix[i][j];
        }
        augmented[i][n + i] = 1.0;
    }

    for col in 0..n {
        // Find pivot with maximum absolute value
        let mut max_row = col;
        let mut max_val = augmented[col][col].abs();
        for r in (col + 1)..n {
            let val = augmented[r][col].abs();
            if val > max_val {
                max_val = val;
                max_row = r;
            }
        }

        if max_val < 1e-13 {
            return Err(AlgoError::ComputationError(
                "Singular matrix: collinear variables detected".to_string(),
            ));
        }

        // Swap rows if necessary
        if max_row != col {
            augmented.swap(col, max_row);
        }

        let pivot = augmented[col][col];
        for c in 0..(2 * n) {
            augmented[col][c] /= pivot;
        }

        for r in 0..n {
            if r != col {
                let factor = augmented[r][col];
                if factor.abs() > 0.0 {
                    for c in 0..(2 * n) {
                        augmented[r][c] -= factor * augmented[col][c];
                    }
                }
            }
        }
    }

    let mut inv = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            inv[i][j] = augmented[i][n + j];
        }
    }

    Ok(inv)
}

/// Computes Ordinary Least Squares (OLS) or Weighted Least Squares (WLS) Linear Regression.
pub fn compute_linear_regression(
    dep_name: &str,
    dep_vals: &[Option<f64>],
    indep_names: &[String],
    indep_vals: &[Vec<Option<f64>>],
    weights: &[f64],
    options: &LinearRegressionOptions,
) -> Result<LinearRegressionResult, AlgoError> {
    if indep_names.is_empty() {
        return Err(AlgoError::InvalidInput(
            "At least one independent variable is required".to_string(),
        ));
    }
    if indep_names.len() != indep_vals.len() {
        return Err(AlgoError::InvalidInput(
            "Mismatch between independent variable names and values count".to_string(),
        ));
    }

    let row_count = dep_vals.len();
    for col in indep_vals {
        if col.len() != row_count {
            return Err(AlgoError::InvalidInput(
                "All variable columns must have the same length".to_string(),
            ));
        }
    }

    let p = indep_names.len();
    let k = p + 1; // Intercept + p predictors

    // 1. Listwise missing value handling and weighting
    let mut valid_y: Vec<f64> = Vec::with_capacity(row_count);
    let mut valid_x: Vec<Vec<f64>> = Vec::with_capacity(row_count);
    let mut valid_w: Vec<f64> = Vec::with_capacity(row_count);
    let mut n_missing: usize = 0;

    for i in 0..row_count {
        let w = if weights.is_empty() {
            1.0
        } else {
            weights.get(i).copied().unwrap_or(0.0)
        };

        if w <= 0.0 || !w.is_finite() {
            n_missing += 1;
            continue;
        }

        let y_opt = dep_vals.get(i).copied().flatten();
        let mut x_row = Vec::with_capacity(p);
        let mut is_valid = matches!(y_opt, Some(y) if y.is_finite());

        if is_valid {
            for col in indep_vals {
                match col.get(i).copied().flatten() {
                    Some(x) if x.is_finite() => x_row.push(x),
                    _ => {
                        is_valid = false;
                        break;
                    }
                }
            }
        }

        if is_valid {
            valid_y.push(y_opt.unwrap_or(0.0));
            valid_x.push(x_row);
            valid_w.push(w);
        } else {
            n_missing += w.round().max(1.0) as usize;
        }
    }

    let n = valid_y.len();
    if n <= k {
        return Err(AlgoError::InsufficientData(format!(
            "Insufficient cases (N = {}), required at least {} valid observations",
            n,
            k + 1
        )));
    }

    let sum_w: f64 = valid_w.iter().sum();
    let y_mean: f64 = valid_y
        .iter()
        .zip(&valid_w)
        .map(|(y, w)| y * w)
        .sum::<f64>()
        / sum_w;

    let ss_total: f64 = valid_y
        .iter()
        .zip(&valid_w)
        .map(|(y, w)| w * (y - y_mean).powi(2))
        .sum();

    if ss_total < 1e-14 {
        return Err(AlgoError::InvalidInput(
            "Dependent variable has zero variance".to_string(),
        ));
    }

    // 2. Build normal equations: A = X^T W X (k x k), b = X^T W y (k x 1)
    let mut a = vec![vec![0.0; k]; k];
    let mut b = vec![0.0; k];

    for i in 0..n {
        let w = valid_w[i];
        let y = valid_y[i];

        // x_full[0] = 1.0 (constant), x_full[1..=p] = valid_x[i]
        let mut x_full = Vec::with_capacity(k);
        x_full.push(1.0);
        x_full.extend_from_slice(&valid_x[i]);

        for r in 0..k {
            b[r] += w * x_full[r] * y;
            for c in 0..k {
                a[r][c] += w * x_full[r] * x_full[c];
            }
        }
    }

    // Invert normal matrix (X^T W X)^-1
    let inv_a = invert_matrix(&a)?;

    // Compute coefficients beta = inv_a * b
    let mut beta = vec![0.0; k];
    for r in 0..k {
        for c in 0..k {
            beta[r] += inv_a[r][c] * b[c];
        }
    }

    // 3. Compute residuals and sums of squares
    let mut ss_resid: f64 = 0.0;
    for i in 0..n {
        let mut y_pred = beta[0];
        for j in 0..p {
            y_pred += beta[j + 1] * valid_x[i][j];
        }
        let e = valid_y[i] - y_pred;
        ss_resid += valid_w[i] * e * e;
    }

    let ss_model = (ss_total - ss_resid).max(0.0);
    let df_model = p;
    let df_total = n - 1;
    let df_resid = n - k;

    let ms_model = ss_model / (df_model as f64);
    let ms_resid = ss_resid / (df_resid as f64);

    let f_stat = if ms_resid > 1e-15 {
        ms_model / ms_resid
    } else {
        0.0
    };
    let f_sig = f_distribution_sf(f_stat, df_model as f64, df_resid as f64);

    let r_squared = (1.0 - (ss_resid / ss_total)).clamp(0.0, 1.0);
    let r = r_squared.sqrt();
    let adj_r_squared = 1.0 - (1.0 - r_squared) * (df_total as f64 / df_resid as f64);
    let std_error_estimate = ms_resid.sqrt();

    // 4. Compute collinearity statistics (Tolerance and VIF)
    let (tolerances, vifs) = if options.collinearity && p > 1 {
        compute_collinearity(&valid_x, &valid_w)?
    } else if p == 1 {
        (vec![Some(1.0)], vec![Some(1.0)])
    } else {
        (vec![None; p], vec![None; p])
    };

    // 5. Standard deviations for Standardized Beta
    let s_y = (ss_total / (df_total as f64)).sqrt();
    let mut s_x = Vec::with_capacity(p);
    for j in 0..p {
        let mean_x = valid_x
            .iter()
            .zip(&valid_w)
            .map(|(row, w)| row[j] * w)
            .sum::<f64>()
            / sum_w;
        let ss_x: f64 = valid_x
            .iter()
            .zip(&valid_w)
            .map(|(row, w)| w * (row[j] - mean_x).powi(2))
            .sum();
        s_x.push((ss_x / (df_total as f64)).sqrt());
    }

    // 6. Build coefficients table
    let alpha = 1.0 - options.ci_level.clamp(0.5, 0.9999);
    let t_crit = student_t_inv_cdf(1.0 - alpha / 2.0, df_resid as f64);

    let mut coefficients = Vec::with_capacity(k);
    for j in 0..k {
        let b_val = beta[j];
        let var_b = (ms_resid * inv_a[j][j]).max(0.0);
        let se_b = var_b.sqrt();
        let t_val = if se_b > 1e-15 { b_val / se_b } else { 0.0 };
        let p_val = student_t_sf_2tailed(t_val.abs(), df_resid as f64);
        let ci_lower = b_val - t_crit * se_b;
        let ci_upper = b_val + t_crit * se_b;

        let (var_name, beta_std, tol, vif) = if j == 0 {
            ("(Constant)".to_string(), None, None, None)
        } else {
            let p_idx = j - 1;
            let std_b = if s_y > 1e-15 {
                Some(b_val * (s_x[p_idx] / s_y))
            } else {
                Some(0.0)
            };
            (
                indep_names[p_idx].clone(),
                std_b,
                tolerances[p_idx],
                vifs[p_idx],
            )
        };

        coefficients.push(RegressionCoefficient {
            var_name,
            b: b_val,
            std_error: se_b,
            beta: beta_std,
            t_stat: t_val,
            p_value: p_val,
            ci_lower,
            ci_upper,
            tolerance: tol,
            vif,
        });
    }

    let anova = RegressionAnova {
        regression: RegressionAnovaRow {
            sum_of_squares: ss_model,
            df: df_model,
            mean_square: Some(ms_model),
            f_stat: Some(f_stat),
            p_value: Some(f_sig),
        },
        residual: RegressionAnovaRow {
            sum_of_squares: ss_resid,
            df: df_resid,
            mean_square: Some(ms_resid),
            f_stat: None,
            p_value: None,
        },
        total: RegressionAnovaRow {
            sum_of_squares: ss_total,
            df: df_total,
            mean_square: None,
            f_stat: None,
            p_value: None,
        },
    };

    Ok(LinearRegressionResult {
        dep_var: dep_name.to_string(),
        indep_vars: indep_names.to_vec(),
        method: "Enter".to_string(),
        n_valid: n,
        n_missing,
        r,
        r_squared,
        adj_r_squared,
        std_error_estimate,
        anova,
        coefficients,
        options: options.clone(),
    })
}

type CollinearityResult = (Vec<Option<f64>>, Vec<Option<f64>>);

/// Computes Tolerance and VIF from predictor correlation matrix.
#[allow(clippy::needless_range_loop)]
fn compute_collinearity(
    x_matrix: &[Vec<f64>],
    weights: &[f64],
) -> Result<CollinearityResult, AlgoError> {
    let p = if let Some(first) = x_matrix.first() {
        first.len()
    } else {
        return Ok((Vec::new(), Vec::new()));
    };

    let sum_w: f64 = weights.iter().sum();
    let mut means = vec![0.0; p];
    let mut stds = vec![0.0; p];

    for j in 0..p {
        means[j] = x_matrix
            .iter()
            .zip(weights)
            .map(|(row, w)| row[j] * w)
            .sum::<f64>()
            / sum_w;
        let ss: f64 = x_matrix
            .iter()
            .zip(weights)
            .map(|(row, w)| w * (row[j] - means[j]).powi(2))
            .sum();
        let var = ss / (weights.len() - 1) as f64;
        stds[j] = var.sqrt().max(1e-15);
    }

    let mut r_matrix = vec![vec![0.0; p]; p];
    for r in 0..p {
        r_matrix[r][r] = 1.0;
        for c in (r + 1)..p {
            let cov: f64 = x_matrix
                .iter()
                .zip(weights)
                .map(|(row, w)| w * (row[r] - means[r]) * (row[c] - means[c]))
                .sum::<f64>()
                / (weights.len() - 1) as f64;
            let corr = (cov / (stds[r] * stds[c])).clamp(-1.0, 1.0);
            r_matrix[r][c] = corr;
            r_matrix[c][r] = corr;
        }
    }

    match invert_matrix(&r_matrix) {
        Ok(inv_r) => {
            let mut tols = Vec::with_capacity(p);
            let mut vifs = Vec::with_capacity(p);
            for j in 0..p {
                let diag = inv_r[j][j].max(1.0);
                let tol = (1.0 / diag).clamp(0.0, 1.0);
                tols.push(Some(tol));
                vifs.push(Some(diag));
            }
            Ok((tols, vifs))
        }
        Err(_) => {
            // Highly collinear or singular correlation matrix
            let tols = vec![Some(0.0); p];
            let vifs = vec![Some(f64::INFINITY); p];
            Ok((tols, vifs))
        }
    }
}
