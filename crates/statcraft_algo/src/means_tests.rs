use super::*;
use crate::means_types::*;

#[test]
fn test_means_breakdown_golden_oracle() {
    // Reference group data:
    // Group 1: [10.0, 12.0, 14.0, 15.0] (mean = 12.75, var = 4.916666666666667, sd = 2.217355782613)
    // Group 2: [20.0, 22.0, 24.0, 25.0, 26.0] (mean = 23.4, var = 5.8, sd = 2.408318915758)
    // Group 3: [30.0, 31.0, 32.0, 35.0] (mean = 32.0, var = 4.666666666666667, sd = 2.160246899469)
    // Total: N = 13, mean = 22.76923076923077
    // SS_between = 744.3576923076923
    // SS_within = 51.95
    // SS_total = 796.3076923076923
    // df1 = 2, df2 = 10, df_total = 12
    // MS_between = 372.17884615384615
    // MS_within = 5.195
    // F = 71.6417413193159
    // eta^2 = 0.9347614064049826
    // eta = 0.9668306008670603

    let mut dep_values: Vec<Option<f64>> = Vec::new();
    let mut factor_values: Vec<Option<String>> = Vec::new();

    for &v in &[10.0, 12.0, 14.0, 15.0] {
        dep_values.push(Some(v));
        factor_values.push(Some("1".to_string()));
    }
    for &v in &[20.0, 22.0, 24.0, 25.0, 26.0] {
        dep_values.push(Some(v));
        factor_values.push(Some("2".to_string()));
    }
    for &v in &[30.0, 31.0, 32.0, 35.0] {
        dep_values.push(Some(v));
        factor_values.push(Some("3".to_string()));
    }

    // Add some missing cases
    dep_values.push(None);
    factor_values.push(Some("1".to_string()));
    dep_values.push(Some(99.0));
    factor_values.push(None);

    let options = MeansOptions::default();
    let res = compute_means_breakdown(
        "score",
        "group",
        &dep_values,
        &factor_values,
        None,
        &options,
    )
    .expect("Should compute means successfully");

    // Case processing summary
    assert_eq!(res.summary.included_n, 13);
    assert_eq!(res.summary.excluded_n, 2);
    assert_eq!(res.summary.total_n, 15);
    assert!((res.summary.included_percent - (13.0 / 15.0 * 100.0)).abs() < 1e-6);

    // Group cells
    assert_eq!(res.cells.len(), 3);

    // Group 1
    let g1 = &res.cells[0];
    assert_eq!(g1.group_label, "1");
    assert_eq!(g1.count, 4);
    assert!((g1.mean.unwrap_or(0.0) - 12.75).abs() < 1e-6);
    assert!((g1.std_dev.unwrap_or(0.0) - 2.217355782613).abs() < 1e-6);
    assert!((g1.variance.unwrap_or(0.0) - (14.75 / 3.0)).abs() < 1e-6);
    assert_eq!(g1.min, Some(10.0));
    assert_eq!(g1.max, Some(15.0));

    // Group 2
    let g2 = &res.cells[1];
    assert_eq!(g2.group_label, "2");
    assert_eq!(g2.count, 5);
    assert!((g2.mean.unwrap_or(0.0) - 23.4).abs() < 1e-6);
    assert!((g2.std_dev.unwrap_or(0.0) - 2.408318915758).abs() < 1e-6);
    assert!((g2.variance.unwrap_or(0.0) - 5.8).abs() < 1e-6);

    // Group 3
    let g3 = &res.cells[2];
    assert_eq!(g3.group_label, "3");
    assert_eq!(g3.count, 4);
    assert!((g3.mean.unwrap_or(0.0) - 32.0).abs() < 1e-6);
    assert!((g3.std_dev.unwrap_or(0.0) - 2.160246899469).abs() < 1e-6);
    assert!((g3.variance.unwrap_or(0.0) - (14.0 / 3.0)).abs() < 1e-6);

    // Total
    assert_eq!(res.total.count, 13);
    assert!((res.total.mean.unwrap_or(0.0) - (296.0 / 13.0)).abs() < 1e-6);

    // ANOVA Table
    let anova = res.anova.as_ref().expect("ANOVA table must be present");
    assert_eq!(anova.between_groups.df, 2);
    assert_eq!(anova.within_groups.df, 10);
    assert_eq!(anova.total.df, 12);

    assert!((anova.between_groups.sum_of_squares - 744.3576923076923).abs() < 1e-6);
    assert!((anova.within_groups.sum_of_squares - 51.95).abs() < 1e-6);
    assert!((anova.total.sum_of_squares - 796.3076923076923).abs() < 1e-6);

    assert!((anova.between_groups.mean_square.unwrap_or(0.0) - 372.17884615384615).abs() < 1e-6);
    assert!((anova.within_groups.mean_square.unwrap_or(0.0) - 5.195).abs() < 1e-6);
    assert!((anova.between_groups.f_stat.unwrap_or(0.0) - 71.6417413193159).abs() < 1e-6);
    assert!(anova.between_groups.p_value.unwrap_or(1.0) < 0.0001);

    // Measures of Association
    let assoc = res.association.as_ref().expect("Measures of association");
    assert!((assoc.eta_squared - 0.9347614064049826).abs() < 1e-6);
    assert!((assoc.eta - 0.9668306008670603).abs() < 1e-6);
}

#[test]
fn test_means_breakdown_single_group() {
    let dep_values = vec![Some(1.0), Some(2.0), Some(3.0)];
    let factor_values = vec![
        Some("A".to_string()),
        Some("A".to_string()),
        Some("A".to_string()),
    ];
    let options = MeansOptions::default();
    let res = compute_means_breakdown("Y", "X", &dep_values, &factor_values, None, &options)
        .expect("Should compute single group");

    assert_eq!(res.cells.len(), 1);
    assert_eq!(res.cells[0].count, 3);
    assert!((res.cells[0].mean.unwrap_or(0.0) - 2.0).abs() < 1e-6);
    assert!(res.anova.is_none());
    assert!(res.association.is_none());
}

#[test]
fn test_means_breakdown_weights() {
    let dep_values = vec![Some(10.0), Some(20.0), Some(30.0)];
    let factor_values = vec![
        Some("G1".to_string()),
        Some("G1".to_string()),
        Some("G2".to_string()),
    ];
    let weights = vec![1.0, 3.0, 2.0];
    let options = MeansOptions::default();

    let res = compute_means_breakdown(
        "Y",
        "X",
        &dep_values,
        &factor_values,
        Some(&weights),
        &options,
    )
    .expect("Should compute with weights");

    // G1: 10*1 + 20*3 = 70 / 4 = 17.5
    let g1 = &res.cells[0];
    assert_eq!(g1.weighted_count, 4.0);
    assert!((g1.mean.unwrap_or(0.0) - 17.5).abs() < 1e-6);

    // G2: 30*2 = 60 / 2 = 30.0
    let g2 = &res.cells[1];
    assert_eq!(g2.weighted_count, 2.0);
    assert!((g2.mean.unwrap_or(0.0) - 30.0).abs() < 1e-6);
}
