use statcraft_app::dialogs::common::{DialogSize, SubdialogSidebar};
use statcraft_app::dialogs::{
    render_anova_oneway_dialog, render_auto_recode_dialog, render_compute_dialog,
    render_correlations_dialog, render_crosstabs_dialog, render_descriptives_dialog,
    render_frequencies_dialog, render_kruskal_wallis_dialog, render_missing_values_dialog,
    render_nonparam_related_dialog, render_nonparam_twosample_dialog, render_recode_dialog,
    render_select_cases_dialog, render_sort_cases_dialog, render_split_file_dialog,
    render_ttest_indep_dialog, render_ttest_onesample_dialog, render_ttest_paired_dialog,
    render_value_labels_dialog, render_variable_type_dialog, AutoRecodeDialogAction,
    AutoRecodeDialogState, AutoRecodePair, ComputeDialogAction, ComputeDialogState,
    CorrelationsDialogAction, CorrelationsDialogState, CrosstabsDialogAction, CrosstabsDialogState,
    DescriptivesDialogAction, DescriptivesDialogState, DialogCasesAction, DialogSelectMode,
    FrequenciesDialogAction, FrequenciesDialogState, IndependentSamplesTTestDialogAction,
    IndependentSamplesTTestDialogState, KruskalWallisDialogAction, KruskalWallisDialogState,
    MissingValuesDialogAction, MissingValuesDialogState, NonparamRelatedDialogAction,
    NonparamRelatedDialogState, NonparamTwoSampleDialogAction, NonparamTwoSampleDialogState,
    OneSampleTTestDialogAction, OneSampleTTestDialogState, OneWayAnovaDialogAction,
    OneWayAnovaDialogState, PairedSamplesTTestDialogAction, PairedSamplesTTestDialogState,
    RecodeDialogAction, RecodeDialogState, RecodeMode, RecodePair, SelectCasesDialogAction,
    SelectCasesDialogState, SortCasesDialogAction, SortCasesDialogState, SplitFileDialogAction,
    SplitFileDialogState, ValueLabelsDialogAction, ValueLabelsDialogState,
    VariableTypeDialogAction, VariableTypeDialogState,
};
use statcraft_core::{DataType, MissingValues, SplitFileMode};
use std::collections::BTreeMap;

#[path = "dialog_invariants/fixtures.rs"]
mod fixtures;
use fixtures::create_test_dataset;

#[path = "dialog_invariants/extended_dialogs.rs"]
mod extended_dialogs;

#[test]
fn test_dialog_canonical_sizes() {
    assert_eq!(DialogSize::Standard.to_vec2(), egui::vec2(560.0, 430.0));
    assert_eq!(DialogSize::Subdialog.to_vec2(), egui::vec2(360.0, 300.0));
    assert_eq!(DialogSize::Properties.to_vec2(), egui::vec2(460.0, 380.0));
    assert_eq!(DialogSize::Compute.to_vec2(), egui::vec2(620.0, 480.0));
    assert_eq!(DialogSize::Alert.to_vec2(), egui::vec2(440.0, 170.0));
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
    assert!(!state.apa_tables);
    assert!(!state.show_statistics);
}

#[test]
fn test_headless_render_frequencies_statistics_subdialog() {
    let ctx = egui::Context::default();
    let ds = create_test_dataset();
    let mut state = FrequenciesDialogState {
        is_open: true,
        show_statistics: true,
        ..Default::default()
    };
    state.stats.mean = true;
    state.stats.median = true;
    state.stats.custom_percentiles.push(25.0);
    state.stats.custom_percentiles.push(75.0);

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_frequencies_dialog(ctx, Some(&ds), &mut state);
        match action {
            FrequenciesDialogAction::None => {}
            _ => panic!("Expected None during subdialog configuration"),
        }
    });

    assert!(state.show_statistics);
    assert!(state.stats.mean);
    assert!(state.stats.median);
    assert_eq!(state.stats.custom_percentiles, vec![25.0, 75.0]);

    let stats_json = state.stats.to_statistics_json().expect("stats JSON");
    assert_eq!(stats_json["mean"], true);
    assert_eq!(stats_json["median"], true);

    let pct_json = state.stats.to_percentiles_json().expect("percentiles JSON");
    assert_eq!(pct_json["custom"], serde_json::json!([25.0, 75.0]));
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

