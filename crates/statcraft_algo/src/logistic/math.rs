use crate::crosstabs::chi_square_sf;
use crate::rank_cases::inv_normal_cdf;

/// Numerically stable sigmoid function.
pub fn sigmoid(eta: f64) -> f64 {
    let p = if eta >= 0.0 {
        1.0 / (1.0 + (-eta).exp())
    } else {
        let e = eta.exp();
        e / (1.0 + e)
    };
    p.clamp(1e-15, 1.0 - 1e-15)
}

/// Computes critical two-sided normal z value for a given confidence level (e.g. 0.95 -> 1.95996).
pub fn normal_critical_value(confidence_level: f64) -> f64 {
    let alpha = 1.0 - confidence_level.clamp(0.5, 0.9999);
    let p = 1.0 - alpha * 0.5;
    inv_normal_cdf(p).abs()
}

/// Survival function (p-value) for Chi-Square distribution with degrees of freedom df.
pub fn chisq_p_value(stat: f64, df: usize) -> f64 {
    if df == 0 || !stat.is_finite() || stat <= 0.0 {
        return 1.0;
    }
    chi_square_sf(stat, df).clamp(0.0, 1.0)
}
