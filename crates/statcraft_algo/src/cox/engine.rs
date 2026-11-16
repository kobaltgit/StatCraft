use crate::anova_oneway::AlgoError;
use crate::cox::types::{CoxDataPoint, CoxRegressionOptions, TieMethod};
use crate::regression::invert_matrix;
use std::collections::BTreeMap;

/// Engine calculation output containing estimated coefficients and variance-covariance.
#[derive(Debug, Clone)]
pub struct CoxEngineOutput {
    pub beta: Vec<f64>,
    pub cov_matrix: Vec<Vec<f64>>,
    pub log_lik_null: f64,
    pub log_lik_final: f64,
    pub iterations: usize,
    pub converged: bool,
}

/// Evaluation output of partial log-likelihood, score vector, and information matrix.
pub type LikelihoodEval = (f64, Vec<f64>, Vec<Vec<f64>>);

/// Evaluates Log Partial Likelihood, Score Vector, and Fisher Information matrix.
pub fn eval_partial_likelihood(
    strata_data: &[Vec<&CoxDataPoint>],
    beta: &[f64],
    p: usize,
    tie_method: TieMethod,
) -> Result<LikelihoodEval, AlgoError> {
    let mut total_log_lik = 0.0_f64;
    let mut total_score = vec![0.0_f64; p];
    let mut total_info = vec![vec![0.0_f64; p]; p];

    for stratum in strata_data {
        if stratum.is_empty() {
            continue;
        }

        // Distinct event times sorted ascending
        let mut distinct_event_times: BTreeMap<i64, f64> = BTreeMap::new();
        for pt in stratum.iter() {
            if pt.is_event {
                let key = (pt.time * 1_000_000.0).round() as i64;
                distinct_event_times.entry(key).or_insert(pt.time);
            }
        }

        // Compute weights w_i = exp(beta^T x_i) for all subjects in stratum
        let mut weights = Vec::with_capacity(stratum.len());
        for pt in stratum.iter() {
            let mut eta = 0.0_f64;
            for (col_idx, &val) in pt.covariates.iter().enumerate().take(p) {
                eta += val * beta[col_idx];
            }
            let clamped_eta = eta.clamp(-700.0, 700.0);
            weights.push(clamped_eta.exp());
        }

        for &event_time in distinct_event_times.values() {
            let mut risk_indices = Vec::new();
            let mut event_indices = Vec::new();

            for (idx, pt) in stratum.iter().enumerate() {
                if pt.time >= event_time - 1e-9 {
                    risk_indices.push(idx);
                }
                if pt.is_event && (pt.time - event_time).abs() < 1e-9 {
                    event_indices.push(idx);
                }
            }

            let d_k = event_indices.len();
            if d_k == 0 {
                continue;
            }

            // Sum of covariates among events s_k
            let mut s_k = vec![0.0_f64; p];
            for &e_idx in &event_indices {
                for m in 0..p {
                    s_k[m] += stratum[e_idx].covariates[m];
                }
            }

            // S^(0), S^(1), S^(2) over the full risk set
            let mut s0 = 0.0_f64;
            let mut s1 = vec![0.0_f64; p];
            let mut s2 = vec![vec![0.0_f64; p]; p];

            for &r_idx in &risk_indices {
                let w = weights[r_idx];
                s0 += w;
                let covs = &stratum[r_idx].covariates;
                for m1 in 0..p {
                    s1[m1] += w * covs[m1];
                    for m2 in 0..p {
                        s2[m1][m2] += w * covs[m1] * covs[m2];
                    }
                }
            }

            if s0 <= 0.0 {
                return Err(AlgoError::ComputationError(
                    "Risk set sum of weights is non-positive".to_string(),
                ));
            }

            match tie_method {
                TieMethod::Breslow => {
                    let mut beta_dot_s = 0.0_f64;
                    for m in 0..p {
                        beta_dot_s += beta[m] * s_k[m];
                    }
                    total_log_lik += beta_dot_s - (d_k as f64) * s0.ln();

                    for m in 0..p {
                        total_score[m] += s_k[m] - (d_k as f64) * (s1[m] / s0);
                    }

                    for m1 in 0..p {
                        let e_m1 = s1[m1] / s0;
                        for m2 in 0..p {
                            let e_m2 = s1[m2] / s0;
                            let v_elem = (s2[m1][m2] / s0) - (e_m1 * e_m2);
                            total_info[m1][m2] += (d_k as f64) * v_elem;
                        }
                    }
                }
                TieMethod::Efron => {
                    let mut beta_dot_s = 0.0_f64;
                    for m in 0..p {
                        beta_dot_s += beta[m] * s_k[m];
                    }
                    total_log_lik += beta_dot_s;

                    // Event sums for Efron adjustment
                    let mut s0_events = 0.0_f64;
                    let mut s1_events = vec![0.0_f64; p];
                    let mut s2_events = vec![vec![0.0_f64; p]; p];

                    for &e_idx in &event_indices {
                        let w = weights[e_idx];
                        s0_events += w;
                        let covs = &stratum[e_idx].covariates;
                        for m1 in 0..p {
                            s1_events[m1] += w * covs[m1];
                            for m2 in 0..p {
                                s2_events[m1][m2] += w * covs[m1] * covs[m2];
                            }
                        }
                    }

                    for m in 0..p {
                        total_score[m] += s_k[m];
                    }

                    let d_f64 = d_k as f64;
                    for l in 0..d_k {
                        let alpha = (l as f64) / d_f64;
                        let s0_l = s0 - alpha * s0_events;
                        if s0_l <= 0.0 {
                            return Err(AlgoError::ComputationError(
                                "Efron adjusted risk sum is non-positive".to_string(),
                            ));
                        }
                        total_log_lik -= s0_l.ln();

                        let mut s1_l = vec![0.0_f64; p];
                        for m in 0..p {
                            s1_l[m] = s1[m] - alpha * s1_events[m];
                            total_score[m] -= s1_l[m] / s0_l;
                        }

                        for m1 in 0..p {
                            let e_m1 = s1_l[m1] / s0_l;
                            for m2 in 0..p {
                                let s2_elem = s2[m1][m2] - alpha * s2_events[m1][m2];
                                let e_m2 = s1_l[m2] / s0_l;
                                let v_elem = (s2_elem / s0_l) - (e_m1 * e_m2);
                                total_info[m1][m2] += v_elem;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((total_log_lik, total_score, total_info))
}

/// Fits Cox Proportional Hazards regression via Newton-Raphson.
pub fn fit_cox_engine(
    strata_data: &[Vec<&CoxDataPoint>],
    p: usize,
    options: &CoxRegressionOptions,
) -> Result<CoxEngineOutput, AlgoError> {
    let mut beta = vec![0.0_f64; p];

    // Initial null log-likelihood at beta = 0
    let (log_lik_null, _, _) = eval_partial_likelihood(strata_data, &beta, p, options.tie_method)?;

    let mut current_log_lik = log_lik_null;
    let mut converged = false;
    let mut iterations = 0;
    let mut last_cov_matrix = vec![vec![0.0_f64; p]; p];

    for iter in 0..options.max_iterations {
        iterations = iter + 1;
        let (_log_lik, score, info) =
            eval_partial_likelihood(strata_data, &beta, p, options.tie_method)?;

        let inv_info = invert_matrix(&info).map_err(|_| {
            AlgoError::ComputationError(
                "Information matrix is singular; collinear covariates detected".to_string(),
            )
        })?;
        last_cov_matrix = inv_info.clone();

        // Delta = inv_info * score
        let mut delta = vec![0.0_f64; p];
        for i in 0..p {
            for j in 0..p {
                delta[i] += inv_info[i][j] * score[j];
            }
        }

        // Backtracking line search with step halving
        let mut step = 1.0_f64;
        let mut new_beta = beta.clone();
        let mut new_log_lik = current_log_lik;
        let mut line_search_success = false;

        for _ in 0..10 {
            for m in 0..p {
                new_beta[m] = beta[m] + step * delta[m];
            }
            if let Ok((cand_ll, _, _)) =
                eval_partial_likelihood(strata_data, &new_beta, p, options.tie_method)
            {
                if cand_ll >= current_log_lik - 1e-6 || step < 0.01 {
                    new_log_lik = cand_ll;
                    line_search_success = true;
                    break;
                }
            }
            step *= 0.5;
        }

        if !line_search_success {
            // Keep delta as best effort
            for m in 0..p {
                new_beta[m] = beta[m] + delta[m];
            }
        }

        let max_delta = delta.iter().map(|d| d.abs()).fold(0.0_f64, f64::max);
        let log_lik_diff = (new_log_lik - current_log_lik).abs();

        beta = new_beta;
        current_log_lik = new_log_lik;

        if max_delta < options.convergence_tol || log_lik_diff < 1e-8 {
            converged = true;
            // Final information matrix evaluation at converged beta
            if let Ok((_, _, final_info)) =
                eval_partial_likelihood(strata_data, &beta, p, options.tie_method)
            {
                if let Ok(cov) = invert_matrix(&final_info) {
                    last_cov_matrix = cov;
                }
            }
            break;
        }
    }

    Ok(CoxEngineOutput {
        beta,
        cov_matrix: last_cov_matrix,
        log_lik_null,
        log_lik_final: current_log_lik,
        iterations,
        converged,
    })
}
