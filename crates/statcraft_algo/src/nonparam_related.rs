use crate::crosstabs::regularized_incomplete_gamma_q;
use crate::nonparam_related_types::{
    RelatedPairTestResult, SignTestDifferences, SignTestResult, WilcoxonRanksSummary,
    WilcoxonSignedRankResult,
};
use crate::nonparam_twosample::compute_midranks_with_ties;
use crate::nonparam_twosample_types::RankGroupStats;
use crate::ttest_onesample::regularized_incomplete_beta;

/// Computes Wilcoxon Signed-Rank Test for two paired variables.
///
/// Matches SPSS `NPAR TESTS /WILCOXON` and standard textbook formulas:
/// - Evaluates difference: `D_i = Y_i - X_i` (Var2 - Var1)
/// - `Negative Ranks`: `Y_i < X_i` (`D_i < 0`)
/// - `Positive Ranks`: `Y_i > X_i` (`D_i > 0`)
/// - `Ties`: `Y_i == X_i` (`D_i == 0`), excluded from ranking
/// - Fractional mid-ranks for tied absolute differences
/// - Tie-corrected variance and continuity-corrected Z
/// - Asymptotic two-tailed p-value
pub fn compute_wilcoxon_signed_rank(
    var1_name: &str,
    var2_name: &str,
    pairs: &[(Option<f64>, Option<f64>)],
) -> WilcoxonSignedRankResult {
    let mut x_vals = Vec::new();
    let mut y_vals = Vec::new();

    for &(opt_x, opt_y) in pairs {
        if let (Some(x), Some(y)) = (opt_x, opt_y) {
            x_vals.push(x);
            y_vals.push(y);
        }
    }

    compute_wilcoxon_signed_rank_from_slices(var1_name, var2_name, &x_vals, &y_vals)
}

/// Computes Wilcoxon Signed-Rank Test from paired numeric slices.
pub fn compute_wilcoxon_signed_rank_from_slices(
    var1_name: &str,
    var2_name: &str,
    x: &[f64],
    y: &[f64],
) -> WilcoxonSignedRankResult {
    let total_cases = x.len().min(y.len());
    let mut diffs: Vec<(f64, bool)> = Vec::new(); // (|diff|, is_negative)
    let mut zero_ties = 0;

    for i in 0..total_cases {
        let d = y[i] - x[i];
        if d.abs() < 1e-12 {
            zero_ties += 1;
        } else {
            diffs.push((d.abs(), d < 0.0));
        }
    }

    let n_nonzero = diffs.len();
    if n_nonzero == 0 {
        return WilcoxonSignedRankResult {
            var1: var1_name.to_string(),
            var2: var2_name.to_string(),
            ranks: WilcoxonRanksSummary {
                negative_ranks: RankGroupStats {
                    group_val: "Negative Ranks".to_string(),
                    n: 0,
                    mean_rank: 0.0,
                    sum_ranks: 0.0,
                },
                positive_ranks: RankGroupStats {
                    group_val: "Positive Ranks".to_string(),
                    n: 0,
                    mean_rank: 0.0,
                    sum_ranks: 0.0,
                },
                ties_count: zero_ties,
                total_n: total_cases,
            },
            test_statistic_w: 0.0,
            z_stat: 0.0,
            asymp_sig_2tailed: 1.0,
            based_on_negative: true,
        };
    }

    // Sort ascending by absolute difference (stable sort)
    diffs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let sorted_abs: Vec<f64> = diffs.iter().map(|item| item.0).collect();
    let (ranks, tie_sum) = compute_midranks_with_ties(&sorted_abs);

    let mut sum_neg = 0.0;
    let mut count_neg = 0;
    let mut sum_pos = 0.0;
    let mut count_pos = 0;

    for (idx, &(_, is_neg)) in diffs.iter().enumerate() {
        if is_neg {
            sum_neg += ranks[idx];
            count_neg += 1;
        } else {
            sum_pos += ranks[idx];
            count_pos += 1;
        }
    }

    let mean_neg = if count_neg > 0 {
        sum_neg / count_neg as f64
    } else {
        0.0
    };
    let mean_pos = if count_pos > 0 {
        sum_pos / count_pos as f64
    } else {
        0.0
    };

    let based_on_negative = sum_neg <= sum_pos;
    let w = if based_on_negative { sum_neg } else { sum_pos };

    // Expected value and variance
    let n_f = n_nonzero as f64;
    let mean_w = (n_f * (n_f + 1.0)) / 4.0;
    let var_w = (n_f * (n_f + 1.0) * (2.0 * n_f + 1.0) - 0.5 * tie_sum) / 24.0;

    let (z, p_val) = if var_w > 1e-14 {
        let std_w = var_w.sqrt();
        let num = if w < mean_w {
            w - mean_w + 0.5
        } else if w > mean_w {
            w - mean_w - 0.5
        } else {
            0.0
        };
        let z_val = num / std_w;
        let p = regularized_incomplete_gamma_q(0.5, (z_val * z_val) / 2.0).clamp(0.0, 1.0);
        (z_val, p)
    } else {
        (0.0, 1.0)
    };

    WilcoxonSignedRankResult {
        var1: var1_name.to_string(),
        var2: var2_name.to_string(),
        ranks: WilcoxonRanksSummary {
            negative_ranks: RankGroupStats {
                group_val: "Negative Ranks".to_string(),
                n: count_neg,
                mean_rank: mean_neg,
                sum_ranks: sum_neg,
            },
            positive_ranks: RankGroupStats {
                group_val: "Positive Ranks".to_string(),
                n: count_pos,
                mean_rank: mean_pos,
                sum_ranks: sum_pos,
            },
            ties_count: zero_ties,
            total_n: total_cases,
        },
        test_statistic_w: w,
        z_stat: z,
        asymp_sig_2tailed: p_val,
        based_on_negative,
    }
}

