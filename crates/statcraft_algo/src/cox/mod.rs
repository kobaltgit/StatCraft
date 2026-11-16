pub mod baseline;
pub mod engine;
#[cfg(test)]
pub mod tests;
pub mod types;

pub use baseline::compute_baseline_survival;
pub use engine::{eval_partial_likelihood, fit_cox_engine, CoxEngineOutput};
pub use types::{
    BaselineSurvivalPoint, CoxCaseSummary, CoxDataPoint, CoxRegressionOptions, CoxRegressionResult,
    CoxVariableInEquation, OmnibusTestsResult, TieMethod,
};

use crate::anova_oneway::AlgoError;
use crate::crosstabs::chi_square_sf;
use std::collections::BTreeMap;

/// Standard normal inverse CDF (quantile function) via rational approximation (Acklam's formula).
#[allow(clippy::excessive_precision)]
pub fn normal_quantile(p: f64) -> f64 {
    let p = p.clamp(1e-12, 1.0 - 1e-12);
    let a = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    let b = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    let c = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    let d = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

/// Computes Cox Proportional Hazards Regression.
pub fn compute_cox_regression(
    data: &[CoxDataPoint],
    time_var: &str,
    status_var: &str,
    strata_var: Option<&str>,
    covariate_names: &[String],
    options: &CoxRegressionOptions,
) -> Result<CoxRegressionResult, AlgoError> {
    if data.is_empty() {
        return Err(AlgoError::InsufficientData("Data is empty".to_string()));
    }
    let p = covariate_names.len();
    if p == 0 {
        return Err(AlgoError::InvalidInput(
            "At least one covariate must be specified for Cox regression".to_string(),
        ));
    }

    // Validate data points
    for pt in data {
        if pt.time < 0.0 {
            return Err(AlgoError::InvalidInput(
                "Survival times must be non-negative".to_string(),
            ));
        }
        if pt.covariates.len() != p {
            return Err(AlgoError::InvalidInput(
                "Covariates dimension mismatch".to_string(),
            ));
        }
    }

    // Partition by stratum
    let mut strata_map: BTreeMap<Option<String>, Vec<&CoxDataPoint>> = BTreeMap::new();
    for pt in data {
        strata_map.entry(pt.stratum.clone()).or_default().push(pt);
    }

    // Calculate Case Summaries
    let mut case_summaries = Vec::new();
    let mut total_all = 0_usize;
    let mut events_all = 0_usize;
    let mut censored_all = 0_usize;

    for (stratum, pts) in &strata_map {
        let total = pts.len();
        let events = pts.iter().filter(|pt| pt.is_event).count();
        let censored = total.saturating_sub(events);
        let pct = if total > 0 {
            (censored as f64) / (total as f64) * 100.0
        } else {
            0.0
        };

        total_all += total;
        events_all += events;
        censored_all += censored;

        case_summaries.push(CoxCaseSummary {
            stratum: stratum.clone(),
            total_n: total,
            events_n: events,
            censored_n: censored,
            censored_percent: pct,
        });
    }

    if events_all == 0 {
        return Err(AlgoError::InsufficientData(
            "No events observed in the dataset".to_string(),
        ));
    }

    let overall_summary = CoxCaseSummary {
        stratum: None,
        total_n: total_all,
        events_n: events_all,
        censored_n: censored_all,
        censored_percent: if total_all > 0 {
            (censored_all as f64) / (total_all as f64) * 100.0
        } else {
            0.0
        },
    };

    // Calculate Covariate Means
    let mut covariate_means = vec![0.0_f64; p];
    let n_f64 = total_all as f64;
    for pt in data {
        for m in 0..p {
            covariate_means[m] += pt.covariates[m];
        }
    }
    for m in 0..p {
        covariate_means[m] /= n_f64;
    }

    // Prepare strata slices for the engine
    let strata_slices: Vec<Vec<&CoxDataPoint>> = strata_map.values().cloned().collect();

    // Run Newton-Raphson fitting
    let engine_out = fit_cox_engine(&strata_slices, p, options)?;

    // Omnibus tests
    let init_2ll = -2.0 * engine_out.log_lik_null;
    let fin_2ll = -2.0 * engine_out.log_lik_final;
    let chi_sq = (init_2ll - fin_2ll).max(0.0);
    let omnibus_p = chi_square_sf(chi_sq, p);

    let omnibus_tests = OmnibusTestsResult {
        initial_minus_2ll: init_2ll,
        final_minus_2ll: fin_2ll,
        chi_square: chi_sq,
        df: p,
        p_value: omnibus_p,
    };

    // Variables in the equation
    let alpha = 1.0 - options.confidence_level;
    let z = normal_quantile(1.0 - alpha / 2.0);

    let mut variables = Vec::with_capacity(p);
    for m in 0..p {
        let b = engine_out.beta[m];
        let var_b = engine_out.cov_matrix[m][m].max(0.0);
        let se = var_b.sqrt();
        let wald = if se > 1e-12 { (b / se).powi(2) } else { 0.0 };
        let p_val = chi_square_sf(wald, 1);
        let exp_b = b.exp();
        let ci_lower = (b - z * se).exp();
        let ci_upper = (b + z * se).exp();

        variables.push(CoxVariableInEquation {
            name: covariate_names[m].clone(),
            b,
            se,
            wald,
            df: 1,
            p_value: p_val,
            exp_b,
            ci_lower,
            ci_upper,
        });
    }

    // Baseline survival if requested
    let baseline_survival = if options.print_baseline {
        let strata_tuples: Vec<(&Option<String>, Vec<&CoxDataPoint>)> =
            strata_map.iter().map(|(k, v)| (k, v.clone())).collect();
        compute_baseline_survival(&strata_tuples, &engine_out.beta, &covariate_means, p)
    } else {
        Vec::new()
    };

    Ok(CoxRegressionResult {
        time_variable: time_var.to_string(),
        status_variable: status_var.to_string(),
        strata_variable: strata_var.map(|s| s.to_string()),
        covariate_names: covariate_names.to_vec(),
        covariate_means,
        case_summaries,
        overall_summary,
        omnibus_tests,
        variables,
        baseline_survival,
        tie_method: options.tie_method,
        iterations: engine_out.iterations,
        converged: engine_out.converged,
    })
}
