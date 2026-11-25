use crate::anova_oneway::AlgoError;
use crate::bootstrap::{
    normal_cdf, run_bootstrap_scalar, BootstrapError, BootstrapOptions, SamplingPlan,
};
use crate::rank_cases::inv_normal_cdf;

use super::types::{
    TheilSenBootstrap, TheilSenOptions, TheilSenPoint, TheilSenResult, MAX_EXACT_PAIRS_N,
};

const EPS: f64 = 1e-12;

fn median_sorted(sorted: &[f64]) -> Option<f64> {
    let n = sorted.len();
    if n == 0 {
        return None;
    }
    if n % 2 == 1 {
        Some(sorted[n / 2])
    } else {
        Some(0.5 * (sorted[n / 2 - 1] + sorted[n / 2]))
    }
}

fn median_of_slice(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut buf = values.to_vec();
    buf.sort_by(f64::total_cmp);
    median_sorted(&buf)
}

/// Returns group sizes of tied values in `vals` (using tolerance `EPS`).
fn tie_group_sizes(vals: &[f64]) -> Vec<usize> {
    if vals.is_empty() {
        return Vec::new();
    }
    let mut sorted = vals.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mut groups = Vec::new();
    let mut count = 1_usize;
    for i in 1..sorted.len() {
        if (sorted[i] - sorted[i - 1]).abs() <= EPS {
            count += 1;
        } else {
            if count > 1 {
                groups.push(count);
            }
            count = 1;
        }
    }
    if count > 1 {
        groups.push(count);
    }
    groups
}

/// Computes Kendall's S variance with tie correction for X and Y (Sen 1968 / Kendall 1970).
fn kendall_s_variance(n: usize, x_ties: &[usize], y_ties: &[usize]) -> f64 {
    let nf = n as f64;
    let v0 = nf * (nf - 1.0) * (2.0 * nf + 5.0);
    let vx: f64 = x_ties
        .iter()
        .map(|&t| {
            let tf = t as f64;
            tf * (tf - 1.0) * (2.0 * tf + 5.0)
        })
        .sum();
    let vy: f64 = y_ties
        .iter()
        .map(|&u| {
            let uf = u as f64;
            uf * (uf - 1.0) * (2.0 * uf + 5.0)
        })
        .sum();

    let mut var_s = (v0 - vx - vy) / 18.0;

    if n > 2 {
        let tx3: f64 = x_ties
            .iter()
            .map(|&t| {
                let tf = t as f64;
                tf * (tf - 1.0) * (tf - 2.0)
            })
            .sum();
        let ty3: f64 = y_ties
            .iter()
            .map(|&u| {
                let uf = u as f64;
                uf * (uf - 1.0) * (uf - 2.0)
            })
            .sum();
        var_s += (tx3 * ty3) / (9.0 * nf * (nf - 1.0) * (nf - 2.0));
    }

    if n > 1 {
        let tx2: f64 = x_ties
            .iter()
            .map(|&t| {
                let tf = t as f64;
                tf * (tf - 1.0)
            })
            .sum();
        let ty2: f64 = y_ties
            .iter()
            .map(|&u| {
                let uf = u as f64;
                uf * (uf - 1.0)
            })
            .sum();
        var_s += (tx2 * ty2) / (2.0 * nf * (nf - 1.0));
    }

    var_s.max(0.0)
}

fn sgn_eps(diff: f64) -> i64 {
    if diff > EPS {
        1
    } else if diff < -EPS {
        -1
    } else {
        0
    }
}

