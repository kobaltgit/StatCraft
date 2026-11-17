use statcraft_algo::{SortDirection, VariableSortAttribute};
use statcraft_app::dialogs::{
    render_sort_variables_dialog, render_transpose_dialog, SortVariablesDialogAction,
    SortVariablesDialogState, TransposeDialogAction, TransposeDialogState,
};
use statcraft_core::{DataSet, DataType, Measure};

fn create_test_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    ds.add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable(
        "gender",
        DataType::Numeric { decimals: 0 },
        Measure::Nominal,
    )
    .unwrap();
    ds.add_empty_row();
    ds.add_empty_row();
    ds
}

#[test]
fn test_headless_render_sort_variables_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = SortVariablesDialogState {
        is_open: true,
        attribute: VariableSortAttribute::Name,
        direction: SortDirection::Descending,
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_sort_variables_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, SortVariablesDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.attribute, VariableSortAttribute::Name);
    assert_eq!(state.direction, SortDirection::Descending);
}

#[test]
fn test_headless_render_transpose_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = TransposeDialogState {
        is_open: true,
        selected_vars: vec!["salary".to_string()],
        name_var: Some("gender".to_string()),
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_transpose_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, TransposeDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars, vec!["salary"]);
    assert_eq!(state.name_var.as_deref(), Some("gender"));
}
