use crate::ttest_onesample::{regularized_incomplete_beta, student_t_inv_cdf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, thiserror::Error, Serialize, Deserialize)]
pub enum AlgoError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
    #[error("Computation error: {0}")]
    ComputationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneWayAnovaOptions {
    pub descriptives: bool,
    pub homogeneity: bool,
    pub confidence_level: f64,
}

impl Default for OneWayAnovaOptions {
    fn default() -> Self {
        Self {
            descriptives: true,
            homogeneity: true,
            confidence_level: 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnovaGroupDescriptives {
    pub group_label: String,
    pub n: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub std_err: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnovaHomogeneityTest {
    pub levene_stat: f64,
    pub df1: usize,
    pub df2: usize,
    pub p_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnovaTableRow {
    pub sum_of_squares: f64,
    pub df: usize,
    pub mean_square: Option<f64>,
    pub f_stat: Option<f64>,
    pub p_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OneWayAnovaResult {
    pub dep_var: String,
    pub factor_var: String,
    pub between_groups: AnovaTableRow,
    pub within_groups: AnovaTableRow,
    pub total: AnovaTableRow,
    pub descriptives: Vec<AnovaGroupDescriptives>,
    pub total_descriptives: Option<AnovaGroupDescriptives>,
    pub homogeneity: Option<AnovaHomogeneityTest>,
}

/// Survival function (upper tail p-value P(X >= f)) for Snedecor-Fisher F-distribution F(df1, df2).
/// Uses regularized incomplete beta function I_{1-x}(df2/2, df1/2) where 1-x = df2 / (df1 * f + df2).
pub fn f_distribution_sf(f: f64, df1: f64, df2: f64) -> f64 {
    if df1 <= 0.0 || df2 <= 0.0 {
        return 1.0;
    }
    if f <= 0.0 {
        return 1.0;
    }
    if f.is_nan() {
        return 1.0;
    }
    if f.is_infinite() {
        return 0.0;
    }

    let x = df2 / (df1 * f + df2);
    regularized_incomplete_beta(x, df2 * 0.5, df1 * 0.5)
}

struct RawGroup {
    label: String,
    values: Vec<f64>,
    mean: f64,
}

/// Computes One-Way ANOVA from raw non-optional slices.
pub fn compute_oneway_anova_from_slices(
    dep_name: &str,
    dep_data: &[f64],
    factor_name: &str,
    factor_data: &[String],
    options: &OneWayAnovaOptions,
) -> Result<OneWayAnovaResult, AlgoError> {
    let dep_opts: Vec<Option<f64>> = dep_data.iter().map(|&x| Some(x)).collect();
    let factor_opts: Vec<Option<String>> = factor_data.iter().map(|s| Some(s.clone())).collect();
    compute_oneway_anova(dep_name, factor_name, &dep_opts, &factor_opts, options)
}

/// Computes One-Way ANOVA on dependent numeric variable across categorical factor groups.
pub fn compute_oneway_anova(
    dep_name: &str,
    factor_name: &str,
    dep_values: &[Option<f64>],
    factor_values: &[Option<String>],
    options: &OneWayAnovaOptions,
) -> Result<OneWayAnovaResult, AlgoError> {
    if dep_values.len() != factor_values.len() {
        return Err(AlgoError::InvalidInput(format!(
            "Length mismatch between dependent variable ({}) and factor variable ({})",
            dep_values.len(),
            factor_values.len()
        )));
    }

    // Filter valid pairs (listwise per analysis)
    let mut valid_pairs: Vec<(f64, String)> = Vec::new();
    for (d_opt, f_opt) in dep_values.iter().zip(factor_values.iter()) {
        if let (Some(y), Some(f)) = (d_opt, f_opt) {
            let f_trimmed = f.trim();
            if !y.is_nan() && !f_trimmed.is_empty() && f_trimmed != "." && f_trimmed != "NA" {
                valid_pairs.push((*y, f_trimmed.to_string()));
            }
        }
    }

    let n_total = valid_pairs.len();
    if n_total == 0 {
        return Err(AlgoError::InsufficientData(format!(
            "No valid cases found for dependent variable '{}' and factor '{}'",
            dep_name, factor_name
        )));
    }

    // Partition by factor group
    let mut group_map: std::collections::BTreeMap<String, Vec<f64>> =
        std::collections::BTreeMap::new();
    for (y, f_label) in &valid_pairs {
        group_map.entry(f_label.clone()).or_default().push(*y);
    }

    let k = group_map.len();
    if k < 2 {
        return Err(AlgoError::InsufficientData(format!(
            "One-Way ANOVA requires at least 2 distinct groups, found {}",
            k
        )));
    }

    if n_total <= k {
        return Err(AlgoError::InsufficientData(format!(
            "Total observations ({}) must be greater than number of groups ({}) to calculate within-group variance",
            n_total, k
        )));
    }

    // Sort group labels: numeric if all parse as float, otherwise lexicographic
    let mut keys: Vec<String> = group_map.keys().cloned().collect();
    let all_numeric = keys.iter().all(|k| k.parse::<f64>().is_ok());
    if all_numeric {
        keys.sort_by(|a, b| {
            let na = a.parse::<f64>().unwrap_or(0.0);
            let nb = b.parse::<f64>().unwrap_or(0.0);
            na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        keys.sort();
    }

    let mut groups: Vec<RawGroup> = Vec::with_capacity(k);
    for key in keys {
        let vals = group_map.remove(&key).unwrap_or_default();
        let sum: f64 = vals.iter().sum();
        let mean = if !vals.is_empty() {
            sum / vals.len() as f64
        } else {
            0.0
        };
        groups.push(RawGroup {
            label: key,
            values: vals,
            mean,
        });
    }

    let grand_sum: f64 = valid_pairs.iter().map(|(y, _)| *y).sum();
    let grand_mean = grand_sum / (n_total as f64);

    // Sums of squares
    let mut ss_between = 0.0_f64;
    let mut ss_within = 0.0_f64;

    for g in &groups {
        let n_g = g.values.len() as f64;
        ss_between += n_g * (g.mean - grand_mean).powi(2);
        for &y in &g.values {
            ss_within += (y - g.mean).powi(2);
        }
    }

    let df_between = k - 1;
    let df_within = n_total - k;
    let df_total = n_total - 1;
    let ss_total = ss_between + ss_within;

    let ms_between = if df_between > 0 {
        ss_between / (df_between as f64)
    } else {
        0.0
    };

    let ms_within = if df_within > 0 {
        ss_within / (df_within as f64)
    } else {
        0.0
    };

    let (f_stat, p_val) = if ms_within > 1e-15 {
        let f = ms_between / ms_within;
        let p = f_distribution_sf(f, df_between as f64, df_within as f64);
        (Some(f), Some(p))
    } else if ms_between <= 1e-15 {
        (Some(0.0), Some(1.0))
    } else {
        (None, None)
    };

    let between_groups = AnovaTableRow {
        sum_of_squares: ss_between,
        df: df_between,
        mean_square: Some(ms_between),
        f_stat,
        p_value: p_val,
    };

    let within_groups = AnovaTableRow {
        sum_of_squares: ss_within,
        df: df_within,
        mean_square: Some(ms_within),
        f_stat: None,
        p_value: None,
    };

    let total = AnovaTableRow {
        sum_of_squares: ss_total,
        df: df_total,
        mean_square: None,
        f_stat: None,
        p_value: None,
    };

    // Descriptives
    let (descriptives, total_descriptives) = if options.descriptives {
        let alpha = (1.0 - options.confidence_level).clamp(0.0001, 0.9999);
        let mut list = Vec::with_capacity(k);

        for g in &groups {
            let n = g.values.len();
            let mean = g.mean;
            let (min, max) = g
                .values
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), &v| {
                    (mn.min(v), mx.max(v))
                });

            let (std_dev, std_err, ci_lower, ci_upper) = if n >= 2 {
                let var: f64 =
                    g.values.iter().map(|&y| (y - mean).powi(2)).sum::<f64>() / ((n - 1) as f64);
                let sd = var.sqrt();
                let se = sd / (n as f64).sqrt();
                let t_crit = student_t_inv_cdf(1.0 - alpha * 0.5, (n - 1) as f64);
                (sd, se, mean - t_crit * se, mean + t_crit * se)
            } else {
                (0.0, 0.0, mean, mean)
            };

            list.push(AnovaGroupDescriptives {
                group_label: g.label.clone(),
                n,
                mean,
                std_dev,
                std_err,
                ci_lower,
                ci_upper,
                min,
                max,
            });
        }

        let (tot_min, tot_max) = valid_pairs
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), &(y, _)| {
                (mn.min(y), mx.max(y))
            });

        let (tot_sd, tot_se, tot_ci_lower, tot_ci_upper) = if n_total >= 2 {
            let tot_var = ss_total / ((n_total - 1) as f64);
            let sd = tot_var.sqrt();
            let se = sd / (n_total as f64).sqrt();
            let t_crit = student_t_inv_cdf(1.0 - alpha * 0.5, (n_total - 1) as f64);
            (sd, se, grand_mean - t_crit * se, grand_mean + t_crit * se)
        } else {
            (0.0, 0.0, grand_mean, grand_mean)
        };

        let total_desc = AnovaGroupDescriptives {
            group_label: "Total".to_string(),
            n: n_total,
            mean: grand_mean,
            std_dev: tot_sd,
            std_err: tot_se,
            ci_lower: tot_ci_lower,
            ci_upper: tot_ci_upper,
            min: tot_min,
            max: tot_max,
        };

        (list, Some(total_desc))
    } else {
        (Vec::new(), None)
    };

    // Levene's Test of Homogeneity of Variances
    let homogeneity = if options.homogeneity {
        // z_ji = |y_ji - mean_j|
        let mut z_groups: Vec<Vec<f64>> = Vec::with_capacity(k);
        let mut sum_all_z = 0.0_f64;

        for g in &groups {
            let z_vals: Vec<f64> = g.values.iter().map(|&y| (y - g.mean).abs()).collect();
            sum_all_z += z_vals.iter().sum::<f64>();
            z_groups.push(z_vals);
        }

        let grand_mean_z = sum_all_z / (n_total as f64);

        let mut ssb_z = 0.0_f64;
        let mut ssw_z = 0.0_f64;

        for z_vals in &z_groups {
            let n_g = z_vals.len() as f64;
            let mean_z = if !z_vals.is_empty() {
                z_vals.iter().sum::<f64>() / n_g
            } else {
                0.0
            };
            ssb_z += n_g * (mean_z - grand_mean_z).powi(2);
            for &z in z_vals {
                ssw_z += (z - mean_z).powi(2);
            }
        }

        let df1 = k - 1;
        let df2 = n_total - k;
        let msb_z = if df1 > 0 { ssb_z / (df1 as f64) } else { 0.0 };
        let msw_z = if df2 > 0 { ssw_z / (df2 as f64) } else { 0.0 };

        if msw_z > 1e-15 {
            let f_l = msb_z / msw_z;
            let p_l = f_distribution_sf(f_l, df1 as f64, df2 as f64);
            Some(AnovaHomogeneityTest {
                levene_stat: f_l,
                df1,
                df2,
                p_value: p_l,
            })
        } else if msb_z <= 1e-15 {
            Some(AnovaHomogeneityTest {
                levene_stat: 0.0,
                df1,
                df2,
                p_value: 1.0,
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(OneWayAnovaResult {
        dep_var: dep_name.to_string(),
        factor_var: factor_name.to_string(),
        between_groups,
        within_groups,
        total,
        descriptives,
        total_descriptives,
        homogeneity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f_distribution_sf_closed_form() {
        // Analytical test for df1=2, where SF = (df2 / (2F + df2))^(df2/2)
        let df1 = 2.0;
        let df2 = 12.0;
        let f = 59.80952380952381;
        let p_calculated = f_distribution_sf(f, df1, df2);
        let expected_closed = (12.0 / (2.0 * f + 12.0)).powf(6.0);
        assert!((p_calculated - expected_closed).abs() < 1e-12);
    }

    #[test]
    fn test_oneway_anova_three_groups_golden_oracle() {
        let dep = vec![
            10.0, 12.0, 11.0, 13.0, 9.0, // Group 1
            15.0, 16.0, 14.0, 15.0, 17.0, // Group 2
            20.0, 22.0, 19.0, 21.0, 23.0, // Group 3
        ];
        let factor = vec![
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "3".to_string(),
            "3".to_string(),
            "3".to_string(),
            "3".to_string(),
            "3".to_string(),
        ];

        let opts = OneWayAnovaOptions::default();
        let res = compute_oneway_anova_from_slices("score", &dep, "group", &factor, &opts)
            .expect("ANOVA computation must succeed");

        // Between groups
        assert_eq!(res.between_groups.df, 2);
        assert!((res.between_groups.sum_of_squares - 251.2).abs() < 1e-7);
        assert!((res.between_groups.mean_square.unwrap_or(0.0) - 125.6).abs() < 1e-7);
        assert!((res.between_groups.f_stat.unwrap_or(0.0) - 59.8095238095).abs() < 1e-7);
        assert!((res.between_groups.p_value.unwrap_or(0.0) - 5.74347867e-7).abs() < 1e-10);

        // Within groups
        assert_eq!(res.within_groups.df, 12);
        assert!((res.within_groups.sum_of_squares - 25.2).abs() < 1e-7);
        assert!((res.within_groups.mean_square.unwrap_or(0.0) - 2.1).abs() < 1e-7);

        // Total
        assert_eq!(res.total.df, 14);
        assert!((res.total.sum_of_squares - 276.4).abs() < 1e-7);

        // Levene test
        let hom = res.homogeneity.expect("Homogeneity test requested");
        assert_eq!(hom.df1, 2);
        assert_eq!(hom.df2, 12);
        assert!((hom.levene_stat - 0.295612).abs() < 1e-5);
        assert!((hom.p_value - 0.749341).abs() < 1e-5);

        // Descriptives
        assert_eq!(res.descriptives.len(), 3);
        assert_eq!(res.descriptives[0].group_label, "1");
        assert_eq!(res.descriptives[0].n, 5);
        assert!((res.descriptives[0].mean - 11.0).abs() < 1e-7);
        assert!((res.descriptives[0].std_dev - 1.5811388).abs() < 1e-6);

        let tot = res.total_descriptives.expect("Total descriptives");
        assert_eq!(tot.n, 15);
        assert!((tot.mean - 15.8).abs() < 1e-7);
    }

    #[test]
    fn test_oneway_anova_insufficient_groups() {
        let dep = vec![10.0, 12.0, 11.0];
        let factor = vec!["1".to_string(), "1".to_string(), "1".to_string()];
        let opts = OneWayAnovaOptions::default();
        let err =
            compute_oneway_anova_from_slices("score", &dep, "group", &factor, &opts).unwrap_err();
        match err {
            AlgoError::InsufficientData(msg) => assert!(msg.contains("at least 2 distinct groups")),
            _ => panic!("Expected InsufficientData"),
        }
    }
}
