use super::engine::{compute_theil_sen, compute_theil_sen_bootstrap};
use super::types::{TheilSenOptions, MAX_EXACT_PAIRS_N};
use crate::anova_oneway::AlgoError;
use crate::bootstrap::{BootstrapCiMethod, BootstrapOptions, BootstrapSamplingMethod};

#[test]
fn test_theil_sen_exact_linear_with_outlier() {
    // True line: y = 2.0 * x + 1.0, with one extreme outlier at x = 5 (y = 100 instead of 11).
    let xs = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
        Some(7.0),
        Some(8.0),
        Some(9.0),
    ];
    let ys = vec![
        Some(3.0),
        Some(5.0),
        Some(7.0),
        Some(9.0),
        Some(100.0), // outlier
        Some(13.0),
        Some(15.0),
        Some(17.0),
        Some(19.0),
    ];

    let opts = TheilSenOptions::default();
    let res = compute_theil_sen("y", &ys, "x", &xs, &opts).unwrap();

    assert_eq!(res.n_valid, 9);
    assert_eq!(res.n_missing, 0);
    assert!((res.slope - 2.0).abs() < 1e-12);
    assert!((res.intercept - 1.0).abs() < 1e-12);
    assert!(res.slope_ci_lower <= res.slope);
    assert!(res.slope_ci_upper >= res.slope);
    assert!(res.kendall_tau.unwrap() > 0.6);
    assert!(res.p_value.unwrap() < 0.05);
    assert!((res.mad_residuals - 0.0).abs() < 1e-12);
}

#[test]
fn test_theil_sen_golden_oracle_with_ties_and_missing() {
    // Classic dataset with ties in X and Y and missing values:
    // Valid pairs (x, y): (1, 2), (2, 3), (2, 5), (4, 6), (5, 8)
    // Pairwise slopes (dx != 0):
    // (1,2)-(2,3)=1, (1,2)-(2,5)=3, (1,2)-(4,6)=4/3, (1,2)-(5,8)=1.5
    // (2,3)-(4,6)=1.5, (2,3)-(5,8)=5/3
    // (2,5)-(4,6)=0.5, (2,5)-(5,8)=1.0
    // (4,6)-(5,8)=2.0
    // Sorted slopes (9 values): [0.5, 1.0, 1.0, 4/3, 1.5, 1.5, 5/3, 2.0, 3.0] -> median = 1.5
    // Intercepts (y - 1.5*x): [0.5, 0.0, 2.0, 0.0, 0.5] -> sorted [0.0, 0.0, 0.5, 0.5, 2.0] -> median = 0.5
    // Kendall S:
    // (1,2) vs (2,3): +1; vs (2,5): +1; vs (4,6): +1; vs (5,8): +1
    // (2,3) vs (2,5): 0 (x tie); vs (4,6): +1; vs (5,8): +1
    // (2,5) vs (4,6): +1; vs (5,8): +1
    // (4,6) vs (5,8): +1
    // S = 9, n0 = 10, n1 = 1 (one tie of size 2 in x), n2 = 0 -> tau_b = 9 / sqrt(9 * 10) = 3 / sqrt(10) = 0.9486832980505138
    let xs = vec![Some(1.0), None, Some(2.0), Some(2.0), Some(4.0), Some(5.0)];
    let ys = vec![
        Some(2.0),
        Some(10.0),
        Some(3.0),
        Some(5.0),
        Some(6.0),
        Some(8.0),
    ];

    let res = compute_theil_sen("y", &ys, "x", &xs, &TheilSenOptions::default()).unwrap();

    assert_eq!(res.n_valid, 5);
    assert_eq!(res.n_missing, 1);
    assert!((res.slope - 1.5).abs() < 1e-12);
    assert!((res.intercept - 0.5).abs() < 1e-12);
    let expected_tau = 3.0 / 10.0_f64.sqrt();
    assert!((res.kendall_tau.unwrap() - expected_tau).abs() < 1e-6);
}

#[test]
fn test_theil_sen_error_cases() {
    let opts = TheilSenOptions::default();

    // Insufficient data
    let err = compute_theil_sen("y", &[Some(1.0)], "x", &[Some(2.0)], &opts).unwrap_err();
    assert!(matches!(err, AlgoError::InsufficientData(_)));

    // Constant X
    let err = compute_theil_sen(
        "y",
        &[Some(1.0), Some(2.0), Some(3.0)],
        "x",
        &[Some(5.0), Some(5.0), Some(5.0)],
        &opts,
    )
    .unwrap_err();
    assert!(matches!(err, AlgoError::InvalidInput(_)));

    // Exceeds MAX_EXACT_PAIRS_N
    let big_n = MAX_EXACT_PAIRS_N + 1;
    let xs: Vec<Option<f64>> = (0..big_n).map(|i| Some(i as f64)).collect();
    let ys: Vec<Option<f64>> = (0..big_n).map(|i| Some(2.0 * i as f64)).collect();
    let err = compute_theil_sen("y", &ys, "x", &xs, &opts).unwrap_err();
    assert!(matches!(err, AlgoError::InvalidInput(_)));
}

#[test]
fn test_theil_sen_bootstrap() {
    let xs: Vec<Option<f64>> = (1..=12).map(|i| Some(i as f64)).collect();
    let ys: Vec<Option<f64>> = (1..=12)
        .map(|i| Some(3.0 * (i as f64) + 2.0 + if i == 6 { 0.5 } else { 0.0 }))
        .collect();

    let opts = TheilSenOptions::default();
    let boot = BootstrapOptions {
        enabled: true,
        samples: 50,
        ci_level: 95.0,
        ci_method: BootstrapCiMethod::Percentile,
        sampling_method: BootstrapSamplingMethod::Simple,
        seed: Some(42),
    };

    let b_res = compute_theil_sen_bootstrap("y", &ys, "x", &xs, &opts, None, &boot).unwrap();
    let slope_boot = b_res.slope.expect("slope bootstrap summary");
    assert!((slope_boot.estimate - 3.0).abs() < 0.2);
    assert!(slope_boot.ci.lower <= slope_boot.ci.upper);
}
