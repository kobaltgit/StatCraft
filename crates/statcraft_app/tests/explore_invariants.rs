use egui::Context;
use statcraft_algo::{compute_explore, ExploreMissingHandling, ExploreOptions};
use statcraft_app::dialogs::common::DialogSize;
use statcraft_app::dialogs::{render_explore_dialog, ExploreDialogAction, ExploreDialogState};
use statcraft_app::views::output_view::{
    explore_to_tsv, render_output_view, OutputItemKind, OutputViewerState,
};
use statcraft_core::{DataSet, DataType, Measure};

fn create_test_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    let col0 = ds
        .add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    let col1 = ds
        .add_variable(
            "gender",
            DataType::Numeric { decimals: 0 },
            Measure::Nominal,
        )
        .unwrap();
    let col2 = ds
        .add_variable("educ", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
        .unwrap();

    if let Some(m) = ds.dictionary.get_mut(col0) {
        m.label = "Current Salary".to_string();
    }
    if let Some(m) = ds.dictionary.get_mut(col1) {
        m.label = "Gender".to_string();
    }
    if let Some(m) = ds.dictionary.get_mut(col2) {
        m.label = "Education Level".to_string();
    }

    let csv = "salary,gender,educ\n35000,1,12\n45000,2,14\n55000,1,16\n65000,2,14\n75000,1,16\n";
    let table =
        statcraft_io::load_csv_from_reader(csv.as_bytes(), statcraft_io::CsvOptions::default())
            .unwrap();
    DataSet::from_raw_table(table).unwrap()
}

#[test]
fn test_explore_dialog_state_defaults() {
    let state = ExploreDialogState::default();
    assert!(!state.is_open);
    assert!(state.selected_deps.is_empty());
    assert!(state.selected_factors.is_empty());
    assert!(state.compute_normality);
    assert!(state.compute_descriptives);
    assert!(state.compute_extremes);
    assert!(state.missing_listwise);
    assert_eq!(state.ci_level_str, "95");
}

#[test]
fn test_explore_dialog_renders_without_panics() {
    let ctx = Context::default();
    let ds = create_test_dataset();
    let mut state = ExploreDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_explore_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, ExploreDialogAction::None));
    });

    // Test transferring variables
    state.selected_deps.push("salary".to_string());
    state.selected_factors.push("gender".to_string());

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_explore_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, ExploreDialogAction::None));
    });

    assert_eq!(DialogSize::Standard.to_vec2(), egui::vec2(560.0, 430.0));
}

#[test]
fn test_explore_output_viewer_invariants() {
    let ds = create_test_dataset();
    let opts = ExploreOptions {
        dependent_vars: vec!["salary".to_string()],
        factor_vars: vec!["gender".to_string()],
        compute_descriptives: true,
        compute_normality: true,
        compute_extremes: true,
        confidence_level: 0.95,
        missing: ExploreMissingHandling::Listwise,
    };

    let result = compute_explore(&ds, &opts).expect("Explore calculation should succeed");

    let mut state = OutputViewerState::new();
    assert_eq!(state.procedures.len(), 0);

    state.add_explore(result.clone());
    assert_eq!(state.procedures.len(), 1);

    let proc = &state.procedures[0];
    assert_eq!(proc.title_key, "output-explore-title");
    assert_eq!(proc.items.len(), 3);
    assert!(matches!(&proc.items[0].kind, OutputItemKind::Log(_)));
    assert!(matches!(&proc.items[1].kind, OutputItemKind::Notes(_)));
    assert!(matches!(&proc.items[2].kind, OutputItemKind::Explore(_)));

    // Verify TSV export contents
    let tsv = explore_to_tsv(&result);
    assert!(tsv.contains("Case Processing Summary"));
    assert!(tsv.contains("Descriptives"));
    assert!(tsv.contains("Tests of Normality"));
    assert!(tsv.contains("Extreme Values"));
    assert!(tsv.contains("Lilliefors"));

    // Verify OutputViewer UI rendering without panics
    let ctx = Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}
