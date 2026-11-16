use crate::anova_oneway::AlgoError;
use crate::multinomial_logistic::types::MultinomialLogisticOptions;
use crate::regression::invert_matrix;

/// Fitted output from the Newton-Raphson multinomial logistic engine.
pub struct MultinomialEngineOutput {
    pub beta: Vec<f64>,
    pub cov_matrix: Vec<Vec<f64>>,
    pub predicted_probs: Vec<Vec<f64>>,
    pub minus_2ll: f64,
    pub minus_2ll_intercept_only: f64,
    pub iterations: usize,
    pub converged: bool,
    pub warnings: Vec<String>,
}

/// Computes stable predicted probabilities for observation row x given beta.
/// Output length is J (M non-reference categories + 1 reference category).
pub fn predict_row_probs(row: &[f64], beta: &[f64], m: usize, p: usize) -> Vec<f64> {
    let mut etas = Vec::with_capacity(m);
    let mut max_eta = 0.0_f64;

    for j in 0..m {
        let mut eta = 0.0;
        let offset = j * p;
        for (col_idx, &val) in row.iter().enumerate().take(p) {
            eta += val * beta[offset + col_idx];
        }
        if eta > max_eta {
            max_eta = eta;
        }
        etas.push(eta);
    }

    let mut sum_exp = (-max_eta).exp(); // reference category eta = 0.0
    for &eta in &etas {
        sum_exp += (eta - max_eta).exp();
    }

    let mut probs = Vec::with_capacity(m + 1);
    for &eta in &etas {
        let pr = ((eta - max_eta).exp() / sum_exp).clamp(1e-15, 1.0 - 1e-15);
        probs.push(pr);
    }
    // Reference category is at index m
    let ref_pr = ((-max_eta).exp() / sum_exp).clamp(1e-15, 1.0 - 1e-15);
    probs.push(ref_pr);

    probs
}

/// Evaluates log-likelihood given current beta.
pub fn eval_log_likelihood(
    x: &[Vec<f64>],
    y_indices: &[usize],
    beta: &[f64],
    m: usize,
    p: usize,
) -> f64 {
    let mut log_lik = 0.0;
    for (i, row) in x.iter().enumerate() {
        let probs = predict_row_probs(row, beta, m, p);
        let yi = y_indices[i];
        let p_yi = probs[yi].max(1e-15);
        log_lik += p_yi.ln();
    }
    log_lik
}

