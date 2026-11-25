use crate::anova_oneway::AlgoError;
use crate::bootstrap::{run_bootstrap_scalar, BootstrapError, BootstrapOptions, SamplingPlan};
use crate::ttest_onesample::student_t_inv_cdf;

use super::types::{DemingBootstrap, DemingPoint, DemingRegressionOptions, DemingRegressionResult};

const EPS: f64 = 1e-12;

struct RawDemingFit {
    slope: f64,
    intercept: f64,
    orthogonal_residual_sd: f64,
    pearson_r: Option<f64>,
}

/// Fits Deming regression on valid finite slices, optionally skipping one index for Jackknife.
fn fit_deming_raw(
    xs: &[f64],
    ys: &[f64],
    lambda: f64,
    skip_idx: Option<usize>,
) -> Result<RawDemingFit, AlgoError> {
    let n_eff = if skip_idx.is_some() {
        xs.len().saturating_sub(1)
    } else {
        xs.len()
    };
    if n_eff < 2 {
        return Err(AlgoError::InvalidInput(
            "At least 2 valid observations are required".to_string(),
        ));
    }

    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;
    for (i, (&x, &y)) in xs.iter().zip(ys.iter()).enumerate() {
        if Some(i) == skip_idx {
            continue;
        }
        sum_x += x;
        sum_y += y;
    }
    let nf = n_eff as f64;
    let mean_x = sum_x / nf;
    let mean_y = sum_y / nf;

    let mut s_xx = 0.0_f64;
    let mut s_yy = 0.0_f64;
    let mut s_xy = 0.0_f64;
    for (i, (&x, &y)) in xs.iter().zip(ys.iter()).enumerate() {
        if Some(i) == skip_idx {
            continue;
        }
        let dx = x - mean_x;
        let dy = y - mean_y;
        s_xx += dx * dx;
        s_yy += dy * dy;
        s_xy += dx * dy;
    }

    if s_xx <= EPS && s_yy <= EPS {
        return Err(AlgoError::InvalidInput(
            "Degenerate data: zero variance in both variables".to_string(),
        ));
    }

    let slope = if s_xy.abs() <= EPS {
        if s_xx <= EPS {
            return Err(AlgoError::InvalidInput(
                "Degenerate data: zero variance in independent variable".to_string(),
            ));
        }
        if s_yy <= lambda * s_xx {
            0.0
        } else {
            return Err(AlgoError::InvalidInput(
                "Degenerate orthogonal fit: s_xy = 0 and s_yy > lambda * s_xx".to_string(),
            ));
        }
    } else {
        let diff = s_yy - lambda * s_xx;
        let disc = (diff * diff + 4.0 * lambda * s_xy * s_xy).sqrt();
        (diff + disc) / (2.0 * s_xy)
    };

    let intercept = mean_y - slope * mean_x;
    let pearson_r = if s_xx > EPS && s_yy > EPS {
        Some((s_xy / (s_xx * s_yy).sqrt()).clamp(-1.0, 1.0))
    } else {
        None
    };

    let scale = (lambda / (lambda + slope * slope)).sqrt();
    let mut rss_orth = 0.0_f64;
    for (i, (&x, &y)) in xs.iter().zip(ys.iter()).enumerate() {
        if Some(i) == skip_idx {
            continue;
        }
        let r = scale * (y - (slope * x + intercept));
        rss_orth += r * r;
    }

    let orthogonal_residual_sd = if n_eff > 2 {
        (rss_orth / ((n_eff - 2) as f64)).sqrt()
    } else {
        0.0
    };

    Ok(RawDemingFit {
        slope,
        intercept,
        orthogonal_residual_sd,
        pearson_r,
    })
}

