use super::fixtures::{assert_reversibility_cycle, create_test_dataset};
use serde_json::json;
use statcraft_syntax::bus::CommandBus;

#[test]
fn test_matrix_compute_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "target": "double_salary",
        "expression": "salary * 2.0",
        "label": "Doubled Salary"
    });

    assert_reversibility_cycle(&mut bus, "transform.compute", params, &mut ds);
}

#[test]
fn test_matrix_recode_same_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "vars": ["gender"],
        "mappings": [
            { "old": { "Exact": 1.0 }, "new": { "Value": 10.0 } },
            { "old": { "Exact": 2.0 }, "new": { "Value": 20.0 } }
        ]
    });

    assert_reversibility_cycle(&mut bus, "transform.recode.same", params, &mut ds);
}

#[test]
fn test_matrix_recode_diff_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "source": "gender",
        "target": "gender_flag",
        "label": "Gender Binary Flag",
        "mappings": [
            { "old": { "Exact": 1.0 }, "new": { "Value": 0.0 } },
            { "old": { "Exact": 2.0 }, "new": { "Value": 1.0 } }
        ]
    });

    assert_reversibility_cycle(&mut bus, "transform.recode.diff", params, &mut ds);
}

#[test]
fn test_matrix_auto_recode_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "pairs": [
            {
                "source_var": "dept",
                "target_var": "dept_id"
            }
        ],
        "order": "lowest"
    });

    assert_reversibility_cycle(&mut bus, "transform.auto_recode", params, &mut ds);
}

#[test]
fn test_matrix_count_values_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "target_var": "count_ones",
        "target_label": "Occurrences of One",
        "variables": ["id", "gender"],
        "criteria": [
            { "ExactNumber": 1.0 }
        ]
    });

    assert_reversibility_cycle(&mut bus, "transform.count_values", params, &mut ds);
}

#[test]
fn test_matrix_create_dummies_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "variables": ["dept"],
        "root_prefix": "dept_",
        "omit_reference": false
    });

    assert_reversibility_cycle(&mut bus, "transform.create_dummies", params, &mut ds);
}

#[test]
fn test_matrix_shift_values_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "source_var": "salary",
        "target_var": "prev_salary",
        "direction": "lag",
        "step": 1,
        "target_label": "Previous Case Salary"
    });

    assert_reversibility_cycle(&mut bus, "transform.shift_values", params, &mut ds);
}

#[test]
fn test_matrix_rank_cases_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "variables": ["id"],
        "direction": "ascending",
        "ties": "mean",
        "rank_type": "rank",
        "prefix": "R_"
    });

    assert_reversibility_cycle(&mut bus, "transform.rank_cases", params, &mut ds);
}

#[test]
fn test_matrix_replace_missing_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "variables": [
            {
                "source_var": "salary",
                "target_var": "imputed_salary",
                "method": "series_mean"
            }
        ]
    });

    assert_reversibility_cycle(&mut bus, "transform.replace_missing", params, &mut ds);
}

#[test]
fn test_matrix_visual_binning_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "source_var": "salary",
        "target_var": "salary_group",
        "target_label": "Grouped Salary",
        "cutpoints": [
            { "upper_bound": 3000.0, "inclusive": true },
            { "upper_bound": 4500.0, "inclusive": true }
        ]
    });

    assert_reversibility_cycle(&mut bus, "transform.visual_binning", params, &mut ds);
}

#[test]
fn test_matrix_optimal_binning_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "variables": ["id"],
        "target_var": "gender",
        "max_bins": 3,
        "prefix": "OB_"
    });

    assert_reversibility_cycle(&mut bus, "transform.optimal_binning", params, &mut ds);
}

#[test]
fn test_matrix_create_time_series_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "variables": [
            {
                "source_var": "id",
                "target_var": "id_diff",
                "function": "diff",
                "order": 1,
                "target_label": "ID First Difference"
            }
        ]
    });

    assert_reversibility_cycle(&mut bus, "transform.create_time_series", params, &mut ds);
}

#[test]
fn test_matrix_date_time_wizard_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "operation": "extract",
        "target_var": "hire_year",
        "primary_var": "hiredate",
        "part": "year",
        "target_label": "Year of Hire"
    });

    assert_reversibility_cycle(&mut bus, "transform.date_time_wizard", params, &mut ds);
}

#[test]
fn test_matrix_prepare_data_modeling_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "inputs": ["gender", "id"],
        "target": "salary",
        "objective": "accuracy",
        "rescale_method": "zscore",
        "partition": false,
        "seed": 2026
    });

    assert_reversibility_cycle(&mut bus, "transform.adp", params, &mut ds);
}

#[test]
fn test_matrix_programmability_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "script": "tax = salary * 0.15\nIF gender == 1 THEN flag = 10 ELSE flag = 20"
    });

    assert_reversibility_cycle(&mut bus, "transform.programmability", params, &mut ds);
}
