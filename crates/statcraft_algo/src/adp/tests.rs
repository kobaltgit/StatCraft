use super::*;
use statcraft_core::{DataType, Measure, RawValue};

#[test]
fn test_screening_filters() {
    let opts = AdpScreeningOptions::default();

    // 1. High missing
    let mut missing_vals = vec![RawValue::Number(10.0); 10];
    for _ in 0..15 {
        missing_vals.push(RawValue::Null);
    }
    let v_missing = evaluate_screening(&missing_vals, None, &opts);
    assert!(v_missing.is_screened_out);
    assert!(v_missing.reason.unwrap().contains("Too many missing"));

    // 2. Quasi-constant
    let mut const_vals = vec![RawValue::Number(1.0); 98];
    const_vals.push(RawValue::Number(2.0));
    const_vals.push(RawValue::Number(3.0));
    let v_const = evaluate_screening(&const_vals, None, &opts);
    assert!(v_const.is_screened_out);
    assert!(v_const
        .reason
        .unwrap()
        .contains("Constant or quasi-constant"));

    // 3. High cardinality identifier (N >= 20, 100% unique)
    let id_vals: Vec<RawValue> = (0..50).map(|i| RawValue::Number(i as f64)).collect();
    let v_id = evaluate_screening(&id_vals, None, &opts);
    assert!(v_id.is_screened_out);
    assert!(v_id.reason.unwrap().contains("High-cardinality identifier"));

    // 4. Good variable
    let mut good_vals = Vec::new();
    for i in 0..100 {
        good_vals.push(RawValue::Number((i % 5) as f64));
    }
    let v_good = evaluate_screening(&good_vals, None, &opts);
    assert!(!v_good.is_screened_out);
}

