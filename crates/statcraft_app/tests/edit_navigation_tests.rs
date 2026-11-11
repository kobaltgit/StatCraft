use serde_json::json;
use statcraft_app::dialogs::{FindReplaceMode, GoToTab};
use statcraft_app::StatCraftApp;
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use statcraft_syntax::CommandBus;

fn make_sample_dataset() -> DataSet {
    let mut ds = DataSet::blank(3, 0).unwrap();
    ds.add_variable("name", DataType::String { max_len: 20 }, Measure::Nominal)
        .unwrap();
    ds.add_variable("score", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();

    ds.set_cell_parsed(0, 0, "Alice").unwrap();
    ds.set_cell_parsed(0, 1, "100").unwrap();

    ds.set_cell_parsed(1, 0, "Bob").unwrap();
    ds.set_cell_parsed(1, 1, "85").unwrap();

    ds.set_cell_parsed(2, 0, "Charlie").unwrap();
    ds.set_cell_parsed(2, 1, "92").unwrap();

    ds
}

#[test]
fn test_batch_replace_command_bus_undo_redo() {
    let mut ds = Some(make_sample_dataset());
    let mut bus = CommandBus::with_defaults();

    let params = json!({
        "find": "Bob",
        "replace": "Robert",
        "match_case": true,
        "match_entire_cell": true
    });

    let res = bus
        .execute("data.replace_values", &mut ds, params)
        .expect("Replace should succeed");
    assert_eq!(res["status"], "success");
    assert_eq!(res["replaced_count"], 1);

    assert_eq!(ds.as_ref().unwrap().get_cell_display(1, 0, false), "Robert");

    // Undo
    assert!(bus.can_undo());
    bus.undo(&mut ds).expect("Undo should succeed");
    assert_eq!(ds.as_ref().unwrap().get_cell_display(1, 0, false), "Bob");

    // Redo
    assert!(bus.can_redo());
    bus.redo(&mut ds).expect("Redo should succeed");
    assert_eq!(ds.as_ref().unwrap().get_cell_display(1, 0, false), "Robert");
}

#[test]
fn test_insert_case_at_index_and_undo() {
    let mut ds = Some(make_sample_dataset());
    let mut bus = CommandBus::with_defaults();

    assert_eq!(ds.as_ref().unwrap().row_count(), 3);

    // Insert row before index 1 (between Alice and Bob)
    let params = json!({ "row_index": 1 });
    let res = bus
        .execute("data.add_row", &mut ds, params)
        .expect("Insert case should succeed");
    assert_eq!(res["status"], "success");
    assert_eq!(res["row_index"], 1);
    assert_eq!(ds.as_ref().unwrap().row_count(), 4);

    // Check that row 1 is now empty / null and Bob shifted to row 2
    assert_eq!(ds.as_ref().unwrap().get_cell_display(0, 0, false), "Alice");
    assert_eq!(ds.as_ref().unwrap().get_cell_display(1, 0, false), ".");
    assert_eq!(ds.as_ref().unwrap().get_cell_display(2, 0, false), "Bob");

    // Undo insert
    bus.undo(&mut ds).expect("Undo should succeed");
    assert_eq!(ds.as_ref().unwrap().row_count(), 3);
    assert_eq!(ds.as_ref().unwrap().get_cell_display(1, 0, false), "Bob");
}

#[test]
fn test_insert_variable_at_index_and_undo() {
    let mut ds = Some(make_sample_dataset());
    let mut bus = CommandBus::with_defaults();

    assert_eq!(ds.as_ref().unwrap().col_count(), 2);

    // Insert new variable at column index 0
    let params = json!({
        "name": "id",
        "col_index": 0
    });
    let res = bus
        .execute("var.add", &mut ds, params)
        .expect("Insert var should succeed");
    assert_eq!(res["status"], "success");
    assert_eq!(res["col_index"], 0);
    assert_eq!(ds.as_ref().unwrap().col_count(), 3);

    // Check column order
    assert_eq!(ds.as_ref().unwrap().dictionary.get(0).unwrap().name, "id");
    assert_eq!(ds.as_ref().unwrap().dictionary.get(1).unwrap().name, "name");
    assert_eq!(
        ds.as_ref().unwrap().dictionary.get(2).unwrap().name,
        "score"
    );

    // Undo insert
    bus.undo(&mut ds).expect("Undo should succeed");
    assert_eq!(ds.as_ref().unwrap().col_count(), 2);
    assert_eq!(ds.as_ref().unwrap().dictionary.get(0).unwrap().name, "name");
}

#[test]
fn test_app_edit_navigation_state_wiring() {
    let mut app = StatCraftApp::new_headless();
    app.dataset = Some(make_sample_dataset());

    // Test opening Find dialog
    app.open_find_dialog();
    assert!(app.find_replace_dialog_state.is_open);
    assert_eq!(app.find_replace_dialog_state.mode, FindReplaceMode::Find);

    // Test opening Replace dialog
    app.open_replace_dialog();
    assert!(app.find_replace_dialog_state.is_open);
    assert_eq!(app.find_replace_dialog_state.mode, FindReplaceMode::Replace);

    // Test opening Go To Case
    app.data_view_state.selected_cell = Some((1, 0));
    app.open_goto_case_dialog();
    assert!(app.goto_dialog_state.is_open);
    assert_eq!(app.goto_dialog_state.active_tab, GoToTab::Case);
    assert_eq!(app.goto_dialog_state.case_number_text, "2");

    // Test opening Go To Variable
    app.open_goto_var_dialog();
    assert!(app.goto_dialog_state.is_open);
    assert_eq!(app.goto_dialog_state.active_tab, GoToTab::Variable);
    assert_eq!(app.goto_dialog_state.selected_var_index, Some(0));

    // Test clear selected cell
    app.clear_selected_cell();
    assert_eq!(
        app.dataset.as_ref().unwrap().get_cell(1, 0).unwrap(),
        &RawValue::Null
    );
}
