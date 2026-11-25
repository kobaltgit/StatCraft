use super::*;

/// Golden Oracle Test: Reference values verified against Python SciPy stats
/// sample: [12.0, 15.0, 18.0, 20.0, 22.0, 25.0, 28.0, 30.0, 35.0, 40.0]
#[test]
fn test_golden_oracle_descriptives_scipy() {
    let sample = vec![
        Some(12.0),
        Some(15.0),
        Some(18.0),
        Some(20.0),
        Some(22.0),
        Some(25.0),
        Some(28.0),
        Some(30.0),
        Some(35.0),
        Some(40.0),
    ];

    let result = compute_descriptives("age", &sample);

    assert_eq!(result.variable, "age");
    assert_eq!(result.n_valid, 10);
    assert_eq!(result.n_missing, 0);

    let epsilon = 1e-6;

    let mean = result.mean.unwrap();
    assert!((mean - 24.5).abs() < epsilon, "mean error: {}", mean);

    let sum = result.sum.unwrap();
    assert!((sum - 245.0).abs() < epsilon, "sum error: {}", sum);

    let min = result.min.unwrap();
    assert_eq!(min, 12.0);

    let max = result.max.unwrap();
    assert_eq!(max, 40.0);

    let range = result.range.unwrap();
    assert_eq!(range, 28.0);

    let var = result.variance.unwrap();
    let expected_var = 708.5 / 9.0;
    assert!((var - expected_var).abs() < epsilon, "var error: {}", var);

    let std_dev = result.std_dev.unwrap();
    let expected_std = (708.5 / 9.0_f64).sqrt();
    assert!(
        (std_dev - expected_std).abs() < epsilon,
        "std_dev error: {}",
        std_dev
    );

    let se_mean = result.se_mean.unwrap();
    let expected_se = expected_std / (10.0_f64).sqrt();
    assert!(
        (se_mean - expected_se).abs() < epsilon,
        "se_mean error: {}",
        se_mean
    );

    let skew = result.skewness.unwrap();
    assert!(
        (skew - 0.3776125768606358).abs() < epsilon,
        "skew error: {}",
        skew
    );

    let kurt = result.kurtosis.unwrap();
    assert!(
        (kurt - (-0.6155351644011618)).abs() < epsilon,
        "kurt error: {}",
        kurt
    );
}

#[test]
fn test_descriptives_empty_sample() {
    let empty: Vec<Option<f64>> = vec![];
    let result = compute_descriptives("empty", &empty);
    assert_eq!(result.n_valid, 0);
    assert_eq!(result.n_missing, 0);
    assert_eq!(result.mean, None);
    assert_eq!(result.variance, None);
}

#[test]
fn test_descriptives_all_missing_and_nans() {
    let sample = vec![None, Some(f64::NAN), Some(f64::INFINITY), None];
    let result = compute_descriptives("missing_var", &sample);
    assert_eq!(result.n_valid, 0);
    assert_eq!(result.n_missing, 4);
    assert_eq!(result.mean, None);
}

#[test]
fn test_descriptives_single_element() {
    let sample = vec![Some(42.0)];
    let result = compute_descriptives("single", &sample);
    assert_eq!(result.n_valid, 1);
    assert_eq!(result.mean, Some(42.0));
    assert_eq!(result.min, Some(42.0));
    assert_eq!(result.max, Some(42.0));
    assert_eq!(result.variance, None);
    assert_eq!(result.skewness, None);
}

#[test]
fn test_descriptives_weighted() {
    let values = vec![Some(10.0), Some(20.0), Some(999.0), None];
    let weights = vec![1.0, 3.0, 0.0, 2.0];

    let result = compute_descriptives_weighted("score", &values, &weights);

    assert_eq!(result.variable, "score");
    assert_eq!(result.n_valid, 4);
    assert_eq!(result.n_missing, 2);

    let epsilon = 1e-6;
    let mean = result.mean.expect("Mean should be present");
    assert!((mean - 17.5).abs() < epsilon, "mean error: {}", mean);

    let var = result.variance.expect("Variance should be present");
    assert!((var - 25.0).abs() < epsilon, "variance error: {}", var);

    let std_dev = result.std_dev.expect("Std dev should be present");
    assert!(
        (std_dev - 5.0).abs() < epsilon,
        "std_dev error: {}",
        std_dev
    );

    let se_mean = result.se_mean.expect("SE mean should be present");
    assert!(
        (se_mean - 2.5).abs() < epsilon,
        "se_mean error: {}",
        se_mean
    );

    assert_eq!(result.min, Some(10.0));
    assert_eq!(result.max, Some(20.0));
    assert_eq!(result.range, Some(10.0));
    assert_eq!(result.sum, Some(70.0));
}

