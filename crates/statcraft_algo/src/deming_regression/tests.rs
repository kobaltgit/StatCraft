use super::engine::{compute_deming_regression, compute_deming_regression_bootstrap};
use super::types::DemingRegressionOptions;
use crate::bootstrap::BootstrapOptions;

#[test]
fn test_deming_exact_linear() {
    let xs = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let ys = vec![Some(3.0), Some(5.0), Some(7.0), Some(9.0), Some(11.0)];

    for lambda in [1.0, 2.5] {
        let opts = DemingRegressionOptions {
            lambda,
            ci_level: 0.95,
        };
        let res = compute_deming_regression("y", &ys, "x", &xs, &opts).unwrap();
        assert!((res.slope - 2.0).abs() < 1e-12);
        assert!((res.intercept - 1.0).abs() < 1e-12);
        assert!(res.slope_se.abs() < 1e-12);
        assert!(res.intercept_se.abs() < 1e-12);
        assert!(res.orthogonal_residual_sd.abs() < 1e-12);
        assert!((res.pearson_r.unwrap() - 1.0).abs() < 1e-12);
    }
}

#[test]
fn test_deming_golden_oracle_lambda_1_and_custom() {
    let xs = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
        None,
    ];
    let ys = vec![
        Some(2.1),
        Some(3.9),
        Some(6.2),
        Some(7.8),
        Some(10.1),
        Some(11.9),
        Some(99.0),
    ];

    // Golden Oracle values verified analytically and against scipy.odr closed-form:
    // s_xx = 17.5, s_yy = 68.52, s_xy = 34.60
    let res_orth =
        compute_deming_regression("y", &ys, "x", &xs, &DemingRegressionOptions::default()).unwrap();
    assert_eq!(res_orth.n_valid, 6);
    assert_eq!(res_orth.n_missing, 1);
    assert!((res_orth.slope - 1.9796948320595082).abs() < 1e-6);
    assert!((res_orth.intercept - 0.0710680877917213).abs() < 1e-6);
    assert!(res_orth.slope_se > 0.0 && res_orth.slope_se.is_finite());
    assert!(res_orth.intercept_se > 0.0 && res_orth.intercept_se.is_finite());
    assert!(res_orth.slope_ci_lower < res_orth.slope && res_orth.slope < res_orth.slope_ci_upper);
    assert!(
        res_orth.intercept_ci_lower < res_orth.intercept
            && res_orth.intercept < res_orth.intercept_ci_upper
    );
    assert!(res_orth.orthogonal_residual_sd > 0.0);

    // Custom Deming ratio lambda = 2.5
    let opts_custom = DemingRegressionOptions {
        lambda: 2.5,
        ci_level: 0.95,
    };
    let res_custom = compute_deming_regression("y", &ys, "x", &xs, &opts_custom).unwrap();
    assert!((res_custom.slope - 1.9790978015868327).abs() < 1e-6);
    assert!((res_custom.intercept - 0.0731576944460855).abs() < 1e-6);
}

#[test]
fn test_deming_edge_cases() {
    let opts = DemingRegressionOptions::default();

    // N < 3
    let err_n = compute_deming_regression(
        "y",
        &[Some(1.0), Some(2.0)],
        "x",
        &[Some(1.0), Some(2.0)],
        &opts,
    )
    .unwrap_err();
    assert!(err_n.to_string().contains("At least 3"));

    // Invalid lambda <= 0
    let err_lam = compute_deming_regression(
        "y",
        &[Some(1.0), Some(2.0), Some(3.0)],
        "x",
        &[Some(1.0), Some(2.0), Some(3.0)],
        &DemingRegressionOptions {
            lambda: 0.0,
            ci_level: 0.95,
        },
    )
    .unwrap_err();
    assert!(err_lam.to_string().contains("lambda"));

    // Zero covariance s_xy = 0 with s_yy <= lambda * s_xx -> horizontal slope = 0.0
    let xs = vec![Some(-1.0), Some(0.0), Some(1.0), Some(0.0)];
    let ys = vec![Some(5.0), Some(5.5), Some(5.0), Some(4.5)];
    let res_zero = compute_deming_regression("y", &ys, "x", &xs, &opts).unwrap();
    assert!(res_zero.slope.abs() < 1e-12);
    assert!((res_zero.intercept - 5.0).abs() < 1e-12);
}

#[test]
fn test_deming_bootstrap() {
    let xs = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
    ];
    let ys = vec![
        Some(2.1),
        Some(3.9),
        Some(6.2),
        Some(7.8),
        Some(10.1),
        Some(11.9),
    ];
    let opts = DemingRegressionOptions::default();
    let boot = BootstrapOptions {
        enabled: true,
        samples: 50,
        seed: Some(42),
        ..BootstrapOptions::default()
    };

    let b_res =
        compute_deming_regression_bootstrap("y", &ys, "x", &xs, &opts, None, &boot).unwrap();
    assert!(b_res.slope.is_some());
    assert!(b_res.intercept.is_some());
}
