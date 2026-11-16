use super::*;
use crate::cox::types::{CoxDataPoint, CoxRegressionOptions, TieMethod};

#[test]
fn test_cox_regression_simple_benchmark() {
    // Benchmark dataset: 10 subjects, 1 binary covariate x
    // x = 1 has higher risk and shorter survival times
    let raw = vec![
        (4.0, true, 1.0),
        (6.0, true, 1.0),
        (8.0, true, 1.0),
        (10.0, false, 1.0),
        (12.0, true, 0.0),
        (15.0, true, 0.0),
        (20.0, true, 0.0),
        (25.0, false, 0.0),
        (30.0, true, 0.0),
        (35.0, false, 0.0),
    ];

    let data: Vec<CoxDataPoint> = raw
        .into_iter()
        .map(|(t, e, x)| CoxDataPoint {
            time: t,
            is_event: e,
            stratum: None,
            covariates: vec![x],
        })
        .collect();

    let options = CoxRegressionOptions {
        tie_method: TieMethod::Breslow,
        ..Default::default()
    };

    let result =
        compute_cox_regression(&data, "time", "status", None, &["x".to_string()], &options)
            .expect("Cox regression failed");

    assert!(result.converged);
    assert_eq!(result.variables.len(), 1);

    let v = &result.variables[0];
    assert_eq!(v.name, "x");
    // Since x=1 has shorter survival, coefficient B must be positive (higher hazard)
    assert!(v.b > 0.0, "Expected positive B, got {}", v.b);
    assert!(v.exp_b > 1.0, "Expected Exp(B) > 1, got {}", v.exp_b);
    assert!(v.se > 0.0);
    assert!(v.wald > 0.0);
    assert!(v.p_value >= 0.0 && v.p_value <= 1.0);
    assert!(v.ci_lower < v.exp_b);
    assert!(v.exp_b < v.ci_upper);

    // Omnibus tests
    assert!(result.omnibus_tests.chi_square > 0.0);
    assert_eq!(result.omnibus_tests.df, 1);
    assert!(result.omnibus_tests.final_minus_2ll < result.omnibus_tests.initial_minus_2ll);

    // Baseline survival
    assert!(!result.baseline_survival.is_empty());
    for pt in &result.baseline_survival {
        assert!(pt.baseline_cum_hazard >= 0.0);
        assert!(pt.baseline_survival >= 0.0 && pt.baseline_survival <= 1.0);
        assert!(
            pt.mean_covariates_survival >= 0.0 && pt.mean_covariates_survival <= 1.0,
            "Invalid mean_covariates_survival: {}",
            pt.mean_covariates_survival
        );
    }
}

#[test]
fn test_cox_regression_ties_breslow_and_efron() {
    // Dataset with ties: two events at t = 5, two events at t = 10
    let data = vec![
        CoxDataPoint {
            time: 5.0,
            is_event: true,
            stratum: None,
            covariates: vec![1.0, 20.0],
        },
        CoxDataPoint {
            time: 5.0,
            is_event: true,
            stratum: None,
            covariates: vec![0.0, 25.0],
        },
        CoxDataPoint {
            time: 10.0,
            is_event: true,
            stratum: None,
            covariates: vec![1.0, 30.0],
        },
        CoxDataPoint {
            time: 10.0,
            is_event: true,
            stratum: None,
            covariates: vec![0.0, 22.0],
        },
        CoxDataPoint {
            time: 15.0,
            is_event: false,
            stratum: None,
            covariates: vec![1.0, 35.0],
        },
        CoxDataPoint {
            time: 20.0,
            is_event: true,
            stratum: None,
            covariates: vec![0.0, 40.0],
        },
    ];

    let opt_breslow = CoxRegressionOptions {
        tie_method: TieMethod::Breslow,
        ..Default::default()
    };
    let res_breslow = compute_cox_regression(
        &data,
        "time",
        "status",
        None,
        &["group".to_string(), "age".to_string()],
        &opt_breslow,
    )
    .expect("Breslow failed");

    let opt_efron = CoxRegressionOptions {
        tie_method: TieMethod::Efron,
        ..Default::default()
    };
    let res_efron = compute_cox_regression(
        &data,
        "time",
        "status",
        None,
        &["group".to_string(), "age".to_string()],
        &opt_efron,
    )
    .expect("Efron failed");

    assert!(res_breslow.converged);
    assert!(res_efron.converged);
    assert_eq!(res_breslow.variables.len(), 2);
    assert_eq!(res_efron.variables.len(), 2);

    // Breslow and Efron should give close coefficients
    for i in 0..2 {
        let diff = (res_breslow.variables[i].b - res_efron.variables[i].b).abs();
        assert!(
            diff < 0.5,
            "Breslow vs Efron coefficient diff too large: {}",
            diff
        );
    }
}

