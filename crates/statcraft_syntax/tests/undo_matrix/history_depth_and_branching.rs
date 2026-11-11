use super::fixtures::create_test_dataset;
use serde_json::json;
use statcraft_io::RawValue;
use statcraft_syntax::bus::CommandBus;

#[test]
fn test_history_depth_eviction_120_commands() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    // Explicitly set max depth to 100
    bus.set_max_history_depth(100);
    assert_eq!(bus.max_history_depth(), 100);

    // Execute 120 modifications
    for i in 0..120 {
        let params = json!({
            "col_index": 3,
            "label": format!("Department iteration {}", i)
        });
        bus.execute("var.set_label", &mut ds, params)
            .unwrap_or_else(|e| panic!("Execution at step {} failed: {:?}", i, e));
    }

    // Verify history stack is exactly 100 (first 20 evicted)
    assert_eq!(bus.history().len(), 100);
    assert!(bus.can_undo());

    // Verify the oldest entry remaining in history is iteration 20
    let oldest_entry = &bus.history()[0];
    assert_eq!(oldest_entry.params["label"], "Department iteration 20");

    // Verify 100 sequential undos execute without panic or failure
    for _ in 0..100 {
        bus.undo(&mut ds).expect("Undo should succeed");
    }

    assert_eq!(bus.history().len(), 0);
    assert!(!bus.can_undo());
    assert_eq!(bus.redo_stack().len(), 100);
    assert!(bus.can_redo());
}

#[test]
fn test_history_coalescence_cell_edits() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    let initial_val = ds.as_ref().unwrap().get_cell(0, 0).cloned();

    // User types into cell (0, 0): 1 -> 12 -> 123 -> 1234 -> 999
    let values = ["1", "12", "123", "1234", "999"];
    for val in values {
        let params = json!({
            "row": 0,
            "col": 0,
            "value": val
        });
        bus.execute_coalesced("data.cell_edit", &mut ds, params)
            .expect("Coalesced execute should succeed");
    }

    // Must be coalesced into exactly ONE history entry
    assert_eq!(bus.history().len(), 1);
    assert_eq!(
        ds.as_ref().unwrap().get_cell(0, 0),
        Some(&RawValue::Number(999.0))
    );

    // A single undo must restore the initial value prior to editing session
    bus.undo(&mut ds).expect("Undo must succeed");
    assert_eq!(bus.history().len(), 0);
    assert_eq!(ds.as_ref().unwrap().get_cell(0, 0), initial_val.as_ref());

    // Redo restores final value (999.0)
    bus.redo(&mut ds).expect("Redo must succeed");
    assert_eq!(
        ds.as_ref().unwrap().get_cell(0, 0),
        Some(&RawValue::Number(999.0))
    );
}

#[test]
fn test_history_branching_clears_redo() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_test_dataset());

    // Execute 3 steps
    bus.execute("data.add_row", &mut ds, json!({})).unwrap();
    bus.execute("data.add_row", &mut ds, json!({})).unwrap();
    bus.execute("data.add_row", &mut ds, json!({})).unwrap();
    assert_eq!(bus.history().len(), 3);
    assert_eq!(bus.redo_stack().len(), 0);

    // Undo 2 steps
    bus.undo(&mut ds).unwrap();
    bus.undo(&mut ds).unwrap();
    assert_eq!(bus.history().len(), 1);
    assert_eq!(bus.redo_stack().len(), 2);
    assert!(bus.can_redo());

    // Branching: execute a NEW command while redo stack is non-empty
    bus.execute(
        "var.set_label",
        &mut ds,
        json!({ "col_index": 0, "label": "Branched ID" }),
    )
    .unwrap();

    // Redo stack must be safely and completely cleared
    assert_eq!(bus.history().len(), 2);
    assert_eq!(bus.redo_stack().len(), 0);
    assert!(!bus.can_redo());
}
