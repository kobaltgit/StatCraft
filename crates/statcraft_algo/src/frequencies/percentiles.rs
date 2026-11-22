use super::options::PercentilesOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PercentileValue {
    pub percentile: f64,
    pub value: f64,
}

impl PercentileValue {
    pub fn new(percentile: f64, value: f64) -> Self {
        Self { percentile, value }
    }
}

/// Computes quartiles (Q1 = 25%, Q2 = 50% / Median, Q3 = 75%)
/// using the standard SPSS HAVERAGE (Weibull / Type 6) method: rank = p * (n + 1).
pub fn compute_quartiles(sorted_numeric_values: &[f64]) -> Option<(f64, f64, f64)> {
    if sorted_numeric_values.is_empty() {
        return None;
    }
    let q1 = compute_percentile(sorted_numeric_values, 0.25)?;
    let q2 = compute_percentile(sorted_numeric_values, 0.50)?;
    let q3 = compute_percentile(sorted_numeric_values, 0.75)?;
    Some((q1, q2, q3))
}

/// Computes a percentile using the SPSS HAVERAGE interpolation method:
/// rank = p * (n + 1) for p in [0.0, 1.0].
pub fn compute_percentile(sorted_values: &[f64], p: f64) -> Option<f64> {
    let n = sorted_values.len();
    if n == 0 {
        return None;
    }
    if n == 1 {
        return Some(sorted_values[0]);
    }

    let r = p * (n as f64 + 1.0);
    if r <= 1.0 {
        return Some(sorted_values[0]);
    }
    if r >= n as f64 {
        return Some(sorted_values[n - 1]);
    }

    let k = r.floor() as usize;
    let d = r - r.floor();
    // 1-based rank k maps to 0-based index k - 1
    if k == 0 {
        return Some(sorted_values[0]);
    }
    let val_k = sorted_values.get(k - 1).copied()?;
    let val_k_next = sorted_values.get(k).copied()?;
    Some(val_k + d * (val_k_next - val_k))
}

/// Computes N-tiles (e.g. 10 for deciles) dividing sorted values into N equal parts.
/// Returns a list of (percentile_in_percent, value) pairs for k = 1..N.
pub fn compute_ntiles(sorted_values: &[f64], n: usize) -> Option<Vec<(f64, f64)>> {
    if n < 2 || sorted_values.is_empty() {
        return None;
    }

    let mut result = Vec::with_capacity(n - 1);
    for k in 1..n {
        let p_pct = (k as f64 / n as f64) * 100.0;
        let p_frac = k as f64 / n as f64;
        let val = compute_percentile(sorted_values, p_frac)?;
        result.push((p_pct, val));
    }
    Some(result)
}

/// Computes custom percentiles given as percentages (e.g., [5.0, 10.0, 90.0, 95.0]).
pub fn compute_custom_percentiles(sorted_values: &[f64], percentiles: &[f64]) -> Vec<(f64, f64)> {
    if sorted_values.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(percentiles.len());
    for &p in percentiles {
        if p <= 0.0 || p >= 100.0 || !p.is_finite() {
            continue;
        }
        if let Some(val) = compute_percentile(sorted_values, p / 100.0) {
            result.push((p, val));
        }
    }
    result
}

/// Computes percentile for grouped data assuming values represent category midpoints
/// of width 1.0 (interval [x - 0.5, x + 0.5]).
pub fn compute_grouped_percentile(sorted_values: &[f64], p: f64) -> Option<f64> {
    let n = sorted_values.len();
    if n == 0 {
        return None;
    }
    if n == 1 {
        return Some(sorted_values[0]);
    }

    // Build unique value frequency distribution
    let mut unique_vals: Vec<(f64, f64)> = Vec::new();
    for &v in sorted_values {
        if let Some(last) = unique_vals.last_mut() {
            if (last.0 - v).abs() < 1e-9 {
                last.1 += 1.0;
                continue;
            }
        }
        unique_vals.push((v, 1.0));
    }

    let total_n = n as f64;
    let target_cum = p * total_n;

    let mut cum = 0.0_f64;
    for &(val, freq) in &unique_vals {
        let next_cum = cum + freq;
        if target_cum <= next_cum {
            let lower_bound = val - 0.5;
            let offset = (target_cum - cum) / freq;
            return Some(lower_bound + offset);
        }
        cum = next_cum;
    }

    unique_vals.last().map(|&(v, _)| v)
}

/// Computes percentiles according to `PercentilesOptions`, sorting and deduplicating
/// any requested quartiles, ntiles, and custom cutpoints.
pub fn compute_percentiles_with_options(
    sorted_values: &[f64],
    options: &PercentilesOptions,
    group_midpoints: bool,
) -> Vec<PercentileValue> {
    if sorted_values.is_empty() {
        return Vec::new();
    }

    let mut requested: Vec<f64> = Vec::new();

    if options.quartiles {
        requested.push(25.0);
        requested.push(50.0);
        requested.push(75.0);
    }

    if let Some(n) = options.ntiles {
        if n >= 2 {
            for k in 1..n {
                requested.push((k as f64 / n as f64) * 100.0);
            }
        }
    }

    for &c in &options.custom {
        if c > 0.0 && c < 100.0 && c.is_finite() {
            requested.push(c);
        }
    }

    if requested.is_empty() {
        return Vec::new();
    }

    // Sort and deduplicate percentiles within 1e-6 tolerance
    requested.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut deduped: Vec<f64> = Vec::new();
    for p in requested {
        if let Some(&last) = deduped.last() {
            if (last - p).abs() < 1e-6 {
                continue;
            }
        }
        deduped.push(p);
    }

    let mut result = Vec::with_capacity(deduped.len());
    for p in deduped {
        let val_opt = if group_midpoints {
            compute_grouped_percentile(sorted_values, p / 100.0)
        } else {
            compute_percentile(sorted_values, p / 100.0)
        };

        if let Some(val) = val_opt {
            result.push(PercentileValue {
                percentile: p,
                value: val,
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_percentile_haverage() {
        let vals = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        // n = 5
        // p = 0.25 -> r = 0.25 * 6 = 1.5 -> 10 + 0.5 * (20 - 10) = 15.0
        // p = 0.50 -> r = 0.50 * 6 = 3.0 -> 30.0
        // p = 0.75 -> r = 0.75 * 6 = 4.5 -> 40 + 0.5 * (50 - 40) = 45.0
        assert_eq!(compute_percentile(&vals, 0.25), Some(15.0));
        assert_eq!(compute_percentile(&vals, 0.50), Some(30.0));
        assert_eq!(compute_percentile(&vals, 0.75), Some(45.0));
    }

    #[test]
    fn test_compute_ntiles() {
        let vals = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let deciles = compute_ntiles(&vals, 10).expect("ntiles");
        assert_eq!(deciles.len(), 9);
        assert!((deciles[0].0 - 10.0).abs() < 1e-6);
        assert!((deciles[4].0 - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_percentiles_with_options_dedup() {
        let vals = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let options = PercentilesOptions {
            quartiles: true,
            ntiles: Some(4), // also generates 25, 50, 75
            custom: vec![50.0, 90.0],
        };
        let res = compute_percentiles_with_options(&vals, &options, false);
        // Expect: 25.0, 50.0, 75.0, 90.0 (no duplicate 50.0)
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].percentile, 25.0);
        assert_eq!(res[1].percentile, 50.0);
        assert_eq!(res[2].percentile, 75.0);
        assert_eq!(res[3].percentile, 90.0);
    }
}
