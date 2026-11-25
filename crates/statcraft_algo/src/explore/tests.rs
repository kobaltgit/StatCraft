use super::*;
use statcraft_core::RawValue;

#[test]
fn test_compute_explore_single_var() {
    let mut ds = DataSet::blank(10, 1).unwrap();
    for r in 0..10 {
        ds.table.columns[0].values[r] = RawValue::Number((r + 1) as f64 * 10.0);
    }

    let options = ExploreOptions {
        dependent_vars: vec!["VAR00001".to_string()],
        ..Default::default()
    };

    let res = compute_explore(&ds, &options).unwrap();
    assert_eq!(res.variables.len(), 1);
    let var_res = &res.variables[0];
    assert_eq!(var_res.groups.len(), 1);

    let group = &var_res.groups[0];
    assert_eq!(group.case_processing.n_valid, 10);
    assert_eq!(group.case_processing.n_missing, 0);

    let desc = group.descriptives.as_ref().unwrap();
    assert_eq!(desc.mean, Some(55.0));

    let norm = group.normality.as_ref().unwrap();
    assert!(norm.kolmogorov_smirnov.is_some());
    assert!(norm.shapiro_wilk.is_some());

    let extremes = group.extremes.as_ref().unwrap();
    assert_eq!(extremes.lowest.len(), 5);
    assert_eq!(extremes.highest.len(), 5);
    assert!(extremes.lowest[0].case_label.is_none());
}

#[test]
fn test_compute_explore_with_factor() {
    let mut ds = DataSet::blank(6, 2).unwrap();
    for r in 0..6 {
        ds.table.columns[0].values[r] = RawValue::Number((r + 1) as f64 * 10.0);
        let grp = if r < 3 { "A" } else { "B" };
        ds.table.columns[1].values[r] = RawValue::Text(grp.to_string());
    }

    let options = ExploreOptions {
        dependent_vars: vec!["VAR00001".to_string()],
        factor_vars: vec!["VAR00002".to_string()],
        ..Default::default()
    };

    let res = compute_explore(&ds, &options).unwrap();
    assert_eq!(res.variables.len(), 1);
    let groups = &res.variables[0].groups;
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].factor_level.as_deref(), Some("A"));
    assert_eq!(groups[0].case_processing.n_valid, 3);
    assert_eq!(groups[1].factor_level.as_deref(), Some("B"));
    assert_eq!(groups[1].case_processing.n_valid, 3);
}

#[test]
fn test_compute_explore_full_options_labels_and_report_missing() {
    let mut ds = DataSet::blank(8, 3).unwrap();
    // Col 0: values, Col 1: factor (with a missing value), Col 2: label_var
    let vals = [10.0, 12.0, 14.0, 20.0, 25.0, 30.0, 50.0, 100.0];
    let facs = ["A", "A", "A", "B", "B", "B", "", "A"];
    let labels = ["id1", "id2", "id3", "id4", "id5", "id6", "id7", "id8"];

    for r in 0..8 {
        ds.table.columns[0].values[r] = RawValue::Number(vals[r]);
        if !facs[r].is_empty() {
            ds.table.columns[1].values[r] = RawValue::Text(facs[r].to_string());
        }
        ds.table.columns[2].values[r] = RawValue::Text(labels[r].to_string());
    }

    let options = ExploreOptions {
        dependent_vars: vec!["VAR00001".to_string()],
        factor_vars: vec!["VAR00002".to_string()],
        label_var: Some("VAR00003".to_string()),
        compute_descriptives: true,
        compute_m_estimators: true,
        compute_percentiles: true,
        compute_normality: false,
        compute_extremes: true,
        spread_level_mode: SpreadLevelMode::PowerEstimation,
        confidence_level: 0.95,
        missing: ExploreMissingHandling::Report,
        ..Default::default()
    };

    let res = compute_explore(&ds, &options).unwrap();
    let var_res = &res.variables[0];

    // Groups: "(Missing)", "A", "B"
    assert_eq!(var_res.groups.len(), 3);
    assert_eq!(var_res.groups[0].factor_level.as_deref(), Some("(Missing)"));
    assert_eq!(var_res.groups[0].case_processing.n_valid, 1);

    // Check case label on highest extreme of group A (value 100.0 -> id8)
    let grp_a = &var_res.groups[1];
    assert_eq!(grp_a.factor_level.as_deref(), Some("A"));
    let ext_a = grp_a.extremes.as_ref().unwrap();
    assert_eq!(ext_a.highest[0].value, 100.0);
    assert_eq!(ext_a.highest[0].case_label.as_deref(), Some("id8"));

    // Check M-estimators & Percentiles
    assert!(grp_a.m_estimators.as_ref().unwrap().huber.is_some());
    assert!(grp_a.percentiles.as_ref().unwrap().tukey_q1.is_some());

    // Check Levene & Spread-vs-Level
    assert!(var_res.levene_test.is_some());
    assert_eq!(var_res.levene_test.as_ref().unwrap().rows.len(), 4);
    assert!(var_res.spread_level.is_some());
}
