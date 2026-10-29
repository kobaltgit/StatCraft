use crate::nonparam_related::{
    compute_related_samples, compute_sign_test, compute_sign_test_from_slices,
    compute_wilcoxon_signed_rank, compute_wilcoxon_signed_rank_from_slices,
};

#[test]
fn test_wilcoxon_signed_rank_oracle_with_ties() {
    let pairs = vec![
        (Some(15.0), Some(18.0)),
        (Some(16.0), Some(16.0)),
        (Some(17.0), Some(20.0)),
        (Some(25.0), Some(22.0)),
        (Some(12.0), Some(15.0)),
    ];
    let res_opt = compute_wilcoxon_signed_rank("pre", "post", &pairs);
    assert_eq!(res_opt.ranks.total_n, 5);
    assert_eq!(res_opt.test_statistic_w, 2.5);

    // Benchmark dataset:
    // Pre (x):  [15, 16, 17, 25, 12]
    // Post (y): [18, 16, 20, 22, 15]
    // D = y - x: [+3, 0, +3, -3, +3]
    // Zero diff (tie): 1
    // Non-zero: 4 (|D| = 3, 3, 3, 3; all tied with midrank 2.5)
    // Negative ranks: 1 (sum = 2.5, mean = 2.5)
    // Positive ranks: 3 (sum = 7.5, mean = 2.5)
    // tie_sum = 4^3 - 4 = 60
    // mu = 4 * 5 / 4 = 5.0
    // var = (4 * 5 * 9 - 30) / 24 = 6.25 -> std = 2.5
    // Z = (2.5 - 5.0 + 0.5) / 2.5 = -0.80
    let pre = [15.0, 16.0, 17.0, 25.0, 12.0];
    let post = [18.0, 16.0, 20.0, 22.0, 15.0];

    let res = compute_wilcoxon_signed_rank_from_slices("pre", "post", &pre, &post);

    assert_eq!(res.ranks.total_n, 5);
    assert_eq!(res.ranks.ties_count, 1);
    assert_eq!(res.ranks.negative_ranks.n, 1);
    assert_eq!(res.ranks.negative_ranks.sum_ranks, 2.5);
    assert_eq!(res.ranks.negative_ranks.mean_rank, 2.5);
    assert_eq!(res.ranks.positive_ranks.n, 3);
    assert_eq!(res.ranks.positive_ranks.sum_ranks, 7.5);
    assert_eq!(res.ranks.positive_ranks.mean_rank, 2.5);
    assert_eq!(res.test_statistic_w, 2.5);
    assert!(res.based_on_negative);

    // Z = -0.80
    assert!((res.z_stat - (-0.80)).abs() < 1e-6);
    // Two-tailed p-value for Z = -0.80: ~0.4237108
    assert!((res.asymp_sig_2tailed - 0.42371).abs() < 1e-4);
}

#[test]
fn test_sign_test_oracle() {
    let pairs = vec![
        (Some(15.0), Some(18.0)),
        (Some(16.0), Some(16.0)),
        (Some(17.0), Some(20.0)),
        (Some(25.0), Some(22.0)),
        (Some(12.0), Some(15.0)),
    ];
    let res_opt = compute_sign_test("pre", "post", &pairs);
    assert_eq!(res_opt.diffs.total_n, 5);
    assert_eq!(res_opt.diffs.positive_count, 3);

    let pre = [15.0, 16.0, 17.0, 25.0, 12.0];
    let post = [18.0, 16.0, 20.0, 22.0, 15.0];

    let res = compute_sign_test_from_slices("pre", "post", &pre, &post);

    assert_eq!(res.diffs.total_n, 5);
    assert_eq!(res.diffs.ties_count, 1);
    assert_eq!(res.diffs.negative_count, 1);
    assert_eq!(res.diffs.positive_count, 3);
    // Binomial(4, 0.5): P(X <= 1) = 5/16 = 0.3125 -> 2-tailed = 0.625
    assert!((res.exact_sig_2tailed - 0.625).abs() < 1e-6);
}

#[test]
fn test_sign_test_equal_halves() {
    // 3 negative, 3 positive -> 2-tailed exact sig = 1.0
    let pre = [10.0, 10.0, 10.0, 20.0, 20.0, 20.0];
    let post = [20.0, 20.0, 20.0, 10.0, 10.0, 10.0];

    let res = compute_sign_test_from_slices("pre", "post", &pre, &post);
    assert_eq!(res.diffs.negative_count, 3);
    assert_eq!(res.diffs.positive_count, 3);
    assert_eq!(res.exact_sig_2tailed, 1.0);
}

#[test]
fn test_wilcoxon_all_zero_ties() {
    let pre = [10.0, 20.0, 30.0];
    let post = [10.0, 20.0, 30.0];

    let res = compute_wilcoxon_signed_rank_from_slices("pre", "post", &pre, &post);
    assert_eq!(res.ranks.ties_count, 3);
    assert_eq!(res.ranks.negative_ranks.n, 0);
    assert_eq!(res.ranks.positive_ranks.n, 0);
    assert_eq!(res.z_stat, 0.0);
    assert_eq!(res.asymp_sig_2tailed, 1.0);
}

#[test]
fn test_related_samples_with_missing_values() {
    let pairs = vec![
        (Some(10.0), Some(12.0)),
        (None, Some(15.0)),
        (Some(20.0), None),
        (Some(30.0), Some(30.0)),
        (Some(40.0), Some(35.0)),
    ];

    let res = compute_related_samples("pre", "post", &pairs, true, true);
    assert!(res.wilcoxon.is_some());
    assert!(res.sign.is_some());

    let w = res.wilcoxon.unwrap();
    assert_eq!(w.ranks.total_n, 3); // 3 valid pairs: (10, 12), (30, 30), (40, 35)
    assert_eq!(w.ranks.ties_count, 1);
    assert_eq!(w.ranks.negative_ranks.n, 1); // 35 < 40
    assert_eq!(w.ranks.positive_ranks.n, 1); // 12 > 10

    let s = res.sign.unwrap();
    assert_eq!(s.diffs.total_n, 3);
    assert_eq!(s.diffs.ties_count, 1);
    assert_eq!(s.diffs.negative_count, 1);
    assert_eq!(s.diffs.positive_count, 1);
    assert_eq!(s.exact_sig_2tailed, 1.0);
}
