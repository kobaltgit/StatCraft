use super::options::FrequenciesStatisticsOptions;
use super::percentiles::{compute_percentile, compute_percentiles_with_options, PercentileValue};
use serde::{Deserialize, Serialize};

/// Result of univariate statistics computation for a frequency analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FrequenciesStatisticsResult {
    pub mean: Option<f64>,
    pub se_mean: Option<f64>,
    pub median: Option<f64>,
    pub mode: Option<f64>,
    pub has_multiple_modes: bool,
    pub std_dev: Option<f64>,
    pub variance: Option<f64>,
    pub skewness: Option<f64>,
    pub se_skewness: Option<f64>,
    pub kurtosis: Option<f64>,
    pub se_kurtosis: Option<f64>,
    pub range: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub sum: Option<f64>,
    pub percentiles: Vec<PercentileValue>,
}

/// Convenience helper to compute statistics from an unweighted slice of numbers.
pub fn compute_statistics_from_slice(
    values: &[f64],
    options: &FrequenciesStatisticsOptions,
) -> FrequenciesStatisticsResult {
    let pairs: Vec<(f64, f64)> = values.iter().map(|&x| (x, 1.0)).collect();
    compute_frequencies_statistics(&pairs, options)
}

/// Computes the requested univariate statistics for a weighted slice of pairs (value, weight).
/// Observations with non-finite values or non-positive/non-finite weights are ignored.
pub fn compute_frequencies_statistics(
    valid_pairs: &[(f64, f64)],
    options: &FrequenciesStatisticsOptions,
) -> FrequenciesStatisticsResult {
    let mut clean_pairs: Vec<(f64, f64)> = Vec::with_capacity(valid_pairs.len());
    for &(x, w) in valid_pairs {
        if x.is_finite() && w.is_finite() && w > 0.0 {
            clean_pairs.push((x, w));
        }
    }

    if clean_pairs.is_empty() {
        return FrequenciesStatisticsResult::default();
    }

    let w_sum: f64 = clean_pairs.iter().map(|&(_, w)| w).sum();
    if w_sum <= 0.0 {
        return FrequenciesStatisticsResult::default();
    }

    let sum_val: f64 = clean_pairs.iter().map(|&(x, w)| w * x).sum();
    let mean_val = sum_val / w_sum;

    let mut min_val = clean_pairs[0].0;
    let mut max_val = clean_pairs[0].0;
    for &(x, _) in &clean_pairs[1..] {
        if x < min_val {
            min_val = x;
        }
        if x > max_val {
            max_val = x;
        }
    }
    let range_val = max_val - min_val;

    // Variance, Standard Deviation, SE of Mean
    let (internal_var, internal_sd, se_mean_val) = if w_sum >= 2.0 {
        let sum_sq_diff: f64 = clean_pairs
            .iter()
            .map(|&(x, w)| w * (x - mean_val).powi(2))
            .sum();
        let var = sum_sq_diff / (w_sum - 1.0);
        let sd = if var >= 0.0 { var.sqrt() } else { 0.0 };
        let se = sd / w_sum.sqrt();
        (Some(var), Some(sd), Some(se))
    } else {
        (None, None, None)
    };

    // Skewness and SE Skewness (SPSS standard formula)
    let (skewness_val, se_skewness_val) = if w_sum >= 3.0 {
        let sd = match internal_sd {
            Some(s) if s > 0.0 => s,
            _ => 0.0,
        };

        if sd == 0.0 {
            (Some(0.0), None)
        } else {
            let sum_cubed_diff: f64 = clean_pairs
                .iter()
                .map(|&(x, w)| w * (x - mean_val).powi(3))
                .sum();
            let factor = w_sum / ((w_sum - 1.0) * (w_sum - 2.0));
            let skew = factor * (sum_cubed_diff / sd.powi(3));

            let se_skew_var =
                (6.0 * w_sum * (w_sum - 1.0)) / ((w_sum - 2.0) * (w_sum + 1.0) * (w_sum + 3.0));
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

    // Kurtosis and SE Kurtosis (SPSS excess kurtosis formula)
    let (kurtosis_val, se_kurtosis_val) = if w_sum >= 4.0 {
        let sd = match internal_sd {
            Some(s) if s > 0.0 => s,
            _ => 0.0,
        };

        if sd == 0.0 {
            (Some(0.0), None)
        } else {
            let m2 = clean_pairs
                .iter()
                .map(|&(x, w)| w * (x - mean_val).powi(2))
                .sum::<f64>()
                / w_sum;
            let m4 = clean_pairs
                .iter()
                .map(|&(x, w)| w * (x - mean_val).powi(4))
                .sum::<f64>()
                / w_sum;

            if m2 > 0.0 {
                let term1 = (w_sum + 1.0) * (m4 / (m2 * m2));
                let term2 = 3.0 * (w_sum - 1.0);
                let multiplier = (w_sum - 1.0) / ((w_sum - 2.0) * (w_sum - 3.0));
                let kurt = multiplier * (term1 - term2);

                let se_kurt = se_skewness_val.and_then(|se_s| {
                    let se_kurt_factor = (w_sum * w_sum - 1.0) / ((w_sum - 3.0) * (w_sum + 5.0));
                    if se_kurt_factor > 0.0 {
                        Some(2.0 * se_s * se_kurt_factor.sqrt())
                    } else {
                        None
                    }
                });
                (Some(kurt), se_kurt)
            } else {
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    // Mode and Multiple Modes Flag (SPSS tie-breaker rule)
    clean_pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut unique_vals: Vec<(f64, f64)> = Vec::new();
    for &(x, w) in &clean_pairs {
        if let Some(last) = unique_vals.last_mut() {
            if (last.0 - x).abs() < 1e-9 {
                last.1 += w;
                continue;
            }
        }
        unique_vals.push((x, w));
    }

    let mut max_freq = 0.0_f64;
    for &(_, w) in &unique_vals {
        if w > max_freq {
            max_freq = w;
        }
    }

    let mut modes: Vec<f64> = Vec::new();
    if max_freq > 0.0 {
        for &(x, w) in &unique_vals {
            if (w - max_freq).abs() < 1e-9 {
                modes.push(x);
            }
        }
    }

    let (mode_val, multiple_modes) = if modes.is_empty() {
        (None, false)
    } else if modes.len() == 1 {
        (Some(modes[0]), false)
    } else {
        // SPSS rule: smallest numeric value is chosen, multiple modes flag is true
        modes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        (Some(modes[0]), true)
    };

    // Build sorted replicated numbers for percentiles and median
    let mut sorted_replicated: Vec<f64> = Vec::new();
    for &(x, w) in &clean_pairs {
        let rep = w.round() as usize;
        for _ in 0..rep.max(1) {
            sorted_replicated.push(x);
        }
    }
    sorted_replicated.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let median_val = if options.median {
        compute_percentile(&sorted_replicated, 0.50)
    } else {
        None
    };

    let percentiles = if options.percentiles.any_requested() {
        compute_percentiles_with_options(
            &sorted_replicated,
            &options.percentiles,
            options.group_midpoints,
        )
    } else {
        Vec::new()
    };

    FrequenciesStatisticsResult {
        mean: if options.mean { Some(mean_val) } else { None },
        se_mean: if options.se_mean { se_mean_val } else { None },
        median: median_val,
        mode: if options.mode { mode_val } else { None },
        has_multiple_modes: if options.mode { multiple_modes } else { false },
        std_dev: if options.std_dev { internal_sd } else { None },
        variance: if options.variance { internal_var } else { None },
        skewness: if options.skewness { skewness_val } else { None },
        se_skewness: if options.skewness {
            se_skewness_val
        } else {
            None
        },
        kurtosis: if options.kurtosis { kurtosis_val } else { None },
        se_kurtosis: if options.kurtosis {
            se_kurtosis_val
        } else {
            None
        },
        range: if options.range { Some(range_val) } else { None },
        min: if options.min { Some(min_val) } else { None },
        max: if options.max { Some(max_val) } else { None },
        sum: if options.sum { Some(sum_val) } else { None },
        percentiles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_tie_breaker_spss_canon() {
        // Sample: 10, 10, 20, 20, 30
        // Both 10 and 20 have frequency 2.
        // SPSS canon: smallest value (10.0) is reported as mode, with has_multiple_modes = true.
        let values = vec![20.0, 10.0, 30.0, 20.0, 10.0];
        let opts = FrequenciesStatisticsOptions {
            mode: true,
            ..Default::default()
        };

        let res = compute_statistics_from_slice(&values, &opts);
        assert_eq!(res.mode, Some(10.0));
        assert!(res.has_multiple_modes);
    }

    #[test]
    fn test_unique_mode() {
        // Sample: 5, 10, 10, 15
        let values = vec![5.0, 10.0, 10.0, 15.0];
        let opts = FrequenciesStatisticsOptions {
            mode: true,
            ..Default::default()
        };

        let res = compute_statistics_from_slice(&values, &opts);
        assert_eq!(res.mode, Some(10.0));
        assert!(!res.has_multiple_modes);
    }

    #[test]
    fn test_all_statistics_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let opts = FrequenciesStatisticsOptions::all();
        let res = compute_statistics_from_slice(&values, &opts);

        assert_eq!(res.mean, Some(3.0));
        assert_eq!(res.min, Some(1.0));
        assert_eq!(res.max, Some(5.0));
        assert_eq!(res.range, Some(4.0));
        assert_eq!(res.sum, Some(15.0));
        assert_eq!(res.median, Some(3.0));
        assert!((res.variance.unwrap() - 2.5).abs() < 1e-6);
        assert!((res.std_dev.unwrap() - 2.5_f64.sqrt()).abs() < 1e-6);
        assert!((res.skewness.unwrap() - 0.0).abs() < 1e-6);
    }
}