/// Computes Orthogonal / Deming regression with Leave-One-Out Jackknife SEs and t-CIs.
pub fn compute_deming_regression(
    dep_var: &str,
    dep_vals: &[Option<f64>],
    indep_var: &str,
    indep_vals: &[Option<f64>],
    options: &DemingRegressionOptions,
) -> Result<DemingRegressionResult, AlgoError> {
    if dep_vals.len() != indep_vals.len() {
        return Err(AlgoError::InvalidInput(
            "Dependent and independent variable columns must have the same length".to_string(),
        ));
    }
    if !options.lambda.is_finite() || options.lambda <= 0.0 {
        return Err(AlgoError::InvalidInput(format!(
            "Error variance ratio lambda must be finite and > 0, got {}",
            options.lambda
        )));
    }
    if !options.ci_level.is_finite() || options.ci_level <= 0.0 || options.ci_level >= 1.0 {
        return Err(AlgoError::InvalidInput(format!(
            "Confidence level must be strictly between 0 and 1, got {}",
            options.ci_level
        )));
    }

    let mut xs = Vec::with_capacity(dep_vals.len());
    let mut ys = Vec::with_capacity(dep_vals.len());
    for (y_opt, x_opt) in dep_vals.iter().zip(indep_vals.iter()) {
        if let (Some(y), Some(x)) = (y_opt, x_opt) {
            if y.is_finite() && x.is_finite() {
                xs.push(*x);
                ys.push(*y);
            }
        }
    }

    let n_valid = xs.len();
    let n_missing = dep_vals.len() - n_valid;
    if n_valid < 3 {
        return Err(AlgoError::InvalidInput(
            "At least 3 valid paired observations are required for Deming regression".to_string(),
        ));
    }

    let full = fit_deming_raw(&xs, &ys, options.lambda, None)?;

    // Leave-One-Out Jackknife for slope and intercept standard errors.
    let mut jk_slopes = Vec::with_capacity(n_valid);
    let mut jk_intercepts = Vec::with_capacity(n_valid);
    for i in 0..n_valid {
        let jk = fit_deming_raw(&xs, &ys, options.lambda, Some(i))?;
        jk_slopes.push(jk.slope);
        jk_intercepts.push(jk.intercept);
    }

    let nf = n_valid as f64;
    let mean_jk_slope = jk_slopes.iter().sum::<f64>() / nf;
    let mean_jk_intercept = jk_intercepts.iter().sum::<f64>() / nf;
    let jk_factor = ((n_valid - 1) as f64) / nf;

    let slope_se = (jk_factor
        * jk_slopes
            .iter()
            .map(|&s| (s - mean_jk_slope) * (s - mean_jk_slope))
            .sum::<f64>())
    .sqrt();
    let intercept_se = (jk_factor
        * jk_intercepts
            .iter()
            .map(|&b| (b - mean_jk_intercept) * (b - mean_jk_intercept))
            .sum::<f64>())
    .sqrt();

    let df = (n_valid - 2) as f64;
    let t_crit = student_t_inv_cdf(0.5 + 0.5 * options.ci_level, df);
    let slope_ci_lower = full.slope - t_crit * slope_se;
    let slope_ci_upper = full.slope + t_crit * slope_se;
    let intercept_ci_lower = full.intercept - t_crit * intercept_se;
    let intercept_ci_upper = full.intercept + t_crit * intercept_se;

    let scale = (options.lambda / (options.lambda + full.slope * full.slope)).sqrt();
    let mut points = Vec::with_capacity(n_valid);
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let fitted = full.slope * x + full.intercept;
        let orthogonal_residual = scale * (y - fitted);
        points.push(DemingPoint {
            x,
            y,
            fitted,
            orthogonal_residual,
        });
    }

    Ok(DemingRegressionResult {
        dep_var: dep_var.to_string(),
        indep_var: indep_var.to_string(),
        n_valid,
        n_missing,
        lambda: options.lambda,
        slope: full.slope,
        intercept: full.intercept,
        slope_se,
        intercept_se,
        slope_ci_lower,
        slope_ci_upper,
        intercept_ci_lower,
        intercept_ci_upper,
        orthogonal_residual_sd: full.orthogonal_residual_sd,
        pearson_r: full.pearson_r,
        points,
        saved_predicted_var: None,
        saved_residuals_var: None,
        options: options.clone(),
        bootstrap: None,
    })
}

/// Computes non-parametric bootstrap summaries for Deming regression slope and intercept.
pub fn compute_deming_regression_bootstrap(
    dep_var: &str,
    dep_vals: &[Option<f64>],
    indep_var: &str,
    indep_vals: &[Option<f64>],
    options: &DemingRegressionOptions,
    strata_keys: Option<&[String]>,
    boot_options: &BootstrapOptions,
) -> Result<DemingBootstrap, BootstrapError> {
    let n = dep_vals.len();
    let plan = SamplingPlan::from_options(n, strata_keys, boot_options)?;

    let slope_summary = run_bootstrap_scalar(&plan, boot_options, |indices| {
        let mut sub_y = Vec::with_capacity(indices.len());
        let mut sub_x = Vec::with_capacity(indices.len());
        for &idx in indices {
            if idx < n {
                sub_y.push(dep_vals[idx]);
                sub_x.push(indep_vals[idx]);
            }
        }
        compute_deming_regression(dep_var, &sub_y, indep_var, &sub_x, options)
            .ok()
            .map(|r| r.slope)
    })
    .ok();

    let intercept_summary = run_bootstrap_scalar(&plan, boot_options, |indices| {
        let mut sub_y = Vec::with_capacity(indices.len());
        let mut sub_x = Vec::with_capacity(indices.len());
        for &idx in indices {
            if idx < n {
                sub_y.push(dep_vals[idx]);
                sub_x.push(indep_vals[idx]);
            }
        }
        compute_deming_regression(dep_var, &sub_y, indep_var, &sub_x, options)
            .ok()
            .map(|r| r.intercept)
    })
    .ok();

    Ok(DemingBootstrap {
        options: boot_options.clone(),
        slope: slope_summary,
        intercept: intercept_summary,
    })
}
