use crate::crosstabs::regularized_incomplete_gamma_q;
use crate::nonparam_twosample_types::{MannWhitneyResult, RankGroupStats};

pub use crate::nonparam_related::{
    compute_wilcoxon_signed_rank, compute_wilcoxon_signed_rank_from_slices,
};

/// Computes fractional mid-ranks for an array of floats and calculates the tie sum `T = sum(t_i^3 - t_i)`.
/// Returns a vector of ranks matching the sorted order of indices, and the tie sum.
pub fn compute_midranks_with_ties(sorted_vals: &[f64]) -> (Vec<f64>, f64) {
    let n = sorted_vals.len();
    if n == 0 {
        return (Vec::new(), 0.0);
    }

    let mut ranks = vec![0.0; n];
    let mut tie_sum = 0.0;
    let mut i = 0;

    while i < n {
        let mut j = i + 1;
        while j < n && (sorted_vals[j] - sorted_vals[i]).abs() < 1e-12 {
            j += 1;
        }

        let tie_count = (j - i) as f64;
        // 1-based ranks from (i + 1) to j. Average rank:
        let avg_rank = ((i + 1) + j) as f64 / 2.0;

        for r in ranks.iter_mut().take(j).skip(i) {
            *r = avg_rank;
        }

        if tie_count > 1.0 {
            tie_sum += tie_count * tie_count * tie_count - tie_count;
        }

        i = j;
    }

    (ranks, tie_sum)
}

fn matches_group(s1: &str, s2: &str) -> bool {
    if s1.trim().eq_ignore_ascii_case(s2.trim()) {
        return true;
    }
    if let (Ok(n1), Ok(n2)) = (s1.trim().parse::<f64>(), s2.trim().parse::<f64>()) {
        if (n1 - n2).abs() < 1e-9 {
            return true;
        }
    }
    false
}

/// Computes the Mann-Whitney U test and Wilcoxon W test for two independent samples.
///
/// Matches SPSS `NPAR TESTS /M-W` and `scipy.stats.mannwhitneyu`:
/// - Mid-ranks for tied values
/// - Tie-corrected variance
/// - Asymptotic two-tailed p-value from standard normal distribution
pub fn compute_mann_whitney(
    var_name: &str,
    grouping_var: &str,
    var_values: &[Option<f64>],
    group_values: &[Option<String>],
    group1_val: &str,
    group2_val: &str,
) -> MannWhitneyResult {
    let mut g1_data = Vec::new();
    let mut g2_data = Vec::new();

    let n = var_values.len().min(group_values.len());
    for i in 0..n {
        if let (Some(val), Some(grp)) = (&var_values[i], &group_values[i]) {
            if matches_group(grp, group1_val) {
                g1_data.push(*val);
            } else if matches_group(grp, group2_val) {
                g2_data.push(*val);
            }
        }
    }

    compute_mann_whitney_from_slices(
        var_name,
        grouping_var,
        &g1_data,
        &g2_data,
        group1_val,
        group2_val,
    )
}

/// Computes Mann-Whitney U from raw slices of floats for the two groups.
pub fn compute_mann_whitney_from_slices(
    var_name: &str,
    grouping_var: &str,
    g1: &[f64],
    g2: &[f64],
    group1_val: &str,
    group2_val: &str,
) -> MannWhitneyResult {
    let n1 = g1.len();
    let n2 = g2.len();
    let total_n = n1 + n2;

    if n1 == 0 || n2 == 0 {
        return MannWhitneyResult {
            variable: var_name.to_string(),
            grouping_var: grouping_var.to_string(),
            group1_stats: RankGroupStats {
                group_val: group1_val.to_string(),
                n: n1,
                mean_rank: 0.0,
                sum_ranks: 0.0,
            },
            group2_stats: RankGroupStats {
                group_val: group2_val.to_string(),
                n: n2,
                mean_rank: 0.0,
                sum_ranks: 0.0,
            },
            total_n,
            mann_whitney_u: 0.0,
            wilcoxon_w: 0.0,
            z_stat: 0.0,
            asymp_sig_2tailed: 1.0,
        };
    }

    // Combine values with group tag (0 = group 1, 1 = group 2)
    let mut combined: Vec<(f64, u8)> = Vec::with_capacity(total_n);
    for &x in g1 {
        combined.push((x, 0));
    }
    for &y in g2 {
        combined.push((y, 1));
    }

    // Sort ascending by value (stable sort)
    combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let sorted_vals: Vec<f64> = combined.iter().map(|item| item.0).collect();
    let (ranks, tie_sum) = compute_midranks_with_ties(&sorted_vals);

    let mut sum_ranks_1 = 0.0;
    let mut sum_ranks_2 = 0.0;

    for (idx, &(_, tag)) in combined.iter().enumerate() {
        if tag == 0 {
            sum_ranks_1 += ranks[idx];
        } else {
            sum_ranks_2 += ranks[idx];
        }
    }

    let n1_f = n1 as f64;
    let n2_f = n2 as f64;
    let n_f = total_n as f64;

    let mean_rank_1 = sum_ranks_1 / n1_f;
    let mean_rank_2 = sum_ranks_2 / n2_f;

    let u1 = sum_ranks_1 - (n1_f * (n1_f + 1.0)) / 2.0;
    let u2 = sum_ranks_2 - (n2_f * (n2_f + 1.0)) / 2.0;

    let u = u1.min(u2);

    // In SPSS: Wilcoxon W is the sum of ranks for the group with the smaller mean rank.
    let wilcoxon_w = if mean_rank_1 <= mean_rank_2 {
        sum_ranks_1
    } else {
        sum_ranks_2
    };

    // Expected value and variance of U with ties correction
    let mean_u = (n1_f * n2_f) / 2.0;
    let var_u = if total_n > 1 {
        let denom = n_f * (n_f - 1.0);
        let correction = if denom > 0.0 { tie_sum / denom } else { 0.0 };
        (n1_f * n2_f / 12.0) * ((n_f + 1.0) - correction)
    } else {
        0.0
    };

    let (z, p_val) = if var_u > 1e-14 {
        let std_u = var_u.sqrt();
        let z_val = (u - mean_u) / std_u;
        let p = regularized_incomplete_gamma_q(0.5, (z_val * z_val) / 2.0).clamp(0.0, 1.0);
        (z_val, p)
    } else {
        (0.0, 1.0)
    };

    MannWhitneyResult {
        variable: var_name.to_string(),
        grouping_var: grouping_var.to_string(),
        group1_stats: RankGroupStats {
            group_val: group1_val.to_string(),
            n: n1,
            mean_rank: mean_rank_1,
            sum_ranks: sum_ranks_1,
        },
        group2_stats: RankGroupStats {
            group_val: group2_val.to_string(),
            n: n2,
            mean_rank: mean_rank_2,
            sum_ranks: sum_ranks_2,
        },
        total_n,
        mann_whitney_u: u,
        wilcoxon_w,
        z_stat: z,
        asymp_sig_2tailed: p_val,
    }
}
