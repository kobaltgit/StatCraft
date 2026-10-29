use crate::crosstabs::chi_square_sf;
use crate::nonparam_twosample::compute_midranks_with_ties;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KruskalWallisGroupStats {
    pub group_val: String,
    pub n: usize,
    pub mean_rank: f64,
    pub sum_ranks: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KruskalWallisResult {
    pub variable: String,
    pub grouping_var: String,
    pub groups: Vec<KruskalWallisGroupStats>,
    pub total_n: usize,
    pub kruskal_wallis_h: f64,
    pub df: usize,
    pub asymp_sig: f64,
}

/// Computes the Kruskal-Wallis H test from raw variable values, grouping variable values,
/// and a specified inclusive range [min_range, max_range].
pub fn compute_kruskal_wallis(
    var_name: &str,
    grouping_var: &str,
    var_values: &[Option<f64>],
    group_values: &[Option<String>],
    min_range: f64,
    max_range: f64,
) -> KruskalWallisResult {
    let n = var_values.len().min(group_values.len());
    let mut parsed_data: Vec<(f64, f64, String)> = Vec::new();

    for i in 0..n {
        if let (Some(val), Some(grp_str)) = (&var_values[i], &group_values[i]) {
            let grp_trimmed = grp_str.trim();
            if let Ok(grp_num) = grp_trimmed.parse::<f64>() {
                if grp_num >= min_range - 1e-9 && grp_num <= max_range + 1e-9 {
                    parsed_data.push((*val, grp_num, grp_trimmed.to_string()));
                }
            }
        }
    }

    // Identify unique groups ordered numerically by group number
    let mut group_keys: Vec<(f64, String)> = parsed_data
        .iter()
        .map(|(_, num, str_val)| (*num, str_val.clone()))
        .collect();
    group_keys.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    group_keys.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-9);

    let mut grouped_slices: Vec<(String, Vec<f64>)> = Vec::with_capacity(group_keys.len());
    for (key_num, key_str) in group_keys {
        let values: Vec<f64> = parsed_data
            .iter()
            .filter(|(_, num, _)| (*num - key_num).abs() < 1e-9)
            .map(|(val, _, _)| *val)
            .collect();
        grouped_slices.push((key_str, values));
    }

    compute_kruskal_wallis_from_groups(var_name, grouping_var, &grouped_slices)
}

/// Computes the Kruskal-Wallis H test from pre-grouped slices of observations.
///
/// Matches SPSS `NPAR TESTS /K-W` and `scipy.stats.kruskal`:
/// - Mid-ranks for tied values
/// - Tie-corrected H statistic: H_adj = H / [1 - sum(t^3 - t) / (N^3 - N)]
/// - Degrees of freedom: df = k - 1
/// - Asymptotic p-value from chi-square distribution with df degrees of freedom
pub fn compute_kruskal_wallis_from_groups(
    var_name: &str,
    grouping_var: &str,
    groups: &[(String, Vec<f64>)],
) -> KruskalWallisResult {
    let k = groups.len();
    let total_n: usize = groups.iter().map(|(_, vals)| vals.len()).sum();

    if k < 2 || total_n == 0 {
        let group_stats = groups
            .iter()
            .map(|(label, vals)| KruskalWallisGroupStats {
                group_val: label.clone(),
                n: vals.len(),
                mean_rank: 0.0,
                sum_ranks: 0.0,
            })
            .collect();

        return KruskalWallisResult {
            variable: var_name.to_string(),
            grouping_var: grouping_var.to_string(),
            groups: group_stats,
            total_n,
            kruskal_wallis_h: 0.0,
            df: 0,
            asymp_sig: 1.0,
        };
    }

    // Combine all values with group index
    let mut combined: Vec<(f64, usize)> = Vec::with_capacity(total_n);
    for (grp_idx, (_, vals)) in groups.iter().enumerate() {
        for &val in vals {
            combined.push((val, grp_idx));
        }
    }

    // Stable sort ascending by value
    combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let sorted_vals: Vec<f64> = combined.iter().map(|(val, _)| *val).collect();
    let (ranks, tie_sum) = compute_midranks_with_ties(&sorted_vals);

    let mut sum_ranks = vec![0.0; k];
    for (idx, &(_, grp_idx)) in combined.iter().enumerate() {
        sum_ranks[grp_idx] += ranks[idx];
    }

    let mut group_stats = Vec::with_capacity(k);
    let mut sum_sq_div_n = 0.0;

    for (grp_idx, (label, vals)) in groups.iter().enumerate() {
        let n_i = vals.len();
        let r_i = sum_ranks[grp_idx];
        let mean_rank = if n_i > 0 { r_i / (n_i as f64) } else { 0.0 };

        if n_i > 0 {
            sum_sq_div_n += (r_i * r_i) / (n_i as f64);
        }

        group_stats.push(KruskalWallisGroupStats {
            group_val: label.clone(),
            n: n_i,
            mean_rank,
            sum_ranks: r_i,
        });
    }

    let n_f = total_n as f64;
    // Unadjusted H: [12 / (N*(N+1))] * sum(R_i^2 / n_i) - 3*(N+1)
    let raw_h = (12.0 / (n_f * (n_f + 1.0))) * sum_sq_div_n - 3.0 * (n_f + 1.0);

    // Tie correction factor: 1 - sum(t^3 - t) / (N^3 - N)
    let denom = n_f * n_f * n_f - n_f;
    let tie_factor = if denom > 1e-12 {
        1.0 - (tie_sum / denom)
    } else {
        1.0
    };

    let adj_h = if tie_factor > 1e-12 {
        (raw_h / tie_factor).max(0.0)
    } else {
        raw_h.max(0.0)
    };

    let df = k - 1;
    let asymp_sig = chi_square_sf(adj_h, df);

    KruskalWallisResult {
        variable: var_name.to_string(),
        grouping_var: grouping_var.to_string(),
        groups: group_stats,
        total_n,
        kruskal_wallis_h: adj_h,
        df,
        asymp_sig,
    }
}
