use crate::regression::compute_linear_regression;
use crate::regression_types::LinearRegressionOptions;

#[test]
fn test_multiple_regression_golden_oracle() {
    // Reference dataset evaluated with Python SciPy / NumPy
    let y: Vec<Option<f64>> = vec![
        Some(12.0),
        Some(15.0),
        Some(18.0),
        Some(20.0),
        Some(24.0),
        Some(28.0),
        Some(30.0),
        Some(35.0),
        Some(38.0),
        Some(42.0),
    ];
    let x1: Vec<Option<f64>> = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
        Some(7.0),
        Some(8.0),
        Some(9.0),
        Some(10.0),
    ];
    let x2: Vec<Option<f64>> = vec![
        Some(2.5),
        Some(3.0),
        Some(4.2),
        Some(5.1),
        Some(5.8),
        Some(7.2),
        Some(8.0),
        Some(8.5),
        Some(9.8),
        Some(10.5),
    ];

    let indep_names = vec!["x1".to_string(), "x2".to_string()];
    let indep_vals = vec![x1, x2];
    let options = LinearRegressionOptions::default();

    let res = compute_linear_regression("y", &y, &indep_names, &indep_vals, &[], &options)
        .expect("Linear regression should succeed");

    let eps = 1e-6;

    // Model Summary Golden Oracle
    // R: 0.9974004406, R2: 0.9948076390, Adj R2: 0.9933241072, SE: 0.8268079705
    assert!((res.r - 0.9974004406).abs() < eps, "R error: {}", res.r);
    assert!(
        (res.r_squared - 0.9948076390).abs() < eps,
        "R2 error: {}",
        res.r_squared
    );
    assert!(
        (res.adj_r_squared - 0.9933241072).abs() < eps,
        "Adj R2 error: {}",
        res.adj_r_squared
    );
    assert!(
        (res.std_error_estimate - 0.8268079705).abs() < eps,
        "SE estimate error: {}",
        res.std_error_estimate
    );

    // ANOVA Golden Oracle
    // SS_model: 916.8147200593, df_m: 2, MS_model: 458.4073600297
    // SS_resid: 4.7852799407, df_r: 7, MS_resid: 0.6836114201
    // F: 670.5671475837, p_F: 1.008735e-08
    assert!((res.anova.regression.sum_of_squares - 916.8147200593).abs() < eps);
    assert_eq!(res.anova.regression.df, 2);
    assert!((res.anova.regression.mean_square.unwrap() - 458.4073600297).abs() < eps);

    assert!((res.anova.residual.sum_of_squares - 4.7852799407).abs() < eps);
    assert_eq!(res.anova.residual.df, 7);
    assert!((res.anova.residual.mean_square.unwrap() - 0.6836114201).abs() < eps);

    assert!((res.anova.total.sum_of_squares - 921.6000000000).abs() < eps);
    assert_eq!(res.anova.total.df, 9);

    assert!((res.anova.regression.f_stat.unwrap() - 670.5671475837).abs() < 1e-4);
    assert!(
        (res.anova.regression.p_value.unwrap() - 1.008735e-08).abs() < 1e-7,
        "ANOVA p error: {:?}",
        res.anova.regression.p_value
    );

    // Coefficients Golden Oracle
    // const: beta=8.8222840193, se=2.1296914354, t=4.1425174899, p=4.335485e-03
    let c0 = &res.coefficients[0];
    assert_eq!(c0.var_name, "(Constant)");
    assert!((c0.b - 8.8222840193).abs() < eps);
    assert!((c0.std_error - 2.1296914354).abs() < eps);
    assert!((c0.t_stat - 4.1425174899).abs() < eps);
    assert!((c0.p_value - 4.335485e-03).abs() < 1e-5);
    assert!((c0.ci_lower - 3.7863640028).abs() < 1e-4);
    assert!((c0.ci_upper - 13.8582040358).abs() < 1e-4);

    // x1: beta=3.9500185391, se=1.3282544530, t=2.9738417441, p=2.069471e-02, beta_std=1.1818296330
    let c1 = &res.coefficients[1];
    assert_eq!(c1.var_name, "x1");
    assert!((c1.b - 3.9500185391).abs() < eps);
    assert!((c1.std_error - 1.3282544530).abs() < eps);
    assert!((c1.t_stat - 2.9738417441).abs() < eps);
    assert!((c1.p_value - 2.069471e-02).abs() < 1e-5);
    assert!((c1.beta.unwrap() - 1.1818296330).abs() < eps);
    assert!((c1.vif.unwrap() - 212.9154616240).abs() < 1e-3);
    assert!((c1.tolerance.unwrap() - 0.0046966998).abs() < 1e-5);

    // x2: beta=-0.6729699666, se=1.4460761243, t=-0.4653765838, p=6.557931e-01, beta_std=-0.1849445547
    let c2 = &res.coefficients[2];
    assert_eq!(c2.var_name, "x2");
    assert!((c2.b - -0.6729699666).abs() < eps);
    assert!((c2.std_error - 1.4460761243).abs() < eps);
    assert!((c2.t_stat - -0.4653765838).abs() < eps);
    assert!((c2.p_value - 6.557931e-01).abs() < 1e-4);
    assert!((c2.beta.unwrap() - -0.1849445547).abs() < eps);
}

