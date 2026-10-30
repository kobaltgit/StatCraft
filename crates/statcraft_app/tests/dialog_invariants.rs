use statcraft_app::dialogs::common::{DialogSize, SubdialogSidebar};
use statcraft_app::dialogs::{
    render_anova_oneway_dialog, render_auto_recode_dialog, render_compute_dialog,
    render_confirm_save_dialog, render_correlations_dialog, render_count_values_dialog,
    render_crosstabs_dialog, render_date_time_dialog, render_descriptives_dialog,
    render_dummy_dialog, render_factor_dialog, render_frequencies_dialog,
    render_kruskal_wallis_dialog, render_linear_regression_dialog, render_means_dialog,
    render_missing_values_dialog, render_nonparam_related_dialog, render_nonparam_twosample_dialog,
    render_optimal_binning_dialog, render_random_seed_dialog, render_rank_cases_dialog,
    render_recode_dialog, render_reliability_dialog, render_replace_missing_dialog,
    render_select_cases_dialog, render_shift_values_dialog, render_sort_cases_dialog,
    render_split_file_dialog, render_time_series_dialog, render_ttest_indep_dialog,
    render_ttest_onesample_dialog, render_ttest_paired_dialog, render_value_labels_dialog,
    render_variable_type_dialog, render_visual_binning_dialog, render_weight_cases_dialog,
    AutoRecodeDialogAction, AutoRecodeDialogState, AutoRecodePair, ComputeDialogAction,
    ComputeDialogState, ConfirmSaveAction, ConfirmSaveDialogState, CorrelationsDialogAction,
    CorrelationsDialogState, CountValuesDialogAction, CountValuesDialogState,
    CrosstabsDialogAction, CrosstabsDialogState, DateTimeDialogAction, DateTimeDialogState,
    DateTimeDialogTask, DescriptivesDialogAction, DescriptivesDialogState, DialogCasesAction,
    DialogImputeMethod, DialogSelectMode, DummyDialogAction, DummyDialogState, FactorDialogAction,
    FactorDialogState, FrequenciesDialogAction, FrequenciesDialogState,
    IndependentSamplesTTestDialogAction, IndependentSamplesTTestDialogState,
    KruskalWallisDialogAction, KruskalWallisDialogState, LinearRegressionDialogAction,
    LinearRegressionDialogState, MeansDialogAction, MeansDialogState, MissingValuesDialogAction,
    MissingValuesDialogState, NonparamRelatedDialogAction, NonparamRelatedDialogState,
    NonparamTwoSampleDialogAction, NonparamTwoSampleDialogState, OneSampleTTestDialogAction,
    OneSampleTTestDialogState, OneWayAnovaDialogAction, OneWayAnovaDialogState,
    OptimalBinningDialogAction, OptimalBinningDialogState, PairedSamplesTTestDialogAction,
    PairedSamplesTTestDialogState, RandomSeedDialogAction, RandomSeedDialogState,
    RankCasesDialogAction, RankCasesDialogState, RecodeDialogAction, RecodeDialogState, RecodeMode,
    RecodePair, ReliabilityDialogAction, ReliabilityDialogState, ReplaceMissingDialogAction,
    ReplaceMissingDialogState, ReplaceMissingItem, SelectCasesDialogAction, SelectCasesDialogState,
    ShiftValuesDialogAction, ShiftValuesDialogState, SortCasesDialogAction, SortCasesDialogState,
    SplitFileDialogAction, SplitFileDialogState, TimeSeriesDialogAction, TimeSeriesDialogState,
    TimeSeriesFnType, TimeSeriesItem, ValueLabelsDialogAction, ValueLabelsDialogState,
    VariableTypeDialogAction, VariableTypeDialogState, VisualBinningDialogAction,
    VisualBinningDialogState, WeightCasesDialogAction, WeightCasesDialogState,
};
use statcraft_core::{DataSet, DataType, Measure, MissingValues, SplitFileMode};
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