#[test]
fn test_headless_render_nonparam_related_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = NonparamRelatedDialogState {
        is_open: true,
        paired_vars: vec![("pre_score".to_string(), "post_score".to_string())],
        test_wilcoxon: true,
        test_sign: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_nonparam_related_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, NonparamRelatedDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.paired_vars.len(), 1);
    assert!(state.test_wilcoxon);
    assert!(state.test_sign);
}

#[test]
fn test_headless_render_kruskal_wallis_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = KruskalWallisDialogState {
        is_open: true,
        selected_vars: vec!["salary".to_string()],
        grouping_var: Some("educ".to_string()),
        min_range: 1.0,
        max_range: 3.0,
        is_range_defined: true,
        is_define_range_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_kruskal_wallis_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, KruskalWallisDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars.len(), 1);
    assert_eq!(state.grouping_var.as_deref(), Some("educ"));
    assert_eq!(state.min_range, 1.0);
    assert_eq!(state.max_range, 3.0);
}

#[test]
fn test_headless_render_compute_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = ComputeDialogState {
        is_open: true,
        target_var: "diff".to_string(),
        expression: "post_score - pre_score".to_string(),
        label: "Score Difference".to_string(),
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_compute_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, ComputeDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.target_var, "diff");
    assert_eq!(state.expression, "post_score - pre_score");
    assert_eq!(state.label, "Score Difference");
}

#[test]
fn test_headless_render_recode_dialog_diff() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = RecodeDialogState {
        is_open: true,
        mode: RecodeMode::Different,
        pairs: vec![RecodePair {
            source: "salary".to_string(),
            target: "sal_cat".to_string(),
            label: "Salary Categories".to_string(),
        }],
        output_name: "sal_cat".to_string(),
        output_label: "Salary Categories".to_string(),
        selected_pair_idx: Some(0),
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_recode_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, RecodeDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.pairs.len(), 1);
    assert_eq!(state.pairs[0].source, "salary");
    assert_eq!(state.pairs[0].target, "sal_cat");
}

#[test]
fn test_headless_render_recode_dialog_same() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = RecodeDialogState {
        is_open: true,
        mode: RecodeMode::Same,
        selected_vars: vec!["salary".to_string(), "pre_score".to_string()],
        selected_var_idx: Some(0),
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_recode_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, RecodeDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars.len(), 2);
}

#[test]
fn test_headless_render_auto_recode_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = AutoRecodeDialogState {
        is_open: true,
        pairs: vec![AutoRecodePair {
            source: "salary".to_string(),
            target: "sal_code".to_string(),
        }],
        new_name: "sal_code".to_string(),
        selected_pair_idx: Some(0),
        order: statcraft_algo::AutoRecodeOrder::LowestValue,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_auto_recode_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, AutoRecodeDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.pairs.len(), 1);
}

#[test]
fn test_headless_render_sort_cases_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = SortCasesDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_sort_cases_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, SortCasesDialogAction::None));
    });

    assert!(state.is_open);
    assert!(state.selected_keys.is_empty());
}

#[test]
fn test_headless_render_select_cases_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = SelectCasesDialogState {
        is_open: true,
        mode: DialogSelectMode::Condition,
        action: DialogCasesAction::Filtered,
        condition: "salary > 30000".to_string(),
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_select_cases_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, SelectCasesDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.condition, "salary > 30000");
}

#[test]
fn test_headless_render_split_file_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = SplitFileDialogState {
        is_open: true,
        mode: SplitFileMode::CompareGroups,
        selected_vars: vec!["gender".to_string()],
        sort_file: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_split_file_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, SplitFileDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars, vec!["gender".to_string()]);
    assert_eq!(state.mode, SplitFileMode::CompareGroups);
}
