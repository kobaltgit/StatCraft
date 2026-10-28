use super::*;

/// Golden Oracle: Verification against Student's t distribution tables and SciPy
#[test]
fn test_golden_oracle_student_t_distribution() {
    let eps = 1e-5;

    // df = 9, t = 2.26215716 => p (two-tailed) = 0.05
    let p1 = student_t_sf_2tailed(2.26215716, 9.0);
    assert!((p1 - 0.05).abs() < eps, "p1 error: {}", p1);

    let t_inv = student_t_inv_cdf(0.975, 9.0);
    assert!((t_inv - 2.26215716).abs() < eps, "t_inv error: {}", t_inv);

    // df = 30, t = 2.042272 => p = 0.05
    let p30 = student_t_sf_2tailed(2.042272, 30.0);
    assert!((p30 - 0.05).abs() < eps, "p30 error: {}", p30);

    let t30_inv = student_t_inv_cdf(0.975, 30.0);
    assert!(
        (t30_inv - 2.042272).abs() < eps,
        "t30_inv error: {}",
        t30_inv
    );
}

/// Golden Oracle Test: Reference values verified against Python SciPy stats
/// sample: [12.0, 15.0, 18.0, 20.0, 22.0, 25.0, 28.0, 30.0, 35.0, 40.0]
/// test_value = 20.0
/// SciPy:
///   statistic = 1.6038503831861783
///   df = 9
///   pvalue = 0.1432247197022207
///   mean = 24.5
///   std = 8.872554436139498
///   se_mean = 2.805748068202517
///   mean_difference = 4.5
///   95% CI of difference = [-1.847042571253724, 10.847042571253724]
#[test]
fn test_golden_oracle_ttest_onesample_scipy() {
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

    let options = OneSampleTTestOptions {
        test_value: 20.0,
        confidence_level: 0.95,
    };

    let res = compute_onesample_ttest("score", &sample, &options);

    assert_eq!(res.variable, "score");
    assert_eq!(res.n_valid, 10);
    assert_eq!(res.n_missing, 0);

    let eps = 1e-5;

    let mean = res.mean.unwrap();
    assert!((mean - 24.5).abs() < eps, "mean: {}", mean);

    let std_dev = res.std_dev.unwrap();
    assert!((std_dev - 8.8725544).abs() < eps, "std_dev: {}", std_dev);

    let se_mean = res.se_mean.unwrap();
    assert!((se_mean - 2.8057481).abs() < eps, "se_mean: {}", se_mean);

    let mean_diff = res.mean_diff.unwrap();
    assert!((mean_diff - 4.5).abs() < eps, "mean_diff: {}", mean_diff);

    let t_stat = res.t_stat.unwrap();
    assert!((t_stat - 1.6038504).abs() < eps, "t_stat: {}", t_stat);

    let df = res.df.unwrap();
    assert_eq!(df, 9.0);

    let sig = res.sig_2tailed.unwrap();
    assert!((sig - 0.14321035).abs() < eps, "sig: {}", sig);

    let ci_lower = res.ci_lower.unwrap();
    assert!(
        (ci_lower - (-1.8470426)).abs() < eps,
        "ci_lower: {}",
        ci_lower
    );

    let ci_upper = res.ci_upper.unwrap();
    assert!(
        (ci_upper - 10.8470426).abs() < eps,
        "ci_upper: {}",
        ci_upper
    );
}

#[test]
fn test_ttest_empty_and_single() {
    let empty: Vec<Option<f64>> = vec![];
    let res_empty = compute_onesample_ttest("empty", &empty, &OneSampleTTestOptions::default());
    assert_eq!(res_empty.n_valid, 0);
    assert_eq!(res_empty.t_stat, None);

    let single = vec![Some(42.0)];
    let res_single = compute_onesample_ttest("single", &single, &OneSampleTTestOptions::default());
    assert_eq!(res_single.n_valid, 1);
    assert_eq!(res_single.mean, Some(42.0));
    assert_eq!(res_single.t_stat, None);
}

#[test]
fn test_ttest_zero_variance() {
    let identical = vec![Some(5.0), Some(5.0), Some(5.0)];
    let opts = OneSampleTTestOptions {
        test_value: 5.0,
        confidence_level: 0.95,
    };
    let res = compute_onesample_ttest("constant", &identical, &opts);
    assert_eq!(res.t_stat, Some(0.0));
    assert_eq!(res.sig_2tailed, Some(1.0));
    assert_eq!(res.ci_lower, Some(0.0));
    assert_eq!(res.ci_upper, Some(0.0));
}