/// Golden Oracle Test against `scipy.stats.zscore([12, 15, 18, 20, 22, 25, 28, 30, 35, 40], ddof=1)`
#[test]
fn test_golden_oracle_zscores_scipy() {
    let sample = vec![
        Some(12.0),
        Some(15.0),
        Some(18.0),
        Some(20.0),
        None,
        Some(22.0),
        Some(25.0),
        Some(28.0),
        Some(30.0),
        Some(35.0),
        Some(40.0),
    ];
    let z = compute_standardized_values(&sample, &[]);
    assert_eq!(z.len(), sample.len());
    assert_eq!(z[4], None);

    // Expected values from scipy.stats.zscore([12,15,18,20,22,25,28,30,35,40], ddof=1):
    // mean = 24.5, sd = sqrt(708.5 / 9) = 8.872554436136317
    let expected = [
        -1.408838923514783,
        -1.070717581871235,
        -0.7325962402276871,
        -0.5071820124653219,
        -0.2817677847029566,
        0.05635355694059132,
        0.39447489858413924,
        0.6198891263465045,
        1.1834246832870959,
        1.7469602402276872,
    ];
    let actual_valid: Vec<f64> = z.iter().filter_map(|&v| v).collect();
    assert_eq!(actual_valid.len(), expected.len());
    for (a, e) in actual_valid.iter().zip(expected.iter()) {
        assert!(
            (a - e).abs() < 1e-6,
            "zscore mismatch: got {a}, expected {e}"
        );
    }
}

#[test]
fn test_zscores_zero_stddev_returns_none() {
    let constant = vec![Some(5.0), Some(5.0), Some(5.0), None];
    let z = compute_standardized_values(&constant, &[]);
    assert_eq!(z, vec![None, None, None, None]);

    let single = vec![Some(10.0)];
    assert_eq!(compute_standardized_values(&single, &[]), vec![None]);
}

#[test]
fn test_apply_options_filter() {
    let sample = vec![Some(10.0), Some(20.0), Some(30.0), Some(40.0)];
    let mut res = compute_descriptives("x", &sample);
    let opts = DescriptivesOptions {
        mean: true,
        std_dev: false,
        variance: false,
        min: true,
        max: false,
        range: false,
        sum: true,
        se_mean: false,
        skewness: false,
        kurtosis: false,
        ..Default::default()
    };

    apply_options_filter(&mut res, &opts);
    assert_eq!(res.mean, Some(25.0));
    assert_eq!(res.min, Some(10.0));
    assert_eq!(res.sum, Some(100.0));
    assert_eq!(res.std_dev, None);
    assert_eq!(res.max, None);
    assert_eq!(res.variance, None);
    assert_eq!(res.skewness, None);
    assert_eq!(res.se_skewness, None);
    assert_eq!(res.kurtosis, None);
    assert_eq!(res.se_kurtosis, None);
    assert_eq!(res.applied_options, Some(opts));
}

#[test]
fn test_sort_descriptives_results() {
    let r_b = compute_descriptives("beta", &[Some(10.0), Some(20.0)]);
    let r_a = compute_descriptives("alpha", &[Some(30.0), Some(40.0)]);
    let r_c = compute_descriptives("gamma", &[Some(1.0), Some(2.0)]);

    let mut list = vec![r_b.clone(), r_a.clone(), r_c.clone()];
    sort_descriptives_results(&mut list, DescriptivesSortOrder::Alphabetic);
    assert_eq!(
        list.iter().map(|r| r.variable.as_str()).collect::<Vec<_>>(),
        vec!["alpha", "beta", "gamma"]
    );

    sort_descriptives_results(&mut list, DescriptivesSortOrder::AlphabeticDesc);
    assert_eq!(
        list.iter().map(|r| r.variable.as_str()).collect::<Vec<_>>(),
        vec!["gamma", "beta", "alpha"]
    );

    sort_descriptives_results(&mut list, DescriptivesSortOrder::AscendingMeans);
    assert_eq!(
        list.iter().map(|r| r.variable.as_str()).collect::<Vec<_>>(),
        vec!["gamma", "beta", "alpha"]
    );

    sort_descriptives_results(&mut list, DescriptivesSortOrder::DescendingMeans);
    assert_eq!(
        list.iter().map(|r| r.variable.as_str()).collect::<Vec<_>>(),
        vec!["alpha", "beta", "gamma"]
    );
}
