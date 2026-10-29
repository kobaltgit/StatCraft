use crate::nonparam_twosample::{
    compute_mann_whitney, compute_mann_whitney_from_slices, compute_midranks_with_ties,
    compute_wilcoxon_signed_rank_from_slices,
};

#[test]
fn test_midranks_no_ties() {
    let vals = vec![1.0, 2.0, 3.0, 4.0];
    let (ranks, tie_sum) = compute_midranks_with_ties(&vals);
    assert_eq!(ranks, vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(tie_sum, 0.0);
}

#[test]
fn test_midranks_with_ties() {
    let vals = vec![1.0, 2.0, 2.0, 2.0, 5.0];
    let (ranks, tie_sum) = compute_midranks_with_ties(&vals);
    // 2.0 occupies positions 2, 3, 4 -> midrank = 3.0
    assert_eq!(ranks, vec![1.0, 3.0, 3.0, 3.0, 5.0]);
    // t = 3 -> 3^3 - 3 = 24
    assert_eq!(tie_sum, 24.0);
}

#[test]
fn test_mann_whitney_oracle_no_ties() {
    // Group 1: 1, 2, 3, 4
    // Group 2: 5, 6, 7, 8
    let g1 = [1.0, 2.0, 3.0, 4.0];
    let g2 = [5.0, 6.0, 7.0, 8.0];

    let res = compute_mann_whitney_from_slices("score", "grp", &g1, &g2, "1", "2");

    assert_eq!(res.group1_stats.n, 4);
    assert_eq!(res.group2_stats.n, 4);
    assert_eq!(res.total_n, 8);
    assert_eq!(res.group1_stats.sum_ranks, 10.0);
    assert_eq!(res.group2_stats.sum_ranks, 26.0);
    assert_eq!(res.group1_stats.mean_rank, 2.5);
    assert_eq!(res.group2_stats.mean_rank, 6.5);
    assert_eq!(res.mann_whitney_u, 0.0);
    assert_eq!(res.wilcoxon_w, 10.0);
    // Z = -8 / sqrt(12) = -2.309401...
    assert!((res.z_stat - (-2.3094010767)).abs() < 1e-5);
    assert!((res.asymp_sig_2tailed - 0.0209213).abs() < 1e-4);
}

#[test]
fn test_mann_whitney_oracle_with_ties() {
    // Hand-verified & SPSS benchmark:
    // Group 1: [10, 15, 15, 20, 25]
    // Group 2: [15, 20, 25, 30, 35]
    let g1 = [10.0, 15.0, 15.0, 20.0, 25.0];
    let g2 = [15.0, 20.0, 25.0, 30.0, 35.0];

    let res = compute_mann_whitney_from_slices("salary", "gender", &g1, &g2, "1", "2");

    assert_eq!(res.group1_stats.n, 5);
    assert_eq!(res.group2_stats.n, 5);
    assert_eq!(res.total_n, 10);
    assert_eq!(res.group1_stats.sum_ranks, 20.0);
    assert_eq!(res.group2_stats.sum_ranks, 35.0);
    assert_eq!(res.group1_stats.mean_rank, 4.0);
    assert_eq!(res.group2_stats.mean_rank, 7.0);
    assert_eq!(res.mann_whitney_u, 5.0);
    assert_eq!(res.wilcoxon_w, 20.0);
    // Z = -1.59598...
    assert!((res.z_stat - (-1.595987)).abs() < 1e-5);
    assert!((res.asymp_sig_2tailed - 0.110495).abs() < 1e-4);
}

#[test]
fn test_mann_whitney_with_missing_values() {
    let vars = vec![
        Some(10.0),
        None,
        Some(15.0),
        Some(20.0),
        Some(30.0),
        Some(40.0),
    ];
    let groups = vec![
        Some("A".to_string()),
        Some("A".to_string()),
        Some("A".to_string()),
        Some("B".to_string()),
        None,
        Some("B".to_string()),
    ];

    let res = compute_mann_whitney("score", "group", &vars, &groups, "A", "B");
    assert_eq!(res.group1_stats.n, 2); // 10.0 and 15.0
    assert_eq!(res.group2_stats.n, 2); // 20.0 and 40.0
    assert_eq!(res.total_n, 4);
    assert_eq!(res.mann_whitney_u, 0.0);
}

#[test]
fn test_wilcoxon_signed_rank_oracle() {
    // Pairs:
    // x: [18, 16, 20, 22, 15]
    // y: [15, 16, 17, 25, 12]
    // diffs: +3, 0, +3, -3, +3
    // non-zero count = 4, absolute diffs = all 3.0 -> all tied at midrank 2.5
    let pre = [15.0, 16.0, 17.0, 25.0, 12.0];
    let post = [18.0, 16.0, 20.0, 22.0, 15.0];

    let res = compute_wilcoxon_signed_rank_from_slices("pre", "post", &pre, &post);

    assert_eq!(res.ranks.total_n, 5);
    assert_eq!(res.ranks.ties_count, 1);
    assert_eq!(res.ranks.negative_ranks.n, 1);
    assert_eq!(res.ranks.negative_ranks.sum_ranks, 2.5);
    assert_eq!(res.ranks.positive_ranks.n, 3);
    assert_eq!(res.ranks.positive_ranks.sum_ranks, 7.5);
    assert_eq!(res.test_statistic_w, 2.5);
}
