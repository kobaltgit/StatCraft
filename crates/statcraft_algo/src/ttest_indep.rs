use crate::ttest_onesample::{student_t_inv_cdf, student_t_sf_2tailed};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndepTTestOptions {
    pub group1_value: String,
    pub group2_value: String,
    pub confidence_level: f64,
}

impl Default for IndepTTestOptions {
    fn default() -> Self {
        Self {
            group1_value: "1".to_string(),
            group2_value: "2".to_string(),
            confidence_level: 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupStats {
    pub group_val: String,
    pub n: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub se_mean: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LeveneResult {
    pub f_stat: Option<f64>,
    pub sig: Option<f64>,
    pub df1: f64,
    pub df2: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TTestTwoGroupDetails {
    pub t_stat: Option<f64>,
    pub df: Option<f64>,
    pub sig_2tailed: Option<f64>,
    pub mean_diff: Option<f64>,
    pub se_diff: Option<f64>,
    pub ci_lower: Option<f64>,
    pub ci_upper: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndepTTestResult {
    pub variable: String,
    pub grouping_var: String,
    pub group1_stats: GroupStats,
    pub group2_stats: GroupStats,
    pub levene: LeveneResult,
    pub equal_variances_assumed: TTestTwoGroupDetails,
    pub equal_variances_not_assumed: TTestTwoGroupDetails,
    pub ci_level: f64,
}

fn matches_group(val_str: &str, target_str: &str) -> bool {
    let s1 = val_str.trim();
    let s2 = target_str.trim();
    if s1.eq_ignore_ascii_case(s2) {
        return true;
    }
    if let (Ok(n1), Ok(n2)) = (s1.parse::<f64>(), s2.parse::<f64>()) {
        if (n1 - n2).abs() < 1e-9 {
            return true;
        }
    }
    false
}

fn calculate_sample_stats(data: &[f64], group_val: &str) -> (GroupStats, Option<f64>) {
    let n = data.len();
    if n == 0 {
        return (
            GroupStats {
                group_val: group_val.to_string(),
                n: 0,
                mean: None,
                std_dev: None,
                se_mean: None,
            },
            None,
        );
    }

    let sum: f64 = data.iter().sum();
    let mean = sum / (n as f64);

    if n < 2 {
        return (
            GroupStats {
                group_val: group_val.to_string(),
                n,
                mean: Some(mean),
                std_dev: None,
                se_mean: None,
            },
            None,
        );
    }

    let var_sum: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum();
    let variance = var_sum / ((n - 1) as f64);
    let std_dev = variance.sqrt();
    let se_mean = std_dev / (n as f64).sqrt();

    (
        GroupStats {
            group_val: group_val.to_string(),
            n,
            mean: Some(mean),
            std_dev: Some(std_dev),
            se_mean: Some(se_mean),
        },
        Some(variance),
    )
}

/// Computes Levene's test for equality of variances between two groups (SPSS canonical center=mean).
fn compute_levene_test(g1: &[f64], g2: &[f64], mean1: f64, mean2: f64) -> LeveneResult {
    let n1 = g1.len();
    let n2 = g2.len();
    let df1 = 1.0;
    let df2 = (n1 + n2).saturating_sub(2) as f64;

    if n1 < 2 || n2 < 2 {
        return LeveneResult {
            f_stat: None,
            sig: None,
            df1,
            df2,
        };
    }

    let z1: Vec<f64> = g1.iter().map(|&x| (x - mean1).abs()).collect();
    let z2: Vec<f64> = g2.iter().map(|&x| (x - mean2).abs()).collect();

    let sum_z1: f64 = z1.iter().sum();
    let sum_z2: f64 = z2.iter().sum();
    let mean_z1 = sum_z1 / (n1 as f64);
    let mean_z2 = sum_z2 / (n2 as f64);

    let grand_mean_z = (sum_z1 + sum_z2) / ((n1 + n2) as f64);

    let ssb = (n1 as f64) * (mean_z1 - grand_mean_z).powi(2)
        + (n2 as f64) * (mean_z2 - grand_mean_z).powi(2);
    let msb = ssb / df1;

    let ssw_1: f64 = z1.iter().map(|&z| (z - mean_z1).powi(2)).sum();
    let ssw_2: f64 = z2.iter().map(|&z| (z - mean_z2).powi(2)).sum();
    let ssw = ssw_1 + ssw_2;
    let msw = ssw / df2;

    if msw <= 1e-15 {
        if msb <= 1e-15 {
            return LeveneResult {
                f_stat: Some(0.0),
                sig: Some(1.0),
                df1,
                df2,
            };
        }
        return LeveneResult {
            f_stat: None,
            sig: None,
            df1,
            df2,
        };
    }

    let f = msb / msw;
    let sig = student_t_sf_2tailed(f.sqrt(), df2);

    LeveneResult {
        f_stat: Some(f),
        sig: Some(sig),
        df1,
        df2,
    }
}

/// Computes Independent-Samples T-Test from raw slices of group observations.
pub fn compute_indep_ttest_from_slices(
    var_name: &str,
    grouping_name: &str,
    g1: &[f64],
    g2: &[f64],
    options: &IndepTTestOptions,
) -> IndepTTestResult {
    let (group1_stats, var1_opt) = calculate_sample_stats(g1, &options.group1_value);
    let (group2_stats, var2_opt) = calculate_sample_stats(g2, &options.group2_value);

    let n1 = g1.len();
    let n2 = g2.len();
    let ci = options.confidence_level;

    // Check if both groups have enough cases and calculable means
    if n1 < 2 || n2 < 2 || group1_stats.mean.is_none() || group2_stats.mean.is_none() {
        let df1 = 1.0;
        let df2 = (n1 + n2).saturating_sub(2) as f64;
        return IndepTTestResult {
            variable: var_name.to_string(),
            grouping_var: grouping_name.to_string(),
            group1_stats,
            group2_stats,
            levene: LeveneResult {
                f_stat: None,
                sig: None,
                df1,
                df2,
            },
            equal_variances_assumed: TTestTwoGroupDetails {
                t_stat: None,
                df: Some(df2),
                sig_2tailed: None,
                mean_diff: None,
                se_diff: None,
                ci_lower: None,
                ci_upper: None,
            },
            equal_variances_not_assumed: TTestTwoGroupDetails {
                t_stat: None,
                df: None,
                sig_2tailed: None,
                mean_diff: None,
                se_diff: None,
                ci_lower: None,
                ci_upper: None,
            },
            ci_level: ci,
        };
    }

    let mean1 = group1_stats.mean.unwrap_or(0.0);
    let mean2 = group2_stats.mean.unwrap_or(0.0);
    let mean_diff = mean1 - mean2;

    let var1 = var1_opt.unwrap_or(0.0);
    let var2 = var2_opt.unwrap_or(0.0);

    let levene = compute_levene_test(g1, g2, mean1, mean2);

    let alpha = (1.0 - ci).clamp(0.0001, 0.9999);
    let half_alpha = alpha * 0.5;

    // 1. Equal variances assumed (Student's t with pooled variance)
    let df_student = (n1 + n2 - 2) as f64;
    let pooled_var = (((n1 - 1) as f64) * var1 + ((n2 - 1) as f64) * var2) / df_student;
    let se_diff_student = (pooled_var * (1.0 / (n1 as f64) + 1.0 / (n2 as f64))).sqrt();

    let equal_variances_assumed = if se_diff_student > 1e-15 {
        let t_val = mean_diff / se_diff_student;
        let sig = student_t_sf_2tailed(t_val, df_student);
        let t_crit = student_t_inv_cdf(1.0 - half_alpha, df_student);
        let ci_lower = mean_diff - t_crit * se_diff_student;
        let ci_upper = mean_diff + t_crit * se_diff_student;

        TTestTwoGroupDetails {
            t_stat: Some(t_val),
            df: Some(df_student),
            sig_2tailed: Some(sig),
            mean_diff: Some(mean_diff),
            se_diff: Some(se_diff_student),
            ci_lower: Some(ci_lower),
            ci_upper: Some(ci_upper),
        }
    } else {
        TTestTwoGroupDetails {
            t_stat: None,
            df: Some(df_student),
            sig_2tailed: None,
            mean_diff: Some(mean_diff),
            se_diff: Some(se_diff_student),
            ci_lower: None,
            ci_upper: None,
        }
    };

    // 2. Equal variances not assumed (Welch's t with Satterthwaite df)
    let v1 = var1 / (n1 as f64);
    let v2 = var2 / (n2 as f64);
    let se_diff_welch = (v1 + v2).sqrt();

    let equal_variances_not_assumed = if se_diff_welch > 1e-15 {
        let t_val = mean_diff / se_diff_welch;
        let denom = (v1 * v1) / ((n1 - 1) as f64) + (v2 * v2) / ((n2 - 1) as f64);
        let df_welch = if denom > 1e-20 {
            (v1 + v2).powi(2) / denom
        } else {
            df_student
        };

        let sig = student_t_sf_2tailed(t_val, df_welch);
        let t_crit = student_t_inv_cdf(1.0 - half_alpha, df_welch);
        let ci_lower = mean_diff - t_crit * se_diff_welch;
        let ci_upper = mean_diff + t_crit * se_diff_welch;

        TTestTwoGroupDetails {
            t_stat: Some(t_val),
            df: Some(df_welch),
            sig_2tailed: Some(sig),
            mean_diff: Some(mean_diff),
            se_diff: Some(se_diff_welch),
            ci_lower: Some(ci_lower),
            ci_upper: Some(ci_upper),
        }
    } else {
        TTestTwoGroupDetails {
            t_stat: None,
            df: None,
            sig_2tailed: None,
            mean_diff: Some(mean_diff),
            se_diff: Some(se_diff_welch),
            ci_lower: None,
            ci_upper: None,
        }
    };

    IndepTTestResult {
        variable: var_name.to_string(),
        grouping_var: grouping_name.to_string(),
        group1_stats,
        group2_stats,
        levene,
        equal_variances_assumed,
        equal_variances_not_assumed,
        ci_level: ci,
    }
}

/// Computes Independent-Samples T-Test from paired optional columns.
pub fn compute_indep_ttest(
    var_name: &str,
    grouping_name: &str,
    var_values: &[Option<f64>],
    group_values: &[Option<String>],
    options: &IndepTTestOptions,
) -> IndepTTestResult {
    let count = var_values.len().min(group_values.len());
    let mut g1 = Vec::new();
    let mut g2 = Vec::new();

    for i in 0..count {
        if let Some(val) = var_values[i] {
            if val.is_finite() {
                if let Some(ref grp_str) = group_values[i] {
                    if matches_group(grp_str, &options.group1_value) {
                        g1.push(val);
                    } else if matches_group(grp_str, &options.group2_value) {
                        g2.push(val);
                    }
                }
            }
        }
    }

    compute_indep_ttest_from_slices(var_name, grouping_name, &g1, &g2, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_oracle_against_scipy() {
        let g1 = [14.0, 15.2, 16.1, 14.8, 17.0, 15.5, 16.2, 14.9, 15.8, 16.5];
        let g2 = [
            18.0, 19.5, 17.8, 20.1, 18.9, 19.2, 21.0, 18.5, 19.8, 20.5, 19.0, 18.2,
        ];

        let opts = IndepTTestOptions {
            group1_value: "1".to_string(),
            group2_value: "2".to_string(),
            confidence_level: 0.95,
        };

        let res = compute_indep_ttest_from_slices("salary", "gender", &g1, &g2, &opts);

        // Group 1 stats
        assert_eq!(res.group1_stats.n, 10);
        assert!((res.group1_stats.mean.unwrap() - 15.60).abs() < 1e-6);
        assert!((res.group1_stats.std_dev.unwrap() - 0.8993825042).abs() < 1e-6);
        assert!((res.group1_stats.se_mean.unwrap() - 0.2844097201).abs() < 1e-6);

        // Group 2 stats
        assert_eq!(res.group2_stats.n, 12);
        assert!((res.group2_stats.mean.unwrap() - 19.2083333333).abs() < 1e-6);
        assert!((res.group2_stats.std_dev.unwrap() - 1.0094628034).abs() < 1e-6);
        assert!((res.group2_stats.se_mean.unwrap() - 0.2914068106).abs() < 1e-6);

        // Levene test (center='mean')
        assert!((res.levene.f_stat.unwrap() - 0.1615102381).abs() < 1e-6);
        assert!((res.levene.sig.unwrap() - 0.6920321765).abs() < 1e-6);
        assert_eq!(res.levene.df1, 1.0);
        assert_eq!(res.levene.df2, 20.0);

        // Student's t (equal variances assumed)
        let s = &res.equal_variances_assumed;
        assert!((s.t_stat.unwrap() - (-8.7648050760)).abs() < 1e-6);
        assert_eq!(s.df, Some(20.0));
        assert!((s.sig_2tailed.unwrap() - 0.0000000277).abs() < 1e-6);
        assert!((s.mean_diff.unwrap() - (-3.6083333333)).abs() < 1e-6);
        assert!((s.se_diff.unwrap() - 0.4116838384).abs() < 1e-6);
        assert!((s.ci_lower.unwrap() - (-4.4670918994)).abs() < 1e-6);
        assert!((s.ci_upper.unwrap() - (-2.7495747673)).abs() < 1e-6);

        // Welch's t (equal variances not assumed)
        let w = &res.equal_variances_not_assumed;
        assert!((w.t_stat.unwrap() - (-8.8614635953)).abs() < 1e-6);
        assert!((w.df.unwrap() - 19.8848975838).abs() < 1e-6);
        assert!((w.sig_2tailed.unwrap() - 0.0000000243).abs() < 1e-6);
        assert!((w.mean_diff.unwrap() - (-3.6083333333)).abs() < 1e-6);
        assert!((w.se_diff.unwrap() - 0.4071938337).abs() < 1e-6);
        assert!((w.ci_lower.unwrap() - (-4.4580400894)).abs() < 1e-6);
        assert!((w.ci_upper.unwrap() - (-2.7586265773)).abs() < 1e-6);
    }

    #[test]
    fn test_compute_indep_ttest_with_group_matching() {
        let var_values = vec![
            Some(10.0),
            Some(12.0),
            None,
            Some(20.0),
            Some(22.0),
            Some(999.0), // group 3, should be ignored
        ];
        let group_values = vec![
            Some("1.0".to_string()),
            Some("1".to_string()),
            Some("1".to_string()),
            Some("2".to_string()),
            Some("2.0".to_string()),
            Some("3".to_string()),
        ];

        let opts = IndepTTestOptions {
            group1_value: "1".to_string(),
            group2_value: "2".to_string(),
            confidence_level: 0.95,
        };

        let res = compute_indep_ttest("x", "grp", &var_values, &group_values, &opts);
        assert_eq!(res.group1_stats.n, 2);
        assert_eq!(res.group2_stats.n, 2);
        assert_eq!(res.group1_stats.mean, Some(11.0));
        assert_eq!(res.group2_stats.mean, Some(21.0));
    }

    #[test]
    fn test_compute_indep_ttest_small_sample_edge_case() {
        let g1 = [10.0];
        let g2 = [20.0];
        let opts = IndepTTestOptions::default();
        let res = compute_indep_ttest_from_slices("x", "grp", &g1, &g2, &opts);

        assert_eq!(res.group1_stats.n, 1);
        assert_eq!(res.group2_stats.n, 1);
        assert!(res.equal_variances_assumed.t_stat.is_none());
        assert!(res.equal_variances_not_assumed.t_stat.is_none());
    }
}
