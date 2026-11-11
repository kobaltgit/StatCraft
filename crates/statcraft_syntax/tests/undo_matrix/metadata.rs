use super::fixtures::{assert_reversibility_cycle, create_test_dataset};
use serde_json::json;
use statcraft_syntax::bus::CommandBus;

#[test]
fn test_matrix_var_add_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "name": "bonus_pct",
        "type": "numeric",
        "measure": "scale"
    });

    assert_reversibility_cycle(&mut bus, "var.add", params, &mut ds);
}

#[test]
fn test_matrix_var_delete_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 2
    });

    assert_reversibility_cycle(&mut bus, "var.delete", params, &mut ds);
}

#[test]
fn test_matrix_var_rename_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 2,
        "new_name": "monthly_income"
    });

    assert_reversibility_cycle(&mut bus, "var.rename", params, &mut ds);
}

#[test]
fn test_matrix_var_set_type_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 2,
        "type": "dollar",
        "width": 10,
        "decimals": 2
    });

    assert_reversibility_cycle(&mut bus, "var.set_type", params, &mut ds);
}

#[test]
fn test_matrix_var_set_width_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 3,
        "width": 25
    });

    assert_reversibility_cycle(&mut bus, "var.set_width", params, &mut ds);
}

#[test]
fn test_matrix_var_set_decimals_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 0,
        "decimals": 3
    });

    assert_reversibility_cycle(&mut bus, "var.set_decimals", params, &mut ds);
}

#[test]
fn test_matrix_var_set_align_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 0,
        "align": "center"
    });

    assert_reversibility_cycle(&mut bus, "var.set_align", params, &mut ds);
}

#[test]
fn test_matrix_var_set_value_labels_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 1,
        "labels": {
            "1": "Man",
            "2": "Woman",
            "9": "Unknown"
        }
    });

    assert_reversibility_cycle(&mut bus, "var.set_value_labels", params, &mut ds);
}

#[test]
fn test_matrix_var_set_missing_values_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 2,
        "missing_type": "discrete",
        "discrete": ["-99", "-999"]
    });

    assert_reversibility_cycle(&mut bus, "var.set_missing_values", params, &mut ds);
}

#[test]
fn test_matrix_set_label_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 3,
        "label": "Corporate Business Unit"
    });

    assert_reversibility_cycle(&mut bus, "var.set_label", params, &mut ds);
}

#[test]
fn test_matrix_set_measure_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "col_index": 1,
        "measure": "ordinal"
    });

    assert_reversibility_cycle(&mut bus, "var.set_measure", params, &mut ds);
}