#[test]
fn test_outlier_winsorization() {
    let mut vals: Vec<RawValue> = (0..100).map(|i| RawValue::Number(i as f64)).collect();
    vals.push(RawValue::Number(1000.0)); // Extreme outlier

    let opts = AdpOutlierOptions {
        method: OutlierMethod::ZScore { threshold: 3.0 },
        treatment: OutlierTreatment::Winsorize,
    };

    let res = treat_outliers(&vals, None, &opts);
    assert_eq!(res.outlier_count, 1);
    let upper = res.upper_bound.unwrap();
    if let RawValue::Number(capped) = res.treated_values.last().unwrap() {
        assert!((*capped - upper).abs() < 1e-6);
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_rescale_zscore_and_minmax() {
    let vals: Vec<RawValue> = vec![
        RawValue::Number(10.0),
        RawValue::Number(20.0),
        RawValue::Number(30.0),
    ];

    // Z-Score
    let z_res = rescale_numeric(&vals, AdpRescaleMethod::ZScore, None);
    if let (RawValue::Number(z1), RawValue::Number(z2), RawValue::Number(z3)) =
        (&z_res[0], &z_res[1], &z_res[2])
    {
        assert!((z1 + 1.0).abs() < 1e-6);
        assert!(z2.abs() < 1e-6);
        assert!((z3 - 1.0).abs() < 1e-6);
    }

    // MinMax [0, 1]
    let mm_res = rescale_numeric(&vals, AdpRescaleMethod::MinMax { min: 0.0, max: 1.0 }, None);
    if let (RawValue::Number(m1), RawValue::Number(m2), RawValue::Number(m3)) =
        (&mm_res[0], &mm_res[1], &mm_res[2])
    {
        assert!(m1.abs() < 1e-6);
        assert!((m2 - 0.5).abs() < 1e-6);
        assert!((m3 - 1.0).abs() < 1e-6);
    }
}

#[test]
fn test_partition_split() {
    let opts = AdpPartitionOptions {
        enabled: true,
        train_pct: 70.0,
        test_pct: 20.0,
        validate_pct: 10.0,
        seed: Some(12345),
    };

    let res = compute_partition(1000, &opts);
    assert_eq!(res.values.len(), 1000);
    assert_eq!(res.counts.total_count, 1000);

    // Verify proportions roughly match 70% / 20% / 10%
    assert!(res.counts.train_count > 650 && res.counts.train_count < 750);
    assert!(res.counts.test_count > 150 && res.counts.test_count < 250);
    assert!(res.counts.validate_count > 50 && res.counts.validate_count < 150);

    // Verify deterministic reproducibility
    let res2 = compute_partition(1000, &opts);
    assert_eq!(res.values, res2.values);
}

#[test]
fn test_predictive_power_pearson_and_anova() {
    // 1. Perfectly correlated continuous variables
    let x: Vec<RawValue> = (1..=20).map(|i| RawValue::Number(i as f64)).collect();
    let y: Vec<RawValue> = (1..=20)
        .map(|i| RawValue::Number(2.0 * i as f64 + 5.0))
        .collect();

    let pwr = compute_predictive_power("x", &x, Measure::Scale, &y, Measure::Scale, None);
    assert_eq!(pwr.method, "Pearson r");
    assert!((pwr.score - 1.0).abs() < 1e-6);
    assert!(pwr.p_value.unwrap() < 1e-5);

    // 2. Clear group separation for ANOVA
    let mut cat_x = Vec::new();
    let mut num_y = Vec::new();
    for _ in 0..10 {
        cat_x.push(RawValue::Text("GroupA".to_string()));
        num_y.push(RawValue::Number(10.0));
    }
    for _ in 0..10 {
        cat_x.push(RawValue::Text("GroupB".to_string()));
        num_y.push(RawValue::Number(100.0));
    }

    let pwr_anova = compute_predictive_power(
        "group",
        &cat_x,
        Measure::Nominal,
        &num_y,
        Measure::Scale,
        None,
    );
    assert_eq!(pwr_anova.method, "ANOVA eta^2");
    assert!((pwr_anova.score - 1.0).abs() < 1e-6);
    assert!(pwr_anova.p_value.unwrap() < 1e-5);
}

#[test]
fn test_full_adp_pipeline() {
    let n = 100;
    let salary_target: Vec<RawValue> = (0..n)
        .map(|i| RawValue::Number(20000.0 + (i as f64) * 500.0))
        .collect();

    let educ: Vec<RawValue> = (0..n)
        .map(|i| RawValue::Number(8.0 + (i % 12) as f64))
        .collect();
    let age: Vec<RawValue> = (0..n)
        .map(|i| RawValue::Number(20.0 + (i % 40) as f64))
        .collect();

    // Constant feature
    let const_feat: Vec<RawValue> = vec![RawValue::Number(1.0); n];

    let inputs = vec![
        (
            "educ".to_string(),
            educ.as_slice(),
            DataType::Numeric { decimals: 0 },
            Measure::Ordinal,
        ),
        (
            "age".to_string(),
            age.as_slice(),
            DataType::Numeric { decimals: 0 },
            Measure::Scale,
        ),
        (
            "const_var".to_string(),
            const_feat.as_slice(),
            DataType::Numeric { decimals: 0 },
            Measure::Nominal,
        ),
    ];

    let target = (
        "salary",
        salary_target.as_slice(),
        DataType::Numeric { decimals: 2 },
        Measure::Scale,
    );

    let opts = AdpOptions {
        objective: AdpObjective::OptimizeAccuracy,
        target_variable: Some("salary".to_string()),
        input_variables: vec![
            "educ".to_string(),
            "age".to_string(),
            "const_var".to_string(),
        ],
        rescale: AdpRescaleMethod::ZScore,
        partition: AdpPartitionOptions {
            enabled: true,
            ..Default::default()
        },
        interactions: AdpInteractionOptions {
            create_interactions: true,
            max_pairs: 2,
        },
        ..Default::default()
    };

    let res = compute_adp(&inputs, Some(target), None, &opts).unwrap();
    assert_eq!(res.total_inputs, 3);
    assert_eq!(res.screened_out_count, 1); // const_var screened out
    assert_eq!(res.recommended_count, 2); // educ, age recommended
    assert!(res.partition_counts.is_some());
    assert!(!res.predictive_powers.is_empty());
}
