use egui::Context;
use statcraft_app::dialogs::{
    render_add_cases_dialog, render_add_vars_dialog, AddCasesDialogState, AddVarsDialogState,
    MergeCasesAction, MergeVarsAction,
};
use statcraft_core::workspace::WorkspaceManager;
use statcraft_core::{DataSet, DataType, Measure};

fn create_test_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    let col0 = ds
        .add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .expect("salary");
    let col1 = ds
        .add_variable(
            "gender",
            DataType::Numeric { decimals: 0 },
            Measure::Nominal,
        )
        .expect("gender");
    let col2 = ds
        .add_variable("educ", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
        .expect("educ");

    ds.table.row_count = 3;
    ds.table.columns[col0].values = vec![
        statcraft_core::RawValue::Number(25000.0),
        statcraft_core::RawValue::Number(32000.0),
        statcraft_core::RawValue::Number(45000.0),
    ];
    ds.table.columns[col1].values = vec![
        statcraft_core::RawValue::Number(1.0),
        statcraft_core::RawValue::Number(0.0),
        statcraft_core::RawValue::Number(1.0),
    ];
    ds.table.columns[col2].values = vec![
        statcraft_core::RawValue::Number(12.0),
        statcraft_core::RawValue::Number(16.0),
        statcraft_core::RawValue::Number(18.0),
    ];
    ds
}

#[test]
fn test_headless_render_add_cases_dialog() {
    let ctx = Context::default();
    let ds = create_test_dataset();
    let ws = WorkspaceManager::new_blank();
    let mut state = AddCasesDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        let action = render_add_cases_dialog(ctx, Some(&ds), Some(&ws), &mut state);
        assert!(matches!(action, MergeCasesAction::None));
    });

    assert!(state.is_open);
}

#[test]
fn test_headless_render_add_vars_dialog() {
    let ctx = Context::default();
    let ds = create_test_dataset();
    let ws = WorkspaceManager::new_blank();
    let mut state = AddVarsDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        let action = render_add_vars_dialog(ctx, Some(&ds), Some(&ws), &mut state);
        assert!(matches!(action, MergeVarsAction::None));
    });

    assert!(state.is_open);
}
