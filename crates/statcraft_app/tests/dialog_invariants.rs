use statcraft_app::dialogs::common::{DialogSize, SubdialogSidebar};
use statcraft_app::dialogs::{
    render_anova_oneway_dialog, render_correlations_dialog, render_crosstabs_dialog,
    render_descriptives_dialog, render_frequencies_dialog, render_missing_values_dialog,
    render_nonparam_twosample_dialog, render_ttest_indep_dialog, render_ttest_onesample_dialog,
    render_ttest_paired_dialog, render_value_labels_dialog, render_variable_type_dialog,
    CorrelationsDialogAction, CorrelationsDialogState, CrosstabsDialogAction, CrosstabsDialogState,
    DescriptivesDialogAction, DescriptivesDialogState, FrequenciesDialogAction,
    FrequenciesDialogState, IndependentSamplesTTestDialogAction,
    IndependentSamplesTTestDialogState, MissingValuesDialogAction, MissingValuesDialogState,
    NonparamTwoSampleDialogAction, NonparamTwoSampleDialogState, OneSampleTTestDialogAction,
    OneSampleTTestDialogState, OneWayAnovaDialogAction, OneWayAnovaDialogState,
    PairedSamplesTTestDialogAction, PairedSamplesTTestDialogState, ValueLabelsDialogAction,
    ValueLabelsDialogState, VariableTypeDialogAction, VariableTypeDialogState,
};
use statcraft_core::{DataSet, DataType, Measure, MissingValues};
use std::collections::BTreeMap;

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
    let col3 = ds
        .add_variable(
            "pre_score",
            DataType::Numeric { decimals: 1 },
            Measure::Scale,
        )
        .unwrap();
    let col4 = ds
        .add_variable(
            "post_score",
            DataType::Numeric { decimals: 1 },
            Measure::Scale,
        )
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
    if let Some(m) = ds.dictionary.get_mut(col3) {
        m.label = "Pre-test Score".to_string();
    }
    if let Some(m) = ds.dictionary.get_mut(col4) {
        m.label = "Post-test Score".to_string();
    }

    ds
}

#[test]
fn test_dialog_canonical_sizes() {
    assert_eq!(DialogSize::Standard.to_vec2(), egui::vec2(560.0, 430.0));
    assert_eq!(DialogSize::Subdialog.to_vec2(), egui::vec2(360.0, 300.0));
    assert_eq!(DialogSize::Properties.to_vec2(), egui::vec2(460.0, 380.0));
    assert_eq!(DialogSize::Compute.to_vec2(), egui::vec2(620.0, 480.0));
    assert_eq!(SubdialogSidebar::BUTTON_WIDTH, 90.0);
    assert_eq!(SubdialogSidebar::BUTTON_HEIGHT, 24.0);
}

#[test]
fn test_headless_render_descriptives_dialog() {
    let ctx = egui::Context::default();
    let ds = create_test_dataset();
    let mut state = DescriptivesDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_descriptives_dialog(ctx, Some(&ds), &mut state);
        match action {
            DescriptivesDialogAction::None => {}
            _ => panic!("Expected None when no variables selected"),
        }
    });

    assert!(state.is_open);
    assert!(state.selected_vars.is_empty());
}

#[test]
fn test_headless_render_frequencies_dialog() {
    let ctx = egui::Context::default();
    let ds = create_test_dataset();
    let mut state = FrequenciesDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_frequencies_dialog(ctx, Some(&ds), &mut state);
        match action {
            FrequenciesDialogAction::None => {}
            _ => panic!("Expected None when no variables selected"),
        }
    });

    assert!(state.is_open);
    assert!(state.display_tables);
}

#[test]
fn test_headless_render_anova_dialog() {
    let ctx = egui::Context::default();
    let ds = create_test_dataset();
    let mut state = OneWayAnovaDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_anova_oneway_dialog(ctx, Some(&ds), &mut state);
        match action {
            OneWayAnovaDialogAction::None => {}
            _ => panic!("Expected None when parameters incomplete"),
        }
    });

    assert!(state.is_open);
    assert!(state.factor_var.is_none());
}

