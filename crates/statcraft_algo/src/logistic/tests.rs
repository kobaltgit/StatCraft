use crate::anova_oneway::AlgoError;
use crate::logistic::{compute_logistic_regression, LogisticRegressionOptions};

#[test]
fn test_logistic_regression_golden_oracle() {
    let x1: Vec<Option<f64>> = vec![
        25.0, 30.0, 35.0, 40.0, 45.0, 50.0, 55.0, 60.0, 65.0, 70.0, 28.0, 32.0, 48.0, 52.0, 58.0,
        62.0,
    ]
    .into_iter()
    .map(Some)
    .collect();

    let x2: Vec<Option<f64>> = vec![
        2.5, 3.0, 1.8, 3.5, 2.2, 4.0, 3.1, 4.5, 3.8, 5.0, 2.0, 2.8, 3.6, 3.9, 4.2, 4.8,
    ]
    .into_iter()
    .map(Some)
    .collect();

    let y: Vec<Option<f64>> = vec![
        0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0,
    ]
    .into_iter()
    .map(Some)
    .collect();

    let dep = ("outcome", y.as_slice());
    let covs = vec![("age", x1.as_slice()), ("score", x2.as_slice())];

    let opts = LogisticRegressionOptions {
        include_constant: true,
        classification_cutoff: 0.5,
        confidence_level: 0.95,
        hosmer_lemeshow: true,
        max_iterations: 25,
        convergence_tol: 1e-6,
    };

    let res = compute_logistic_regression(dep, &covs, &opts).expect("should compute");
    assert_eq!(res.dependent_var, "outcome");
    assert_eq!(res.covariates, vec!["age", "score"]);
    assert_eq!(res.n_cases, 16);
    assert!(res.converged);

    // Check coefficients table
    assert_eq!(res.coefficients.len(), 3);
    let c_const = &res.coefficients[0];
    let c_age = &res.coefficients[1];
    let c_score = &res.coefficients[2];

    assert_eq!(c_const.param_name, "(Constant)");
    assert_eq!(c_age.param_name, "age");
    assert_eq!(c_score.param_name, "score");

    // Golden Oracle checks with SciPy/Statsmodels
    let exp_b_const = -11.360214;
    let exp_b_age = -0.058516;
    let exp_b_score = 4.258627;

    assert!(
        (c_const.b - exp_b_const).abs() < 1e-4,
        "const b diff: {}",
        (c_const.b - exp_b_const).abs()
    );
    assert!(
        (c_age.b - exp_b_age).abs() < 1e-4,
        "age b diff: {}",
        (c_age.b - exp_b_age).abs()
    );
    assert!(
        (c_score.b - exp_b_score).abs() < 1e-4,
        "score b diff: {}",
        (c_score.b - exp_b_score).abs()
    );

    // Standard errors & Wald
    assert!((c_const.se - 5.924935).abs() < 1e-3);
    assert!((c_age.se - 0.098285).abs() < 1e-3);
    assert!((c_score.se - 2.523231).abs() < 1e-3);

    // Odds ratios Exp(B)
    assert!((c_age.exp_b - 0.943163).abs() < 1e-3);
    assert!((c_score.exp_b - 70.712825).abs() < 1e-1);

    // Model Summary
    assert!((res.model_summary.minus_2ll - 9.353664).abs() < 1e-3);
    assert!((res.omnibus.chi_square - 12.57639).abs() < 1e-3);
    assert_eq!(res.omnibus.df, 2);
    assert!((res.model_summary.cox_snell_r2 - 0.544347).abs() < 1e-3);
    assert!((res.model_summary.nagelkerke_r2 - 0.729637).abs() < 1e-3);

    // Classification table
    let ct = &res.classification_table;
    assert_eq!(ct.rows[0].pred_zero, 6);
    assert_eq!(ct.rows[0].pred_one, 1);
    assert!((ct.rows[0].percent_correct - 85.714).abs() < 0.1);

    assert_eq!(ct.rows[1].pred_zero, 1);
    assert_eq!(ct.rows[1].pred_one, 8);
    assert!((ct.rows[1].percent_correct - 88.888).abs() < 0.1);

    assert!((ct.overall_percent_correct - 87.5).abs() < 0.1);

    // Hosmer-Lemeshow
    assert!(res.hosmer_lemeshow.is_some());
    let hl = res.hosmer_lemeshow.as_ref().unwrap();
    assert!(hl.df > 0);
    assert!(hl.sig > 0.05); // Good fit
}

#[test]
fn test_logistic_regression_validation_errors() {
    let y_three_levels: Vec<Option<f64>> =
        vec![Some(0.0), Some(1.0), Some(2.0), Some(1.0), Some(0.0)];
    let y_one_level: Vec<Option<f64>> = vec![Some(1.0), Some(1.0), Some(1.0), Some(1.0), Some(1.0)];
    let x: Vec<Option<f64>> = vec![Some(10.0), Some(20.0), Some(30.0), Some(40.0), Some(50.0)];

    let opts = LogisticRegressionOptions::default();

    // 3 levels in dependent variable
    let res3 = compute_logistic_regression(
        ("y", y_three_levels.as_slice()),
        &[("x", x.as_slice())],
        &opts,
    );
    assert!(matches!(res3, Err(AlgoError::InvalidInput(_))));

    // 1 level in dependent variable
    let res1 =
        compute_logistic_regression(("y", y_one_level.as_slice()), &[("x", x.as_slice())], &opts);
    assert!(matches!(res1, Err(AlgoError::InvalidInput(_))));

    // Empty covariates
    let res_empty = compute_logistic_regression(("y", x.as_slice()), &[], &opts);
    assert!(matches!(res_empty, Err(AlgoError::InvalidInput(_))));

    // Duplicate variable name in dependent and covariate
    let res_dup = compute_logistic_regression(("x", x.as_slice()), &[("x", x.as_slice())], &opts);
    assert!(matches!(res_dup, Err(AlgoError::InvalidInput(_))));
}
