use crate::rank_cases::inv_normal_cdf;
use crate::AlgoError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalityTestRow {
    pub statistic: f64,
    pub df: usize,
    pub sig: f64,
}

/// Evaluates a polynomial with given coefficients using Horner's method:
/// P(x) = coeffs[0] * x^(deg) + coeffs[1] * x^(deg-1) + ... + coeffs[deg].
pub(crate) fn poly_eval(coeffs: &[f64], x: f64) -> f64 {
    let mut res = 0.0;
    for &c in coeffs {
        res = res * x + c;
    }
    res
}

/// Computes the standard normal cumulative distribution function Phi(z)
/// using Abramowitz and Stegun (1964) formula 7.1.26 with error < 1.5e-7.
pub(crate) fn normal_cdf(z: f64) -> f64 {
    if z.is_nan() {
        return f64::NAN;
    }
    if z < -10.0 {
        return 0.0;
    }
    if z > 10.0 {
        return 1.0;
    }

    let x = z / std::f64::consts::SQRT_2;
    let abs_x = x.abs();

    const P: f64 = 0.3275911;
    const A1: f64 = 0.254829592;
    const A2: f64 = -0.284496736;
    const A3: f64 = 1.421413741;
    const A4: f64 = -1.453152027;
    const A5: f64 = 1.061405429;

    let t = 1.0 / (1.0 + P * abs_x);
    let poly = ((((A5 * t + A4) * t + A3) * t + A2) * t + A1) * t;
    let erf_val = 1.0 - poly * (-abs_x * abs_x).exp();

    let signed_erf = if x >= 0.0 { erf_val } else { -erf_val };
    0.5 * (1.0 + signed_erf)
}

// Royston (1992) polynomial approximations for Shapiro-Wilk test
const POLY3: [f64; 4] = [-0.0006714, 0.0250540, -0.39978, 0.54400];
const POLY4: [f64; 4] = [-0.0020322, 0.0627670, -0.77857, 1.38220];
const POLY5: [f64; 4] = [0.00389150, -0.083751, -0.31082, -1.5861];
const POLY6: [f64; 3] = [0.00303020, -0.082676, -0.48030];
const POLY7: [f64; 2] = [0.459, -2.273];

