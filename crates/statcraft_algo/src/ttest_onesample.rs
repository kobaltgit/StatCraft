use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneSampleTTestOptions {
    pub test_value: f64,
    pub confidence_level: f64,
}

impl Default for OneSampleTTestOptions {
    fn default() -> Self {
        Self {
            test_value: 0.0,
            confidence_level: 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OneSampleTTestResult {
    pub variable: String,
    pub n_valid: usize,
    pub n_missing: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub se_mean: Option<f64>,
    pub test_value: f64,
    pub t_stat: Option<f64>,
    pub df: Option<f64>,
    pub sig_2tailed: Option<f64>,
    pub mean_diff: Option<f64>,
    pub ci_level: f64,
    pub ci_lower: Option<f64>,
    pub ci_upper: Option<f64>,
}

/// Natural logarithm of the Gamma function via Lanczos approximation (g=7, n=9).
/// Accurate to ~15 decimal digits for all positive z.
#[allow(clippy::excessive_precision)]
pub fn ln_gamma(z: f64) -> f64 {
    let p = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];

    if z < 0.5 {
        // Reflection formula: Gamma(1 - z) * Gamma(z) = pi / sin(pi * z)
        let pi = std::f64::consts::PI;
        return (pi / (pi * z).sin()).ln() - ln_gamma(1.0 - z);
    }

    let z_adj = z - 1.0;
    let mut x = p[0];
    for (i, &coeff) in p.iter().enumerate().skip(1) {
        x += coeff / (z_adj + i as f64);
    }
    let t = z_adj + 7.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z_adj + 0.5) * t.ln() - t + x.ln()
}

/// Natural logarithm of the Beta function B(a, b).
pub fn ln_beta(a: f64, b: f64) -> f64 {
    ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)
}

/// Regularized incomplete beta function I_x(a, b) using continued fractions (Lentz's method).
pub fn regularized_incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Symmetry transformation if x > (a + 1)/(a + b + 2)
    if x > (a + 1.0) / (a + b + 2.0) {
        return 1.0 - regularized_incomplete_beta(1.0 - x, b, a);
    }

    let front = (a * x.ln() + b * (1.0 - x).ln() - ln_beta(a, b) - a.ln()).exp();

    // Continued fraction evaluation via modified Lentz's method
    const TINY: f64 = 1.0e-30;
    const MAX_ITER: usize = 200;
    const EPS: f64 = 1.0e-15;

    let mut f = 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < TINY {
        d = TINY;
    }
    d = 1.0 / d;
    f *= d;

    for m in 1..=MAX_ITER {
        let mf = m as f64;

        // Even step: 2m
        let num_even = mf * (b - mf) * x / ((a + 2.0 * mf - 1.0) * (a + 2.0 * mf));
        d = 1.0 + num_even * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + num_even / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        f *= c * d;

        // Odd step: 2m + 1
        let num_odd = -(a + mf) * (a + b + mf) * x / ((a + 2.0 * mf) * (a + 2.0 * mf + 1.0));
        d = 1.0 + num_odd * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + num_odd / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;

        if (delta - 1.0).abs() < EPS {
            break;
        }
    }

    (front * f).clamp(0.0, 1.0)
}

/// Computes the two-tailed significance (p-value) for Student's t distribution with `df` degrees of freedom.
pub fn student_t_sf_2tailed(t: f64, df: f64) -> f64 {
    if df <= 0.0 || !t.is_finite() {
        return 1.0;
    }
    if t == 0.0 {
        return 1.0;
    }
    let t_sq = t * t;
    let x = df / (df + t_sq);
    regularized_incomplete_beta(x, df * 0.5, 0.5)
}

/// Cumulative distribution function for Student's t distribution.
pub fn student_t_cdf(t: f64, df: f64) -> f64 {
    if df <= 0.0 {
        return 0.5;
    }
    if t == 0.0 {
        return 0.5;
    }
    let p_2tailed = student_t_sf_2tailed(t, df);
    if t > 0.0 {
        1.0 - 0.5 * p_2tailed
    } else {
        0.5 * p_2tailed
    }
}

/// Probability density function for Student's t distribution.
pub fn student_t_pdf(t: f64, df: f64) -> f64 {
    if df <= 0.0 {
        return 0.0;
    }
    let pi = std::f64::consts::PI;
    let log_val = ln_gamma((df + 1.0) * 0.5)
        - ln_gamma(df * 0.5)
        - 0.5 * (pi * df).ln()
        - 0.5 * (df + 1.0) * (1.0 + t * t / df).ln();
    log_val.exp()
}

/// Rational approximation for standard normal inverse CDF (Acklam's algorithm).
/// Accurate to ~1.15e-9.
#[allow(clippy::excessive_precision)]
fn standard_normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    // Coefficients in rational approximations
    const A: [f64; 6] = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];

    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

