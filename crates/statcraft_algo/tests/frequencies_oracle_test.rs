//! Golden Oracle verification tests for FREQUENCIES univariate statistics and percentiles.
//! Validated against IBM SPSS Statistics canon and scipy.stats reference formulas.
//! Precision target: <= 1e-6.

use statcraft_algo::frequencies::{
    compute_custom_percentiles, compute_frequencies_full, compute_frequencies_statistics,
    compute_ntiles, compute_percentile, compute_quartiles, compute_statistics_from_slice,
    FrequenciesStatisticsOptions, FrequencyInputItem,
};

#[test]
fn test_golden_oracle_symmetric_uniform_sample() {
    // Reference sample: integers 1 to 10
    // X = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    let x: Vec<f64> = (1..=10).map(|v| v as f64).collect();
    let opts = FrequenciesStatisticsOptions::all();
    let res = compute_statistics_from_slice(&x, &opts);

    // 1. Mean: (1+10)/2 = 5.5
    assert!(
        (res.mean.expect("mean") - 5.5).abs() < 1e-6,
        "Mean mismatch"
    );

    // 2. Sum: 55.0
    assert!((res.sum.expect("sum") - 55.0).abs() < 1e-6, "Sum mismatch");

    // 3. Min, Max, Range
    assert_eq!(res.min, Some(1.0));
    assert_eq!(res.max, Some(10.0));
    assert_eq!(res.range, Some(9.0));

    // 4. Sample Variance: s^2 = 82.5 / 9 = 9.166666666666666
    let expected_var = 82.5 / 9.0;
    assert!(
        (res.variance.expect("variance") - expected_var).abs() < 1e-6,
        "Variance mismatch"
    );

    // 5. Sample Std Dev: s = sqrt(9.166666666666666) ≈ 3.0276503540974917
    let expected_sd = expected_var.sqrt();
    assert!(
        (res.std_dev.expect("std_dev") - expected_sd).abs() < 1e-6,
        "Std Dev mismatch"
    );

    // 6. Standard Error of Mean: SE = s / sqrt(10) ≈ 0.9574271077563381
    let expected_se_mean = expected_sd / 10.0_f64.sqrt();
    assert!(
        (res.se_mean.expect("se_mean") - expected_se_mean).abs() < 1e-6,
        "SE Mean mismatch"
    );

    // 7. Median: HAVERAGE r = 0.5 * 11 = 5.5 -> 5.5
    assert!(
        (res.median.expect("median") - 5.5).abs() < 1e-6,
        "Median mismatch"
    );

    // 8. Mode: all frequencies = 1 -> smallest value 1.0, multiple modes = true
    assert_eq!(res.mode, Some(1.0));
    assert!(res.has_multiple_modes);

    // 9. Skewness: symmetric -> 0.0
    assert!(
        (res.skewness.expect("skewness") - 0.0).abs() < 1e-6,
        "Skewness mismatch"
    );
    // SE Skewness: sqrt(540 / 1144) ≈ 0.6870429186
    let expected_se_skew = (540.0 / 1144.0_f64).sqrt();
    assert!(
        (res.se_skewness.expect("se_skew") - expected_se_skew).abs() < 1e-6,
        "SE Skewness mismatch"
    );

    // 10. Kurtosis (SPSS excess kurtosis): exactly -1.2
    assert!(
        (res.kurtosis.expect("kurtosis") - (-1.2)).abs() < 1e-6,
        "Kurtosis mismatch"
    );
    // SE Kurtosis: 2 * se_skew * sqrt(99 / 105) ≈ 1.33418579
    let expected_se_kurt = 2.0 * expected_se_skew * (99.0 / 105.0_f64).sqrt();
    assert!(
        (res.se_kurtosis.expect("se_kurt") - expected_se_kurt).abs() < 1e-6,
        "SE Kurtosis mismatch"
    );

    // 11. Quartiles: Q1 = 2.75, Q2 = 5.5, Q3 = 8.25
    let q = compute_quartiles(&x).expect("quartiles");
    assert!((q.0 - 2.75).abs() < 1e-6, "Q1 mismatch");
    assert!((q.1 - 5.50).abs() < 1e-6, "Q2 mismatch");
    assert!((q.2 - 8.25).abs() < 1e-6, "Q3 mismatch");
}

