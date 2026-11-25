use crate::frequencies::compute_percentile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplorePercentilesResult {
    pub p5: Option<f64>,
    pub p10: Option<f64>,
    pub p25: Option<f64>,
    pub p50: Option<f64>,
    pub p75: Option<f64>,
    pub p90: Option<f64>,
    pub p95: Option<f64>,
    pub tukey_q1: Option<f64>,
    pub tukey_q2: Option<f64>,
    pub tukey_q3: Option<f64>,
}

/// Evaluates a 1-based depth (which is either integer or half-integer) on a sorted slice.
fn value_at_depth(sorted: &[f64], depth_1based: f64) -> Option<f64> {
    let n = sorted.len();
    if n == 0 {
        return None;
    }
    let clamped = depth_1based.clamp(1.0, n as f64);
    let lo_idx = (clamped.floor() as usize).saturating_sub(1);
    let hi_idx = (clamped.ceil() as usize).saturating_sub(1);
    let lo = sorted.get(lo_idx).copied()?;
    let hi = sorted.get(hi_idx).copied()?;
    let frac = clamped - clamped.floor();
    Some(lo + frac * (hi - lo))
}

/// Computes Tukey's Hinges (Q1, Q2, Q3) per IBM SPSS Statistics `EXAMINE` algorithm:
/// - Median depth: `d_med = (n + 1) / 2`
/// - Lower hinge depth: `d_1 = (floor(d_med) + 1) / 2`
/// - Upper hinge depth: `d_3 = n + 1 - d_1`
pub fn compute_tukey_hinges(sorted_values: &[f64]) -> Option<(f64, f64, f64)> {
    let n = sorted_values.len();
    if n == 0 {
        return None;
    }
    let n_f = n as f64;
    let d_med = (n_f + 1.0) / 2.0;
    let d_1 = (d_med.floor() + 1.0) / 2.0;
    let d_3 = n_f + 1.0 - d_1;

    let q1 = value_at_depth(sorted_values, d_1)?;
    let q2 = value_at_depth(sorted_values, d_med)?;
    let q3 = value_at_depth(sorted_values, d_3)?;
    Some((q1, q2, q3))
}

/// Computes standard SPSS `EXAMINE` percentiles (5, 10, 25, 50, 75, 90, 95 via HAVERAGE)
/// alongside Tukey's Hinges.
pub fn compute_explore_percentiles(valid_values: &[f64]) -> ExplorePercentilesResult {
    if valid_values.is_empty() {
        return ExplorePercentilesResult {
            p5: None,
            p10: None,
            p25: None,
            p50: None,
            p75: None,
            p90: None,
            p95: None,
            tukey_q1: None,
            tukey_q2: None,
            tukey_q3: None,
        };
    }

    let mut sorted = valid_values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let hinges = compute_tukey_hinges(&sorted);

    ExplorePercentilesResult {
        p5: compute_percentile(&sorted, 0.05),
        p10: compute_percentile(&sorted, 0.10),
        p25: compute_percentile(&sorted, 0.25),
        p50: compute_percentile(&sorted, 0.50),
        p75: compute_percentile(&sorted, 0.75),
        p90: compute_percentile(&sorted, 0.90),
        p95: compute_percentile(&sorted, 0.95),
        tukey_q1: hinges.map(|h| h.0),
        tukey_q2: hinges.map(|h| h.1),
        tukey_q3: hinges.map(|h| h.2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tukey_hinges_odd_and_even() {
        // n = 4: [10, 20, 30, 40] -> d_med = 2.5, d_1 = 1.5, d_3 = 3.5 -> (15, 25, 35)
        let res4 = compute_explore_percentiles(&[40.0, 10.0, 30.0, 20.0]);
        assert_eq!(res4.tukey_q1, Some(15.0));
        assert_eq!(res4.tukey_q2, Some(25.0));
        assert_eq!(res4.tukey_q3, Some(35.0));

        // n = 5: [10, 20, 30, 40, 50] -> d_med = 3.0, d_1 = 2.0, d_3 = 4.0 -> (20, 30, 40)
        let res5 = compute_explore_percentiles(&[10.0, 20.0, 30.0, 40.0, 50.0]);
        assert_eq!(res5.tukey_q1, Some(20.0));
        assert_eq!(res5.tukey_q2, Some(30.0));
        assert_eq!(res5.tukey_q3, Some(40.0));

        // n = 7: [10..70] -> d_med = 4.0, d_1 = 2.5, d_3 = 5.5 -> (25, 40, 55)
        let res7 = compute_explore_percentiles(&[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0]);
        assert_eq!(res7.tukey_q1, Some(25.0));
        assert_eq!(res7.tukey_q2, Some(40.0));
        assert_eq!(res7.tukey_q3, Some(55.0));
    }

    #[test]
    fn test_haverage_percentiles_n10() {
        let data: Vec<f64> = (1..=10).map(|x| x as f64 * 10.0).collect();
        let res = compute_explore_percentiles(&data);
        // p25: r = 0.25 * 11 = 2.75 -> 20 + 0.75 * 10 = 27.5
        assert!((res.p25.unwrap() - 27.5).abs() < 1e-6);
        // p50: r = 0.50 * 11 = 5.5 -> 55.0
        assert!((res.p50.unwrap() - 55.0).abs() < 1e-6);
        // p75: r = 0.75 * 11 = 8.25 -> 82.5
        assert!((res.p75.unwrap() - 82.5).abs() < 1e-6);
        // p5: r = 0.05 * 11 = 0.55 <= 1 -> 10.0
        assert!((res.p5.unwrap() - 10.0).abs() < 1e-6);
        // p95: r = 0.95 * 11 = 10.45 >= 10 -> 100.0
        assert!((res.p95.unwrap() - 100.0).abs() < 1e-6);
    }
}
