use crate::anova_oneway::AlgoError;
use crate::logistic::math::{chisq_p_value, normal_critical_value, sigmoid};
use crate::logistic::types::{
    LogisticCoefficientRow, LogisticModelSummary, LogisticRegressionOptions, OmnibusTestsRow,
};
use crate::regression::invert_matrix;

pub struct FittedEngineOutput {
    pub coefficients: Vec<LogisticCoefficientRow>,
    pub omnibus: OmnibusTestsRow,
    pub model_summary: LogisticModelSummary,
    pub predicted_probs: Vec<f64>,
    pub iterations: usize,
    pub converged: bool,
    pub warnings: Vec<String>,
}

/// Executes Newton-Raphson / IRLS optimization for binary logistic regression.
pub fn fit_logistic_engine(
    x: &[Vec<f64>],
    y: &[f64],
    param_names: &[String],
    options: &LogisticRegressionOptions,
) -> Result<FittedEngineOutput, AlgoError> {
    let n = x.len();
    let p = if n > 0 { x[0].len() } else { 0 };

    if n == 0 || p == 0 {
        return Err(AlgoError::InvalidInput("Empty design matrix".to_string()));
    }

    let mut warnings = Vec::new();

    let n_pos = y.iter().filter(|&&val| (val - 1.0).abs() < 1e-6).count();
    let n_neg = n - n_pos;

    if n_pos == 0 || n_neg == 0 {
        return Err(AlgoError::InvalidInput(
            "Dependent variable must have both 0 and 1 outcomes in valid cases".to_string(),
        ));
    }

    // Initial coefficients: constant = ln(p / (1-p)), all others 0.0
    let mut beta = vec![0.0; p];
    if options.include_constant && p > 0 {
        let p_init = (n_pos as f64) / (n as f64);
        beta[0] = (p_init / (1.0 - p_init)).ln();
    }

    let mut converged = false;
    let mut actual_iterations = 0;

    for iter in 0..options.max_iterations {
        actual_iterations = iter + 1;

        // 1. Calculate predicted probabilities pi and weights w
        let mut pi = Vec::with_capacity(n);
        let mut w = Vec::with_capacity(n);
        for row in x {
            let mut eta = 0.0;
            for j in 0..p {
                eta += row[j] * beta[j];
            }
            let prob = sigmoid(eta);
            pi.push(prob);
            w.push(prob * (1.0 - prob));
        }

        // 2. Gradient g = X^T (y - pi)
        let mut g = vec![0.0; p];
        for i in 0..n {
            let err = y[i] - pi[i];
            for j in 0..p {
                g[j] += err * x[i][j];
            }
        }

        // 3. Hessian / Fisher Info matrix A = X^T W X
        let mut a = vec![vec![0.0; p]; p];
        for i in 0..n {
            let wi = w[i];
            for j in 0..p {
                let x_ij = x[i][j];
                for k in j..p {
                    let term = wi * x_ij * x[i][k];
                    a[j][k] += term;
                    if j != k {
                        a[k][j] += term;
                    }
                }
            }
        }

        // Small ridge damping for numerical conditioning
        for (j, row) in a.iter_mut().enumerate().take(p) {
            row[j] += 1e-11;
        }

        // 4. Invert Fisher information matrix
        let inv_a = match invert_matrix(&a) {
            Ok(inv) => inv,
            Err(_) => {
                // Try stronger regularization if ill-conditioned
                for (j, row) in a.iter_mut().enumerate().take(p) {
                    row[j] += 1e-5;
                }
                let inv = invert_matrix(&a)?;
                warnings.push(
                    "Fisher information matrix is ill-conditioned: applied ridge regularization"
                        .to_string(),
                );
                inv
            }
        };

        // 5. Delta = inv_A * g
        let mut delta = vec![0.0; p];
        let mut max_abs_delta = 0.0;
        for j in 0..p {
            let mut step = 0.0;
            for k in 0..p {
                step += inv_a[j][k] * g[k];
            }
            delta[j] = step;
            if step.abs() > max_abs_delta {
                max_abs_delta = step.abs();
            }
        }

        // 6. Update beta
        for j in 0..p {
            beta[j] += delta[j];
        }

        if max_abs_delta < options.convergence_tol {
            converged = true;
            break;
        }
    }

    if !converged {
        warnings.push(format!(
            "IRLS optimization did not converge within {} iterations",
            options.max_iterations
        ));
    }

    // Final predictions, weights, and covariance matrix
    let mut predicted_probs = Vec::with_capacity(n);
    let mut w_final = Vec::with_capacity(n);
    for row in x {
        let mut eta = 0.0;
        for j in 0..p {
            eta += row[j] * beta[j];
        }
        let prob = sigmoid(eta);
        predicted_probs.push(prob);
        w_final.push(prob * (1.0 - prob));
    }

    let mut a_final = vec![vec![0.0; p]; p];
    for i in 0..n {
        let wi = w_final[i];
        for j in 0..p {
            let x_ij = x[i][j];
            for k in j..p {
                let term = wi * x_ij * x[i][k];
                a_final[j][k] += term;
                if j != k {
                    a_final[k][j] += term;
                }
            }
        }
    }

    let cov_matrix = invert_matrix(&a_final).unwrap_or_else(|_| {
        for (j, row) in a_final.iter_mut().enumerate().take(p) {
            row[j] += 1e-5;
        }
        invert_matrix(&a_final).unwrap_or_else(|_| vec![vec![0.0; p]; p])
    });

    let z_crit = normal_critical_value(options.confidence_level);

    let mut coefficients = Vec::with_capacity(p);
    for j in 0..p {
        let b = beta[j];
        let var_b = if j < cov_matrix.len() && j < cov_matrix[j].len() {
            cov_matrix[j][j].max(0.0)
        } else {
            0.0
        };
        let se = var_b.sqrt();
        let wald = if se > 1e-14 { (b / se).powi(2) } else { 0.0 };
        let sig = chisq_p_value(wald, 1);
        let exp_b = b.exp();
        let lower_ci = (b - z_crit * se).exp();
        let upper_ci = (b + z_crit * se).exp();

        coefficients.push(LogisticCoefficientRow {
            param_name: param_names.get(j).cloned().unwrap_or_default(),
            b,
            se,
            wald,
            df: 1,
            sig,
            exp_b,
            lower_ci,
            upper_ci,
        });
    }

    // Log-likelihood calculations
    let mut log_lik = 0.0;
    for i in 0..n {
        let pi_val = predicted_probs[i];
        log_lik += y[i] * pi_val.ln() + (1.0 - y[i]) * (1.0 - pi_val).ln();
    }
    let minus_2ll = -2.0 * log_lik;

    let n_f64 = n as f64;
    let n_pos_f = n_pos as f64;
    let n_neg_f = n_neg as f64;
    let ll0 = n_pos_f * (n_pos_f / n_f64).ln() + n_neg_f * (n_neg_f / n_f64).ln();
    let minus_2ll0 = -2.0 * ll0;

    let chi_sq_omni = (minus_2ll0 - minus_2ll).max(0.0);
    let df_omni = if options.include_constant && p > 1 {
        p - 1
    } else {
        p
    };
    let sig_omni = chisq_p_value(chi_sq_omni, df_omni);

    let r2_cs = (1.0 - (-chi_sq_omni / n_f64).exp()).clamp(0.0, 1.0);
    let r2_max = (1.0 - (-minus_2ll0 / n_f64).exp()).clamp(1e-14, 1.0);
    let r2_n = (r2_cs / r2_max).clamp(0.0, 1.0);

    Ok(FittedEngineOutput {
        coefficients,
        omnibus: OmnibusTestsRow {
            chi_square: chi_sq_omni,
            df: df_omni,
            sig: sig_omni,
        },
        model_summary: LogisticModelSummary {
            minus_2ll,
            cox_snell_r2: r2_cs,
            nagelkerke_r2: r2_n,
        },
        predicted_probs,
        iterations: actual_iterations,
        converged,
        warnings,
    })
}
