use super::*;

#[test]
fn test_f_distribution_sf_closed_form() {
    // Analytical test for df1=2, where SF = (df2 / (2F + df2))^(df2/2)
    let df1 = 2.0;
    let df2 = 12.0;
    let f = 59.80952380952381;
    let p_calculated = f_distribution_sf(f, df1, df2);
    let expected_closed = (12.0 / (2.0 * f + 12.0)).powf(6.0);
    assert!((p_calculated - expected_closed).abs() < 1e-12);
}

#[test]
fn test_oneway_anova_three_groups_golden_oracle() {
    let dep = vec![
        10.0, 12.0, 11.0, 13.0, 9.0, // Group 1
        15.0, 16.0, 14.0, 15.0, 17.0, // Group 2
        20.0, 22.0, 19.0, 21.0, 23.0, // Group 3
    ];
    let factor = vec![
        "1".to_string(),
        "1".to_string(),
        "1".to_string(),
        "1".to_string(),
        "1".to_string(),
        "2".to_string(),
        "2".to_string(),
        "2".to_string(),
        "2".to_string(),
        "2".to_string(),
        "3".to_string(),
        "3".to_string(),
        "3".to_string(),
        "3".to_string(),
        "3".to_string(),
    ];

    let opts = OneWayAnovaOptions::default();
    let res = compute_oneway_anova_from_slices("score", &dep, "group", &factor, &opts)
        .expect("ANOVA computation must succeed");

    // Between groups
    assert_eq!(res.between_groups.df, 2);
    assert!((res.between_groups.sum_of_squares - 251.2).abs() < 1e-7);
    assert!((res.between_groups.mean_square.unwrap_or(0.0) - 125.6).abs() < 1e-7);
    assert!((res.between_groups.f_stat.unwrap_or(0.0) - 59.8095238095).abs() < 1e-7);
    assert!((res.between_groups.p_value.unwrap_or(0.0) - 5.74347867e-7).abs() < 1e-10);

    // Within groups
    assert_eq!(res.within_groups.df, 12);
    assert!((res.within_groups.sum_of_squares - 25.2).abs() < 1e-7);
    assert!((res.within_groups.mean_square.unwrap_or(0.0) - 2.1).abs() < 1e-7);

    // Total
    assert_eq!(res.total.df, 14);
    assert!((res.total.sum_of_squares - 276.4).abs() < 1e-7);

    // Levene test
    let hom = res.homogeneity.expect("Homogeneity test requested");
    assert_eq!(hom.df1, 2);
    assert_eq!(hom.df2, 12);
    assert!((hom.levene_stat - 0.295612).abs() < 1e-5);
    assert!((hom.p_value - 0.749341).abs() < 1e-5);

    // Descriptives
    assert_eq!(res.descriptives.len(), 3);
    assert_eq!(res.descriptives[0].group_label, "1");
    assert_eq!(res.descriptives[0].n, 5);
    assert!((res.descriptives[0].mean - 11.0).abs() < 1e-7);
    assert!((res.descriptives[0].std_dev - 1.5811388).abs() < 1e-6);

    let tot = res.total_descriptives.expect("Total descriptives");
    assert_eq!(tot.n, 15);
    assert!((tot.mean - 15.8).abs() < 1e-7);
}

#[test]
fn test_oneway_anova_insufficient_groups() {
    let dep = vec![10.0, 12.0, 11.0];
    let factor = vec!["1".to_string(), "1".to_string(), "1".to_string()];
    let opts = OneWayAnovaOptions::default();
    let err = compute_oneway_anova_from_slices("score", &dep, "group", &factor, &opts).unwrap_err();
    match err {
        AlgoError::InsufficientData(msg) => assert!(msg.contains("at least 2 distinct groups")),
        _ => panic!("Expected InsufficientData"),
    }
}