#[test]
fn test_golden_oracle_asymmetric_sample_with_unique_mode() {
    // Sample: [1, 2, 2, 3, 4, 4, 4, 5, 8, 12]
    // N = 10, Sum = 45, Mean = 4.5
    // Unique mode = 4 (frequency 3)
    let y = vec![1.0, 2.0, 2.0, 3.0, 4.0, 4.0, 4.0, 5.0, 8.0, 12.0];
    let opts = FrequenciesStatisticsOptions::all();
    let res = compute_statistics_from_slice(&y, &opts);

    assert_eq!(res.mean, Some(4.5));
    assert_eq!(res.sum, Some(45.0));
    assert_eq!(res.min, Some(1.0));
    assert_eq!(res.max, Some(12.0));
    assert_eq!(res.range, Some(11.0));
    assert_eq!(res.mode, Some(4.0));
    assert!(!res.has_multiple_modes);

    // Median: r = 0.5 * 11 = 5.5 -> index 5 and 6 (both are 4.0) -> 4.0
    assert_eq!(res.median, Some(4.0));

    // Positive skewness check
    let skew = res.skewness.expect("skewness");
    assert!(skew > 1.0, "Expected positive skewness, got {}", skew);
}

#[test]
fn test_golden_oracle_weighted_equivalence() {
    // Weighted sample: (10, weight 2), (20, weight 1)
    // Equivalent to: [10, 10, 20]
    let weighted_pairs = vec![(10.0, 2.0), (20.0, 1.0)];
    let opts = FrequenciesStatisticsOptions::all();
    let res_weighted = compute_frequencies_statistics(&weighted_pairs, &opts);

    let unweighted_raw = vec![10.0, 10.0, 20.0];
    let res_unweighted = compute_statistics_from_slice(&unweighted_raw, &opts);

    assert!((res_weighted.mean.unwrap() - res_unweighted.mean.unwrap()).abs() < 1e-6);
    assert!((res_weighted.variance.unwrap() - res_unweighted.variance.unwrap()).abs() < 1e-6);
    assert!((res_weighted.std_dev.unwrap() - res_unweighted.std_dev.unwrap()).abs() < 1e-6);
    assert_eq!(res_weighted.mode, res_unweighted.mode);
    assert_eq!(res_weighted.median, res_unweighted.median);
}

#[test]
fn test_golden_oracle_percentiles_ntiles_and_custom() {
    let vals: Vec<f64> = (1..=100).map(|v| v as f64).collect();

    // Percentile 50th for 1..100: r = 0.50 * 101 = 50.5 -> 50.5
    assert_eq!(compute_percentile(&vals, 0.50), Some(50.5));

    // Deciles (10-tiles)
    let deciles = compute_ntiles(&vals, 10).expect("deciles");
    assert_eq!(deciles.len(), 9);
    // 10th percentile: r = 0.10 * 101 = 10.1 -> 10 + 0.1 * 1 = 10.1
    assert!((deciles[0].1 - 10.1).abs() < 1e-6);

    // Custom percentiles
    let custom = compute_custom_percentiles(&vals, &[5.0, 95.0]);
    assert_eq!(custom.len(), 2);
    // 5th: r = 0.05 * 101 = 5.05 -> 5.05
    assert!((custom[0].1 - 5.05).abs() < 1e-6);
    // 95th: r = 0.95 * 101 = 95.95 -> 95.95
    assert!((custom[1].1 - 95.95).abs() < 1e-6);
}

#[test]
fn test_compute_frequencies_full_integration() {
    let items = vec![
        FrequencyInputItem {
            value: "10".to_string(),
            label: Some("Ten".to_string()),
            is_missing: false,
            numeric_value: Some(10.0),
            weight: 1.0,
        },
        FrequencyInputItem {
            value: "20".to_string(),
            label: Some("Twenty".to_string()),
            is_missing: false,
            numeric_value: Some(20.0),
            weight: 1.0,
        },
        FrequencyInputItem {
            value: "99".to_string(),
            label: Some("Missing".to_string()),
            is_missing: true,
            numeric_value: Some(99.0),
            weight: 1.0,
        },
    ];

    let opts = FrequenciesStatisticsOptions::all();
    let res = compute_frequencies_full("score", &items, Some(&opts));

    assert_eq!(res.variable, "score");
    assert_eq!(res.n_valid, 2);
    assert_eq!(res.n_missing, 1);
    assert_eq!(res.categories.len(), 2);
    assert!(res.quartiles.is_some());

    let stats = res.statistics.expect("statistics should be computed");
    assert_eq!(stats.mean, Some(15.0));
    assert_eq!(stats.min, Some(10.0));
    assert_eq!(stats.max, Some(20.0));
    assert_eq!(stats.range, Some(10.0));
    assert_eq!(stats.sum, Some(30.0));
}
