use statcraft_algo::glm::{
    MultivariateEffectRow, MultivariateGlmResult, MultivariateStatName, RepeatedGlmResult,
    SphericityTestResult, UnivariateGlmResult, WithinSubjectsEffectRow,
};
use statcraft_app::dialogs::{
    render_multivariate_glm_dialog, render_repeated_glm_dialog, render_univariate_glm_dialog,
    MultivariateGlmDialogAction, MultivariateGlmDialogState, RepeatedGlmDialogAction,
    RepeatedGlmDialogState, UnivariateGlmDialogAction, UnivariateGlmDialogState,
};
use statcraft_app::views::output_view::{
    multivariate_glm_to_tsv, render_output_view, repeated_glm_to_tsv, univariate_glm_to_tsv,
    OutputViewerState,
};
use statcraft_core::{DataSet, DataType, Measure};

fn create_sample_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    ds.add_variable("score", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("group", DataType::String { max_len: 8 }, Measure::Nominal)
        .unwrap();
    ds.add_variable("cov", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("y1", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("y2", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("t1", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("t2", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("t3", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds
}

#[test]
fn test_headless_render_univariate_glm_dialog() {
    let _ = statcraft_i18n::init_i18n();
    let ctx = egui::Context::default();
    let ds = create_sample_dataset();
    let mut state = UnivariateGlmDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_univariate_glm_dialog(ctx, Some(&ds), &mut state);
        match action {
            UnivariateGlmDialogAction::None => {}
            _ => panic!("Expected None when no variables selected"),
        }
    });

    assert!(state.is_open);
    assert!(state.dep_var.is_none());
}

#[test]
fn test_headless_render_multivariate_glm_dialog() {
    let _ = statcraft_i18n::init_i18n();
    let ctx = egui::Context::default();
    let ds = create_sample_dataset();
    let mut state = MultivariateGlmDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_multivariate_glm_dialog(ctx, Some(&ds), &mut state);
        match action {
            MultivariateGlmDialogAction::None => {}
            _ => panic!("Expected None when no variables selected"),
        }
    });

    assert!(state.is_open);
    assert!(state.dep_vars.is_empty());
}

#[test]
fn test_headless_render_repeated_glm_dialog() {
    let _ = statcraft_i18n::init_i18n();
    let ctx = egui::Context::default();
    let ds = create_sample_dataset();
    let mut state = RepeatedGlmDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_repeated_glm_dialog(ctx, Some(&ds), &mut state);
        match action {
            RepeatedGlmDialogAction::None => {}
            _ => panic!("Expected None when no measures selected"),
        }
    });

    assert!(state.is_open);
    assert_eq!(state.factor_name, "time");
}

#[test]
fn test_output_viewer_univariate_glm_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    let sample_res = UnivariateGlmResult {
        dependent: "Score".to_string(),
        n_valid: 20,
        r_squared: 0.852,
        adjusted_r_squared: 0.821,
        tests: vec![
            statcraft_algo::glm::BetweenSubjectsRow {
                source: "Corrected Model".to_string(),
                type3_ss: 150.25,
                df: 3,
                mean_square: Some(50.083),
                f_stat: Some(25.4),
                p_value: Some(0.0001),
                partial_eta_squared: Some(0.852),
            },
            statcraft_algo::glm::BetweenSubjectsRow {
                source: "Group".to_string(),
                type3_ss: 120.50,
                df: 2,
                mean_square: Some(60.25),
                f_stat: Some(30.5),
                p_value: Some(0.00005),
                partial_eta_squared: Some(0.792),
            },
            statcraft_algo::glm::BetweenSubjectsRow {
                source: "Error".to_string(),
                type3_ss: 31.55,
                df: 16,
                mean_square: Some(1.972),
                f_stat: None,
                p_value: None,
                partial_eta_squared: None,
            },
        ],
        levene_test: None,
        group_descriptives: Vec::new(),
    };

    state.add_univariate_glm(sample_res.clone());
    assert_eq!(state.procedures.len(), 1);

    // Headless UI render
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = univariate_glm_to_tsv(&sample_res);
    assert!(tsv.contains("Corrected Model"));
    assert!(tsv.contains("Group"));
    assert!(tsv.contains("Error"));
    assert!(tsv.contains("150.250"));
    assert!(tsv.contains("0.852"));
}

