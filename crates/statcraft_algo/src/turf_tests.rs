use crate::turf::{compute_turf, estimate_turf_operations};
use crate::turf_types::TurfOptions;
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use std::collections::HashMap;

fn make_survey_dataset(names: &[&str], rows: &[Vec<Option<f64>>]) -> DataSet {
    let mut ds = DataSet::empty();
    for name in names {
        let _ = ds.add_variable(name, DataType::Numeric { decimals: 0 }, Measure::Nominal);
    }
    for (r_idx, row) in rows.iter().enumerate() {
        ds.add_empty_row();
        for (c_idx, v) in row.iter().enumerate() {
            let val = match v {
                Some(n) => RawValue::Number(*n),
                None => RawValue::Null,
            };
            let _ = ds.set_cell(r_idx, c_idx, val);
        }
    }
    ds
}

#[test]
fn golden_oracle_exact_reach_and_frequency() {
    // 5 respondents x 4 items:
    // R0: A=1, B=0, C=0, D=1
    // R1: A=1, B=1, C=0, D=0
    // R2: A=0, B=1, C=1, D=0
    // R3: A=0, B=0, C=1, D=1
    // R4: A=0, B=0, C=0, D=0
    // Individual reach: A=2(40%), B=2(40%), C=2(40%), D=2(40%)
    // Pair reach:
    //   A+C -> R0,R1,R2,R3 = 4 (80%), freq = 4 (80%), mean = 1.0
    //   B+D -> R0,R1,R2,R3 = 4 (80%), freq = 4 (80%), mean = 1.0
    //   A+B -> R0,R1,R2    = 3 (60%), freq = 4 (80%)
    // Triple reach:
    //   A+B+C -> R0,R1,R2,R3 = 4 (80%), freq = 6 (120%), mean = 1.5
    let ds = make_survey_dataset(
        &["A", "B", "C", "D"],
        &[
            vec![Some(1.0), Some(0.0), Some(0.0), Some(1.0)],
            vec![Some(1.0), Some(1.0), Some(0.0), Some(0.0)],
            vec![Some(0.0), Some(1.0), Some(1.0), Some(0.0)],
            vec![Some(0.0), Some(0.0), Some(1.0), Some(1.0)],
            vec![Some(0.0), Some(0.0), Some(0.0), Some(0.0)],
        ],
    );
    let opts = TurfOptions {
        variables: vec!["A".into(), "B".into(), "C".into(), "D".into()],
        max_combinations: Some(3),
        ..Default::default()
    };
    let res = compute_turf(&ds, &opts).expect("turf should succeed");
    assert!((res.total_valid_cases - 5.0).abs() < 1e-9);
    assert_eq!(res.cycles.len(), 1);
    let c0 = &res.cycles[0];
    assert!(!c0.used_heats);
    assert_eq!(c0.rows_by_size.len(), 3);

    // Best pair: A+C (reach 4, 80%, freq 4, 50% of 8 total positive responses)
    let pairs = &c0.rows_by_size[1].1;
    assert_eq!(pairs[0].variables, vec!["A", "C"]);
    assert!((pairs[0].reach_count - 4.0).abs() < 1e-9);
    assert!((pairs[0].reach_pct - 80.0).abs() < 1e-9);
    assert!((pairs[0].frequency_count - 4.0).abs() < 1e-9);
    assert!((pairs[0].frequency_pct - 50.0).abs() < 1e-9);
    assert!((pairs[0].mean_responses_per_reached - 1.0).abs() < 1e-9);

    // Best triple: reach 4, freq 6, 75% of 8 total positive responses, mean 1.5
    let triples = &c0.rows_by_size[2].1;
    assert!((triples[0].reach_count - 4.0).abs() < 1e-9);
    assert!((triples[0].frequency_count - 6.0).abs() < 1e-9);
    assert!((triples[0].frequency_pct - 75.0).abs() < 1e-9);
    assert!((triples[0].mean_responses_per_reached - 1.5).abs() < 1e-9);
}

