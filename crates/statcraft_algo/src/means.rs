use crate::anova_oneway::{f_distribution_sf, AlgoError};
use crate::frequencies::compute_percentile;
use crate::means_types::{
    MeansAnovaRow, MeansAnovaTable, MeansAssociationMeasures, MeansCaseProcessingSummary,
    MeansCellStats, MeansOptions, MeansTableResult,
};
use std::cmp::Ordering;
use std::collections::BTreeMap;

fn compare_category_keys(a: &str, b: &str) -> Ordering {
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(Ordering::Equal),
        _ => a.cmp(b),
    }
}

fn compute_single_cell_stats(
    group_label: &str,
    values_with_weights: &[(f64, f64)],
) -> MeansCellStats {
    if values_with_weights.is_empty() {
        return MeansCellStats {
            group_label: group_label.to_string(),
            count: 0,
            weighted_count: 0.0,
            mean: None,
            std_dev: None,
            variance: None,
            median: None,
            min: None,
            max: None,
            sum: None,
        };
    }

    let count = values_with_weights.len();
    let mut weighted_count = 0.0;
    let mut sum = 0.0;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut sorted_vals: Vec<f64> = Vec::with_capacity(count);

    for &(val, w) in values_with_weights {
        weighted_count += w;
        sum += val * w;
        if val < min {
            min = val;
        }
        if val > max {
            max = val;
        }
        sorted_vals.push(val);
    }

    let mean = if weighted_count > 0.0 {
        Some(sum / weighted_count)
    } else {
        None
    };

    let (variance, std_dev) = if let Some(m) = mean {
        if weighted_count > 1.0 {
            let ss: f64 = values_with_weights
                .iter()
                .map(|&(v, w)| w * (v - m).powi(2))
                .sum();
            let var = ss / (weighted_count - 1.0);
            let sd = if var >= 0.0 { var.sqrt() } else { 0.0 };
            (Some(var), Some(sd))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let median = compute_percentile(&sorted_vals, 0.50);

    MeansCellStats {
        group_label: group_label.to_string(),
        count,
        weighted_count,
        mean,
        std_dev,
        variance,
        median,
        min: if min.is_finite() { Some(min) } else { None },
        max: if max.is_finite() { Some(max) } else { None },
        sum: Some(sum),
    }
}

pub fn compute_means_breakdown(
    dep_name: &str,
    factor_name: &str,
    dep_values: &[Option<f64>],
    factor_values: &[Option<String>],
    weights: Option<&[f64]>,
    options: &MeansOptions,
) -> Result<MeansTableResult, AlgoError> {
    let total_rows = dep_values.len();
    if total_rows != factor_values.len() {
        return Err(AlgoError::InvalidInput(format!(
            "Length mismatch between dependent '{}' ({}) and factor '{}' ({})",
            dep_name,
            total_rows,
            factor_name,
            factor_values.len()
        )));
    }
    if let Some(w) = weights {
        if w.len() != total_rows {
            return Err(AlgoError::InvalidInput(format!(
                "Length mismatch between data ({}) and weights ({})",
                total_rows,
                w.len()
            )));
        }
    }

    let mut valid_records: Vec<(f64, String, f64)> = Vec::with_capacity(total_rows);
    let mut included_n = 0_usize;

    for r in 0..total_rows {
        let w = match weights {
            Some(w_slice) => {
                let weight_val = w_slice[r];
                if weight_val <= 0.0 || weight_val.is_nan() {
                    continue;
                }
                weight_val
            }
            None => 1.0,
        };

        let y_opt = dep_values[r];
        let f_opt = &factor_values[r];

        if let (Some(y), Some(f)) = (y_opt, f_opt) {
            let f_trimmed = f.trim();
            if y.is_finite()
                && !f_trimmed.is_empty()
                && f_trimmed != "."
                && f_trimmed != "NA"
                && f_trimmed != "sysmis"
            {
                valid_records.push((y, f_trimmed.to_string(), w));
                included_n += 1;
            }
        }
    }

    let excluded_n = total_rows.saturating_sub(included_n);
    let total_n = total_rows;
    let included_percent = if total_n > 0 {
        (included_n as f64 / total_n as f64) * 100.0
    } else {
        0.0
    };
    let excluded_percent = if total_n > 0 {
        (excluded_n as f64 / total_n as f64) * 100.0
    } else {
        0.0
    };

    let summary = MeansCaseProcessingSummary {
        dep_var: dep_name.to_string(),
        factor_var: factor_name.to_string(),
        included_n,
        included_percent,
        excluded_n,
        excluded_percent,
        total_n,
        total_percent: 100.0,
    };

    if valid_records.is_empty() {
        return Err(AlgoError::InsufficientData(format!(
            "No valid cases found for '{}' by '{}'",
            dep_name, factor_name
        )));
    }

    let mut group_map: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    let mut all_pairs: Vec<(f64, f64)> = Vec::with_capacity(valid_records.len());

    for (y, f, w) in valid_records {
        group_map.entry(f).or_default().push((y, w));
        all_pairs.push((y, w));
    }

    let mut group_keys: Vec<String> = group_map.keys().cloned().collect();
    group_keys.sort_by(|a, b| compare_category_keys(a, b));

    let mut cells: Vec<MeansCellStats> = Vec::with_capacity(group_keys.len());
    for key in &group_keys {
        if let Some(list) = group_map.get(key) {
            cells.push(compute_single_cell_stats(key, list));
        }
    }

    let total_cell = compute_single_cell_stats("Total", &all_pairs);

    let (anova, association) = if options.anova
        && cells.len() >= 2
        && total_cell.weighted_count > cells.len() as f64
        && total_cell.mean.is_some()
    {
        let tot_mean = total_cell.mean.unwrap_or(0.0);
        let mut ss_between = 0.0_f64;
        let mut ss_within = 0.0_f64;

        for cell in &cells {
            if let Some(grp_mean) = cell.mean {
                ss_between += cell.weighted_count * (grp_mean - tot_mean).powi(2);
            }
            if let Some(grp_var) = cell.variance {
                if cell.weighted_count > 1.0 {
                    ss_within += grp_var * (cell.weighted_count - 1.0);
                }
            }
        }

        let ss_total = ss_between + ss_within;
        let k = cells.len();
        let n_eff = total_cell.weighted_count.round() as usize;
        let df_between = k - 1;
        let df_within = n_eff.saturating_sub(k);
        let df_total = df_between + df_within;

        let ms_between = if df_between > 0 {
            Some(ss_between / (df_between as f64))
        } else {
            None
        };

        let ms_within = if df_within > 0 {
            Some(ss_within / (df_within as f64))
        } else {
            None
        };

        let f_stat = match (ms_between, ms_within) {
            (Some(ms_b), Some(ms_w)) if ms_w > 1e-12 => Some(ms_b / ms_w),
            _ => None,
        };

        let p_value = f_stat.map(|f| f_distribution_sf(f, df_between as f64, df_within as f64));

        let anova_table = MeansAnovaTable {
            between_groups: MeansAnovaRow {
                source: "Between Groups".to_string(),
                sum_of_squares: ss_between,
                df: df_between,
                mean_square: ms_between,
                f_stat,
                p_value,
            },
            within_groups: MeansAnovaRow {
                source: "Within Groups".to_string(),
                sum_of_squares: ss_within,
                df: df_within,
                mean_square: ms_within,
                f_stat: None,
                p_value: None,
            },
            total: MeansAnovaRow {
                source: "Total".to_string(),
                sum_of_squares: ss_total,
                df: df_total,
                mean_square: None,
                f_stat: None,
                p_value: None,
            },
        };

        let assoc = if ss_total > 1e-12 && ss_between >= 0.0 {
            let eta2 = (ss_between / ss_total).clamp(0.0, 1.0);
            let eta = eta2.sqrt();
            Some(MeansAssociationMeasures {
                eta,
                eta_squared: eta2,
            })
        } else {
            Some(MeansAssociationMeasures {
                eta: 0.0,
                eta_squared: 0.0,
            })
        };

        (Some(anova_table), assoc)
    } else {
        (None, None)
    };

    Ok(MeansTableResult {
        dep_var: dep_name.to_string(),
        factor_var: factor_name.to_string(),
        summary,
        cells,
        total: total_cell,
        anova,
        association,
    })
}
