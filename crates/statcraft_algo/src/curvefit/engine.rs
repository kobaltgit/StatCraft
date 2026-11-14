use crate::anova_oneway::{f_distribution_sf, AlgoError};
use crate::curvefit::types::{CurveModelSummaryRow, CurveModelType};
use crate::regression::invert_matrix;

/// Fits a single curve model to the given raw x and y data.
pub fn fit_single_curve_model(
    model: CurveModelType,
    raw_x: &[f64],
    raw_y: &[f64],
    include_constant: bool,
) -> Result<CurveModelSummaryRow, AlgoError> {
    let n_raw = raw_x.len();
    if n_raw < 3 {
        return Err(AlgoError::InsufficientData(format!(
            "Model {} requires at least 3 valid observations",
            model.name()
        )));
    }

    // 1. Transform variables based on model domain constraints
    let mut trans_x: Vec<f64> = Vec::with_capacity(n_raw);
    let mut trans_y: Vec<f64> = Vec::with_capacity(n_raw);

    for (&xi, &yi) in raw_x.iter().zip(raw_y.iter()) {
        match model {
            CurveModelType::Linear => {
                trans_x.push(xi);
                trans_y.push(yi);
            }
            CurveModelType::Logarithmic => {
                if xi > 0.0 {
                    trans_x.push(xi.ln());
                    trans_y.push(yi);
                }
            }
            CurveModelType::Inverse => {
                if xi.abs() > 1e-15 {
                    trans_x.push(1.0 / xi);
                    trans_y.push(yi);
                }
            }
            CurveModelType::Quadratic => {
                trans_x.push(xi);
                trans_y.push(yi);
            }
            CurveModelType::Power => {
                if xi > 0.0 && yi > 0.0 {
                    trans_x.push(xi.ln());
                    trans_y.push(yi.ln());
                }
            }
            CurveModelType::Exponential => {
                if yi > 0.0 {
                    trans_x.push(xi);
                    trans_y.push(yi.ln());
                }
            }
        }
    }

    let n = trans_x.len();
    let min_needed = if model == CurveModelType::Quadratic {
        if include_constant {
            4
        } else {
            3
        }
    } else if include_constant {
        3
    } else {
        2
    };

    if n < min_needed {
        return Err(AlgoError::InsufficientData(format!(
            "Model {} has insufficient observations ({n}) after domain filtering (requires {min_needed})",
            model.name()
        )));
    }

    // 2. Fit Ordinary Least Squares
    if model == CurveModelType::Quadratic {
        fit_quadratic_ols(&trans_x, &trans_y, include_constant, n)
    } else {
        fit_simple_ols(model, &trans_x, &trans_y, include_constant, n)
    }
}