#[test]
fn test_weight_and_filter_respect() {
    let mut ds = make_survey_dataset(
        &["A", "B", "wt", "flt"],
        &[
            vec![Some(1.0), Some(0.0), Some(2.0), Some(1.0)],
            vec![Some(0.0), Some(1.0), Some(3.0), Some(1.0)],
            vec![Some(1.0), Some(1.0), Some(10.0), Some(0.0)], // filtered out
        ],
    );
    ds.set_weight_variable(Some("wt".into())).expect("wt");
    ds.set_filter_variable(Some("flt".into())).expect("flt");

    let opts = TurfOptions {
        variables: vec!["A".into(), "B".into()],
        max_combinations: Some(2),
        ..Default::default()
    };
    let res = compute_turf(&ds, &opts).expect("turf weighted");
    assert_eq!(res.total_cases_unweighted, 2);
    assert!((res.total_valid_cases - 5.0).abs() < 1e-9);
    let size1 = &res.cycles[0].rows_by_size[0].1;
    assert_eq!(size1[0].variables, vec!["B"]);
    assert!((size1[0].reach_count - 3.0).abs() < 1e-9);
    assert!((size1[0].reach_pct - 60.0).abs() < 1e-9);
}

#[test]
fn test_force_minpercent_and_removal_cycles() {
    let ds = make_survey_dataset(
        &["F1", "V1", "V2", "Low"],
        &[
            vec![Some(1.0), Some(0.0), Some(0.0), Some(0.0)],
            vec![Some(1.0), Some(1.0), Some(0.0), Some(0.0)],
            vec![Some(0.0), Some(1.0), Some(1.0), Some(0.0)],
            vec![Some(0.0), Some(0.0), Some(1.0), Some(0.0)],
            vec![Some(0.0), Some(0.0), Some(0.0), Some(1.0)], // Low = 20%
        ],
    );
    let opts = TurfOptions {
        variables: vec!["F1".into(), "V1".into(), "V2".into(), "Low".into()],
        always_required: 1,
        min_positive_pct: Some(25.0),
        max_combinations: Some(2),
        removal_cycles: 1,
        ..Default::default()
    };
    let res = compute_turf(&ds, &opts).expect("turf force");
    assert_eq!(res.excluded_variables.len(), 1);
    assert_eq!(res.excluded_variables[0].0, "Low");
    assert_eq!(res.cycles.len(), 2);

    // Cycle 0: F1 must be in every combination
    for (_, rows) in &res.cycles[0].rows_by_size {
        for r in rows {
            assert!(r.variables.contains(&"F1".to_string()));
        }
    }
    // Cycle 1: one non-forced variable was removed
    assert_eq!(res.cycles[1].removed_variables.len(), 1);
}

#[test]
fn test_importance_weights_strict_and_ranking() {
    let ds = make_survey_dataset(
        &["A", "B"],
        &[vec![Some(1.0), Some(0.0)], vec![Some(0.0), Some(1.0)]],
    );
    let mut weights = HashMap::new();
    weights.insert("A".to_string(), 1.0);
    let strict_opts = TurfOptions {
        variables: vec!["A".into(), "B".into()],
        importance_weights: weights.clone(),
        weights_strict: true,
        ..Default::default()
    };
    assert!(compute_turf(&ds, &strict_opts).is_err());

    let non_strict = TurfOptions {
        variables: vec!["A".into(), "B".into()],
        importance_weights: {
            let mut w = HashMap::new();
            w.insert("B".to_string(), 3.0);
            w
        },
        weights_strict: false,
        max_combinations: Some(1),
        ..Default::default()
    };
    let res = compute_turf(&ds, &non_strict).expect("non-strict ok");
    let top = &res.cycles[0].rows_by_size[0].1[0];
    assert_eq!(top.variables, vec!["B"]);
    assert_eq!(top.weighted_score, Some(3.0));
}

#[test]
fn test_max_operations_abort_and_heats_fallback() {
    let mut names = Vec::new();
    for i in 0..24 {
        names.push(format!("V{i:02}"));
    }
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut rows = Vec::new();
    for r in 0..10 {
        let mut row = vec![Some(0.0); 24];
        row[r % 24] = Some(1.0);
        row[(r * 3 + 1) % 24] = Some(1.0);
        rows.push(row);
    }
    let ds = make_survey_dataset(&name_refs, &rows);

    let abort_opts = TurfOptions {
        variables: names.clone(),
        max_combinations: Some(4),
        max_operations: 500,
        use_advanced_selection: false,
        limit_exceeded_abort: true,
        ..Default::default()
    };
    assert!(compute_turf(&ds, &abort_opts).is_err());

    let heats_opts = TurfOptions {
        variables: names,
        max_combinations: Some(4),
        max_operations: 500,
        use_advanced_selection: true,
        advanced_auto: true,
        advanced_size: 8,
        advanced_winners: 2,
        ..Default::default()
    };
    let res = compute_turf(&ds, &heats_opts).expect("heats should succeed");
    assert!(res.cycles[0].used_heats);
    assert!(!res.cycles[0].rows_by_size.is_empty());
    assert!(estimate_turf_operations(24, 0, 4) > 500);
}
