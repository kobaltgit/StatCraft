use statcraft_app::dialogs::{
    render_curvefit_dialog, render_logistic_regression_dialog, render_multinomial_logistic_dialog,
    render_partial_corr_dialog, CurveFitDialogAction, CurveFitDialogState,
    LogisticRegressionDialogAction, LogisticRegressionDialogState, MultinomialLogisticDialogAction,
    MultinomialLogisticDialogState, PartialCorrDialogAction, PartialCorrDialogState,
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

#[test]
fn test_headless_render_multinomial_logistic_dialog() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = MultinomialLogisticDialogState {
        is_open: true,
        dep_var: Some("educ".to_string()),
        factors: vec!["gender".to_string()],
        covariates: vec!["salary".to_string(), "pre_score".to_string()],
        print_classification: true,
        print_goodness_of_fit: true,
        print_lrt: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_multinomial_logistic_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, MultinomialLogisticDialogAction::None));
    });

    assert!(state.is_open);
    assert_eq!(state.dep_var, Some("educ".to_string()));
    assert_eq!(state.factors.len(), 1);
    assert_eq!(state.covariates.len(), 2);
    assert!(state.print_classification);
    assert!(state.print_goodness_of_fit);
    assert!(state.print_lrt);
}

#[test]
fn test_logistic_regression_validation_with_cases() {
    let mut ds = DataSet::empty();
    ds.add_variable("age", DataType::Numeric { decimals: 0 }, Measure::Scale)
        .unwrap();
    ds.add_variable(
        "gender",
        DataType::Numeric { decimals: 0 },
        Measure::Nominal,
    )
    .unwrap();
    ds.add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();

    for i in 0..20 {
        ds.insert_row(
            i,
            &[
                statcraft_core::RawValue::Number(20.0 + i as f64),
                statcraft_core::RawValue::Number(if i % 2 == 0 { 1.0 } else { 2.0 }),
                statcraft_core::RawValue::Number(30000.0 + (i * 1000) as f64),
            ],
        );
    }

    let ctx = egui::Context::default();

    // 1. When age (20 unique values) is selected:
    let mut state_age = LogisticRegressionDialogState {
        is_open: true,
        dep_var: Some("age".to_string()),
        covariates: vec!["salary".to_string()],
        ..Default::default()
    };
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_logistic_regression_dialog(ctx, Some(&ds), &mut state_age);
        assert!(matches!(action, LogisticRegressionDialogAction::None));
    });

    // 2. When gender (2 unique values) is selected:
    let mut state_gender = LogisticRegressionDialogState {
        is_open: true,
        dep_var: Some("gender".to_string()),
        covariates: vec!["salary".to_string()],
        ..Default::default()
    };
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_logistic_regression_dialog(ctx, Some(&ds), &mut state_gender);
        assert!(matches!(action, LogisticRegressionDialogAction::None));
    });
}
