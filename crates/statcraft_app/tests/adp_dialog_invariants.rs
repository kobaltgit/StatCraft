use statcraft_algo::{
    AdpFieldAction, AdpObjective, AdpPredictivePower, AdpResult, PartitionCounts,
};
use statcraft_app::dialogs::{render_adp_dialog, AdpDialogAction, AdpDialogState};
use statcraft_app::views::output_view::{adp_to_tsv, render_output_view, OutputViewerState};
use statcraft_core::{DataSet, DataType, Measure, RawValue};

fn create_test_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    ds.add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("educ", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
        .unwrap();
    ds.add_variable("gender", DataType::String { max_len: 10 }, Measure::Nominal)
        .unwrap();

    for _ in 0..5 {
        ds.add_empty_row();
    }
    ds.set_cell(0, 0, RawValue::Number(50000.0)).unwrap();
    ds.set_cell(0, 1, RawValue::Number(16.0)).unwrap();
    ds.set_cell(0, 2, RawValue::Text("F".to_string())).unwrap();

    ds
}

#[test]
fn test_headless_render_adp_dialog() {
    let ctx = egui::Context::default();
    let ds = create_test_dataset();
    let mut state = AdpDialogState::default();

    // 1. Initially closed
    let action = render_adp_dialog(&ctx, &mut state, Some(&ds));
    assert!(matches!(action, AdpDialogAction::None));

    // 2. Open dialog
    state.open();
    assert!(state.is_open);

    let _ = ctx.run(Default::default(), |ctx| {
        let _ = render_adp_dialog(ctx, &mut state, Some(&ds));
    });

    // 3. Open subdialogs
    state.show_screening = true;
    state.show_transform = true;
    state.show_partition = true;

    let _ = ctx.run(Default::default(), |ctx| {
        let _ = render_adp_dialog(ctx, &mut state, Some(&ds));
    });

    // 4. Test build options
    state.target_var = Some("salary".to_string());
    state.input_vars = vec!["educ".to_string(), "gender".to_string()];
    let opts = state.build_options();
    assert_eq!(opts.target_variable.as_deref(), Some("salary"));
    assert_eq!(opts.input_variables, vec!["educ", "gender"]);
    assert_eq!(opts.objective, AdpObjective::BalanceSpeedAccuracy);
}

#[test]
fn test_adp_output_viewer_and_tsv() {
    let mut output_state = OutputViewerState::new();

    let sample_adp_res = AdpResult {
        target_variable: Some("salary".to_string()),
        objective: AdpObjective::OptimizeAccuracy,
        total_inputs: 3,
        screened_out_count: 1,
        recommended_count: 2,
        field_actions: vec![
            AdpFieldAction {
                variable_name: "educ".to_string(),
                is_screened_out: false,
                screening_reason: None,
                action_type: "Transformed".to_string(),
                new_field_name: Some("P_educ".to_string()),
                details: "Z-score standardized".to_string(),
            },
            AdpFieldAction {
                variable_name: "id".to_string(),
                is_screened_out: true,
                screening_reason: Some("High-cardinality identifier".to_string()),
                action_type: "Excluded".to_string(),
                new_field_name: None,
                details: "Screened out by identifier filter".to_string(),
            },
        ],
        predictive_powers: vec![AdpPredictivePower {
            variable_name: "educ".to_string(),
            method: "Pearson r".to_string(),
            score: 0.825,
            p_value: Some(0.0001),
        }],
        partition_counts: Some(PartitionCounts {
            train_count: 70,
            test_count: 20,
            validate_count: 10,
            total_count: 100,
        }),
        transformed_columns: Vec::new(),
    };

    // 1. Add to OutputViewer
    output_state.add_adp(
        sample_adp_res.clone(),
        Some("ADP /FIELDS TARGET=salary.".to_string()),
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
    let tsv = adp_to_tsv(&sample_adp_res);
    assert!(tsv.contains("Property\tValue"));
    assert!(tsv.contains("Target Variable\tsalary"));
    assert!(tsv.contains("Total Inputs\t3"));
    assert!(tsv.contains("Field\tStatus\tTransformed Field\tDetails"));
    assert!(tsv.contains("educ\tRecommended\tP_educ\tZ-score standardized"));
    assert!(tsv.contains("id\tExcluded\t-\tScreened out by identifier filter"));
    assert!(tsv.contains("Predictor\tMethod\tImportance\tSig."));
    assert!(tsv.contains("educ\tPearson r\t0.8250\t< .001"));
}
