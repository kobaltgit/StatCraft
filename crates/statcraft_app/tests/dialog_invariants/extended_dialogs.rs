use super::fixtures::create_test_dataset;
use statcraft_app::dialogs::{
    render_confirm_save_dialog, render_count_values_dialog, render_date_time_dialog,
    render_dummy_dialog, render_factor_dialog, render_linear_regression_dialog,
    render_means_dialog, render_optimal_binning_dialog, render_random_seed_dialog,
    render_rank_cases_dialog, render_reliability_dialog, render_replace_missing_dialog,
    render_shift_values_dialog, render_time_series_dialog, render_visual_binning_dialog,
    render_weight_cases_dialog, ConfirmSaveAction, ConfirmSaveDialogState, CountValuesDialogAction,
    CountValuesDialogState, DateTimeDialogAction, DateTimeDialogState, DateTimeDialogTask,
    DialogImputeMethod, DummyDialogAction, DummyDialogState, FactorDialogAction, FactorDialogState,
    LinearRegressionDialogAction, LinearRegressionDialogState, MeansDialogAction, MeansDialogState,
    OptimalBinningDialogAction, OptimalBinningDialogState, RandomSeedDialogAction,
    RandomSeedDialogState, RankCasesDialogAction, RankCasesDialogState, ReliabilityDialogAction,
    ReliabilityDialogState, ReplaceMissingDialogAction, ReplaceMissingDialogState,
    ReplaceMissingItem, ShiftValuesDialogAction, ShiftValuesDialogState, TimeSeriesDialogAction,
    TimeSeriesDialogState, TimeSeriesFnType, TimeSeriesItem, VisualBinningDialogAction,
    VisualBinningDialogState, WeightCasesDialogAction, WeightCasesDialogState,
};

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

#[test]
fn test_headless_render_mcp_bridge_dialog() {
    let ctx = egui::Context::default();
    let mut state = statcraft_app::dialogs::McpBridgeDialogState {
        is_open: true,
        copied_banner: false,
    };
    let mut bridge = statcraft_app::bridge_service::BridgeService::new(false);

    let _ = ctx.run(Default::default(), |ctx| {
        statcraft_app::dialogs::render_mcp_bridge_dialog(ctx, &mut state, &mut bridge);
    });

    assert!(state.is_open);
}
