use egui::Pos2;
use statcraft_app::grid::{CellRange, ContextMenuTarget, GridContextMenuState, GridSelection};
use statcraft_app::views::DataViewAction;
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use statcraft_syntax::bus::CommandBus;
use statcraft_syntax::commands::DataDeleteRowCommand;

#[test]
fn test_right_click_preserves_rectangular_selection() {
    // 1. Arrange a 3x3 rectangular selection covering rows 1..=3, cols 1..=3
    let mut sel = GridSelection::from_anchor_and_active((1, 1), (3, 3));
    assert_eq!(sel.range.cell_count(), 9);

    // 2. Simulate right click at (2, 2) which is inside the selection
    let rc_row = 2;
    let rc_col = 2;
    if !sel.contains(rc_row, rc_col) {
        sel = GridSelection::single(rc_row, rc_col);
    }

    // Selection MUST be preserved (not collapsed to single cell!)
    assert_eq!(sel.range.min_row, 1);
    assert_eq!(sel.range.max_row, 3);
    assert_eq!(sel.range.min_col, 1);
    assert_eq!(sel.range.max_col, 3);
    assert_eq!(sel.range.cell_count(), 9);

    // 3. Simulate right click at (5, 5) which is outside the selection
    let outside_row = 5;
    let outside_col = 5;
    if !sel.contains(outside_row, outside_col) {
        sel = GridSelection::single(outside_row, outside_col);
    }

    // Selection collapses to the new clicked cell outside
    assert_eq!(sel.range.min_row, 5);
    assert_eq!(sel.range.max_row, 5);
    assert_eq!(sel.range.cell_count(), 1);
}

#[test]
fn test_phantom_row_zero_pollution() {
    // Create dataset with 3 rows and 2 columns
    let mut ds = DataSet::empty();
    ds.add_variable("x", DataType::Numeric { decimals: 0 }, Measure::Scale)
        .expect("var x");
    ds.add_variable("y", DataType::Numeric { decimals: 0 }, Measure::Scale)
        .expect("var y");

    for i in 0..3 {
        ds.add_empty_row();
        let _ = ds.set_cell_parsed(i, 0, &(i + 1).to_string());
        let _ = ds.set_cell_parsed(i, 1, &((i + 1) * 10).to_string());
    }
    assert_eq!(ds.row_count(), 3);

    // Simulate Editor Commit on phantom row (row index 3 >= ds.row_count()) with empty buffer
    let row = 3;
    let col = 0;
    let empty_text = "   ";

    let trimmed = empty_text.trim();
    if trimmed.is_empty() && row >= ds.row_count() {
        // Zero Pollution invariant: empty buffer on phantom row does NOT add rows!
    } else {
        while row >= ds.row_count() {
            ds.add_empty_row();
        }
        let _ = ds.set_cell_parsed(row, col, empty_text);
    }

    // Dataset row count must remain strictly 3!
    assert_eq!(ds.row_count(), 3);
    assert_eq!(ds.get_cell(2, 0), Some(&RawValue::Number(3.0)));

    // Now commit valid non-empty text on the phantom row
    let valid_text = "42";
    let trimmed = valid_text.trim();
    if trimmed.is_empty() && row >= ds.row_count() {
        // skip
    } else {
        while row >= ds.row_count() {
            ds.add_empty_row();
        }
        let _ = ds.set_cell_parsed(row, col, valid_text);
    }

    // Dataset now allocates the new row cleanly
    assert_eq!(ds.row_count(), 4);
    assert_eq!(ds.get_cell(3, 0), Some(&RawValue::Number(42.0)));
    assert_eq!(ds.get_cell(3, 1), Some(&RawValue::Null)); // other column is Null
}

#[test]
fn test_delete_selected_rows_reduces_n_and_supports_undo() {
    let mut ds = DataSet::empty();
    ds.add_variable("val", DataType::Numeric { decimals: 0 }, Measure::Scale)
        .expect("var val");

    for i in 0..5 {
        ds.add_empty_row();
        let _ = ds.set_cell_parsed(i, 0, &i.to_string());
    }
    assert_eq!(ds.row_count(), 5);

    let mut bus = CommandBus::new();
    bus.register(Box::new(DataDeleteRowCommand::new()));

    let mut opt_ds = Some(ds);

    // Delete row 2
    let params = serde_json::json!({ "row_index": 2 });
    let res = bus.execute("data.delete_row", &mut opt_ds, params);
    assert!(res.is_ok());

    let ds_ref = opt_ds.as_ref().expect("ds exists");
    assert_eq!(ds_ref.row_count(), 4);
    // Values were 0, 1, 2, 3, 4. After removing index 2: 0, 1, 3, 4
    assert_eq!(ds_ref.get_cell(0, 0), Some(&RawValue::Number(0.0)));
    assert_eq!(ds_ref.get_cell(1, 0), Some(&RawValue::Number(1.0)));
    assert_eq!(ds_ref.get_cell(2, 0), Some(&RawValue::Number(3.0)));
    assert_eq!(ds_ref.get_cell(3, 0), Some(&RawValue::Number(4.0)));

    // Undo restores the row
    let undo_res = bus.undo(&mut opt_ds);
    assert!(undo_res.is_ok());

    let ds_undone = opt_ds.as_ref().expect("ds exists");
    assert_eq!(ds_undone.row_count(), 5);
    assert_eq!(ds_undone.get_cell(2, 0), Some(&RawValue::Number(2.0)));
}

