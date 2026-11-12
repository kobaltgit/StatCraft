use crate::frequencies::compute_percentile;
use crate::ttest_onesample::student_t_inv_cdf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreDescriptives {
    pub n_valid: usize,
    pub n_missing: usize,
    pub mean: Option<f64>,
    pub mean_se: Option<f64>,
    pub mean_ci_lower: Option<f64>,
    pub mean_ci_upper: Option<f64>,
    pub ci_level: f64,
    pub trimmed_mean_5pct: Option<f64>,
    pub median: Option<f64>,
    pub variance: Option<f64>,
    pub std_dev: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub range: Option<f64>,
    pub iqr: Option<f64>,
    pub q1: Option<f64>,
    pub q3: Option<f64>,
    pub skewness: Option<f64>,
    pub skewness_se: Option<f64>,
    pub kurtosis: Option<f64>,
    pub kurtosis_se: Option<f64>,
}

/// Computes the 5% trimmed mean per SPSS 24 Algorithms Guide.
/// Trims g = 0.05 * N observations from each end.
/// If g = k + r, drops k observations on each end, weights observation
/// k (0-based) and N - 1 - k by (1 - r), and intermediate observations by 1.0.
pub fn compute_5pct_trimmed_mean(sorted_values: &[f64]) -> Option<f64> {
    let n = sorted_values.len();
    if n == 0 {
        return None;
    }
    if n == 1 {
        return Some(sorted_values[0]);
    }

    let g = 0.05 * (n as f64);
    let k = g.floor() as usize;
    let r = g - (k as f64);

    let mut weight_sum = 0.0_f64;
    let mut weighted_val_sum = 0.0_f64;

    for (i, &val) in sorted_values.iter().enumerate() {
        let w = if i < k || i >= n - k {
            0.0
        } else if i == k || i == n - 1 - k {
            1.0 - r
        } else {
            1.0
        };

        if w > 0.0 {
            weight_sum += w;
            weighted_val_sum += w * val;
        }
    }

    if weight_sum > 0.0 {
        Some(weighted_val_sum / weight_sum)
    } else {
        None
    }
}