/// Computes the inverse CDF (quantile function) of Student's t distribution.
/// Uses normal quantile + Cornish-Fisher expansion followed by Newton-Raphson refinement.
pub fn student_t_inv_cdf(p: f64, df: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if (p - 0.5).abs() < 1e-15 {
        return 0.0;
    }

    let z = standard_normal_quantile(p);
    // Initial guess from Cornish-Fisher expansion for Student's t
    let z2 = z * z;
    let mut t = z
        + (z * z2 + z) / (4.0 * df)
        + (5.0 * z * z2 * z2 + 16.0 * z * z2 + 3.0 * z) / (96.0 * df * df);

    // Refine with Newton-Raphson iterations
    for _ in 0..10 {
        let pdf = student_t_pdf(t, df);
        if pdf <= 1e-30 {
            break;
        }
        let cdf = student_t_cdf(t, df);
        let diff = cdf - p;
        if diff.abs() < 1e-13 {
            break;
        }
        let step = diff / pdf;
        t -= step;
        if step.abs() < 1e-12 {
            break;
        }
    }

    t
}

/// Computes One-Sample T-Test for a variable with optional floating-point values.
/// Missing, NaN, or infinite values are excluded listwise.
pub fn compute_onesample_ttest(
    var_name: &str,
    values: &[Option<f64>],
    options: &OneSampleTTestOptions,
) -> OneSampleTTestResult {
    let mut valid_values = Vec::with_capacity(values.len());
    let mut n_missing = 0_usize;

    for v in values {
        match v {
            Some(x) if x.is_finite() => valid_values.push(*x),
            _ => n_missing += 1,
        }
    }

    let n = valid_values.len();
    let test_val = options.test_value;
    let ci_level = if options.confidence_level > 0.0 && options.confidence_level < 1.0 {
        options.confidence_level
    } else {
        0.95
    };

    if n == 0 {
        return OneSampleTTestResult {
            variable: var_name.to_string(),
            n_valid: 0,
            n_missing,
            mean: None,
            std_dev: None,
            se_mean: None,
            test_value: test_val,
            t_stat: None,
            df: None,
            sig_2tailed: None,
            mean_diff: None,
            ci_level,
            ci_lower: None,
            ci_upper: None,
        };
    }

    let sum: f64 = valid_values.iter().sum();
    let mean = sum / (n as f64);
    let mean_diff = mean - test_val;

    if n < 2 {
        return OneSampleTTestResult {
            variable: var_name.to_string(),
            n_valid: 1,
            n_missing,
            mean: Some(mean),
            std_dev: None,
            se_mean: None,
            test_value: test_val,
            t_stat: None,
            df: None,
            sig_2tailed: None,
            mean_diff: Some(mean_diff),
            ci_level,
            ci_lower: None,
            ci_upper: None,
        };
    }

    let df = (n - 1) as f64;
    let sum_sq_diff: f64 = valid_values.iter().map(|&x| (x - mean).powi(2)).sum();
    let variance = sum_sq_diff / df;
    let std_dev = variance.sqrt();
    let se_mean = std_dev / (n as f64).sqrt();

    // Degenerate variance edge cases
    if std_dev == 0.0 || se_mean == 0.0 {
        let (t_stat, sig, lower, upper) = if mean_diff.abs() < 1e-12 {
            (Some(0.0), Some(1.0), Some(0.0), Some(0.0))
        } else {
            (None, None, None, None)
        };
        return OneSampleTTestResult {
            variable: var_name.to_string(),
            n_valid: n,
            n_missing,
            mean: Some(mean),
            std_dev: Some(std_dev),
            se_mean: Some(se_mean),
            test_value: test_val,
            t_stat,
            df: Some(df),
            sig_2tailed: sig,
            mean_diff: Some(mean_diff),
            ci_level,
            ci_lower: lower,
            ci_upper: upper,
        };
    }

    let t_stat = mean_diff / se_mean;
    let sig_2tailed = student_t_sf_2tailed(t_stat, df);

    // Confidence interval for the mean difference
    let alpha = 1.0 - ci_level;
    let t_crit = student_t_inv_cdf(1.0 - alpha * 0.5, df);
    let margin = t_crit * se_mean;
    let ci_lower = mean_diff - margin;
    let ci_upper = mean_diff + margin;

    OneSampleTTestResult {
        variable: var_name.to_string(),
        n_valid: n,
        n_missing,
        mean: Some(mean),
        std_dev: Some(std_dev),
        se_mean: Some(se_mean),
        test_value: test_val,
        t_stat: Some(t_stat),
        df: Some(df),
        sig_2tailed: Some(sig_2tailed),
        mean_diff: Some(mean_diff),
        ci_level,
        ci_lower: Some(ci_lower),
        ci_upper: Some(ci_upper),
    }
}

#[cfg(test)]
#[path = "ttest_onesample_tests.rs"]
mod tests;
