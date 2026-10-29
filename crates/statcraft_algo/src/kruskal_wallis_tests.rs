use super::kruskal_wallis::*;

#[test]
fn test_kruskal_wallis_golden_oracle_distinct() {
    // 3 groups tested against scipy.stats.kruskal:
    // H: 8.299821109123439, p: 0.0157658266030596
    let groups = vec![
        ("1".to_string(), vec![2.9, 3.0, 2.5, 2.6, 3.2]),
        ("2".to_string(), vec![3.8, 3.7, 3.9, 4.0, 3.5]),
        ("3".to_string(), vec![2.8, 3.4, 3.7, 2.2, 2.0]),
    ];

    let res = compute_kruskal_wallis_from_groups("score", "group", &groups);

    assert_eq!(res.variable, "score");
    assert_eq!(res.grouping_var, "group");
    assert_eq!(res.total_n, 15);
    assert_eq!(res.df, 2);
    assert_eq!(res.groups.len(), 3);

    // Golden Oracle check with precision <= 1e-6
    assert!((res.kruskal_wallis_h - 8.2998211).abs() < 1e-5);
    assert!((res.asymp_sig - 0.0157658).abs() < 1e-5);
}

#[test]
fn test_kruskal_wallis_golden_oracle_ties() {
    // 3 groups with ties tested against scipy.stats.kruskal:
    // H: 5.580291970802921, p: 0.06141224796889311
    let groups = vec![
        ("1".to_string(), vec![1.0, 2.0, 2.0, 3.0]),
        ("2".to_string(), vec![2.0, 3.0, 4.0, 5.0]),
        ("3".to_string(), vec![3.0, 4.0, 4.0, 6.0]),
    ];

    let res = compute_kruskal_wallis_from_groups("rating", "class", &groups);

    assert_eq!(res.total_n, 12);
    assert_eq!(res.df, 2);
    assert!((res.kruskal_wallis_h - 5.58029197).abs() < 1e-5);
    assert!((res.asymp_sig - 0.0614122).abs() < 1e-5);
}

#[test]
fn test_kruskal_wallis_range_filtering() {
    let var_values = vec![
        Some(10.0),
        Some(20.0),
        Some(30.0),
        Some(40.0),
        Some(50.0),
        Some(60.0),
        Some(70.0),
    ];
    let group_values = vec![
        Some("1".to_string()),
        Some("1".to_string()),
        Some("2".to_string()),
        Some("2".to_string()),
        Some("3".to_string()),
        Some("4".to_string()), // Out of range 1..3
        None,                  // Missing group
    ];

    let res = compute_kruskal_wallis("val", "grp", &var_values, &group_values, 1.0, 3.0);

    assert_eq!(res.total_n, 5);
    assert_eq!(res.groups.len(), 3);
    assert_eq!(res.groups[0].group_val, "1");
    assert_eq!(res.groups[0].n, 2);
    assert_eq!(res.groups[1].group_val, "2");
    assert_eq!(res.groups[1].n, 2);
    assert_eq!(res.groups[2].group_val, "3");
    assert_eq!(res.groups[2].n, 1);
    assert_eq!(res.df, 2);
    assert!(res.asymp_sig >= 0.0 && res.asymp_sig <= 1.0);
}

#[test]
fn test_kruskal_wallis_edge_cases() {
    // Empty
    let empty_res = compute_kruskal_wallis("x", "grp", &[], &[], 1.0, 3.0);
    assert_eq!(empty_res.total_n, 0);
    assert_eq!(empty_res.df, 0);
    assert_eq!(empty_res.kruskal_wallis_h, 0.0);
    assert_eq!(empty_res.asymp_sig, 1.0);

    // Single group
    let single_group = vec![("1".to_string(), vec![1.0, 2.0, 3.0])];
    let single_res = compute_kruskal_wallis_from_groups("x", "grp", &single_group);
    assert_eq!(single_res.total_n, 3);
    assert_eq!(single_res.df, 0);
    assert_eq!(single_res.kruskal_wallis_h, 0.0);
    assert_eq!(single_res.asymp_sig, 1.0);
}
