use egui::Context;
use statcraft_algo::{
    BaselineSurvivalPoint, CoxCaseSummary, CoxRegressionResult, CoxVariableInEquation,
    OmnibusTestsResult, TieMethod,
};
use statcraft_app::dialogs::{
    render_cox_regression_dialog, CoxRegressionDialogAction, CoxRegressionDialogState,
};
use statcraft_app::views::output_view::{cox_to_tsv, render_output_view, OutputViewerState};
use statcraft_core::{DataSet, DataType, Measure};

fn create_test_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    let col0 = ds
        .add_variable("time", DataType::Numeric { decimals: 1 }, Measure::Scale)
        .unwrap();
    let col1 = ds
        .add_variable(
            "status",
            DataType::Numeric { decimals: 0 },
            Measure::Nominal,
        )
        .unwrap();
    let col2 = ds
        .add_variable("age", DataType::Numeric { decimals: 1 }, Measure::Scale)
        .unwrap();

    if let Some(m) = ds.dictionary.get_mut(col0) {
        m.label = "Survival Time".to_string();
    }
    if let Some(m) = ds.dictionary.get_mut(col1) {
        m.label = "Terminal Status".to_string();
    }
    if let Some(m) = ds.dictionary.get_mut(col2) {
        m.label = "Age of Patient".to_string();
    }

    ds
}

fn create_sample_cox_result() -> CoxRegressionResult {
    CoxRegressionResult {
        time_variable: "time".to_string(),
        status_variable: "status".to_string(),
        strata_variable: None,
        covariate_names: vec!["age".to_string()],
        covariate_means: vec![48.75],
        case_summaries: vec![CoxCaseSummary {
            stratum: None,
            total_n: 4,
            events_n: 3,
            censored_n: 1,
            censored_percent: 25.0,
        }],
        overall_summary: CoxCaseSummary {
            stratum: None,
            total_n: 4,
            events_n: 3,
            censored_n: 1,
            censored_percent: 25.0,
        },
        omnibus_tests: OmnibusTestsResult {
            initial_minus_2ll: 6.931,
            final_minus_2ll: 4.125,
            chi_square: 2.806,
            df: 1,
            p_value: 0.0939,
        },
        variables: vec![CoxVariableInEquation {
            name: "age".to_string(),
            b: 0.0521,
            se: 0.0315,
            wald: 2.735,
            df: 1,
            p_value: 0.0982,
            exp_b: 1.0535,
            ci_lower: 0.9904,
            ci_upper: 1.1206,
        }],
        baseline_survival: vec![
            BaselineSurvivalPoint {
                stratum: None,
                time: 10.0,
                baseline_cum_hazard: 0.25,
                baseline_survival: 0.7788,
                mean_covariates_survival: 0.7512,
            },
            BaselineSurvivalPoint {
                stratum: None,
                time: 30.0,
                baseline_cum_hazard: 0.65,
                baseline_survival: 0.5220,
                mean_covariates_survival: 0.4815,
            },
        ],
        tie_method: TieMethod::Breslow,
        iterations: 4,
        converged: true,
    }
}

#[test]
fn test_cox_output_viewer_procedure_and_tsv() {
    let mut state = OutputViewerState::default();
    let res = create_sample_cox_result();

    state.add_cox_regression(res.clone());

    assert_eq!(state.procedures.len(), 1);
    let proc = &state.procedures[0];
    assert_eq!(proc.title_key, "output-cox-title");
    assert_eq!(proc.items.len(), 3); // Log, Notes, CoxRegression Table

    // Verify TSV export
    let tsv = cox_to_tsv(&res);
    assert!(tsv.contains("=== Case Processing Summary ==="));
    assert!(tsv.contains("=== Omnibus Tests of Model Coefficients ==="));
    assert!(tsv.contains("=== Variables in the Equation ==="));
    assert!(tsv.contains("age"));
    assert!(tsv.contains("1.0535"));
}

#[test]
fn test_cox_headless_ui_rendering() {
    let ctx = Context::default();
    let ds = create_test_dataset();
    let mut state = CoxRegressionDialogState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_cox_regression_dialog(ctx, Some(&ds), &mut state);
        assert!(matches!(action, CoxRegressionDialogAction::None));
    });

    let mut viewer_state = OutputViewerState::default();
    viewer_state.add_cox_regression(create_sample_cox_result());

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut viewer_state);
        });
    });
}
