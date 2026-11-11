use super::fixtures::{assert_reversibility_cycle, create_test_dataset};
use serde_json::json;
use statcraft_syntax::bus::CommandBus;

#[test]
fn test_matrix_sort_cases_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "keys": [
            { "column_name": "dept", "direction": "ascending" },
            { "column_name": "salary", "direction": "descending" }
        ]
    });

    assert_reversibility_cycle(&mut bus, "data.sort_cases", params, &mut ds);
}

#[test]
fn test_matrix_select_cases_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "mode": "condition",
        "condition": "salary > 3000",
        "action": "filter"
    });

    assert_reversibility_cycle(&mut bus, "data.select_cases", params, &mut ds);
}

#[test]
fn test_matrix_split_file_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "mode": "compare_groups",
        "variables": ["gender"]
    });

    assert_reversibility_cycle(&mut bus, "data.split_file", params, &mut ds);
}

#[test]
fn test_matrix_weight_cases_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "variable": "id"
    });

    assert_reversibility_cycle(&mut bus, "data.weight_cases", params, &mut ds);
}

#[test]
fn test_matrix_random_seed_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "seed": 20261111
    });

    assert_reversibility_cycle(&mut bus, "transform.random_seed", params, &mut ds);
}

#[test]
fn test_matrix_analytical_readonly_procedures_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    // Analytical procedures do not modify dataset
    let analytical_calls = vec![
        ("stat.descriptives", json!({ "vars": ["salary"] })),
        ("stat.frequencies", json!({ "vars": ["gender"] })),
        (
            "stat.means",
            json!({ "dependent_vars": ["salary"], "factor_vars": ["gender"] }),
        ),
        (
            "stat.ttest.onesample",
            json!({ "vars": ["salary"], "test_value": 3000.0 }),
        ),
        (
            "stat.ttest.indep",
            json!({ "vars": ["salary"], "grouping_var": "gender", "group1": 1.0, "group2": 2.0 }),
        ),
        (
            "stat.anova.oneway",
            json!({ "dependent_vars": ["salary"], "factor_var": "gender" }),
        ),
        ("stat.correlations", json!({ "vars": ["id", "salary"] })),
    ];

    for (cmd_id, params) in analytical_calls {
        assert_reversibility_cycle(&mut bus, cmd_id, params, &mut ds);
    }
}
