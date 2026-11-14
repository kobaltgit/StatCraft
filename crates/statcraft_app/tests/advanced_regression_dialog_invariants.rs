use statcraft_app::dialogs::{
    render_curvefit_dialog, render_logistic_regression_dialog, render_partial_corr_dialog,
    CurveFitDialogAction, CurveFitDialogState, LogisticRegressionDialogAction,
    LogisticRegressionDialogState, PartialCorrDialogAction, PartialCorrDialogState,
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
    let col3 = ds
        .add_variable(
            "pre_score",
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

    ds
}

#[test]
fn test_headless_render_partial_corr_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = PartialCorrDialogState {
        is_open: true,
        selected_vars: vec!["salary".to_string(), "pre_score".to_string()],
        selected_controls: vec!["educ".to_string()],
        two_tailed: true,
        show_zero_order: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_partial_corr_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, PartialCorrDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.selected_vars.len(), 2);
    assert_eq!(state.selected_controls.len(), 1);
    assert!(state.two_tailed);
    assert!(state.show_zero_order);
}

#[test]
fn test_headless_render_logistic_regression_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = LogisticRegressionDialogState {
        is_open: true,
        dep_var: Some("gender".to_string()),
        covariates: vec!["salary".to_string(), "educ".to_string()],
        cutoff_str: "0.5".to_string(),
        ci_str: "95".to_string(),
        hosmer_lemeshow: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_logistic_regression_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, LogisticRegressionDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.dep_var, Some("gender".to_string()));
    assert_eq!(state.covariates.len(), 2);
    assert!(state.hosmer_lemeshow);
}

#[test]
fn test_headless_render_curvefit_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = CurveFitDialogState {
        is_open: true,
        dep_var: Some("salary".to_string()),
        indep_var: Some("educ".to_string()),
        linear: true,
        quadratic: true,
        exponential: true,
        include_constant: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_curvefit_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, CurveFitDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.dep_var, Some("salary".to_string()));
    assert_eq!(state.indep_var, Some("educ".to_string()));
    let models = state.selected_models();
    assert_eq!(models.len(), 3);
}