/// Computes the Shapiro-Wilk W test for normality and its p-value using Royston (1992)
/// polynomial approximations (the standard algorithm used in SciPy and SPSS).
/// Requires sample size 3 <= N <= 5000 and non-zero variance.
pub fn compute_shapiro_wilk(values: &[f64]) -> Result<NormalityTestRow, AlgoError> {
    let n = values.len();
    if !(3..=5000).contains(&n) {
        return Err(AlgoError::InvalidInput(format!(
            "Shapiro-Wilk test requires sample size between 3 and 5000, got {n}"
        )));
    }

    let mut sorted_x = values.to_vec();
    sorted_x.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n_f = n as f64;
    let mean: f64 = sorted_x.iter().sum::<f64>() / n_f;
    let ss: f64 = sorted_x.iter().map(|&x| (x - mean).powi(2)).sum();

    if ss <= 1e-15 || !ss.is_finite() {
        return Err(AlgoError::InvalidInput(
            "Shapiro-Wilk test cannot be computed for constant data (variance is zero)".to_string(),
        ));
    }

    // Step 1: Blom's expected normal order statistics
    let mut m = Vec::with_capacity(n);
    let mut m_sq_sum = 0.0;
    for i in 0..n {
        let u_i = ((i + 1) as f64 - 0.375) / (n_f + 0.25);
        let m_i = inv_normal_cdf(u_i);
        m_sq_sum += m_i * m_i;
        m.push(m_i);
    }

    if m_sq_sum <= 0.0 {
        return Err(AlgoError::ComputationError(
            "Failed to compute normal order statistics for Shapiro-Wilk test".to_string(),
        ));
    }

    let m_norm = m_sq_sum.sqrt();
    let c: Vec<f64> = m.iter().map(|&mi| mi / m_norm).collect();

    // Step 2: Weights a_i
    let mut weights = vec![0.0; n];
    if n == 3 {
        let sqrt_half = 0.5_f64.sqrt();
        weights[0] = -sqrt_half;
        weights[1] = 0.0;
        weights[2] = sqrt_half;
    } else {
        let u = 1.0 / n_f.sqrt();
        let poly1 = [
            -2.706056,
            4.434685,
            -2.071190,
            -0.147981,
            0.221157,
            c[n - 1],
        ];
        weights[n - 1] = poly_eval(&poly1, u);
        weights[0] = -weights[n - 1];

        if n >= 6 {
            let poly2 = [
                -3.582633,
                5.682633,
                -1.752461,
                -0.293762,
                0.042981,
                c[n - 2],
            ];
            weights[n - 2] = poly_eval(&poly2, u);
            weights[1] = -weights[n - 2];

            let phi_num = m_sq_sum - 2.0 * m[n - 1].powi(2) - 2.0 * m[n - 2].powi(2);
            let phi_den = 1.0 - 2.0 * weights[n - 1].powi(2) - 2.0 * weights[n - 2].powi(2);
            let phi = if phi_den > 1e-12 && phi_num > 0.0 {
                phi_num / phi_den
            } else {
                1.0
            };
            let phi_sqrt = phi.sqrt();
            for i in 2..(n - 2) {
                weights[i] = m[i] / phi_sqrt;
            }
        } else {
            // n is 4 or 5
            let phi_num = m_sq_sum - 2.0 * m[n - 1].powi(2);
            let phi_den = 1.0 - 2.0 * weights[n - 1].powi(2);
            let phi = if phi_den > 1e-12 && phi_num > 0.0 {
                phi_num / phi_den
            } else {
                1.0
            };
            let phi_sqrt = phi.sqrt();
            for i in 1..(n - 1) {
                weights[i] = m[i] / phi_sqrt;
            }
        }
    }

    // Step 3: Compute W statistic
    let sum_ax: f64 = weights
        .iter()
        .zip(sorted_x.iter())
        .map(|(&w, &x)| w * x)
        .sum();
    let w_raw = (sum_ax * sum_ax) / ss;
    let w_stat = w_raw.clamp(1e-10, 1.0);

    // Step 4: Compute p-value using Royston's transformations
    let p_val = if n == 3 {
        let pi = std::f64::consts::PI;
        let diff = w_stat.sqrt().asin() - 0.75_f64.sqrt().asin();
        let exact_p = (6.0 / pi) * diff;
        exact_p.clamp(0.0, 1.0)
    } else if n <= 11 {
        let mu = poly_eval(&POLY3, n_f);
        let sigma = poly_eval(&POLY4, n_f).exp();
        let gam = poly_eval(&POLY7, n_f);
        let term = gam - (1.0 - w_stat).max(1e-15).ln();
        let y = -term.max(1e-15).ln();
        let z = (y - mu) / sigma;
        (1.0 - normal_cdf(z)).clamp(0.0, 1.0)
    } else {
        let ln_n = n_f.ln();
        let mu = poly_eval(&POLY5, ln_n);
        let sigma = poly_eval(&POLY6, ln_n).exp();
        let y = (1.0 - w_stat).max(1e-15).ln();
        let z = (y - mu) / sigma;
        (1.0 - normal_cdf(z)).clamp(0.0, 1.0)
    };

    Ok(NormalityTestRow {
        statistic: w_stat,
        df: n,
        sig: p_val,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_cdf() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-6);
        assert!((normal_cdf(1.95996) - 0.975).abs() < 1e-4);
        assert!((normal_cdf(-1.95996) - 0.025).abs() < 1e-4);
        assert!((normal_cdf(1.0) - 0.84134).abs() < 1e-4);
    }

    #[test]
    fn test_shapiro_wilk_n3() {
        // Sample of 3 values
        let data = vec![1.0, 2.0, 3.0];
        let res = compute_shapiro_wilk(&data).unwrap();
        assert_eq!(res.df, 3);
        assert!((res.statistic - 1.0).abs() < 1e-5);
        assert!((res.sig - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_shapiro_wilk_golden_oracle_n10() {
        // Standard sequence: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let res = compute_shapiro_wilk(&data).unwrap();
        assert_eq!(res.df, 10);
        // SciPy scipy.stats.shapiro(range(1, 11)) gives W = 0.970165, p = 0.8924
        assert!((res.statistic - 0.970165).abs() < 1e-4);
        assert!((res.sig - 0.8924).abs() < 1e-2);
    }

    #[test]
    fn test_shapiro_wilk_skewed_data() {
        // Highly skewed data [1, 1, 1, 1, 1, 1, 2, 3, 10, 100]
        let data = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 10.0, 100.0];
        let res = compute_shapiro_wilk(&data).unwrap();
        assert!(res.statistic < 0.6);
        assert!(res.sig < 0.01);
    }

    #[test]
    fn test_shapiro_wilk_constant_data() {
        let data = vec![5.0, 5.0, 5.0, 5.0];
        assert!(compute_shapiro_wilk(&data).is_err());
    }

    #[test]
    fn test_shapiro_wilk_too_small() {
        let data = vec![1.0, 2.0];
        assert!(compute_shapiro_wilk(&data).is_err());
    }
}
