use super::{BootstrapCiMethod, BootstrapError};
use crate::rank_cases::inv_normal_cdf;
use serde::{Deserialize, Serialize};

/// Bootstrap confidence interval result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BootstrapCi {
    pub lower: f64,
    pub upper: f64,
    pub level: f64,
    pub method: BootstrapCiMethod,
}

/// Complete bootstrap summary for a scalar statistic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BootstrapSummary {
    pub estimate: f64,
    pub bias: f64,
    pub std_error: f64,
    pub ci: BootstrapCi,
    pub valid_samples: usize,
}

/// High-precision standard normal CDF $\Phi(z)$ using Cody's rational approximation
/// for $\operatorname{erfc}$ (relative accuracy $\approx 10^{-15}$).
pub fn normal_cdf(z: f64) -> f64 {
    if z.is_nan() {
        return f64::NAN;
    }
    if z <= -38.0 {
        return 0.0;
    }
    if z >= 38.0 {
        return 1.0;
    }
    0.5 * erfc_cody(-z * std::f64::consts::FRAC_1_SQRT_2)
}

/// Complementary error function $\operatorname{erfc}(x)$ using Cody's rational Chebyshev approximation.
fn erfc_cody(x: f64) -> f64 {
    const THRESHOLD: f64 = 0.46875;
    let abs_x = x.abs();

    let res = if abs_x <= THRESHOLD {
        const A: [f64; 5] = [
            3.161_123_743_870_565_5e0,
            1.138_641_541_510_501_6e2,
            3.774_852_376_853_02e2,
            3.209_377_589_138_469_5e3,
            1.857_777_061_846_031_5e-1,
        ];
        const B: [f64; 4] = [
            2.360_129_095_234_412_1e1,
            2.440_246_379_344_441_7e2,
            1.282_616_526_077_372_3e3,
            2.844_236_833_439_171e3,
        ];
        let xsq = abs_x * abs_x;
        let num = (((A[4] * xsq + A[0]) * xsq + A[1]) * xsq + A[2]) * xsq + A[3];
        let den = (((xsq + B[0]) * xsq + B[1]) * xsq + B[2]) * xsq + B[3];
        let erf_abs = abs_x * (num / den);
        1.0 - if x >= 0.0 { erf_abs } else { -erf_abs }
    } else if abs_x <= 4.0 {
        const C: [f64; 9] = [
            5.641_884_969_886_701e-1,
            8.883_149_794_388_376e0,
            6.611_919_063_714_163e1,
            2.986_351_381_974_001e2,
            8.819_522_212_417_69e2,
            1.712_047_612_634_071e3,
            2.051_078_377_826_071e3,
            1.230_339_354_797_997e3,
            2.153_115_354_744_038e-8,
        ];
        const D: [f64; 8] = [
            1.574_492_611_070_983_5e1,
            1.176_939_508_913_125e2,
            5.371_811_018_620_099e2,
            1.621_389_574_566_69e3,
            3.290_799_235_733_459_6e3,
            4.362_619_090_143_247e3,
            3.439_367_674_143_722e3,
            1.230_339_354_803_749_4e3,
        ];
        let mut xnum = C[8] * abs_x;
        let mut xden = abs_x;
        for i in 0..7 {
            xnum = (xnum + C[i]) * abs_x;
            xden = (xden + D[i]) * abs_x;
        }
        let temp = (xnum + C[7]) / (xden + D[7]);
        let erfc_abs = temp * (-abs_x * abs_x).exp();
        if x >= 0.0 {
            erfc_abs
        } else {
            2.0 - erfc_abs
        }
    } else {
        const P: [f64; 6] = [
            3.053_266_349_612_323_4e-1,
            3.603_448_999_498_044e-1,
            1.257_817_261_112_292_5e-1,
            1.608_378_514_874_228e-2,
            6.587_491_615_298_378e-4,
            1.631_538_713_730_209_8e-2,
        ];
        const Q: [f64; 5] = [
            2.568_520_192_289_822e0,
            1.872_952_849_923_460_4e0,
            5.279_051_029_514_284e-1,
            6.051_834_131_244_132e-2,
            2.335_204_976_268_692e-3,
        ];
        let xsq = 1.0 / (abs_x * abs_x);
        let mut xnum = P[5] * xsq;
        let mut xden = xsq;
        for i in 0..4 {
            xnum = (xnum + P[i]) * xsq;
            xden = (xden + Q[i]) * xsq;
        }
        let temp = xsq * (xnum + P[4]) / (xden + Q[4]);
        let inv_sqrt_pi = std::f64::consts::FRAC_2_SQRT_PI * 0.5;
        let erfc_abs = (-abs_x * abs_x).exp() * ((inv_sqrt_pi - temp) / abs_x);
        if x >= 0.0 {
            erfc_abs
        } else {
            2.0 - erfc_abs
        }
    };

    res.clamp(0.0, 2.0)
}

/// Linearly interpolates the quantile at probability `p` in `[0, 1]` from a sorted slice.
fn sorted_quantile(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return sorted[0];
    }
    let p_clamped = p.clamp(0.0, 1.0);
    let pos = p_clamped * (n - 1) as f64;
    let lo = (pos.floor() as usize).min(n - 1);
    let hi = (pos.ceil() as usize).min(n - 1);
    let frac = pos - lo as f64;
    sorted[lo] + frac * (sorted[hi] - sorted[lo])
}