/// Computes all canonical SPSS Explore descriptive statistics for a slice of valid numbers.
pub fn compute_explore_descriptives(
    valid_values: &[f64],
    n_missing: usize,
    confidence_level: f64,
) -> ExploreDescriptives {
    let n = valid_values.len();
    if n == 0 {
        return ExploreDescriptives {
            n_valid: 0,
            n_missing,
            mean: None,
            mean_se: None,
            mean_ci_lower: None,
            mean_ci_upper: None,
            ci_level: confidence_level,
            trimmed_mean_5pct: None,
            median: None,
            variance: None,
            std_dev: None,
            min: None,
            max: None,
            range: None,
            iqr: None,
            q1: None,
            q3: None,
            skewness: None,
            skewness_se: None,
            kurtosis: None,
            kurtosis_se: None,
        };
    }

    let mut sorted = valid_values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n_f = n as f64;
    let sum: f64 = sorted.iter().sum();
    let mean = sum / n_f;

    let min = sorted[0];
    let max = sorted[n - 1];
    let range = max - min;

    let trimmed_mean_5pct = compute_5pct_trimmed_mean(&sorted);

    // Median
    let median = if n % 2 == 1 {
        Some(sorted[n / 2])
    } else {
        Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
    };

    // Quartiles and IQR using SPSS HAVERAGE
    let q1 = compute_percentile(&sorted, 0.25);
    let q3 = compute_percentile(&sorted, 0.75);
    let iqr = match (q1, q3) {
        (Some(q1_val), Some(q3_val)) => Some(q3_val - q1_val),
        _ => None,
    };

    // Variance, SD, SE Mean, and CI
    let (variance, std_dev, mean_se, mean_ci_lower, mean_ci_upper) = if n >= 2 {
        let sum_sq_diff: f64 = sorted.iter().map(|&x| (x - mean).powi(2)).sum();
        let var = sum_sq_diff / (n_f - 1.0);
        let sd = var.sqrt();
        let se = sd / n_f.sqrt();

        let alpha = 1.0 - confidence_level;
        let t_crit = student_t_inv_cdf(1.0 - alpha * 0.5, n_f - 1.0);
        let ci_low = mean - t_crit * se;
        let ci_high = mean + t_crit * se;

        (Some(var), Some(sd), Some(se), Some(ci_low), Some(ci_high))
    } else {
        (None, None, None, None, None)
    };

    // Skewness
    let (skewness, skewness_se) = if n >= 3 {
        let sd = std_dev.unwrap_or(0.0);
        if sd == 0.0 {
            (Some(0.0), None)
        } else {
            let m3: f64 = sorted.iter().map(|&x| (x - mean).powi(3)).sum();
            let factor = n_f / ((n_f - 1.0) * (n_f - 2.0));
            let skew = factor * (m3 / sd.powi(3));

            let se_skew_var = (6.0 * n_f * (n_f - 1.0)) / ((n_f - 2.0) * (n_f + 1.0) * (n_f + 3.0));
            let se_skew = if se_skew_var > 0.0 {
                Some(se_skew_var.sqrt())
            } else {
                None
            };
            (Some(skew), se_skew)
        }
    } else {
        (None, None)
    };

    // Kurtosis
    let (kurtosis, kurtosis_se) = if n >= 4 {
        let sd = std_dev.unwrap_or(0.0);
        if sd == 0.0 {
            (Some(0.0), None)
        } else {
            let m2: f64 = sorted.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n_f;
            let m4: f64 = sorted.iter().map(|&x| (x - mean).powi(4)).sum::<f64>() / n_f;

            if m2 > 0.0 {
                let term1 = (n_f + 1.0) * (m4 / (m2 * m2));
                let term2 = 3.0 * (n_f - 1.0);
                let mult = (n_f - 1.0) / ((n_f - 2.0) * (n_f - 3.0));
                let kurt = mult * (term1 - term2);

                let se_kurt = skewness_se.and_then(|se_s| {
                    let se_kurt_factor = (n_f * n_f - 1.0) / ((n_f - 3.0) * (n_f + 5.0));
                    if se_kurt_factor > 0.0 {
                        Some(2.0 * se_s * se_kurt_factor.sqrt())
                    } else {
                        None
                    }
                });

                (Some(kurt), se_kurt)
            } else {
                (Some(0.0), None)
            }
        }
    } else {
        (None, None)
    };

    ExploreDescriptives {
        n_valid: n,
        n_missing,
        mean: Some(mean),
        mean_se,
        mean_ci_lower,
        mean_ci_upper,
        ci_level: confidence_level,
        trimmed_mean_5pct,
        median,
        variance,
        std_dev,
        min: Some(min),
        max: Some(max),
        range: Some(range),
        iqr,
        q1,
        q3,
        skewness,
        skewness_se,
        kurtosis,
        kurtosis_se,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_5pct_trimmed_mean() {
        // N = 20, g = 1.0, k = 1, r = 0. Drops x[0] and x[19], weights x[1]..x[18] with 1.0
        let data: Vec<f64> = (1..=20).map(|x| x as f64).collect();
        let tm = compute_5pct_trimmed_mean(&data).unwrap();
        // sum(2..=19) = 189 / 18 = 10.5
        assert!((tm - 10.5).abs() < 1e-6);
    }

    #[test]
    fn test_5pct_trimmed_mean_fractional() {
        // N = 10, g = 0.5, k = 0, r = 0.5
        // x[0] has weight 0.5, x[9] has weight 0.5, x[1]..x[8] have weight 1.0
        let data: Vec<f64> = (1..=10).map(|x| x as f64).collect();
        let tm = compute_5pct_trimmed_mean(&data).unwrap();
        // weights: 0.5 * 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 0.5 * 10 = 0.5 + 44 + 5 = 49.5
        // weight_sum = 9.0 -> 49.5 / 9.0 = 5.5
        assert!((tm - 5.5).abs() < 1e-6);
    }

    #[test]
    fn test_explore_descriptives_values() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let desc = compute_explore_descriptives(&data, 2, 0.95);

        assert_eq!(desc.n_valid, 5);
        assert_eq!(desc.n_missing, 2);
        assert_eq!(desc.mean, Some(30.0));
        assert_eq!(desc.median, Some(30.0));
        assert_eq!(desc.min, Some(10.0));
        assert_eq!(desc.max, Some(50.0));
        assert_eq!(desc.range, Some(40.0));
        assert!(desc.mean_ci_lower.unwrap() < 30.0);
        assert!(desc.mean_ci_upper.unwrap() > 30.0);
        assert!(desc.iqr.is_some());
    }

    #[test]
    fn test_explore_descriptives_empty() {
        let desc = compute_explore_descriptives(&[], 5, 0.95);
        assert_eq!(desc.n_valid, 0);
        assert_eq!(desc.n_missing, 5);
        assert!(desc.mean.is_none());
    }
}
