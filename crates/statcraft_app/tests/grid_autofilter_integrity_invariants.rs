use egui::{pos2, Context};
use statcraft_app::grid::{
    convert_text_to_number, is_number_stored_as_text, ColumnFilter, GridFilterState,
};
use statcraft_app::views::{render_data_view, DataViewAction, DataViewState};
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use std::collections::HashSet;

fn create_sample_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    let _ = ds.add_variable("ID", DataType::Numeric { decimals: 0 }, Measure::Scale);
    let _ = ds.add_variable("DEPT", DataType::String { max_len: 10 }, Measure::Nominal);
    let _ = ds.add_variable("SALARY", DataType::String { max_len: 20 }, Measure::Scale);

    ds.add_empty_row();
    let _ = ds.set_cell(0, 0, RawValue::Number(1.0));
    let _ = ds.set_cell(0, 1, RawValue::Text("IT".into()));
    let _ = ds.set_cell(0, 2, RawValue::Text(" 5000.50 ".into())); // number stored as text

    ds.add_empty_row();
    let _ = ds.set_cell(1, 0, RawValue::Number(2.0));
    let _ = ds.set_cell(1, 1, RawValue::Text("HR".into()));
    let _ = ds.set_cell(1, 2, RawValue::Text("4200".into())); // number stored as text

    ds.add_empty_row();
    let _ = ds.set_cell(2, 0, RawValue::Number(3.0));
    let _ = ds.set_cell(2, 1, RawValue::Text("IT".into()));
    let _ = ds.set_cell(2, 2, RawValue::Text("6100".into())); // number stored as text

    ds.add_empty_row();
    let _ = ds.set_cell(3, 0, RawValue::Number(4.0));
    let _ = ds.set_cell(3, 1, RawValue::Text("Sales".into()));
    let _ = ds.set_cell(3, 2, RawValue::Text("NotANumber".into())); // plain text

    ds
}

#[test]
fn test_autofilter_matching_and_inactive_row_mask() {
    let ds = create_sample_dataset();
    let mut filter_state = GridFilterState::default();

    assert!(filter_state.is_row_matched(&ds, 0));
    assert!(filter_state.is_row_matched(&ds, 1));
    assert!(filter_state.is_row_matched(&ds, 2));
    assert!(filter_state.is_row_matched(&ds, 3));

    // Filter DEPT (col 1) only to "IT"
    let mut it_filter = HashSet::new();
    it_filter.insert("IT".to_string());
    filter_state.active_filters.insert(
        1,
        ColumnFilter {
            allowed_values: it_filter,
            include_blanks: false,
        },
    );

    assert!(filter_state.is_col_filtered(1));
    assert!(!filter_state.is_col_filtered(0));
    assert!(filter_state.has_any_filter());

    // Row 0 and 2 are IT -> matched
    assert!(filter_state.is_row_matched(&ds, 0));
    assert!(filter_state.is_row_matched(&ds, 2));

    // Row 1 (HR) and 3 (Sales) -> filtered out (inactive)
    assert!(!filter_state.is_row_matched(&ds, 1));
    assert!(!filter_state.is_row_matched(&ds, 3));
}

#[test]
fn test_integrity_guard_number_stored_as_text() {
    let ds = create_sample_dataset();

    // Check cells in row 0, 1, 2 for SALARY (col 2)
    let cell_0 = ds.get_cell(0, 2).expect("cell 0,2 exists");
    assert!(is_number_stored_as_text(cell_0));
    let converted_0 = convert_text_to_number(cell_0).expect("must convert to float");
    assert_eq!(converted_0, RawValue::Number(5000.5));

    let cell_1 = ds.get_cell(1, 2).expect("cell 1,2 exists");
    assert!(is_number_stored_as_text(cell_1));
    let converted_1 = convert_text_to_number(cell_1).expect("must convert to float");
    assert_eq!(converted_1, RawValue::Number(4200.0));

    // Row 3 is "NotANumber" -> should not be flagged
    let cell_3 = ds.get_cell(3, 2).expect("cell 3,2 exists");
    assert!(!is_number_stored_as_text(cell_3));
    assert_eq!(convert_text_to_number(cell_3), None);

    // Number cell in col 0 is already numeric -> not flagged
    let cell_num = ds.get_cell(0, 0).expect("cell 0,0 exists");
    assert!(!is_number_stored_as_text(cell_num));
}

#[test]
fn test_headless_render_data_view_with_autofilter_and_context_menus() {
    let ds = create_sample_dataset();
    let mut state = DataViewState::default();

    // 1. Open filter menu for column 1
    state.filter_state.open_for_col(1, &ds, pos2(200.0, 50.0));
    assert_eq!(state.filter_state.open_col, Some(1));

    let ctx = Context::default();
    let mut dataset_opt = Some(ds.clone());

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let action = render_data_view(ui, &mut dataset_opt, &mut state);
            assert_eq!(action, DataViewAction::None);
        });
    });

    // 2. Open context menu for cell (0, 1)
    state.context_menu_state.open(0, 1, pos2(150.0, 120.0));
    assert!(state.context_menu_state.is_open());

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let action = render_data_view(ui, &mut dataset_opt, &mut state);
            assert_eq!(action, DataViewAction::None);
        });
    });

    // Close menus
    state.filter_state.close();
    state.context_menu_state.close();
    assert_eq!(state.filter_state.open_col, None);
    assert!(!state.context_menu_state.is_open());
}