#[test]
fn test_weighted_regression_golden_oracle() {
    let y: Vec<Option<f64>> = vec![Some(12.0), Some(15.0), Some(18.0), Some(20.0), Some(24.0)];
    let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let w = vec![1.0, 2.0, 1.5, 3.0, 2.5];

    let indep_names = vec!["x".to_string()];
    let indep_vals = vec![x];
    let options = LinearRegressionOptions::default();

    let res = compute_linear_regression("y", &y, &indep_names, &indep_vals, &w, &options)
        .expect("Weighted regression should succeed");

    let eps = 1e-6;
    // Weighted OLS Python Golden Oracle:
    // R: 0.9935090450, R2: 0.9870602224, SE: 0.7987058498, F: 228.8432432432, p_F: 6.271529e-04
    assert!((res.r - 0.9935090450).abs() < eps);
    assert!((res.r_squared - 0.9870602224).abs() < eps);
    assert!((res.std_error_estimate - 0.7987058498).abs() < eps);
    assert!((res.anova.regression.f_stat.unwrap() - 228.8432432432).abs() < 1e-4);
    assert!((res.anova.regression.p_value.unwrap() - 6.271529e-04).abs() < 1e-5);

    // const: beta=9.0517241379, se=0.6982935999, t=12.9626336821, p=9.912056e-04
    let c0 = &res.coefficients[0];
    assert!((c0.b - 9.0517241379).abs() < eps);
    assert!((c0.std_error - 0.6982935999).abs() < eps);
    assert!((c0.t_stat - 12.9626336821).abs() < eps);
    assert!((c0.p_value - 9.912056e-04).abs() < 1e-5);

    // x: beta=2.8965517241, se=0.1914750718, t=15.1275656747, p=6.271529e-04
    let c1 = &res.coefficients[1];
    assert!((c1.b - 2.8965517241).abs() < eps);
    assert!((c1.std_error - 0.1914750718).abs() < eps);
    assert!((c1.t_stat - 15.1275656747).abs() < eps);
    assert!((c1.p_value - 6.271529e-04).abs() < 1e-5);
}

#[test]
fn test_insufficient_data_error() {
    let y = vec![Some(1.0), Some(2.0)];
    let x = vec![Some(1.0), Some(2.0)];
    let options = LinearRegressionOptions::default();

    let err = compute_linear_regression("y", &y, &["x".to_string()], &[x], &[], &options)
        .expect_err("Should fail due to insufficient data");
    assert!(err.to_string().contains("Insufficient cases"));
}
