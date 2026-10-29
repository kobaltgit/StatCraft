use crate::reliability::compute_reliability;
use crate::reliability_types::{ReliabilityModel, ReliabilityOptions};

#[test]
fn test_reliability_golden_oracle_3_items() {
    let q1 = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let q2 = vec![Some(2.0), Some(3.0), Some(4.0), Some(5.0), Some(6.0)];
    let q3 = vec![Some(1.0), Some(3.0), Some(2.0), Some(4.0), Some(5.0)];

    let items = vec![
        ("q1", q1.as_slice()),
        ("q2", q2.as_slice()),
        ("q3", q3.as_slice()),
    ];

    let options = ReliabilityOptions {
        scale_name: "ALL VARIABLES".to_string(),
        model: ReliabilityModel::Alpha,
        item_statistics: true,
        scale_statistics: true,
        scale_if_item_deleted: true,
    };

    let result = compute_reliability(&items, &[], &options);

    assert_eq!(result.case_processing.n_valid, 5);
    assert_eq!(result.case_processing.n_excluded, 0);
    assert_eq!(result.case_processing.n_total, 5);
    assert!((result.case_processing.percent_valid - 100.0).abs() < 1e-9);

    // Alpha = 1.5 * (1 - 7.5 / 21.5) = 0.976744186
    let alpha = result.reliability_stats.cronbach_alpha.unwrap();
    assert!((alpha - 0.9767441860465116).abs() < 1e-6);

    let alpha_std = result
        .reliability_stats
        .cronbach_alpha_standardized
        .unwrap();
    assert!((alpha_std - 0.9767441860465116).abs() < 1e-6);

    // Item statistics
    assert_eq!(result.item_stats.len(), 3);
    assert_eq!(result.item_stats[0].item_name, "q1");
    assert!((result.item_stats[0].mean - 3.0).abs() < 1e-9);
    assert!((result.item_stats[0].std_dev - 2.5_f64.sqrt()).abs() < 1e-9);

    assert_eq!(result.item_stats[1].item_name, "q2");
    assert!((result.item_stats[1].mean - 4.0).abs() < 1e-9);
    assert!((result.item_stats[1].std_dev - 2.5_f64.sqrt()).abs() < 1e-9);

    assert_eq!(result.item_stats[2].item_name, "q3");
    assert!((result.item_stats[2].mean - 3.0).abs() < 1e-9);
    assert!((result.item_stats[2].std_dev - 2.5_f64.sqrt()).abs() < 1e-9);

    // Item-total statistics for q1
    assert_eq!(result.item_total_stats.len(), 3);
    let it1 = &result.item_total_stats[0];
    assert_eq!(it1.item_name, "q1");
    assert!((it1.scale_mean_if_deleted - 7.0).abs() < 1e-9);
    assert!((it1.scale_variance_if_deleted - 9.5).abs() < 1e-9);

    // Corrected item-total correlation: 4.75 / sqrt(23.75) = 0.9746794344808963
    let r1 = it1.corrected_item_total_correlation.unwrap();
    assert!((r1 - 0.9746794344808963).abs() < 1e-6);

    // Alpha if deleted: 2 * (1 - 5 / 9.5) = 0.9473684210526315
    let a1 = it1.cronbach_alpha_if_deleted.unwrap();
    assert!((a1 - 0.9473684210526315).abs() < 1e-6);

    // Scale statistics
    let scale_stats = result.scale_stats.unwrap();
    assert!((scale_stats.mean - 10.0).abs() < 1e-9);
    assert!((scale_stats.variance - 21.5).abs() < 1e-9);
    assert!((scale_stats.std_dev - 21.5_f64.sqrt()).abs() < 1e-9);
    assert_eq!(scale_stats.n_of_items, 3);
}

#[test]
fn test_reliability_missing_listwise() {
    let q1 = vec![Some(1.0), None, Some(3.0), Some(4.0), Some(5.0)];
    let q2 = vec![Some(2.0), Some(3.0), Some(4.0), Some(5.0), Some(6.0)];
    let q3 = vec![Some(1.0), Some(3.0), Some(2.0), Some(4.0), None];

    let items = vec![
        ("q1", q1.as_slice()),
        ("q2", q2.as_slice()),
        ("q3", q3.as_slice()),
    ];

    let options = ReliabilityOptions::default();
    let result = compute_reliability(&items, &[], &options);

    // Rows 1 and 4 have missing values, so 3 valid rows remaining (0, 2, 3)
    assert_eq!(result.case_processing.n_valid, 3);
    assert_eq!(result.case_processing.n_excluded, 2);
    assert_eq!(result.case_processing.n_total, 5);
    assert!((result.case_processing.percent_valid - 60.0).abs() < 1e-9);
    assert!((result.case_processing.percent_excluded - 40.0).abs() < 1e-9);

    assert!(result.reliability_stats.cronbach_alpha.is_some());
}

#[test]
fn test_reliability_edge_cases_never_crash() {
    let options = ReliabilityOptions::default();

    // 1. Empty items
    let empty_res = compute_reliability(&[], &[], &options);
    assert!(empty_res.reliability_stats.cronbach_alpha.is_none());

    // 2. Only 1 item
    let q1 = vec![Some(1.0), Some(2.0), Some(3.0)];
    let res_1item = compute_reliability(&[("q1", q1.as_slice())], &[], &options);
    assert!(res_1item.reliability_stats.cronbach_alpha.is_none());

    // 3. Constant values (zero variance)
    let c1 = vec![Some(3.0), Some(3.0), Some(3.0)];
    let c2 = vec![Some(3.0), Some(3.0), Some(3.0)];
    let res_const = compute_reliability(
        &[("c1", c1.as_slice()), ("c2", c2.as_slice())],
        &[],
        &options,
    );
    assert!(res_const.reliability_stats.cronbach_alpha.is_none());

    // 4. Exactly 2 items (alpha if deleted should be None because 1 item left)
    let a1 = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0)];
    let a2 = vec![Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let res_2items = compute_reliability(
        &[("a1", a1.as_slice()), ("a2", a2.as_slice())],
        &[],
        &options,
    );
    assert!(res_2items.reliability_stats.cronbach_alpha.is_some());
    for it in &res_2items.item_total_stats {
        assert!(it.cronbach_alpha_if_deleted.is_none());
    }
}
