use crate::anova_oneway::f_distribution_sf;
use crate::explore::descriptives::compute_5pct_trimmed_mean;
use crate::frequencies::compute_percentile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeveneBasis {
    BasedOnMean,
    BasedOnMedian,
    BasedOnMedianAdjustedDf,
    BasedOnTrimmedMean,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LeveneTestRow {
    pub basis: LeveneBasis,
    pub levene_statistic: f64,
    pub df1: f64,
    pub df2: f64,
    pub sig: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreLeveneResult {
    pub variable: String,
    pub factor_var: String,
    pub rows: Vec<LeveneTestRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum SpreadLevelMode {
    #[default]
    None,
    PowerEstimation,
    NaturalLog,
    Power(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpreadLevelGroupRow {
    pub factor_level: String,
    pub n_valid: usize,
    pub median: Option<f64>,
    pub iqr: Option<f64>,
    pub ln_median: Option<f64>,
    pub ln_iqr: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpreadLevelResult {
    pub variable: String,
    pub factor_var: String,
    pub mode: SpreadLevelMode,
    pub groups: Vec<SpreadLevelGroupRow>,
    pub slope: Option<f64>,
    pub power: Option<f64>,
}

fn median_sorted(sorted: &[f64]) -> Option<f64> {
    let n = sorted.len();
    if n == 0 {
        return None;
    }
    if n % 2 == 1 {
        Some(sorted[n / 2])
    } else {
        Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
    }
}

fn oneway_f_on_z(z_groups: &[Vec<f64>]) -> Option<(f64, f64, f64, f64)> {
    let k = z_groups.len();
    if k < 2 {
        return None;
    }

    let n_total: usize = z_groups.iter().map(|g| g.len()).sum();
    if n_total <= k {
        return None;
    }

    let mut grand_sum = 0.0_f64;
    let mut group_means = Vec::with_capacity(k);
    for g in z_groups {
        if g.is_empty() {
            return None;
        }
        let s: f64 = g.iter().sum();
        grand_sum += s;
        group_means.push(s / g.len() as f64);
    }
    let grand_mean = grand_sum / n_total as f64;

    let mut ss_between = 0.0_f64;
    let mut ss_within = 0.0_f64;
    let mut group_ss_within = Vec::with_capacity(k);

    for (g, &g_mean) in z_groups.iter().zip(group_means.iter()) {
        let n_i = g.len() as f64;
        ss_between += n_i * (g_mean - grand_mean).powi(2);
        let u_i: f64 = g.iter().map(|&z| (z - g_mean).powi(2)).sum();
        ss_within += u_i;
        group_ss_within.push(u_i);
    }

    let df1 = (k - 1) as f64;
    let df2 = (n_total - k) as f64;

    let f_stat = if ss_within <= 1e-30 {
        0.0
    } else {
        (ss_between / df1) / (ss_within / df2)
    };

    // Welch-Satterthwaite adjusted df2: (sum u_i)^2 / sum(u_i^2 / (n_i - 1))
    let mut denom = 0.0_f64;
    for (g, &u_i) in z_groups.iter().zip(group_ss_within.iter()) {
        if g.len() > 1 {
            denom += (u_i * u_i) / ((g.len() - 1) as f64);
        }
    }
    let df2_adj = if denom > 1e-30 && ss_within > 1e-30 {
        (ss_within * ss_within) / denom
    } else {
        df2
    };

    Some((f_stat, df1, df2, df2_adj))
}

/// Computes the 4-row IBM SPSS Statistics `EXAMINE` Levene test of homogeneity of variance.
/// Returns `None` if there are fewer than 2 non-empty groups or `N <= k`.
pub fn compute_explore_levene(
    variable: &str,
    factor_var: &str,
    groups: &[(String, Vec<f64>)],
) -> Option<ExploreLeveneResult> {
    let non_empty: Vec<&[f64]> = groups
        .iter()
        .map(|(_, v)| v.as_slice())
        .filter(|v| !v.is_empty())
        .collect();

    if non_empty.len() < 2 {
        return None;
    }

    let mut z_mean = Vec::with_capacity(non_empty.len());
    let mut z_median = Vec::with_capacity(non_empty.len());
    let mut z_trimmed = Vec::with_capacity(non_empty.len());

    for &vals in &non_empty {
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        let mut sorted = vals.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let med = median_sorted(&sorted)?;
        let trim = compute_5pct_trimmed_mean(&sorted)?;

        z_mean.push(vals.iter().map(|&x| (x - mean).abs()).collect());
        z_median.push(vals.iter().map(|&x| (x - med).abs()).collect());
        z_trimmed.push(vals.iter().map(|&x| (x - trim).abs()).collect());
    }

    let (f_mean, df1, df2, _) = oneway_f_on_z(&z_mean)?;
    let (f_med, _, _, df2_adj) = oneway_f_on_z(&z_median)?;
    let (f_trim, _, _, _) = oneway_f_on_z(&z_trimmed)?;

    let sig_mean = if f_mean == 0.0 {
        1.0
    } else {
        f_distribution_sf(f_mean, df1, df2).clamp(0.0, 1.0)
    };
    let sig_med = if f_med == 0.0 {
        1.0
    } else {
        f_distribution_sf(f_med, df1, df2).clamp(0.0, 1.0)
    };
    let sig_med_adj = if f_med == 0.0 {
        1.0
    } else {
        f_distribution_sf(f_med, df1, df2_adj).clamp(0.0, 1.0)
    };
    let sig_trim = if f_trim == 0.0 {
        1.0
    } else {
        f_distribution_sf(f_trim, df1, df2).clamp(0.0, 1.0)
    };

    Some(ExploreLeveneResult {
        variable: variable.to_string(),
        factor_var: factor_var.to_string(),
        rows: vec![
            LeveneTestRow {
                basis: LeveneBasis::BasedOnMean,
                levene_statistic: f_mean,
                df1,
                df2,
                sig: sig_mean,
            },
            LeveneTestRow {
                basis: LeveneBasis::BasedOnMedian,
                levene_statistic: f_med,
                df1,
                df2,
                sig: sig_med,
            },
            LeveneTestRow {
                basis: LeveneBasis::BasedOnMedianAdjustedDf,
                levene_statistic: f_med,
                df1,
                df2: df2_adj,
                sig: sig_med_adj,
            },
            LeveneTestRow {
                basis: LeveneBasis::BasedOnTrimmedMean,
                levene_statistic: f_trim,
                df1,
                df2,
                sig: sig_trim,
            },
        ],
    })
}

fn transform_value(x: f64, mode: SpreadLevelMode) -> Option<f64> {
    match mode {
        SpreadLevelMode::None | SpreadLevelMode::PowerEstimation => Some(x),
        SpreadLevelMode::NaturalLog => {
            if x > 0.0 {
                let v = x.ln();
                v.is_finite().then_some(v)
            } else {
                None
            }
        }
        SpreadLevelMode::Power(p) => {
            if p.abs() < 1e-12 {
                if x > 0.0 {
                    let v = x.ln();
                    v.is_finite().then_some(v)
                } else {
                    None
                }
            } else {
                let v = x.powf(p);
                v.is_finite().then_some(v)
            }
        }
    }
}

/// Computes Spread-vs-Level statistics (group Medians, IQRs, log-log OLS slope, and power = 1 - slope).
pub fn compute_spread_level(
    variable: &str,
    factor_var: &str,
    groups: &[(String, Vec<f64>)],
    mode: SpreadLevelMode,
) -> Option<SpreadLevelResult> {
    if mode == SpreadLevelMode::None || groups.is_empty() {
        return None;
    }

    let mut group_rows = Vec::with_capacity(groups.len());
    let mut log_pairs = Vec::new();

    for (level, vals) in groups {
        let mut transformed: Vec<f64> = vals
            .iter()
            .filter_map(|&x| transform_value(x, mode))
            .collect();
        transformed.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n_valid = transformed.len();
        let median = median_sorted(&transformed);
        let q1 = compute_percentile(&transformed, 0.25);
        let q3 = compute_percentile(&transformed, 0.75);
        let iqr = match (q1, q3) {
            (Some(a), Some(b)) => Some(b - a),
            _ => None,
        };

        let ln_median = median.and_then(|m| (m > 0.0).then(|| m.ln()));
        let ln_iqr = iqr.and_then(|r| (r > 0.0).then(|| r.ln()));

        if let (Some(lm), Some(li)) = (ln_median, ln_iqr) {
            log_pairs.push((lm, li));
        }

        group_rows.push(SpreadLevelGroupRow {
            factor_level: level.clone(),
            n_valid,
            median,
            iqr,
            ln_median,
            ln_iqr,
        });
    }

    let (slope, power) = if mode == SpreadLevelMode::PowerEstimation && log_pairs.len() >= 2 {
        let n = log_pairs.len() as f64;
        let mean_x = log_pairs.iter().map(|&(x, _)| x).sum::<f64>() / n;
        let mean_y = log_pairs.iter().map(|&(_, y)| y).sum::<f64>() / n;
        let num: f64 = log_pairs
            .iter()
            .map(|&(x, y)| (x - mean_x) * (y - mean_y))
            .sum();
        let den: f64 = log_pairs.iter().map(|&(x, _)| (x - mean_x).powi(2)).sum();
        if den > 1e-15 {
            let s = num / den;
            (Some(s), Some(1.0 - s))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Some(SpreadLevelResult {
        variable: variable.to_string(),
        factor_var: factor_var.to_string(),
        mode,
        groups: group_rows,
        slope,
        power,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levene_4_rows_golden_oracle() {
        // 3 groups with unequal sample sizes:
        // G1: [10, 12, 14, 16]
        // G2: [20, 25, 30, 35, 40]
        // G3: [50, 60, 70]
        let groups = vec![
            ("G1".to_string(), vec![10.0, 12.0, 14.0, 16.0]),
            ("G2".to_string(), vec![20.0, 25.0, 30.0, 35.0, 40.0]),
            ("G3".to_string(), vec![50.0, 60.0, 70.0]),
        ];

        let res = compute_explore_levene("score", "grp", &groups).unwrap();
        assert_eq!(res.rows.len(), 4);

        let r_mean = &res.rows[0];
        assert_eq!(r_mean.basis, LeveneBasis::BasedOnMean);
        assert_eq!(r_mean.df1, 2.0);
        assert_eq!(r_mean.df2, 9.0);
        // Closed-form verification: F(2, 9) = 661.5 / 422.0 = 1.567535545, p = (9 / (9 + 2*F))^4.5 = 0.260556
        assert!((r_mean.levene_statistic - 1.5675355).abs() < 1e-5);
        assert!((r_mean.sig - 0.260556).abs() < 1e-5);

        let r_med = &res.rows[1];
        assert_eq!(r_med.basis, LeveneBasis::BasedOnMedian);
        assert!((r_med.levene_statistic - r_mean.levene_statistic).abs() < 1e-6);

        let r_adj = &res.rows[2];
        assert_eq!(r_adj.basis, LeveneBasis::BasedOnMedianAdjustedDf);
        assert_eq!(r_adj.levene_statistic, r_med.levene_statistic);
        assert!(r_adj.df2 < 9.0 && r_adj.df2 > 2.0);
    }

    #[test]
    fn test_spread_level_power_estimation() {
        // Construct groups where IQR scales linearly with Median (slope = 1 => power = 0 => log transform)
        let groups = vec![
            ("A".to_string(), vec![8.0, 9.0, 10.0, 11.0, 12.0]),
            ("B".to_string(), vec![16.0, 18.0, 20.0, 22.0, 24.0]),
            ("C".to_string(), vec![32.0, 36.0, 40.0, 44.0, 48.0]),
        ];

        let res =
            compute_spread_level("val", "grp", &groups, SpreadLevelMode::PowerEstimation).unwrap();
        assert!((res.slope.unwrap() - 1.0).abs() < 1e-6);
        assert!((res.power.unwrap() - 0.0).abs() < 1e-6);
    }
}