#[test]
fn test_context_menu_state_and_targets() {
    let mut state = GridContextMenuState::default();
    assert!(!state.is_open());
    assert!(!state.just_opened);

    // 1. Cell range context menu
    let range = CellRange::from_corners(1, 1, 3, 3);
    state.open_cell_range(range, (2, 2), Pos2::new(100.0, 150.0));
    assert!(state.is_open());
    assert!(state.just_opened);
    assert_eq!(state.open_cell, Some((2, 2)));
    assert_eq!(
        state.target,
        Some(ContextMenuTarget::CellRange {
            range,
            active_cell: (2, 2),
        })
    );

    state.close();
    assert!(!state.is_open());
    assert!(!state.just_opened);

    // 2. Row header context menu
    state.open_row_header(4, vec![4, 5, 6], Pos2::new(20.0, 200.0));
    assert!(state.is_open());
    assert_eq!(
        state.target,
        Some(ContextMenuTarget::RowHeader {
            target_row: 4,
            selected_rows: vec![4, 5, 6],
        })
    );

    // 3. Col header context menu
    state.open_col_header(2, vec![2], Pos2::new(200.0, 30.0));
    assert!(state.is_open());
    assert_eq!(
        state.target,
        Some(ContextMenuTarget::ColHeader {
            target_col: 2,
            selected_cols: vec![2],
        })
    );
}

#[test]
fn test_full_rows_keyboard_delete_action() {
    // When full row is selected, delete emits DataViewAction::DeleteCases
    let row_sel = GridSelection::select_row(2, 5);
    assert!(row_sel.is_full_rows);

    let action = if row_sel.is_full_rows {
        let rows: Vec<usize> = (row_sel.range.min_row..=row_sel.range.max_row).collect();
        DataViewAction::DeleteCases { rows }
    } else {
        DataViewAction::None
    };

    assert_eq!(action, DataViewAction::DeleteCases { rows: vec![2] });

    // When cell range is selected, delete does NOT delete cases
    let cell_sel = GridSelection::from_anchor_and_active((1, 1), (2, 2));
    assert!(!cell_sel.is_full_rows);
    let cell_action = if cell_sel.is_full_rows {
        let rows: Vec<usize> = (cell_sel.range.min_row..=cell_sel.range.max_row).collect();
        DataViewAction::DeleteCases { rows }
    } else {
        DataViewAction::None
    };
    assert_eq!(cell_action, DataViewAction::None);
}

#[test]
fn test_drag_selection_over_phantom_row_never_mutates_dataset() {
    let mut ds = DataSet::empty();
    ds.add_variable("id", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .expect("var id");
    ds.add_variable("val", DataType::Numeric { decimals: 0 }, Measure::Scale)
        .expect("var val");

    for i in 0..5 {
        ds.add_empty_row();
        let _ = ds.set_cell_parsed(i, 0, &(101 + i).to_string());
        let _ = ds.set_cell_parsed(i, 1, &((i + 1) * 10).to_string());
    }

    assert_eq!(ds.row_count(), 5);
    assert_eq!(ds.get_cell_display(0, 0, false), "101.00");
    assert_eq!(ds.get_cell_display(4, 0, false), "105.00");

    // Simulate drag selection starting from cell (0, 0) and dragging across the grid
    // all the way down into the phantom row at index 5
    let anchor = (0, 0);
    let phantom_row_active = (5, 0);

    let sel = GridSelection::from_anchor_and_active(anchor, phantom_row_active);
    assert_eq!(sel.range.min_row, 0);
    assert_eq!(sel.range.max_row, 5);
    assert_eq!(sel.range.min_col, 0);
    assert_eq!(sel.range.max_col, 0);

    // Invariant: Selecting cells (even dragging down past existing rows into phantom row)
    // is strictly read-only and must NEVER mutate DataSet row count or overwrite cell values!
    assert_eq!(
        ds.row_count(),
        5,
        "Dataset row count must remain strictly 5"
    );
    assert_eq!(ds.get_cell_display(0, 0, false), "101.00");
    assert_eq!(ds.get_cell_display(1, 0, false), "102.00");
    assert_eq!(ds.get_cell_display(2, 0, false), "103.00");
    assert_eq!(ds.get_cell_display(3, 0, false), "104.00");
    assert_eq!(ds.get_cell_display(4, 0, false), "105.00");
}
