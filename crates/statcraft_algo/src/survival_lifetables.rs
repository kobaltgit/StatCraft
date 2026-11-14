//! Actuarial Life Tables procedure (SURVIVAL / stat.survival.lifetables).
//! Computes interval-based mortality rates, cumulative survival, probability density,
//! and hazard rate per IBM SPSS Statistics 24 Algorithms Guide.

use crate::anova_oneway::AlgoError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeTableOptions {
    pub interval_start: f64,
    pub interval_end: f64,
    pub interval_width: f64,
}

impl Default for LifeTableOptions {
    fn default() -> Self {
        Self {
            interval_start: 0.0,
            interval_end: 60.0,
            interval_width: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LifeTableRow {
    pub interval_start: f64,
    pub interval_end: f64,
    pub cases_entering: usize,
    pub cases_withdrawn: usize,
    pub cases_exposed: f64,
    pub terminal_events: usize,
    pub proportion_terminating: f64,
    pub proportion_surviving: f64,
    pub cumulative_surviving: f64,
    pub cumulative_surviving_se: f64,
    pub probability_density: f64,
    pub probability_density_se: f64,
    pub hazard_rate: f64,
    pub hazard_rate_se: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LifeTableResult {
    pub time_variable: String,
    pub status_variable: String,
    pub rows: Vec<LifeTableRow>,
    pub total_cases: usize,
}

pub struct LifeTableCase {
    pub time: f64,
    pub is_event: bool,
}

/// Compute Actuarial Life Table for a time/event variable.
pub fn compute_life_table(
    cases: &[LifeTableCase],
    time_var: &str,
    status_var: &str,
    options: &LifeTableOptions,
) -> Result<LifeTableResult, AlgoError> {
    if cases.is_empty() {
        return Err(AlgoError::InsufficientData("Cases are empty".to_string()));
    }
    if options.interval_width <= 0.0 {
        return Err(AlgoError::InvalidInput(
            "Interval width must be strictly positive".to_string(),
        ));
    }
    if options.interval_end <= options.interval_start {
        return Err(AlgoError::InvalidInput(
            "Interval end must be greater than interval start".to_string(),
        ));
    }

    let mut intervals = Vec::new();
    let mut t = options.interval_start;
    while t < options.interval_end - 1e-9 {
        let t_next = (t + options.interval_width).min(options.interval_end);
        intervals.push((t, t_next));
        t = t_next;
    }

    let num_intervals = intervals.len();
    let mut events_in_int = vec![0usize; num_intervals];
    let mut withdrawn_in_int = vec![0usize; num_intervals];

    for c in cases {
        if c.time < options.interval_start {
            continue;
        }
        let mut placed = false;
        for (idx, &(t_s, t_e)) in intervals.iter().enumerate() {
            // Semi-open interval [t_s, t_e), or closed [t_s, t_e] for the last interval
            let in_interval = if idx == num_intervals - 1 {
                c.time >= t_s && c.time <= t_e
            } else {
                c.time >= t_s && c.time < t_e
            };

            if in_interval {
                if c.is_event {
                    events_in_int[idx] += 1;
                } else {
                    withdrawn_in_int[idx] += 1;
                }
                placed = true;
                break;
            }
        }
        if !placed && c.time >= options.interval_end {
            // Withdrawn after end of study
            withdrawn_in_int[num_intervals - 1] += 1;
        }
    }

    let mut rows = Vec::with_capacity(num_intervals);
    let mut current_entering = cases.len();
    let mut cum_p = 1.0f64;
    let mut cum_se_sum = 0.0f64;

    for idx in 0..num_intervals {
        let (t_s, t_e) = intervals[idx];
        let h = t_e - t_s;
        let d = events_in_int[idx];
        let w = withdrawn_in_int[idx];

        let n_exposed = (current_entering as f64) - (w as f64 / 2.0);
        let q = if n_exposed > 1e-12 {
            ((d as f64) / n_exposed).min(1.0)
        } else {
            0.0
        };
        let p = 1.0 - q;

        let cum_surv = cum_p;
        let cum_surv_se = cum_surv * cum_se_sum.sqrt();

        // Density f = (P_k * q_k) / h_k
        let f = if h > 1e-12 { (cum_surv * q) / h } else { 0.0 };

        // Hazard rate h = (2 * q_k) / (h_k * (1 + p_k))
        let denom_h = h * (1.0 + p);
        let hazard = if denom_h > 1e-12 {
            (2.0 * q) / denom_h
        } else {
            0.0
        };

        let f_se = if h > 1e-12 && n_exposed > 1e-12 && q > 1e-12 {
            let inner = cum_se_sum + (p / (n_exposed * q));
            (f * inner.sqrt()).max(0.0)
        } else {
            0.0
        };

        let hazard_se = if d > 0 {
            hazard / (d as f64).sqrt()
        } else {
            0.0
        };

        rows.push(LifeTableRow {
            interval_start: t_s,
            interval_end: t_e,
            cases_entering: current_entering,
            cases_withdrawn: w,
            cases_exposed: n_exposed,
            terminal_events: d,
            proportion_terminating: q,
            proportion_surviving: p,
            cumulative_surviving: cum_surv,
            cumulative_surviving_se: cum_surv_se,
            probability_density: f,
            probability_density_se: f_se,
            hazard_rate: hazard,
            hazard_rate_se: hazard_se,
        });

        if n_exposed > 1e-12 && p > 1e-12 {
            cum_se_sum += q / (n_exposed * p);
        }
        cum_p *= p;
        current_entering = current_entering.saturating_sub(d + w);
    }

    Ok(LifeTableResult {
        time_variable: time_var.to_string(),
        status_variable: status_var.to_string(),
        rows,
        total_cases: cases.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_life_table_basic() {
        let cases = vec![
            LifeTableCase {
                time: 5.0,
                is_event: true,
            },
            LifeTableCase {
                time: 8.0,
                is_event: false,
            },
            LifeTableCase {
                time: 15.0,
                is_event: true,
            },
            LifeTableCase {
                time: 25.0,
                is_event: true,
            },
        ];
        let options = LifeTableOptions {
            interval_start: 0.0,
            interval_end: 30.0,
            interval_width: 10.0,
        };

        let res =
            compute_life_table(&cases, "time", "status", &options).expect("Life table failed");
        assert_eq!(res.rows.len(), 3);
        assert_eq!(res.rows[0].cases_entering, 4);
        assert_eq!(res.rows[0].terminal_events, 1);
        assert_eq!(res.rows[0].cases_withdrawn, 1);
        assert_eq!(res.rows[0].cases_exposed, 3.5);
    }
}