/// Computes the Percentile bootstrap confidence interval from sorted replicates.
pub fn compute_percentile_ci(
    sorted_replicates: &[f64],
    ci_level: f64,
) -> Result<BootstrapCi, BootstrapError> {
    if sorted_replicates.len() < 2 {
        return Err(BootstrapError::InsufficientValidReplicates {
            valid: sorted_replicates.len(),
        });
    }
    if !ci_level.is_finite() || ci_level <= 0.0 || ci_level >= 100.0 {
        return Err(BootstrapError::InvalidCiLevel { ci_level });
    }

    let alpha = 1.0 - ci_level / 100.0;
    let lower = sorted_quantile(sorted_replicates, alpha * 0.5);
    let upper = sorted_quantile(sorted_replicates, 1.0 - alpha * 0.5);

    Ok(BootstrapCi {
        lower,
        upper,
        level: ci_level,
        method: BootstrapCiMethod::Percentile,
    })
}

/// Computes the BCa (Bias-Corrected and Accelerated) bootstrap confidence interval
/// per Efron & Tibshirani (1993).
pub fn compute_bca_ci(
    estimate: f64,
    sorted_replicates: &[f64],
    jackknife: Option<&[f64]>,
    ci_level: f64,
) -> Result<BootstrapCi, BootstrapError> {
    if sorted_replicates.len() < 2 {
        return Err(BootstrapError::InsufficientValidReplicates {
            valid: sorted_replicates.len(),
        });
    }
    if !ci_level.is_finite() || ci_level <= 0.0 || ci_level >= 100.0 {
        return Err(BootstrapError::InvalidCiLevel { ci_level });
    }
    if !estimate.is_finite() {
        return Err(BootstrapError::InvalidPointEstimate);
    }

    let b = sorted_replicates.len() as f64;
    let less_count = sorted_replicates.iter().filter(|&&v| v < estimate).count() as f64;
    let eq_count = sorted_replicates
        .iter()
        .filter(|&&v| (v - estimate) == 0.0)
        .count() as f64;
    let prop = ((less_count + 0.5 * eq_count) / b).clamp(1e-12, 1.0 - 1e-12);
    let z0 = inv_normal_cdf(prop);

    let a_hat = match jackknife {
        Some(jk) if jk.len() >= 2 => {
            let valid_jk: Vec<f64> = jk.iter().copied().filter(|v| v.is_finite()).collect();
            if valid_jk.len() < 2 {
                0.0
            } else {
                let jk_mean = valid_jk.iter().sum::<f64>() / valid_jk.len() as f64;
                let mut sum_sq = 0.0;
                let mut sum_cu = 0.0;
                for &v in &valid_jk {
                    let d = jk_mean - v;
                    let d2 = d * d;
                    sum_sq += d2;
                    sum_cu += d2 * d;
                }
                let den = 6.0 * sum_sq.powf(1.5);
                if den.abs() < 1e-15 {
                    0.0
                } else {
                    sum_cu / den
                }
            }
        }
        _ => 0.0,
    };

    let alpha = 1.0 - ci_level / 100.0;
    let z_low = inv_normal_cdf(alpha * 0.5);
    let z_high = inv_normal_cdf(1.0 - alpha * 0.5);

    let adjust = |z_q: f64| -> f64 {
        let num = z0 + z_q;
        let denom = 1.0 - a_hat * num;
        if denom.abs() < 1e-12 {
            if num >= 0.0 {
                1.0
            } else {
                0.0
            }
        } else {
            normal_cdf(z0 + num / denom)
        }
    };

    let alpha_1 = adjust(z_low);
    let alpha_2 = adjust(z_high);
    let (p_lo, p_hi) = if alpha_1 <= alpha_2 {
        (alpha_1, alpha_2)
    } else {
        (alpha_2, alpha_1)
    };

    let lower = sorted_quantile(sorted_replicates, p_lo);
    let upper = sorted_quantile(sorted_replicates, p_hi);

    Ok(BootstrapCi {
        lower,
        upper,
        level: ci_level,
        method: BootstrapCiMethod::Bca,
    })
}

/// Summarizes bootstrap replicates into bias, standard error, and confidence interval.
pub fn summarize_bootstrap(
    estimate: f64,
    replicates: &[f64],
    jackknife: Option<&[f64]>,
    ci_level: f64,
    ci_method: BootstrapCiMethod,
) -> Result<BootstrapSummary, BootstrapError> {
    if !estimate.is_finite() {
        return Err(BootstrapError::InvalidPointEstimate);
    }

    let mut sorted: Vec<f64> = replicates
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    if sorted.len() < 2 {
        return Err(BootstrapError::InsufficientValidReplicates {
            valid: sorted.len(),
        });
    }
    sorted.sort_by(f64::total_cmp);

    let b = sorted.len() as f64;
    let mean_rep = sorted.iter().sum::<f64>() / b;
    let bias = mean_rep - estimate;
    let var_rep = sorted
        .iter()
        .map(|&v| {
            let d = v - mean_rep;
            d * d
        })
        .sum::<f64>()
        / (b - 1.0);
    let std_error = var_rep.max(0.0).sqrt();

    let ci = match ci_method {
        BootstrapCiMethod::Percentile => compute_percentile_ci(&sorted, ci_level)?,
        BootstrapCiMethod::Bca => compute_bca_ci(estimate, &sorted, jackknife, ci_level)?,
    };

    Ok(BootstrapSummary {
        estimate,
        bias,
        std_error,
        ci,
        valid_samples: sorted.len(),
    })
}
