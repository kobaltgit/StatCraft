use crate::multinomial_logistic::{
    compute_multinomial_logistic, MultinomialLogisticOptions, ReferenceCategory,
};

#[test]
fn test_multinomial_logistic_three_categories_covariates() {
    // 3 categories: 1.0 (program A), 2.0 (program B), 3.0 (program C, reference)
    // Predictors: math score, writing score
    let math: Vec<Option<f64>> = vec![
        45.0, 52.0, 38.0, 65.0, 70.0, 58.0, 42.0, 49.0, 62.0, 75.0, 35.0, 48.0, 55.0, 68.0, 72.0,
        40.0, 50.0, 60.0, 70.0, 80.0, 32.0, 44.0, 56.0, 64.0, 76.0,
    ]
    .into_iter()
    .map(Some)
    .collect();

    let write: Vec<Option<f64>> = vec![
        50.0, 48.0, 42.0, 60.0, 68.0, 55.0, 46.0, 51.0, 59.0, 71.0, 39.0, 47.0, 53.0, 66.0, 70.0,
        43.0, 52.0, 58.0, 69.0, 78.0, 36.0, 45.0, 54.0, 63.0, 74.0,
    ]
    .into_iter()
    .map(Some)
    .collect();

    let prog: Vec<Option<f64>> = vec![
        1.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 1.0, 2.0, 3.0, 1.0, 1.0, 2.0, 2.0, 3.0, 1.0, 2.0, 2.0,
        3.0, 3.0, 1.0, 1.0, 2.0, 2.0, 3.0,
    ]
    .into_iter()
    .map(Some)
    .collect();

    let dep = ("prog", prog.as_slice());
    let covs = vec![("math", math.as_slice()), ("write", write.as_slice())];

    let opts = MultinomialLogisticOptions {
        reference_category: ReferenceCategory::Last,
        include_constant: true,
        confidence_level: 0.95,
        max_iterations: 50,
        convergence_tol: 1e-6,
        print_classification: true,
        print_goodness_of_fit: true,
        print_lrt: true,
    };

    let res = compute_multinomial_logistic(dep, &[], &covs, None, &opts)
        .expect("Multinomial logistic regression should succeed");

    assert_eq!(res.dependent_var, "prog");
    assert_eq!(res.reference_category_val, 3.0);
    assert_eq!(res.categories, vec![1.0, 2.0, 3.0]);
    assert_eq!(res.n_cases, 25);
    assert!(res.converged);

    // Case Processing Summary
    assert_eq!(res.case_processing.valid_n, 25);
    assert_eq!(res.case_processing.categories.len(), 3);

    // Model Fitting Info
    assert!(res.model_fitting.final_minus_2ll < res.model_fitting.intercept_only_minus_2ll);
    assert!(res.model_fitting.chi_square > 0.0);
    assert_eq!(res.model_fitting.df, 4); // (3 - 1) * 2 covariates = 4 df
    assert!(res.model_fitting.sig >= 0.0 && res.model_fitting.sig <= 1.0);

    // Pseudo R2
    assert!(res.pseudo_r_squared.cox_and_snell > 0.0);
    assert!(res.pseudo_r_squared.nagelkerke > 0.0);
    assert!(res.pseudo_r_squared.mcfadden > 0.0);

    // Parameter estimates: 2 non-reference categories * 3 parameters + 1 ref category * 3 = 9 rows
    assert_eq!(res.parameter_estimates.len(), 9);

    // Check that reference category rows have is_reference_category == true and b == 0
    let ref_rows: Vec<_> = res
        .parameter_estimates
        .iter()
        .filter(|r| r.is_reference_category)
        .collect();
    assert_eq!(ref_rows.len(), 3);
    for r in ref_rows {
        assert_eq!(r.category_value, 3.0);
        assert_eq!(r.b, 0.0);
    }

    // Likelihood Ratio Tests
    assert_eq!(res.likelihood_ratio_tests.len(), 3); // Intercept, math, write
    for lrt in &res.likelihood_ratio_tests {
        assert!(lrt.chi_square >= 0.0);
        assert!(lrt.df > 0);
    }

    // Classification table
    let ct = res
        .classification_table
        .as_ref()
        .expect("Classification table present");
    assert_eq!(ct.rows.len(), 3);
    assert!(ct.overall_percent > 50.0);
}

#[test]
fn test_multinomial_logistic_reference_category_first() {
    let x: Vec<Option<f64>> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]
        .into_iter()
        .map(Some)
        .collect();
    let y: Vec<Option<f64>> = vec![10.0, 10.0, 10.0, 20.0, 20.0, 20.0, 30.0, 30.0, 30.0]
        .into_iter()
        .map(Some)
        .collect();

    let dep = ("outcome", y.as_slice());
    let covs = vec![("x", x.as_slice())];

    let opts = MultinomialLogisticOptions {
        reference_category: ReferenceCategory::First,
        ..Default::default()
    };

    let res = compute_multinomial_logistic(dep, &[], &covs, None, &opts)
        .expect("Should compute with First as ref");

    assert_eq!(res.reference_category_val, 10.0);
    assert_eq!(res.categories, vec![20.0, 30.0, 10.0]);
}

#[test]
fn test_multinomial_logistic_with_factor() {
    let factor_group: Vec<Option<f64>> =
        vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0]
            .into_iter()
            .map(Some)
            .collect();

    let score: Vec<Option<f64>> = vec![
        10.0, 12.0, 14.0, 18.0, 20.0, 22.0, 11.0, 19.0, 13.0, 21.0, 15.0, 23.0,
    ]
    .into_iter()
    .map(Some)
    .collect();

    let response: Vec<Option<f64>> =
        vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 0.0, 1.0, 1.0, 2.0, 0.0, 2.0]
            .into_iter()
            .map(Some)
            .collect();

    let dep = ("choice", response.as_slice());
    let factors = vec![("group", factor_group.as_slice())];
    let covs = vec![("score", score.as_slice())];

    let opts = MultinomialLogisticOptions::default();
    let res = compute_multinomial_logistic(dep, &factors, &covs, None, &opts)
        .expect("Should compute with factor and covariate");

    assert_eq!(res.factors, vec!["group"]);
    assert_eq!(res.covariates, vec!["score"]);
    assert!(res.converged);
}

#[test]
fn test_multinomial_logistic_invalid_inputs() {
    let y: Vec<Option<f64>> = vec![1.0, 1.0, 1.0].into_iter().map(Some).collect();
    let x: Vec<Option<f64>> = vec![2.0, 3.0, 4.0].into_iter().map(Some).collect();

    let dep = ("y", y.as_slice());
    let covs = vec![("x", x.as_slice())];
    let opts = MultinomialLogisticOptions::default();

    // Only 1 unique category should fail
    let res = compute_multinomial_logistic(dep, &[], &covs, None, &opts);
    assert!(res.is_err());
}