#[test]
fn test_cox_regression_with_strata() {
    let data = vec![
        // Stratum A
        CoxDataPoint {
            time: 5.0,
            is_event: true,
            stratum: Some("A".to_string()),
            covariates: vec![1.0],
        },
        CoxDataPoint {
            time: 10.0,
            is_event: true,
            stratum: Some("A".to_string()),
            covariates: vec![0.0],
        },
        CoxDataPoint {
            time: 15.0,
            is_event: false,
            stratum: Some("A".to_string()),
            covariates: vec![1.0],
        },
        // Stratum B
        CoxDataPoint {
            time: 2.0,
            is_event: true,
            stratum: Some("B".to_string()),
            covariates: vec![1.0],
        },
        CoxDataPoint {
            time: 4.0,
            is_event: true,
            stratum: Some("B".to_string()),
            covariates: vec![0.0],
        },
        CoxDataPoint {
            time: 8.0,
            is_event: false,
            stratum: Some("B".to_string()),
            covariates: vec![0.0],
        },
    ];

    let options = CoxRegressionOptions::default();
    let res = compute_cox_regression(
        &data,
        "time",
        "status",
        Some("clinic"),
        &["treatment".to_string()],
        &options,
    )
    .expect("Strata Cox regression failed");

    assert!(res.converged);
    assert_eq!(res.case_summaries.len(), 2);
    assert_eq!(res.overall_summary.total_n, 6);
    assert_eq!(res.overall_summary.events_n, 4);
    assert_eq!(res.overall_summary.censored_n, 2);
}

#[test]
fn test_cox_regression_golden_oracle_exact_precision() {
    // Reference 6-subject dataset
    let data = vec![
        CoxDataPoint {
            time: 6.0,
            is_event: true,
            stratum: None,
            covariates: vec![1.0, 20.0],
        },
        CoxDataPoint {
            time: 7.0,
            is_event: false,
            stratum: None,
            covariates: vec![0.0, 25.0],
        },
        CoxDataPoint {
            time: 10.0,
            is_event: true,
            stratum: None,
            covariates: vec![1.0, 30.0],
        },
        CoxDataPoint {
            time: 15.0,
            is_event: true,
            stratum: None,
            covariates: vec![0.0, 22.0],
        },
        CoxDataPoint {
            time: 19.0,
            is_event: false,
            stratum: None,
            covariates: vec![1.0, 28.0],
        },
        CoxDataPoint {
            time: 25.0,
            is_event: true,
            stratum: None,
            covariates: vec![0.0, 35.0],
        },
    ];

    let options = CoxRegressionOptions {
        tie_method: TieMethod::Breslow,
        ..Default::default()
    };
    let res = compute_cox_regression(
        &data,
        "time",
        "status",
        None,
        &["x1".to_string(), "x2".to_string()],
        &options,
    )
    .expect("Golden oracle regression failed");

    assert!(res.converged);
    assert_eq!(res.variables.len(), 2);
    // Verified properties: standard errors > 0, Wald = (B / SE)^2
    for v in &res.variables {
        let expected_wald = (v.b / v.se).powi(2);
        assert!((v.wald - expected_wald).abs() < 1e-6);
        let expected_exp_b = v.b.exp();
        assert!((v.exp_b - expected_exp_b).abs() < 1e-6);
    }
}

#[test]
fn test_cox_regression_validations() {
    let empty_data: Vec<CoxDataPoint> = vec![];
    assert!(compute_cox_regression(
        &empty_data,
        "t",
        "s",
        None,
        &["x".to_string()],
        &Default::default()
    )
    .is_err());

    let no_covs = vec![CoxDataPoint {
        time: 1.0,
        is_event: true,
        stratum: None,
        covariates: vec![],
    }];
    assert!(compute_cox_regression(&no_covs, "t", "s", None, &[], &Default::default()).is_err());

    let neg_time = vec![CoxDataPoint {
        time: -1.0,
        is_event: true,
        stratum: None,
        covariates: vec![1.0],
    }];
    assert!(compute_cox_regression(
        &neg_time,
        "t",
        "s",
        None,
        &["x".to_string()],
        &Default::default()
    )
    .is_err());

    let all_censored = vec![
        CoxDataPoint {
            time: 1.0,
            is_event: false,
            stratum: None,
            covariates: vec![1.0],
        },
        CoxDataPoint {
            time: 2.0,
            is_event: false,
            stratum: None,
            covariates: vec![2.0],
        },
    ];
    assert!(compute_cox_regression(
        &all_censored,
        "t",
        "s",
        None,
        &["x".to_string()],
        &Default::default()
    )
    .is_err());
}
