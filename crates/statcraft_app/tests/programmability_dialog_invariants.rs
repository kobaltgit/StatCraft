use statcraft_algo::ProgrammabilityResult;
use statcraft_app::dialogs::{
    render_programmability_dialog, ProgrammabilityDialogAction, ProgrammabilityDialogState,
};
use statcraft_app::views::output_view::{
    programmability_to_tsv, render_output_view, OutputViewerState,
};
use statcraft_core::{DataSet, DataType, Measure, RawValue};

fn create_test_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    ds.add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("educ", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
        .unwrap();

    for _ in 0..4 {
        ds.add_empty_row();
    }
    ds.set_cell(0, 0, RawValue::Number(50000.0)).unwrap();
    ds.set_cell(0, 1, RawValue::Number(16.0)).unwrap();

    ds
}

#[test]
fn test_headless_render_programmability_dialog() {
    let ctx = egui::Context::default();
    let ds = create_test_dataset();
    let mut state = ProgrammabilityDialogState::default();

    // 1. Initially closed
    let action = render_programmability_dialog(&ctx, Some(&ds), &mut state);
    assert!(matches!(action, ProgrammabilityDialogAction::None));

    // 2. Open dialog
    state.is_open = true;
    assert!(state.is_open);

    let _ = ctx.run(Default::default(), |ctx| {
        let _ = render_programmability_dialog(ctx, Some(&ds), &mut state);
    });

    // 3. Test script modification
    state.script = "bonus = salary * 0.1\nIF educ >= 16 THEN high_educ = 1 ELSE 0\n".to_string();

    let _ = ctx.run(Default::default(), |ctx| {
        let _ = render_programmability_dialog(ctx, Some(&ds), &mut state);
    });

    assert!(state.is_open);
}

#[test]
fn test_programmability_output_viewer_and_tsv() {
    let mut output_state = OutputViewerState::new();

    let sample_res = ProgrammabilityResult {
        created_vars: vec!["bonus".to_string(), "high_educ".to_string()],
        modified_vars: vec!["salary".to_string()],
        rows_affected: 4,
        statements_count: 2,
        logs: vec![
            "Executed: bonus = salary * 0.1 (rows affected: 4)".to_string(),
            "Executed: IF educ >= 16 THEN high_educ = 1 ELSE 0 (rows affected: 4)".to_string(),
        ],
    };

    // 1. Add to OutputViewer
    output_state.add_programmability(
        sample_res.clone(),
        Some("BEGIN PROGRAM.\n# Test\nEND PROGRAM.\nEXECUTE.".to_string()),
    );
    assert_eq!(output_state.procedures.len(), 1);

    // 2. Render in headless egui context
    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut output_state);
        });
    });

    // 3. TSV export integrity
    let tsv = programmability_to_tsv(&sample_res);
    assert!(tsv.contains(&statcraft_i18n::t!("output-programmability-property")));
    assert!(tsv.contains(&statcraft_i18n::t!("output-programmability-value")));
    assert!(tsv.contains(&statcraft_i18n::t!(
        "output-programmability-prop-statements"
    )));
    assert!(tsv.contains("bonus, high_educ"));
    assert!(tsv.contains("salary"));
    assert!(tsv.contains("\t4"));
}