/// Computes the Sign Test for two paired variables.
///
/// Matches SPSS `NPAR TESTS /SIGN`:
/// - Counts differences `D_i = Y_i - X_i`
/// - Negative differences: `Y_i < X_i`
/// - Positive differences: `Y_i > X_i`
/// - Ties: `Y_i == X_i`
/// - Exact two-tailed p-value via Binomial CDF
pub fn compute_sign_test(
    var1_name: &str,
    var2_name: &str,
    pairs: &[(Option<f64>, Option<f64>)],
) -> SignTestResult {
    let mut x_vals = Vec::new();
    let mut y_vals = Vec::new();

    for &(opt_x, opt_y) in pairs {
        if let (Some(x), Some(y)) = (opt_x, opt_y) {
            x_vals.push(x);
            y_vals.push(y);
        }
    }

    compute_sign_test_from_slices(var1_name, var2_name, &x_vals, &y_vals)
}

/// Computes the Sign Test from paired numeric slices.
pub fn compute_sign_test_from_slices(
    var1_name: &str,
    var2_name: &str,
    x: &[f64],
    y: &[f64],
) -> SignTestResult {
    let total_cases = x.len().min(y.len());
    let mut neg_count = 0;
    let mut pos_count = 0;
    let mut tie_count = 0;

    for i in 0..total_cases {
        let d = y[i] - x[i];
        if d.abs() < 1e-12 {
            tie_count += 1;
        } else if d < 0.0 {
            neg_count += 1;
        } else {
            pos_count += 1;
        }
    }

    let n = neg_count + pos_count;
    let k = neg_count.min(pos_count);

    let p_val = if n == 0 || k * 2 == n {
        1.0
    } else {
        let cdf_k = regularized_incomplete_beta(0.5, (n - k) as f64, (k + 1) as f64);
        (2.0 * cdf_k).clamp(0.0, 1.0)
    };

    SignTestResult {
        var1: var1_name.to_string(),
        var2: var2_name.to_string(),
        diffs: SignTestDifferences {
            negative_count: neg_count,
            positive_count: pos_count,
            ties_count: tie_count,
            total_n: total_cases,
        },
        exact_sig_2tailed: p_val,
    }
}

/// Computes requested related samples tests (Wilcoxon and/or Sign) for a pair of variables.
pub fn compute_related_samples(
    var1_name: &str,
    var2_name: &str,
    pairs: &[(Option<f64>, Option<f64>)],
    do_wilcoxon: bool,
    do_sign: bool,
) -> RelatedPairTestResult {
    let mut x_vals = Vec::new();
    let mut y_vals = Vec::new();

    for &(opt_x, opt_y) in pairs {
        if let (Some(x), Some(y)) = (opt_x, opt_y) {
            x_vals.push(x);
            y_vals.push(y);
        }
    }

    let wilcoxon = if do_wilcoxon {
        Some(compute_wilcoxon_signed_rank_from_slices(
            var1_name, var2_name, &x_vals, &y_vals,
        ))
    } else {
        None
    };

    let sign = if do_sign {
        Some(compute_sign_test_from_slices(
            var1_name, var2_name, &x_vals, &y_vals,
        ))
    } else {
        None
    };

    RelatedPairTestResult {
        var1: var1_name.to_string(),
        var2: var2_name.to_string(),
        wilcoxon,
        sign,
    }
}