#[test]
fn test_output_viewer_multivariate_glm_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    let sample_res = MultivariateGlmResult {
        dependents: vec!["Y1".to_string(), "Y2".to_string()],
        n_valid: 25,
        effects: vec![
            MultivariateEffectRow {
                effect: "Group".to_string(),
                stat_name: MultivariateStatName::PillaiTrace,
                value: 0.85,
                f_stat: 12.5,
                hypothesis_df: 4.0,
                error_df: 42.0,
                p_value: 0.0001,
                partial_eta_squared: Some(0.425),
            },
            MultivariateEffectRow {
                effect: "Group".to_string(),
                stat_name: MultivariateStatName::WilksLambda,
                value: 0.18,
                f_stat: 14.2,
                hypothesis_df: 4.0,
                error_df: 40.0,
                p_value: 0.00005,
                partial_eta_squared: Some(0.575),
            },
        ],
        between_subjects: Vec::new(),
    };

    state.add_multivariate_glm(sample_res.clone());
    assert_eq!(state.procedures.len(), 1);

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = multivariate_glm_to_tsv(&sample_res);
    assert!(tsv.contains("Group"));
    assert!(tsv.contains("Pillai's Trace"));
    assert!(tsv.contains("Wilks' Lambda"));
    assert!(tsv.contains("12.500"));
}

#[test]
fn test_output_viewer_repeated_glm_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    let sample_res = RepeatedGlmResult {
        factor_name: "time".to_string(),
        measure_variables: vec!["T1".to_string(), "T2".to_string(), "T3".to_string()],
        n_valid: 15,
        sphericity: SphericityTestResult {
            within_subject_effect: "time".to_string(),
            mauchly_w: 0.725,
            approx_chi_square: 4.12,
            df: 2,
            p_value: 0.127,
            greenhouse_geisser: 0.784,
            huynh_feldt: 0.865,
            lower_bound: 0.500,
        },
        within_subjects_tests: vec![WithinSubjectsEffectRow {
            source: "time".to_string(),
            sphericity_assumed_ss: 85.5,
            sphericity_assumed_df: 2.0,
            sphericity_assumed_ms: 42.75,
            sphericity_assumed_f: Some(18.5),
            sphericity_assumed_p: Some(0.0001),
            greenhouse_geisser_ss: 85.5,
            greenhouse_geisser_df: 1.568,
            greenhouse_geisser_ms: 54.528,
            greenhouse_geisser_f: Some(18.5),
            greenhouse_geisser_p: Some(0.0002),
            huynh_feldt_ss: 85.5,
            huynh_feldt_df: 1.73,
            huynh_feldt_ms: 49.42,
            huynh_feldt_f: Some(18.5),
            huynh_feldt_p: Some(0.00015),
            lower_bound_ss: 85.5,
            lower_bound_df: 1.0,
            lower_bound_ms: 85.5,
            lower_bound_f: Some(18.5),
            lower_bound_p: Some(0.001),
        }],
        error_row: WithinSubjectsEffectRow {
            source: "Error(time)".to_string(),
            sphericity_assumed_ss: 64.7,
            sphericity_assumed_df: 28.0,
            sphericity_assumed_ms: 2.31,
            sphericity_assumed_f: None,
            sphericity_assumed_p: None,
            greenhouse_geisser_ss: 64.7,
            greenhouse_geisser_df: 21.95,
            greenhouse_geisser_ms: 2.947,
            greenhouse_geisser_f: None,
            greenhouse_geisser_p: None,
            huynh_feldt_ss: 64.7,
            huynh_feldt_df: 24.22,
            huynh_feldt_ms: 2.671,
            huynh_feldt_f: None,
            huynh_feldt_p: None,
            lower_bound_ss: 64.7,
            lower_bound_df: 14.0,
            lower_bound_ms: 4.62,
            lower_bound_f: None,
            lower_bound_p: None,
        },
    };

    state.add_repeated_glm(sample_res.clone());
    assert_eq!(state.procedures.len(), 1);

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = repeated_glm_to_tsv(&sample_res);
    assert!(tsv.contains("time"));
    assert!(tsv.contains("Sphericity Assumed"));
    assert!(tsv.contains("85.500"));
}
