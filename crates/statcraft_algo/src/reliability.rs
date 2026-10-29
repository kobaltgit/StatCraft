use crate::reliability_types::{
    CaseProcessingSummary, ItemStatistic, ItemTotalStatistic, ReliabilityOptions,
    ReliabilityResult, ReliabilityStatistics, ScaleStatistics,
};

/// Computes scale reliability analysis according to the specified options.
///
/// Supports Cronbach's alpha and alpha based on standardized items,
/// item statistics, item-total statistics (scale mean/variance if deleted,
/// corrected item-total correlation, alpha if item deleted),
/// as well as case processing summary and scale statistics.
///
/// Handles listwise missing value exclusion and case weighting safely without panicking.
pub fn compute_reliability(
    items: &[(&str, &[Option<f64>])],
    weights: &[f64],
    options: &ReliabilityOptions,
) -> ReliabilityResult {
    let k = items.len();
    let n_total = items.first().map(|(_, s)| s.len()).unwrap_or(0);

    // Listwise extraction of valid cases
    let mut valid_rows: Vec<(Vec<f64>, f64)> = Vec::new();
    let mut total_cases_count = 0_usize;

    for r in 0..n_total {
        let w = if weights.is_empty() {
            1.0
        } else {
            weights.get(r).copied().unwrap_or(0.0)
        };

        if w <= 0.0 || !w.is_finite() {
            total_cases_count += 1;
            continue;
        }

        total_cases_count += 1;

        let mut row_vals = Vec::with_capacity(k);
        let mut is_valid = true;

        for (_, series) in items {
            match series.get(r).copied().flatten() {
                Some(v) if v.is_finite() => row_vals.push(v),
                _ => {
                    is_valid = false;
                    break;
                }
            }
        }

        if is_valid && row_vals.len() == k {
            valid_rows.push((row_vals, w));
        }
    }

    let w_valid_sum: f64 = valid_rows.iter().map(|(_, w)| *w).sum();
    let n_valid = if weights.is_empty() {
        valid_rows.len()
    } else {
        w_valid_sum.round() as usize
    };
    let n_excluded = total_cases_count.saturating_sub(n_valid);

    let (percent_valid, percent_excluded, percent_total) = if total_cases_count > 0 {
        let p_val = (n_valid as f64 / total_cases_count as f64) * 100.0;
        let p_ex = (n_excluded as f64 / total_cases_count as f64) * 100.0;
        (p_val, p_ex, 100.0)
    } else {
        (0.0, 0.0, 0.0)
    };

    let case_processing = CaseProcessingSummary {
        n_valid,
        percent_valid,
        n_excluded,
        percent_excluded,
        n_total: total_cases_count,
        percent_total,
    };

    if k < 2 || valid_rows.is_empty() || w_valid_sum < 2.0 {
        return ReliabilityResult {
            scale_name: options.scale_name.clone(),
            model: options.model,
            case_processing,
            reliability_stats: ReliabilityStatistics {
                cronbach_alpha: None,
                cronbach_alpha_standardized: None,
                n_of_items: k,
            },
            item_stats: Vec::new(),
            item_total_stats: Vec::new(),
            scale_stats: None,
        };
    }

    let df = w_valid_sum - 1.0;

    // Calculate means and variances for each item
    let mut item_means = vec![0.0; k];
    let mut item_vars = vec![0.0; k];
    let mut item_sds = vec![0.0; k];

    for j in 0..k {
        let sum: f64 = valid_rows.iter().map(|(row, w)| w * row[j]).sum();
        let m = sum / w_valid_sum;
        item_means[j] = m;

        let sum_sq_diff: f64 = valid_rows
            .iter()
            .map(|(row, w)| w * (row[j] - m).powi(2))
            .sum();
        let var = sum_sq_diff / df;
        item_vars[j] = var.max(0.0);
        item_sds[j] = item_vars[j].sqrt();
    }

    // Calculate total scale score for each row
    let mut total_scale_mean = 0.0;
    for &m in &item_means {
        total_scale_mean += m;
    }

    let sum_sq_total_diff: f64 = valid_rows
        .iter()
        .map(|(row, w)| {
            let row_total: f64 = row.iter().sum();
            w * (row_total - total_scale_mean).powi(2)
        })
        .sum();
    let total_scale_var = (sum_sq_total_diff / df).max(0.0);
    let total_scale_sd = total_scale_var.sqrt();

    // Cronbach's Alpha
    let sum_item_vars: f64 = item_vars.iter().sum();
    let k_f64 = k as f64;
    let cronbach_alpha = if total_scale_var > 1e-12 {
        let alpha = (k_f64 / (k_f64 - 1.0)) * (1.0 - (sum_item_vars / total_scale_var));
        Some(alpha)
    } else {
        None
    };

    // Standardized Cronbach's Alpha (based on average inter-item correlation)
    let mut sum_r = 0.0;
    let mut pair_count = 0_usize;

    for i in 0..k {
        for j in (i + 1)..k {
            let cov_ij: f64 = valid_rows
                .iter()
                .map(|(row, w)| w * (row[i] - item_means[i]) * (row[j] - item_means[j]))
                .sum::<f64>()
                / df;

            let denom = item_sds[i] * item_sds[j];
            let r_ij = if denom > 1e-12 {
                (cov_ij / denom).clamp(-1.0, 1.0)
            } else {
                0.0
            };

            sum_r += r_ij;
            pair_count += 1;
        }
    }

    let cronbach_alpha_standardized = if pair_count > 0 {
        let mean_r = sum_r / (pair_count as f64);
        let denom = 1.0 + (k_f64 - 1.0) * mean_r;
        if denom.abs() > 1e-12 {
            Some((k_f64 * mean_r) / denom)
        } else {
            None
        }
    } else {
        None
    };

    let reliability_stats = ReliabilityStatistics {
        cronbach_alpha,
        cronbach_alpha_standardized,
        n_of_items: k,
    };

    // Item Statistics
    let mut item_stats = Vec::with_capacity(k);
    if options.item_statistics {
        for j in 0..k {
            item_stats.push(ItemStatistic {
                item_name: items[j].0.to_string(),
                mean: item_means[j],
                std_dev: item_sds[j],
                n: n_valid,
            });
        }
    }

    // Item-Total Statistics
    let mut item_total_stats = Vec::with_capacity(k);
    if options.scale_if_item_deleted {
        for i in 0..k {
            let scale_mean_if_deleted = total_scale_mean - item_means[i];

            // Covariance between item i and total score
            let cov_i_total: f64 = valid_rows
                .iter()
                .map(|(row, w)| {
                    let row_total: f64 = row.iter().sum();
                    w * (row[i] - item_means[i]) * (row_total - total_scale_mean)
                })
                .sum::<f64>()
                / df;

            // Var(Total - Item_i) = Var(Total) + Var(Item_i) - 2 * Cov(Item_i, Total)
            let var_if_deleted = (total_scale_var + item_vars[i] - 2.0 * cov_i_total).max(0.0);
            let sd_if_deleted = var_if_deleted.sqrt();

            // Corrected Item-Total correlation: Cov(Item_i, Total - Item_i) / (sd_i * sd_if_deleted)
            let cov_i_subtotal = cov_i_total - item_vars[i];
            let corrected_r = if item_sds[i] > 1e-12 && sd_if_deleted > 1e-12 {
                Some((cov_i_subtotal / (item_sds[i] * sd_if_deleted)).clamp(-1.0, 1.0))
            } else {
                None
            };

            // Cronbach's alpha if item deleted
            let alpha_if_deleted = if k > 2 {
                if var_if_deleted > 1e-12 {
                    let sum_vars_deleted = sum_item_vars - item_vars[i];
                    let k_rem = (k - 1) as f64;
                    let a = (k_rem / (k_rem - 1.0)) * (1.0 - (sum_vars_deleted / var_if_deleted));
                    Some(a)
                } else {
                    None
                }
            } else {
                None
            };

            item_total_stats.push(ItemTotalStatistic {
                item_name: items[i].0.to_string(),
                scale_mean_if_deleted,
                scale_variance_if_deleted: var_if_deleted,
                corrected_item_total_correlation: corrected_r,
                cronbach_alpha_if_deleted: alpha_if_deleted,
            });
        }
    }

    // Scale Statistics
    let scale_stats = if options.scale_statistics {
        Some(ScaleStatistics {
            mean: total_scale_mean,
            variance: total_scale_var,
            std_dev: total_scale_sd,
            n_of_items: k,
        })
    } else {
        None
    };

    ReliabilityResult {
        scale_name: options.scale_name.clone(),
        model: options.model,
        case_processing,
        reliability_stats,
        item_stats,
        item_total_stats,
        scale_stats,
    }
}
