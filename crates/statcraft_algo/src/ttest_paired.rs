use crate::ttest_onesample::{student_t_inv_cdf, student_t_sf_2tailed};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairedVarStats {
    pub var_name: String,
    pub n: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub se_mean: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairedCorrelation {
    pub var1: String,
    pub var2: String,
    pub n: usize,
    pub correlation: Option<f64>,
    pub sig: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairedDifferences {
    pub mean_diff: Option<f64>,
    pub std_dev_diff: Option<f64>,
    pub se_diff: Option<f64>,
    pub ci_lower: Option<f64>,
    pub ci_upper: Option<f64>,
    pub t_stat: Option<f64>,
    pub df: Option<f64>,
    pub sig_2tailed: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairedTTestResult {
    pub pair_num: usize,
    pub var1_stats: PairedVarStats,
    pub var2_stats: PairedVarStats,
    pub correlation: PairedCorrelation,
    pub differences: PairedDifferences,
    pub confidence_level: f64,
}

pub type PairedDataInput<'a> = (&'a str, &'a [Option<f64>], &'a str, &'a [Option<f64>]);
pub type PairedSliceInput<'a> = (&'a str, &'a [f64], &'a str, &'a [f64]);

/// Computes paired-samples t-test for pairs of variables, handling optional (missing) values per pair.
/// Follows listwise deletion per pair: cases where either var1 or var2 is None/NaN are excluded.
pub fn compute_paired_ttest(
    pairs: &[PairedDataInput<'_>],
    confidence_level: f64,
) -> Vec<PairedTTestResult> {
    let mut results = Vec::with_capacity(pairs.len());

    for (idx, (var1_name, var1_vals, var2_name, var2_vals)) in pairs.iter().enumerate() {
        let pair_num = idx + 1;
        let min_len = var1_vals.len().min(var2_vals.len());

        let mut valid_x: Vec<f64> = Vec::with_capacity(min_len);
        let mut valid_y: Vec<f64> = Vec::with_capacity(min_len);

        for i in 0..min_len {
            if let (Some(x), Some(y)) = (var1_vals[i], var2_vals[i]) {
                if x.is_finite() && y.is_finite() {
                    valid_x.push(x);
                    valid_y.push(y);
                }
            }
        }

        let res = compute_single_pair(
            pair_num,
            var1_name,
            &valid_x,
            var2_name,
            &valid_y,
            confidence_level,
        );
        results.push(res);
    }

    results
}

/// Computes paired-samples t-test from clean f64 slices.
pub fn compute_paired_ttest_from_slices(
    pairs: &[PairedSliceInput<'_>],
    confidence_level: f64,
) -> Vec<PairedTTestResult> {
    let mut results = Vec::with_capacity(pairs.len());

    for (idx, (var1_name, var1_vals, var2_name, var2_vals)) in pairs.iter().enumerate() {
        let pair_num = idx + 1;
        let min_len = var1_vals.len().min(var2_vals.len());

        let mut valid_x: Vec<f64> = Vec::with_capacity(min_len);
        let mut valid_y: Vec<f64> = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let x = var1_vals[i];
            let y = var2_vals[i];
            if x.is_finite() && y.is_finite() {
                valid_x.push(x);
                valid_y.push(y);
            }
        }

        let res = compute_single_pair(
            pair_num,
            var1_name,
            &valid_x,
            var2_name,
            &valid_y,
            confidence_level,
        );
        results.push(res);
    }

    results
}

fn compute_single_pair(
    pair_num: usize,
    var1_name: &str,
    x: &[f64],
    var2_name: &str,
    y: &[f64],
    confidence_level: f64,
) -> PairedTTestResult {
    // 1. Descriptive stats for Var 1 and Var 2
    let var1_stats = compute_var_stats(var1_name, x);
    let var2_stats = compute_var_stats(var2_name, y);

    // 2. Pearson Correlation & Significance
    let correlation = compute_paired_correlation(var1_name, var2_name, x, y);

    // 3. Paired Differences Analysis
    let differences = compute_differences(x, y, confidence_level);

    PairedTTestResult {
        pair_num,
        var1_stats,
        var2_stats,
        correlation,
        differences,
        confidence_level,
    }
}

fn compute_var_stats(name: &str, data: &[f64]) -> PairedVarStats {
    let n = data.len();
    if n == 0 {
        return PairedVarStats {
            var_name: name.to_string(),
            n: 0,
            mean: None,
            std_dev: None,
            se_mean: None,
        };
    }

    let sum: f64 = data.iter().sum();
    let mean = sum / (n as f64);

    if n < 2 {
        return PairedVarStats {
            var_name: name.to_string(),
            n,
            mean: Some(mean),
            std_dev: None,
            se_mean: None,
        };
    }

    let ss: f64 = data.iter().map(|&v| (v - mean).powi(2)).sum();
    let variance = ss / ((n - 1) as f64);
    let std_dev = variance.max(0.0).sqrt();
    let se_mean = std_dev / (n as f64).sqrt();

    PairedVarStats {
        var_name: name.to_string(),
        n,
        mean: Some(mean),
        std_dev: Some(std_dev),
        se_mean: Some(se_mean),
    }
}

fn compute_paired_correlation(
    var1_name: &str,
    var2_name: &str,
    x: &[f64],
    y: &[f64],
) -> PairedCorrelation {
    let n = x.len();
    if n < 2 {
        return PairedCorrelation {
            var1: var1_name.to_string(),
            var2: var2_name.to_string(),
            n,
            correlation: None,
            sig: None,
        };
    }

    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let mean_x = sum_x / (n as f64);
    let mean_y = sum_y / (n as f64);

    let mut ss_xx = 0.0;
    let mut ss_yy = 0.0;
    let mut sp_xy = 0.0;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        ss_xx += dx * dx;
        ss_yy += dy * dy;
        sp_xy += dx * dy;
    }

    if ss_xx <= 0.0 || ss_yy <= 0.0 {
        return PairedCorrelation {
            var1: var1_name.to_string(),
            var2: var2_name.to_string(),
            n,
            correlation: None,
            sig: None,
        };
    }

    let denom = (ss_xx * ss_yy).sqrt();
    let r = if denom > 0.0 {
        (sp_xy / denom).clamp(-1.0, 1.0)
    } else {
        0.0
    };

    let sig = if n <= 2 {
        None
    } else {
        let df = (n - 2) as f64;
        let one_minus_r2 = (1.0 - r * r).max(0.0);
        if one_minus_r2 < 1e-15 {
            Some(0.0)
        } else {
            let t_r = r * (df / one_minus_r2).sqrt();
            Some(student_t_sf_2tailed(t_r, df))
        }
    };

    PairedCorrelation {
        var1: var1_name.to_string(),
        var2: var2_name.to_string(),
        n,
        correlation: Some(r),
        sig,
    }
}

fn compute_differences(x: &[f64], y: &[f64], confidence_level: f64) -> PairedDifferences {
    let n = x.len();
    if n == 0 {
        return PairedDifferences {
            mean_diff: None,
            std_dev_diff: None,
            se_diff: None,
            ci_lower: None,
            ci_upper: None,
            t_stat: None,
            df: None,
            sig_2tailed: None,
        };
    }

    let diffs: Vec<f64> = x.iter().zip(y.iter()).map(|(&a, &b)| a - b).collect();
    let sum_d: f64 = diffs.iter().sum();
    let mean_d = sum_d / (n as f64);

    if n < 2 {
        return PairedDifferences {
            mean_diff: Some(mean_d),
            std_dev_diff: None,
            se_diff: None,
            ci_lower: None,
            ci_upper: None,
            t_stat: None,
            df: None,
            sig_2tailed: None,
        };
    }

    let df = (n - 1) as f64;
    let ss_d: f64 = diffs.iter().map(|&d| (d - mean_d).powi(2)).sum();
    let var_d = ss_d / df;
    let std_dev_d = var_d.max(0.0).sqrt();
    let se_d = std_dev_d / (n as f64).sqrt();

    let alpha = (1.0 - confidence_level).clamp(0.0001, 0.9999);
    let t_crit = student_t_inv_cdf(1.0 - alpha / 2.0, df);

    if se_d <= 1e-15 {
        // Zero variance edge-case
        let t_stat = if mean_d.abs() < 1e-15 {
            Some(0.0)
        } else {
            None
        };
        let sig = if mean_d.abs() < 1e-15 {
            Some(1.0)
        } else {
            None
        };
        return PairedDifferences {
            mean_diff: Some(mean_d),
            std_dev_diff: Some(std_dev_d),
            se_diff: Some(se_d),
            ci_lower: Some(mean_d),
            ci_upper: Some(mean_d),
            t_stat,
            df: Some(df),
            sig_2tailed: sig,
        };
    }

    let t_stat = mean_d / se_d;
    let sig_2tailed = student_t_sf_2tailed(t_stat, df);
    let margin = t_crit * se_d;
    let ci_lower = mean_d - margin;
    let ci_upper = mean_d + margin;

    PairedDifferences {
        mean_diff: Some(mean_d),
        std_dev_diff: Some(std_dev_d),
        se_diff: Some(se_d),
        ci_lower: Some(ci_lower),
        ci_upper: Some(ci_upper),
        t_stat: Some(t_stat),
        df: Some(df),
        sig_2tailed: Some(sig_2tailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paired_ttest_golden_oracle() {
        let x = [10.0, 12.0, 15.0, 11.0, 14.0, 16.0, 13.0, 18.0];
        let y = [8.0, 11.0, 13.0, 10.0, 15.0, 14.0, 12.0, 16.0];

        let results = compute_paired_ttest_from_slices(&[("before", &x, "after", &y)], 0.95);
        assert_eq!(results.len(), 1);

        let res = &results[0];
        assert_eq!(res.pair_num, 1);

        // Var 1 (before)
        assert_eq!(res.var1_stats.var_name, "before");
        assert_eq!(res.var1_stats.n, 8);
        assert!((res.var1_stats.mean.unwrap() - 13.625).abs() < 1e-6);
        assert!((res.var1_stats.std_dev.unwrap() - 2.669269563).abs() < 1e-6);
        assert!((res.var1_stats.se_mean.unwrap() - 0.943729304).abs() < 1e-6);

        // Var 2 (after)
        assert_eq!(res.var2_stats.var_name, "after");
        assert_eq!(res.var2_stats.n, 8);
        assert!((res.var2_stats.mean.unwrap() - 12.375).abs() < 1e-6);
        assert!((res.var2_stats.std_dev.unwrap() - 2.669269563).abs() < 1e-6);
        assert!((res.var2_stats.se_mean.unwrap() - 0.943729304).abs() < 1e-6);

        // Correlation
        assert_eq!(res.correlation.var1, "before");
        assert_eq!(res.correlation.var2, "after");
        assert_eq!(res.correlation.n, 8);
        assert!((res.correlation.correlation.unwrap() - 0.92481203).abs() < 1e-6);
        // df = 6, t_r = 5.954712 -> p_value ~ 0.001002
        assert!((res.correlation.sig.unwrap() - 0.0010023).abs() < 1e-5);

        // Paired Differences
        let d = &res.differences;
        assert!((d.mean_diff.unwrap() - 1.25).abs() < 1e-6);
        assert!((d.std_dev_diff.unwrap() - 1.035098339).abs() < 1e-6);
        assert!((d.se_diff.unwrap() - 0.365962527).abs() < 1e-6);
        assert_eq!(d.df, Some(7.0));
        assert!((d.t_stat.unwrap() - 3.415650255).abs() < 1e-6);
        assert!((d.sig_2tailed.unwrap() - 0.011195).abs() < 1e-4);

        // 95% Confidence Interval
        // t_crit for df=7, 95% is ~ 2.364624
        // margin = 2.364624 * 0.365962527 ~ 0.865364
        // ci_lower = 1.25 - 0.865364 ~ 0.384636
        // ci_upper = 1.25 + 0.865364 ~ 2.115364
        assert!((d.ci_lower.unwrap() - 0.384636).abs() < 1e-4);
        assert!((d.ci_upper.unwrap() - 2.115364).abs() < 1e-4);
    }

    #[test]
    fn test_paired_ttest_missing_values_listwise() {
        let x = [Some(10.0), Some(12.0), None, Some(14.0)];
        let y = [Some(8.0), None, Some(13.0), Some(12.0)];

        let pairs = [("x", &x[..], "y", &y[..])];
        let results = compute_paired_ttest(&pairs, 0.95);

        assert_eq!(results.len(), 1);
        let res = &results[0];
        // Only rows 0 and 3 are valid for both
        assert_eq!(res.var1_stats.n, 2);
        assert_eq!(res.var2_stats.n, 2);
        assert_eq!(res.differences.mean_diff, Some(2.0));
    }

    #[test]
    fn test_paired_ttest_edge_cases() {
        // Empty
        let res_empty = compute_paired_ttest_from_slices(&[("a", &[], "b", &[])], 0.95);
        assert_eq!(res_empty[0].var1_stats.n, 0);
        assert!(res_empty[0].differences.t_stat.is_none());

        // Single pair (N = 1)
        let res_single = compute_paired_ttest_from_slices(&[("a", &[5.0], "b", &[2.0])], 0.95);
        assert_eq!(res_single[0].var1_stats.n, 1);
        assert_eq!(res_single[0].differences.mean_diff, Some(3.0));
        assert!(res_single[0].differences.std_dev_diff.is_none());
        assert!(res_single[0].differences.t_stat.is_none());

        // Zero variance in diffs (diffs identical: 2.0, 2.0, 2.0)
        let res_const = compute_paired_ttest_from_slices(
            &[("a", &[12.0, 14.0, 16.0], "b", &[10.0, 12.0, 14.0])],
            0.95,
        );
        assert_eq!(res_const[0].differences.mean_diff, Some(2.0));
        assert_eq!(res_const[0].differences.std_dev_diff, Some(0.0));
        assert_eq!(res_const[0].differences.se_diff, Some(0.0));
    }
}