#[test]
fn test_headless_render_ttest_onesample_dialog() {
    let ctx = egui::Context::default();
    let ds = create_test_dataset();
    let mut state = OneSampleTTestDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_ttest_onesample_dialog(ctx, Some(&ds), &mut state);
        match action {
            OneSampleTTestDialogAction::None => {}
            _ => panic!("Expected None when no variables selected"),
        }
    });

    assert!(state.is_open);
    assert_eq!(state.test_value_str, "0");
}

#[test]
fn test_headless_render_ttest_indep_dialog() {
    let ctx = egui::Context::default();
    let ds = create_test_dataset();
    let mut state = IndependentSamplesTTestDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_ttest_indep_dialog(ctx, Some(&ds), &mut state);
        match action {
            IndependentSamplesTTestDialogAction::None => {}
            _ => panic!("Expected None when parameters incomplete"),
        }
    });

    assert!(state.is_open);
    assert!(state.grouping_var.is_none());
}

#[test]
fn test_headless_render_ttest_paired_dialog() {
    let ctx = egui::Context::default();
    let ds = create_test_dataset();
    let mut state = PairedSamplesTTestDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_ttest_paired_dialog(ctx, Some(&ds), &mut state);
        match action {
            PairedSamplesTTestDialogAction::None => {}
            _ => panic!("Expected None when no pairs selected"),
        }
    });

    assert!(state.is_open);
    assert!(state.paired_vars.is_empty());
}

#[test]
fn test_headless_render_properties_dialogs() {
    let ctx = egui::Context::default();

    // 1. Value Labels
    let mut vl_state = ValueLabelsDialogState::default();
    let mut mapping = BTreeMap::new();
    mapping.insert("1".to_string(), "Male".to_string());
    mapping.insert("2".to_string(), "Female".to_string());
    vl_state.open_for_variable(0, "gender", &mapping);

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_value_labels_dialog(ctx, &mut vl_state);
        assert!(matches!(action, ValueLabelsDialogAction::None));
    });

    // 2. Missing Values
    let mut mv_state = MissingValuesDialogState::default();
    let missing = MissingValues {
        discrete: vec!["99".to_string()],
        range: None,
    };
    mv_state.open_for_variable(0, "salary", &missing);

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_missing_values_dialog(ctx, &mut mv_state);
        assert!(matches!(action, MissingValuesDialogAction::None));
    });

    // 3. Variable Type
    let mut vt_state =
        VariableTypeDialogState::open(0, "salary", &DataType::Numeric { decimals: 2 }, 8, 2);

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_variable_type_dialog(ctx, &mut vt_state);
        assert!(matches!(action, VariableTypeDialogAction::None));
    });
}

#[test]
fn test_descriptives_reset_defaults() {
    let mut state = DescriptivesDialogState {
        is_open: true,
        selected_vars: vec!["salary".to_string()],
        source_selected: Some(2),
        target_selected: Some(0),
        ..Default::default()
    };
    state.options.mean = false;

    // Resetting state
    state.selected_vars.clear();
    state.source_selected = None;
    state.target_selected = None;
    state.options = statcraft_algo::DescriptivesOptions::spps_default();

    assert!(state.selected_vars.is_empty());
    assert!(state.source_selected.is_none());
    assert!(state.target_selected.is_none());
    assert!(state.options.mean);
    assert!(state.options.std_dev);
    assert!(state.options.min);
    assert!(state.options.max);
}

#[test]
fn test_headless_render_crosstabs_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = CrosstabsDialogState {
        is_open: true,
        row_vars: vec!["gender".to_string()],
        col_vars: vec!["educ".to_string()],
        show_cells: true,
        show_stats: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_crosstabs_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, CrosstabsDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.row_vars.len(), 1);
    assert_eq!(state.col_vars.len(), 1);
}

#[test]
fn test_headless_render_correlations_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = CorrelationsDialogState {
        is_open: true,
        selected_vars: vec!["salary".to_string(), "pre_score".to_string()],
        show_options: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_correlations_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, CorrelationsDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars.len(), 2);
}

#[test]
fn test_headless_render_nonparam_twosample_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = NonparamTwoSampleDialogState {
        is_open: true,
        selected_vars: vec!["salary".to_string()],
        grouping_var: Some("gender".to_string()),
        group1_val: "1".to_string(),
        group2_val: "2".to_string(),
        is_define_groups_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_nonparam_twosample_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, NonparamTwoSampleDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars.len(), 1);
    assert_eq!(state.grouping_var.as_deref(), Some("gender"));
}