/// Computes Theil-Sen robust simple linear regression (Sen 1968) and optional Mann-Kendall test.
pub fn compute_theil_sen(
    dep_var: &str,
    dep_vals: &[Option<f64>],
    indep_var: &str,
    indep_vals: &[Option<f64>],
    options: &TheilSenOptions,
) -> Result<TheilSenResult, AlgoError> {
    if dep_vals.len() != indep_vals.len() {
        return Err(AlgoError::InvalidInput(
            "Dependent and independent variable columns must have the same length".to_string(),
        ));
    }
    if !options.ci_level.is_finite() || options.ci_level <= 0.0 || options.ci_level >= 1.0 {
        return Err(AlgoError::InvalidInput(format!(
            "Confidence level must be strictly between 0 and 1, got {}",
            options.ci_level
        )));
    }

    let mut xs = Vec::with_capacity(dep_vals.len());
    let mut ys = Vec::with_capacity(dep_vals.len());
    let mut n_missing = 0_usize;

    for (y_opt, x_opt) in dep_vals.iter().zip(indep_vals.iter()) {
        match (y_opt, x_opt) {
            (Some(y), Some(x)) if y.is_finite() && x.is_finite() => {
                xs.push(*x);
                ys.push(*y);
            }
            _ => {
                n_missing += 1;
            }
        }
    }

    let n_valid = xs.len();
    if n_valid < 2 {
        return Err(AlgoError::InsufficientData(
            "Theil-Sen regression requires at least 2 valid observations".to_string(),
        ));
    }
    if n_valid > MAX_EXACT_PAIRS_N {
        return Err(AlgoError::InvalidInput(format!(
            "Theil-Sen exact pairwise computation supports at most {MAX_EXACT_PAIRS_N} observations (got {n_valid}). Consider subsampling the dataset."
        )));
    }

    let max_pairs = n_valid * (n_valid - 1) / 2;
    let mut slopes = Vec::with_capacity(max_pairs);
    let mut s_stat: i64 = 0;

    for i in 0..n_valid {
        for j in (i + 1)..n_valid {
            let dx = xs[j] - xs[i];
            let dy = ys[j] - ys[i];
            s_stat += sgn_eps(dx) * sgn_eps(dy);
            if dx.abs() > EPS {
                slopes.push(dy / dx);
            }
        }
    }

    if slopes.is_empty() {
        return Err(AlgoError::InvalidInput(
            "All independent variable values are identical; slope is undefined".to_string(),
        ));
    }

    slopes.sort_by(f64::total_cmp);
    let slope = median_sorted(&slopes).ok_or_else(|| {
        AlgoError::ComputationError("Failed to compute median of pairwise slopes".to_string())
    })?;

    let intercepts: Vec<f64> = xs
        .iter()
        .zip(ys.iter())
        .map(|(&x, &y)| y - slope * x)
        .collect();
    let intercept = median_of_slice(&intercepts).ok_or_else(|| {
        AlgoError::ComputationError("Failed to compute median intercept".to_string())
    })?;

    let x_ties = tie_group_sizes(&xs);
    let y_ties = tie_group_sizes(&ys);
    let var_s = kendall_s_variance(n_valid, &x_ties, &y_ties);
    let sd_s = var_s.sqrt();

    let alpha = 1.0 - options.ci_level;
    let z_crit = inv_normal_cdf(1.0 - alpha * 0.5);
    let c_alpha = z_crit * sd_s;
    let ns = slopes.len();
    let lower_idx = (((ns as f64 - c_alpha) * 0.5).floor().max(0.0) as usize).min(ns - 1);
    let upper_idx = (((ns as f64 + c_alpha) * 0.5).ceil().max(0.0) as usize).min(ns - 1);
    let (slope_ci_lower, slope_ci_upper) = if lower_idx <= upper_idx {
        (slopes[lower_idx], slopes[upper_idx])
    } else {
        (slopes[upper_idx], slopes[lower_idx])
    };

    let (kendall_tau, z_stat, p_value) = if options.mann_kendall {
        let n0 = (n_valid * (n_valid - 1)) as f64 * 0.5;
        let n1: f64 = x_ties.iter().map(|&t| (t * (t - 1)) as f64 * 0.5).sum();
        let n2: f64 = y_ties.iter().map(|&u| (u * (u - 1)) as f64 * 0.5).sum();
        let denom = ((n0 - n1) * (n0 - n2)).max(0.0).sqrt();
        let tau = if denom > EPS {
            (s_stat as f64 / denom).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let z = if sd_s > EPS {
            s_stat as f64 / sd_s
        } else {
            0.0
        };
        let p = (2.0 * (1.0 - normal_cdf(z.abs()))).clamp(0.0, 1.0);
        (Some(tau), Some(z), Some(p))
    } else {
        (None, None, None)
    };

    let mut points = Vec::with_capacity(n_valid);
    let mut residuals = Vec::with_capacity(n_valid);
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let fitted = slope * x + intercept;
        let residual = y - fitted;
        residuals.push(residual);
        points.push(TheilSenPoint {
            x,
            y,
            fitted,
            residual,
        });
    }

    let med_res = median_of_slice(&residuals).unwrap_or(0.0);
    let abs_dev: Vec<f64> = residuals.iter().map(|&r| (r - med_res).abs()).collect();
    let mad_residuals = median_of_slice(&abs_dev).unwrap_or(0.0);

    Ok(TheilSenResult {
        dep_var: dep_var.to_string(),
        indep_var: indep_var.to_string(),
        n_valid,
        n_missing,
        slope,
        intercept,
        slope_ci_lower,
        slope_ci_upper,
        ci_level: options.ci_level,
        kendall_tau,
        z_stat,
        p_value,
        mad_residuals,
        points,
        saved_predicted_var: None,
        saved_residuals_var: None,
        options: options.clone(),
        bootstrap: None,
    })
}

/// Computes non-parametric bootstrap summaries for Theil-Sen slope and intercept.
pub fn compute_theil_sen_bootstrap(
    dep_var: &str,
    dep_vals: &[Option<f64>],
    indep_var: &str,
    indep_vals: &[Option<f64>],
    options: &TheilSenOptions,
    strata_keys: Option<&[String]>,
    boot_options: &BootstrapOptions,
) -> Result<TheilSenBootstrap, BootstrapError> {
    let n = dep_vals.len();
    let plan = SamplingPlan::from_options(n, strata_keys, boot_options)?;
    let boot_opts = TheilSenOptions {
        ci_level: options.ci_level,
        mann_kendall: false,
    };

    let slope_summary = run_bootstrap_scalar(&plan, boot_options, |indices| {
        let mut sub_y = Vec::with_capacity(indices.len());
        let mut sub_x = Vec::with_capacity(indices.len());
        for &idx in indices {
            if idx < n {
                sub_y.push(dep_vals[idx]);
                sub_x.push(indep_vals[idx]);
            }
        }
        compute_theil_sen(dep_var, &sub_y, indep_var, &sub_x, &boot_opts)
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
        compute_theil_sen(dep_var, &sub_y, indep_var, &sub_x, &boot_opts)
            .ok()
            .map(|r| r.intercept)
    })
    .ok();

    Ok(TheilSenBootstrap {
        options: boot_options.clone(),
        slope: slope_summary,
        intercept: intercept_summary,
    })
}
