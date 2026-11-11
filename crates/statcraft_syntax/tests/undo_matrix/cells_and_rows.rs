use super::fixtures::{assert_reversibility_cycle, create_test_dataset};
use serde_json::json;
use statcraft_syntax::bus::CommandBus;

#[test]
fn test_matrix_cell_edit_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "row": 0,
        "col": 2,
        "value": "9999.5"
    });

    assert_reversibility_cycle(&mut bus, "data.cell_edit", params, &mut ds);
}

#[test]
fn test_matrix_add_row_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({});

    assert_reversibility_cycle(&mut bus, "data.add_row", params, &mut ds);
}

#[test]
fn test_matrix_delete_row_reversibility() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "row_index": 2
    });

    assert_reversibility_cycle(&mut bus, "data.delete_row", params, &mut ds);
}