#[test]
fn test_headless_render_weight_cases_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = WeightCasesDialogState {
        is_open: true,
        weight_active: true,
        selected_var: Some("salary".to_string()),
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_weight_cases_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, WeightCasesDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_var, Some("salary".to_string()));
    assert!(state.weight_active);
}

#[test]
fn test_headless_render_linear_regression_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = LinearRegressionDialogState {
        is_open: true,
        dep_var: Some("salary".to_string()),
        indep_vars: vec!["educ".to_string()],
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_linear_regression_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, LinearRegressionDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.dep_var, Some("salary".to_string()));
    assert_eq!(state.indep_vars, vec!["educ".to_string()]);
}

#[test]
fn test_headless_render_reliability_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = ReliabilityDialogState {
        is_open: true,
        selected_vars: vec![
            "salary".to_string(),
            "pre_score".to_string(),
            "post_score".to_string(),
        ],
        scale_name: "TEST_SCALE".to_string(),
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_reliability_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, ReliabilityDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars.len(), 3);
    assert_eq!(state.scale_name, "TEST_SCALE");
}

#[test]
fn test_headless_render_factor_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = FactorDialogState {
        is_open: true,
        selected_vars: vec![
            "salary".to_string(),
            "pre_score".to_string(),
            "post_score".to_string(),
        ],
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_factor_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, FactorDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars.len(), 3);
    assert!(state.kmo_bartlett);
}

#[test]
fn test_headless_render_means_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = MeansDialogState {
        is_open: true,
        selected_deps: vec!["salary".to_string()],
        selected_factors: vec!["gender".to_string()],
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_means_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, MeansDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_deps.len(), 1);
    assert_eq!(state.selected_factors.len(), 1);
}

#[test]
fn test_headless_render_confirm_save_dialog() {
    let ctx = egui::Context::default();
    let mut state = ConfirmSaveDialogState { is_open: false };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_confirm_save_dialog(ctx, &mut state);
        assert_eq!(action, ConfirmSaveAction::None);
    });
    assert!(!state.is_open);

    state.is_open = true;
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_confirm_save_dialog(ctx, &mut state);
        assert_eq!(action, ConfirmSaveAction::None);
    });
    assert!(state.is_open);
}

#[test]
fn test_headless_render_shift_values_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = ShiftValuesDialogState {
        is_open: true,
        source_var: Some("salary".to_string()),
        target_var: "salary_lag1".to_string(),
        direction: statcraft_algo::ShiftDirection::Lag,
        step: 1,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_shift_values_dialog(ctx, Some(&ds), &mut state);
        assert_eq!(action, ShiftValuesDialogAction::None);
    });

    assert!(state.is_open);
    assert_eq!(state.source_var.as_deref(), Some("salary"));
    assert_eq!(state.target_var, "salary_lag1");
}

#[test]
fn test_headless_render_dummy_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = DummyDialogState {
        is_open: true,
        selected_vars: vec!["gender".to_string()],
        root_prefix: "d_".to_string(),
        omit_reference: false,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_dummy_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, DummyDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars, vec!["gender".to_string()]);
    assert_eq!(state.root_prefix, "d_");
}

#[test]
fn test_headless_render_random_seed_dialog() {
    let ctx = egui::Context::default();
    let mut state = RandomSeedDialogState {
        is_open: true,
        is_random: false,
        seed_text: "2000000".to_string(),
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_random_seed_dialog(ctx, &mut state);
        assert_eq!(action, RandomSeedDialogAction::None);
    });

    assert!(state.is_open);
    assert!(!state.is_random);
    assert_eq!(state.seed_text, "2000000");

    // Test with random mode
    state.is_random = true;
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_random_seed_dialog(ctx, &mut state);
        assert_eq!(action, RandomSeedDialogAction::None);
    });
    assert!(state.is_random);
}

#[test]
fn test_headless_render_count_values_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = CountValuesDialogState {
        is_open: true,
        target_var: "high_scores".to_string(),
        target_label: "Count of High Scores".to_string(),
        variables: vec!["salary".to_string(), "educ".to_string()],
        criteria: vec![statcraft_algo::ValueCriterion::ExactNumber(1.0)],
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_count_values_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, CountValuesDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.target_var, "high_scores");
    assert_eq!(state.variables.len(), 2);
    assert_eq!(state.criteria.len(), 1);
}