/// Fits a multinomial logit model via Newton-Raphson optimization.
pub fn fit_multinomial_engine(
    x: &[Vec<f64>],
    y_indices: &[usize],
    category_counts: &[usize],
    m: usize,
    p: usize,
    options: &MultinomialLogisticOptions,
) -> Result<MultinomialEngineOutput, AlgoError> {
    let n = x.len();
    let k = m * p;
    let mut warnings = Vec::new();

    if n == 0 || p == 0 || m == 0 {
        return Err(AlgoError::InvalidInput(
            "Empty design matrix or insufficient categories".to_string(),
        ));
    }

    // Intercept-only log-likelihood
    let n_f64 = n as f64;
    let mut minus_2ll_null = 0.0;
    for &cnt in category_counts {
        if cnt > 0 {
            let p_null = (cnt as f64) / n_f64;
            minus_2ll_null += -2.0 * (cnt as f64) * p_null.ln();
        }
    }

    // Initial beta
    let mut beta = vec![0.0; k];
    if options.include_constant {
        let ref_cnt = category_counts[m] as f64;
        if ref_cnt > 0.0 {
            for j in 0..m {
                let cnt = category_counts[j] as f64;
                if cnt > 0.0 {
                    beta[j * p] = (cnt / ref_cnt).ln();
                }
            }
        }
    }

    let mut converged = false;
    let mut actual_iterations = 0;
    let mut current_ll = eval_log_likelihood(x, y_indices, &beta, m, p);

    for iter in 0..options.max_iterations {
        actual_iterations = iter + 1;

        // 1. Calculate probabilities for each observation
        let mut all_probs = Vec::with_capacity(n);
        for row in x {
            all_probs.push(predict_row_probs(row, &beta, m, p));
        }

        // 2. Score vector g
        let mut g = vec![0.0; k];
        for i in 0..n {
            let yi = y_indices[i];
            let row = &x[i];
            let probs = &all_probs[i];
            for j in 0..m {
                let y_ij = if yi == j { 1.0 } else { 0.0 };
                let diff = y_ij - probs[j];
                let offset = j * p;
                for col in 0..p {
                    g[offset + col] += diff * row[col];
                }
            }
        }

        // 3. Information matrix I = -H
        let mut info_matrix = vec![vec![0.0; k]; k];
        for i in 0..n {
            let row = &x[i];
            let probs = &all_probs[i];
            for j in 0..m {
                let pj = probs[j];
                let offset_j = j * p;
                for k_idx in j..m {
                    let pk = probs[k_idx];
                    let offset_k = k_idx * p;
                    let w_jk = if j == k_idx {
                        pj * (1.0 - pj)
                    } else {
                        -pj * pk
                    };

                    for col1 in 0..p {
                        let term_base = w_jk * row[col1];
                        let r1 = offset_j + col1;
                        for col2 in 0..p {
                            let r2 = offset_k + col2;
                            let val = term_base * row[col2];
                            info_matrix[r1][r2] += val;
                            if j != k_idx {
                                info_matrix[r2][r1] += val;
                            }
                        }
                    }
                }
            }
        }

        // Ridge regularization to ensure non-singularity
        let mut inv_info = None;
        for &ridge in &[1e-9, 1e-7, 1e-5, 1e-3, 1e-2] {
            let mut reg_info = info_matrix.clone();
            for r in 0..k {
                reg_info[r][r] += ridge;
            }
            if let Ok(inv) = invert_matrix(&reg_info) {
                inv_info = Some(inv);
                break;
            }
        }

        let inv_matrix = match inv_info {
            Some(inv) => inv,
            None => {
                warnings.push(
                    "Information matrix is singular; parameter estimates may not be unique."
                        .to_string(),
                );
                break;
            }
        };

        // Step delta = inv_info * g
        let mut delta = vec![0.0; k];
        for r1 in 0..k {
            for r2 in 0..k {
                delta[r1] += inv_matrix[r1][r2] * g[r2];
            }
        }

        // Step halving backtracking line search
        let mut step_size = 1.0;
        let mut next_beta = beta.clone();
        let mut next_ll = current_ll;
        let mut improved = false;

        for _ in 0..5 {
            for r in 0..k {
                next_beta[r] = beta[r] + step_size * delta[r];
            }
            let cand_ll = eval_log_likelihood(x, y_indices, &next_beta, m, p);
            if cand_ll >= current_ll - 1e-7 {
                next_ll = cand_ll;
                improved = true;
                break;
            }
            step_size *= 0.5;
        }

        if !improved {
            for r in 0..k {
                next_beta[r] = beta[r] + step_size * delta[r];
            }
            next_ll = eval_log_likelihood(x, y_indices, &next_beta, m, p);
        }

        // Convergence test
        let mut max_abs_diff = 0.0_f64;
        for r in 0..k {
            let diff = (next_beta[r] - beta[r]).abs();
            if diff > max_abs_diff {
                max_abs_diff = diff;
            }
        }

        let ll_diff = (next_ll - current_ll).abs();
        let rel_ll_diff = ll_diff / (current_ll.abs() + 1.0);
        beta = next_beta;
        current_ll = next_ll;

        if max_abs_diff < options.convergence_tol
            || rel_ll_diff < options.convergence_tol
            || ll_diff < 1e-6
        {
            converged = true;
            break;
        }
    }

    // Final predicted probabilities
    let mut final_probs = Vec::with_capacity(n);
    for row in x {
        final_probs.push(predict_row_probs(row, &beta, m, p));
    }

    // Final covariance matrix = inv(I)
    let mut final_cov = vec![vec![0.0; k]; k];
    let mut final_info = vec![vec![0.0; k]; k];
    for i in 0..n {
        let row = &x[i];
        let probs = &final_probs[i];
        for j in 0..m {
            let pj = probs[j];
            let offset_j = j * p;
            for k_idx in j..m {
                let pk = probs[k_idx];
                let offset_k = k_idx * p;
                let w_jk = if j == k_idx {
                    pj * (1.0 - pj)
                } else {
                    -pj * pk
                };

                for col1 in 0..p {
                    let term = w_jk * row[col1];
                    let r1 = offset_j + col1;
                    for col2 in 0..p {
                        let r2 = offset_k + col2;
                        let val = term * row[col2];
                        final_info[r1][r2] += val;
                        if j != k_idx {
                            final_info[r2][r1] += val;
                        }
                    }
                }
            }
        }
    }
    for r in 0..k {
        final_info[r][r] += 1e-8;
    }
    if let Ok(inv) = invert_matrix(&final_info) {
        final_cov = inv;
    }

    Ok(MultinomialEngineOutput {
        beta,
        cov_matrix: final_cov,
        predicted_probs: final_probs,
        minus_2ll: -2.0 * current_ll,
        minus_2ll_intercept_only: minus_2ll_null,
        iterations: actual_iterations,
        converged,
        warnings,
    })
}