/// Fits simple 1-predictor OLS: y' = b0 + b1 * x'.
fn fit_simple_ols(
    model: CurveModelType,
    x: &[f64],
    y: &[f64],
    include_constant: bool,
    n: usize,
) -> Result<CurveModelSummaryRow, AlgoError> {
    let n_f = n as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let mean_y = sum_y / n_f;

    let (b0, b1) = if include_constant {
        let mean_x = sum_x / n_f;
        let mut ss_x = 0.0;
        let mut ss_xy = 0.0;
        for i in 0..n {
            let dx = x[i] - mean_x;
            let dy = y[i] - mean_y;
            ss_x += dx * dx;
            ss_xy += dx * dy;
        }
        if ss_x <= 1e-15 {
            return Err(AlgoError::InvalidInput(
                "Zero variance in predictor variable".to_string(),
            ));
        }
        let slope = ss_xy / ss_x;
        let intercept = mean_y - slope * mean_x;
        (intercept, slope)
    } else {
        let mut s_xx = 0.0;
        let mut s_xy = 0.0;
        for i in 0..n {
            s_xx += x[i] * x[i];
            s_xy += x[i] * y[i];
        }
        if s_xx <= 1e-15 {
            return Err(AlgoError::InvalidInput(
                "Zero sum of squares in predictor variable".to_string(),
            ));
        }
        (0.0, s_xy / s_xx)
    };

    // Calculate sums of squares
    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for i in 0..n {
        let y_pred = b0 + b1 * x[i];
        let res = y[i] - y_pred;
        ss_res += res * res;
        let dev = y[i] - mean_y;
        ss_tot += dev * dev;
    }

    let ss_reg = (ss_tot - ss_res).max(0.0);
    let r2 = if ss_tot > 1e-14 {
        (ss_reg / ss_tot).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let df1 = 1usize;
    let df2 = n.saturating_sub(if include_constant { 2 } else { 1 });
    let ms_reg = ss_reg / (df1 as f64);
    let ms_res = if df2 > 0 { ss_res / (df2 as f64) } else { 0.0 };

    let f_stat = if ms_res > 1e-14 { ms_reg / ms_res } else { 0.0 };
    let sig = if df2 > 0 {
        f_distribution_sf(f_stat, df1 as f64, df2 as f64).clamp(0.0, 1.0)
    } else {
        1.0
    };

    // Convert linearized intercept if power or exponential
    let param_b0 = match model {
        CurveModelType::Power | CurveModelType::Exponential => b0.exp(),
        _ => b0,
    };

    Ok(CurveModelSummaryRow {
        model_type: model,
        r_squared: r2,
        f_stat,
        df1,
        df2,
        sig,
        b0: param_b0,
        b1,
        b2: None,
        n_cases: n,
    })
}

/// Fits quadratic OLS: y = b0 + b1 * x + b2 * x^2.
fn fit_quadratic_ols(
    x: &[f64],
    y: &[f64],
    include_constant: bool,
    n: usize,
) -> Result<CurveModelSummaryRow, AlgoError> {
    let p = if include_constant { 3 } else { 2 };
    let mut a = vec![vec![0.0; p]; p];
    let mut g = vec![0.0; p];

    for i in 0..n {
        let xi = x[i];
        let yi = y[i];
        let row_x = if include_constant {
            vec![1.0, xi, xi * xi]
        } else {
            vec![xi, xi * xi]
        };

        for r in 0..p {
            g[r] += row_x[r] * yi;
            for c in r..p {
                let term = row_x[r] * row_x[c];
                a[r][c] += term;
                if r != c {
                    a[c][r] += term;
                }
            }
        }
    }

    let inv_a = invert_matrix(&a).map_err(|_| {
        AlgoError::ComputationError(
            "Singular matrix in quadratic regression estimation".to_string(),
        )
    })?;

    let mut beta = vec![0.0; p];
    for r in 0..p {
        let mut sum = 0.0;
        for c in 0..p {
            sum += inv_a[r][c] * g[c];
        }
        beta[r] = sum;
    }

    let (b0, b1, b2) = if include_constant {
        (beta[0], beta[1], beta[2])
    } else {
        (0.0, beta[0], beta[1])
    };

    let mean_y = y.iter().sum::<f64>() / (n as f64);
    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for i in 0..n {
        let xi = x[i];
        let y_pred = b0 + b1 * xi + b2 * xi * xi;
        let res = y[i] - y_pred;
        ss_res += res * res;
        let dev = y[i] - mean_y;
        ss_tot += dev * dev;
    }

    let ss_reg = (ss_tot - ss_res).max(0.0);
    let r2 = if ss_tot > 1e-14 {
        (ss_reg / ss_tot).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let df1 = 2usize;
    let df2 = n.saturating_sub(if include_constant { 3 } else { 2 });
    let ms_reg = ss_reg / (df1 as f64);
    let ms_res = if df2 > 0 { ss_res / (df2 as f64) } else { 0.0 };

    let f_stat = if ms_res > 1e-14 { ms_reg / ms_res } else { 0.0 };
    let sig = if df2 > 0 {
        f_distribution_sf(f_stat, df1 as f64, df2 as f64).clamp(0.0, 1.0)
    } else {
        1.0
    };

    Ok(CurveModelSummaryRow {
        model_type: CurveModelType::Quadratic,
        r_squared: r2,
        f_stat,
        df1,
        df2,
        sig,
        b0,
        b1,
        b2: Some(b2),
        n_cases: n,
    })
}
