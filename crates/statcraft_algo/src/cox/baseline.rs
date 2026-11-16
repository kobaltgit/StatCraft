use crate::cox::types::{BaselineSurvivalPoint, CoxDataPoint};
use std::collections::BTreeMap;

/// Computes Breslow baseline cumulative hazard and survival curves.
pub fn compute_baseline_survival(
    strata_data: &[(&Option<String>, Vec<&CoxDataPoint>)],
    beta: &[f64],
    covariate_means: &[f64],
    p: usize,
) -> Vec<BaselineSurvivalPoint> {
    let mut baseline_points = Vec::new();

    // Overall mean prognostic index: eta_mean = beta^T * x_mean
    let mut eta_mean = 0.0_f64;
    for (m, &mean_val) in covariate_means.iter().enumerate().take(p) {
        eta_mean += beta[m] * mean_val;
    }
    let exp_eta_mean = eta_mean.clamp(-700.0, 700.0).exp();

    for (stratum_name, pts) in strata_data {
        if pts.is_empty() {
            continue;
        }

        // Distinct event times sorted ascending
        let mut distinct_event_times: BTreeMap<i64, f64> = BTreeMap::new();
        for pt in pts {
            if pt.is_event {
                let key = (pt.time * 1_000_000.0).round() as i64;
                distinct_event_times.entry(key).or_insert(pt.time);
            }
        }

        // Precompute weights w_i = exp(beta^T x_i)
        let mut weights = Vec::with_capacity(pts.len());
        for pt in pts {
            let mut eta = 0.0_f64;
            for (col_idx, &val) in pt.covariates.iter().enumerate().take(p) {
                eta += val * beta[col_idx];
            }
            weights.push(eta.clamp(-700.0, 700.0).exp());
        }

        let mut cum_hazard = 0.0_f64;

        for &event_time in distinct_event_times.values() {
            let mut sum_risk_w = 0.0_f64;
            let mut d_k = 0_usize;

            for (idx, pt) in pts.iter().enumerate() {
                if pt.time >= event_time - 1e-9 {
                    sum_risk_w += weights[idx];
                }
                if pt.is_event && (pt.time - event_time).abs() < 1e-9 {
                    d_k += 1;
                }
            }

            if sum_risk_w > 0.0 && d_k > 0 {
                let delta_h = (d_k as f64) / sum_risk_w;
                cum_hazard += delta_h;

                let s0 = (-cum_hazard).exp();
                let s_mean = (-cum_hazard * exp_eta_mean).exp();

                baseline_points.push(BaselineSurvivalPoint {
                    stratum: (*stratum_name).clone(),
                    time: event_time,
                    baseline_cum_hazard: cum_hazard,
                    baseline_survival: s0,
                    mean_covariates_survival: s_mean,
                });
            }
        }
    }

    baseline_points
}