#[test]
fn test_headless_render_time_series_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = TimeSeriesDialogState {
        is_open: true,
        items: vec![
            TimeSeriesItem {
                source_var: "salary".to_string(),
                target_var: "diff_salary".to_string(),
                fn_type: TimeSeriesFnType::Diff,
                order: 1,
                target_label: "Diff 1".to_string(),
            },
            TimeSeriesItem {
                source_var: "salary".to_string(),
                target_var: "csum_salary".to_string(),
                fn_type: TimeSeriesFnType::CumulativeSum,
                order: 1,
                target_label: String::new(),
            },
        ],
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_time_series_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, TimeSeriesDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.items.len(), 2);
    assert_eq!(state.items[0].target_var, "diff_salary");
}

#[test]
fn test_headless_render_rank_cases_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = RankCasesDialogState {
        is_open: true,
        selected_vars: vec!["salary".to_string()],
        direction: statcraft_algo::RankDirection::Ascending,
        ties: statcraft_algo::TiesHandling::Mean,
        rank_type: statcraft_algo::RankType::SimpleRank,
        ntiles_groups: 4,
        prefix: "R_".to_string(),
        show_types: false,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_rank_cases_dialog(ctx, Some(&ds), &mut state);
        assert_eq!(action, RankCasesDialogAction::None);
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars, vec!["salary".to_string()]);
    assert_eq!(state.prefix, "R_");

    state.show_types = true;
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_rank_cases_dialog(ctx, Some(&ds), &mut state);
        assert_eq!(action, RankCasesDialogAction::None);
    });
    assert!(state.show_types);
}

#[test]
fn test_headless_render_replace_missing_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = ReplaceMissingDialogState {
        is_open: true,
        items: vec![
            ReplaceMissingItem {
                source: "salary".to_string(),
                target: "new_salary".to_string(),
                method: DialogImputeMethod::SeriesMean,
                span: 2,
            },
            ReplaceMissingItem {
                source: "salary".to_string(),
                target: "linear_salary".to_string(),
                method: DialogImputeMethod::LinearInterpolation,
                span: 2,
            },
        ],
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_replace_missing_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, ReplaceMissingDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.items.len(), 2);
    assert_eq!(state.items[0].target, "new_salary");
    assert_eq!(
        state.items[0].display_label(),
        "new_salary -> SMEAN(salary)"
    );
}

#[test]
fn test_headless_render_visual_binning_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = VisualBinningDialogState {
        is_open: true,
        selected_var: Some("salary".to_string()),
        selected_var_idx: Some(0),
        target_var: "salary_binned".to_string(),
        target_label: "Binned Salary".to_string(),
        inclusive: true,
        cutpoints: vec![25000.0, 50000.0],
        custom_labels: vec![
            "<= 25000".to_string(),
            "25000 - 50000".to_string(),
            "> 50000".to_string(),
        ],
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_visual_binning_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, VisualBinningDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.cutpoints.len(), 2);
    assert_eq!(state.target_var, "salary_binned");
}

#[test]
fn test_headless_render_date_time_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = DateTimeDialogState {
        is_open: true,
        task: DateTimeDialogTask::Extract,
        primary_var: Some("salary".to_string()),
        target_var: "extracted_year".to_string(),
        target_label: "Extracted Year".to_string(),
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_date_time_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, DateTimeDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.primary_var.as_deref(), Some("salary"));
    assert_eq!(state.target_var, "extracted_year");
}

#[test]
fn test_headless_render_optimal_binning_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = OptimalBinningDialogState {
        is_open: true,
        selected_vars: vec!["salary".to_string()],
        guide_var: Some("gender".to_string()),
        max_bins: 5,
        prefix: "OB_".to_string(),
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_optimal_binning_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, OptimalBinningDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars, vec!["salary".to_string()]);
    assert_eq!(state.guide_var.as_deref(), Some("gender"));
}
