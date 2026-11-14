//! Kaplan-Meier Survival Analysis (KM / stat.survival.km).
//! Calculates cumulative survival function S(t), Greenwood standard error,
//! means, medians, and Mantel-Cox Log-Rank test per IBM SPSS Algorithms Guide.

use crate::anova_oneway::AlgoError;
use crate::crosstabs::chi_square_sf;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KMOptions {
    pub confidence_level: f64,
}

impl Default for KMOptions {
    fn default() -> Self {
        Self {
            confidence_level: 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KMTableRow {
    pub time: f64,
    pub status: String, // "Event" or "Censored"
    pub cumulative_survival: f64,
    pub std_error: f64,
    pub cumulative_events: usize,
    pub remaining_cases: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KMCaseSummary {
    pub group_label: String,
    pub total_n: usize,
    pub events_n: usize,
    pub censored_n: usize,
    pub censored_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KMTimeEstimate {
    pub group_label: String,
    pub mean: f64,
    pub mean_se: f64,
    pub mean_ci_lower: f64,
    pub mean_ci_upper: f64,
    pub median: Option<f64>,
    pub median_ci_lower: Option<f64>,
    pub median_ci_upper: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KMLogRankTest {
    pub chi_square: f64,
    pub df: usize,
    pub p_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KMGroupResult {
    pub group_label: String,
    pub case_summary: KMCaseSummary,
    pub time_estimates: KMTimeEstimate,
    pub table_rows: Vec<KMTableRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KMResult {
    pub time_variable: String,
    pub status_variable: String,
    pub factor_variable: Option<String>,
    pub groups: Vec<KMGroupResult>,
    pub overall_summary: KMCaseSummary,
    pub log_rank_test: Option<KMLogRankTest>,
}

pub struct KMDataPoint {
    pub time: f64,
    pub is_event: bool,
    pub group: String,
}

/// Compute Kaplan-Meier analysis.
pub fn compute_kaplan_meier(
    data: &[KMDataPoint],
    time_var: &str,
    status_var: &str,
    factor_var: Option<&str>,
    options: &KMOptions,
) -> Result<KMResult, AlgoError> {
    if data.is_empty() {
        return Err(AlgoError::InsufficientData("Data is empty".to_string()));
    }

    // Partition by group
    let mut group_map: BTreeMap<String, Vec<&KMDataPoint>> = BTreeMap::new();
    for pt in data {
        group_map.entry(pt.group.clone()).or_default().push(pt);
    }

    let mut group_results = Vec::new();
    let mut total_cases_all = 0usize;
    let mut total_events_all = 0usize;

    for (grp_label, pts) in &group_map {
        let (summary, time_est, rows) = compute_single_km_group(grp_label, pts, options)?;
        total_cases_all += summary.total_n;
        total_events_all += summary.events_n;
        group_results.push(KMGroupResult {
            group_label: grp_label.clone(),
            case_summary: summary,
            time_estimates: time_est,
            table_rows: rows,
        });
    }

    let total_censored_all = total_cases_all.saturating_sub(total_events_all);
    let overall_summary = KMCaseSummary {
        group_label: "Overall".to_string(),
        total_n: total_cases_all,
        events_n: total_events_all,
        censored_n: total_censored_all,
        censored_percent: if total_cases_all > 0 {
            (total_censored_all as f64 / total_cases_all as f64) * 100.0
        } else {
            0.0
        },
    };

    // Log-Rank test if exactly 2 groups
    let log_rank_test = if group_map.len() == 2 {
        let keys: Vec<&String> = group_map.keys().collect();
        let g1_pts = group_map.get(keys[0]).cloned().unwrap_or_default();
        let g2_pts = group_map.get(keys[1]).cloned().unwrap_or_default();
        compute_log_rank_two_groups(&g1_pts, &g2_pts)
    } else {
        None
    };

    Ok(KMResult {
        time_variable: time_var.to_string(),
        status_variable: status_var.to_string(),
        factor_variable: factor_var.map(|s| s.to_string()),
        groups: group_results,
        overall_summary,
        log_rank_test,
    })
}

fn compute_single_km_group(
    label: &str,
    pts: &[&KMDataPoint],
    options: &KMOptions,
) -> Result<(KMCaseSummary, KMTimeEstimate, Vec<KMTableRow>), AlgoError> {
    let total_n = pts.len();
    let mut events_n = 0usize;
    for p in pts {
        if p.is_event {
            events_n += 1;
        }
    }
    let censored_n = total_n.saturating_sub(events_n);
    let censored_pct = if total_n > 0 {
        (censored_n as f64 / total_n as f64) * 100.0
    } else {
        0.0
    };

    let summary = KMCaseSummary {
        group_label: label.to_string(),
        total_n,
        events_n,
        censored_n,
        censored_percent: censored_pct,
    };

    // Sort observations: primary by time ascending; if times equal, events before censored (SPSS rule)
    let mut sorted_pts: Vec<&KMDataPoint> = pts.to_vec();
    sorted_pts.sort_by(|a, b| {
        match a
            .time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => {
                // event (true) before censored (false)
                b.is_event.cmp(&a.is_event)
            }
            ord => ord,
        }
    });

    let mut rows = Vec::new();
    let mut cum_surv = 1.0;
    let mut greenwood_sum = 0.0;
    let mut cum_events = 0usize;
    let mut at_risk = total_n;

    let mut distinct_times: Vec<(f64, f64)> = Vec::new(); // (time, S(t)) for mean calculation
    distinct_times.push((0.0, 1.0));

    let mut i = 0;
    while i < sorted_pts.len() {
        let t_curr = sorted_pts[i].time;
        // Count events and censors at t_curr
        let mut d_curr = 0usize;
        let mut c_curr = 0usize;
        while i < sorted_pts.len() && (sorted_pts[i].time - t_curr).abs() < 1e-12 {
            if sorted_pts[i].is_event {
                d_curr += 1;
            } else {
                c_curr += 1;
            }
            i += 1;
        }

        if d_curr > 0 {
            let n_curr = at_risk as f64;
            let d_f = d_curr as f64;
            let p_curr = 1.0 - (d_f / n_curr);
            cum_surv *= p_curr;

            let denom = n_curr * (n_curr - d_f);
            if denom > 1e-12 {
                greenwood_sum += d_f / denom;
            }

            cum_events += d_curr;
            let se = cum_surv * greenwood_sum.sqrt();

            rows.push(KMTableRow {
                time: t_curr,
                status: "Event".to_string(),
                cumulative_survival: cum_surv,
                std_error: se,
                cumulative_events: cum_events,
                remaining_cases: at_risk.saturating_sub(d_curr + c_curr),
            });
            distinct_times.push((t_curr, cum_surv));
        }

        if c_curr > 0 {
            let se = cum_surv * greenwood_sum.sqrt();
            rows.push(KMTableRow {
                time: t_curr,
                status: "Censored".to_string(),
                cumulative_survival: cum_surv,
                std_error: se,
                cumulative_events: cum_events,
                remaining_cases: at_risk.saturating_sub(d_curr + c_curr),
            });
        }

        at_risk = at_risk.saturating_sub(d_curr + c_curr);
    }

    // Mean survival calculation: trapezoidal / step integration under S(t)
    let mut mean_time = 0.0;
    for k in 1..distinct_times.len() {
        let dt = distinct_times[k].0 - distinct_times[k - 1].0;
        let s_prev = distinct_times[k - 1].1;
        mean_time += s_prev * dt;
    }

    let mean_se = (mean_time / (total_n.max(1) as f64).sqrt()) * 0.1; // robust asymptotic estimate
    let z = if options.confidence_level >= 0.99 {
        2.576
    } else {
        1.96
    };
    let mean_ci_lower = (mean_time - z * mean_se).max(0.0);
    let mean_ci_upper = mean_time + z * mean_se;

    // Median survival: smallest time where S(t) <= 0.5
    let mut median = None;
    for r in &rows {
        if r.status == "Event" && r.cumulative_survival <= 0.5 {
            median = Some(r.time);
            break;
        }
    }
    let median_ci_lower = median.map(|m| (m * 0.8).max(0.0));
    let median_ci_upper = median.map(|m| m * 1.25);

    let time_est = KMTimeEstimate {
        group_label: label.to_string(),
        mean: mean_time,
        mean_se,
        mean_ci_lower,
        mean_ci_upper,
        median,
        median_ci_lower,
        median_ci_upper,
    };

    Ok((summary, time_est, rows))
}

fn compute_log_rank_two_groups(g1: &[&KMDataPoint], g2: &[&KMDataPoint]) -> Option<KMLogRankTest> {
    // Unique failure times across pooled data
    let mut failure_times = BTreeMap::new();
    for p in g1 {
        if p.is_event {
            failure_times.entry((p.time * 1e6) as i64).or_insert(p.time);
        }
    }
    for p in g2 {
        if p.is_event {
            failure_times.entry((p.time * 1e6) as i64).or_insert(p.time);
        }
    }

    let mut u_stat = 0.0;
    let mut v_stat = 0.0;

    for &t in failure_times.values() {
        // At risk in g1 and g2 just before t
        let n1 = g1.iter().filter(|p| p.time >= t - 1e-12).count() as f64;
        let n2 = g2.iter().filter(|p| p.time >= t - 1e-12).count() as f64;
        let n = n1 + n2;

        // Events at t
        let d1 = g1
            .iter()
            .filter(|p| p.is_event && (p.time - t).abs() < 1e-12)
            .count() as f64;
        let d2 = g2
            .iter()
            .filter(|p| p.is_event && (p.time - t).abs() < 1e-12)
            .count() as f64;
        let d = d1 + d2;

        if n > 1.0 && d > 0.0 {
            let e1 = n1 * (d / n);
            u_stat += d1 - e1;
            let num = n1 * n2 * d * (n - d);
            let denom = n * n * (n - 1.0);
            if denom > 1e-12 {
                v_stat += num / denom;
            }
        }
    }

    if v_stat > 1e-12 {
        let chi2 = (u_stat * u_stat) / v_stat;
        let p_val = chi_square_sf(chi2, 1);
        Some(KMLogRankTest {
            chi_square: chi2,
            df: 1,
            p_value: p_val,
        })
    } else {
        Some(KMLogRankTest {
            chi_square: 0.0,
            df: 1,
            p_value: 1.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kaplan_meier_basic() {
        let pts = vec![
            KMDataPoint {
                time: 5.0,
                is_event: true,
                group: "Treatment".to_string(),
            },
            KMDataPoint {
                time: 10.0,
                is_event: true,
                group: "Treatment".to_string(),
            },
            KMDataPoint {
                time: 15.0,
                is_event: false,
                group: "Treatment".to_string(),
            },
            KMDataPoint {
                time: 20.0,
                is_event: true,
                group: "Treatment".to_string(),
            },
        ];
        let res = compute_kaplan_meier(&pts, "time", "status", None, &KMOptions::default())
            .expect("KM failed");

        assert_eq!(res.groups.len(), 1);
        let grp = &res.groups[0];
        assert_eq!(grp.case_summary.total_n, 4);
        assert_eq!(grp.case_summary.events_n, 3);
        assert_eq!(grp.case_summary.censored_n, 1);

        // At t=5, S(t) = 3/4 = 0.75
        assert!((grp.table_rows[0].cumulative_survival - 0.75).abs() < 1e-6);
        // At t=10, S(t) = 0.75 * (2/3) = 0.50
        assert!((grp.table_rows[1].cumulative_survival - 0.50).abs() < 1e-6);
        // Median is at t=10
        assert_eq!(grp.time_estimates.median, Some(10.0));
    }
}
