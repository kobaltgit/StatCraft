use statcraft_algo::{
    CaseProcessingCategory, GoodnessOfFit, LikelihoodRatioTestRow,
    MultinomialCaseProcessingSummary, MultinomialClassificationTable,
    MultinomialClassificationTableRow, MultinomialLogisticResult, MultinomialModelFittingInfo,
    MultinomialPseudoRSquared, ParameterEstimateRow,
};
use statcraft_app::views::output_view::{
    multinomial_logistic_to_tsv, render_output_view, OutputItemKind, OutputViewerState,
};

fn sample_multinomial_result() -> MultinomialLogisticResult {
    MultinomialLogisticResult {
        dependent_var: "brand".to_string(),
        factors: vec!["region".to_string()],
        covariates: vec!["price".to_string()],
        reference_category_val: 3.0,
        categories: vec![1.0, 2.0, 3.0],
        n_cases: 150,
        case_processing: MultinomialCaseProcessingSummary {
            categories: vec![
                CaseProcessingCategory {
                    category_value: 1.0,
                    category_label: Some("Brand A".to_string()),
                    n: 50,
                    percent: 33.3,
                },
                CaseProcessingCategory {
                    category_value: 2.0,
                    category_label: Some("Brand B".to_string()),
                    n: 60,
                    percent: 40.0,
                },
                CaseProcessingCategory {
                    category_value: 3.0,
                    category_label: Some("Brand C".to_string()),
                    n: 40,
                    percent: 26.7,
                },
            ],
            valid_n: 150,
            valid_percent: 100.0,
            missing_n: 0,
            total_n: 150,
        },
        model_fitting: MultinomialModelFittingInfo {
            intercept_only_minus_2ll: 325.45,
            final_minus_2ll: 280.12,
            chi_square: 45.33,
            df: 4,
            sig: 0.0001,
            aic: 290.12,
            bic: 305.18,
        },
        goodness_of_fit: Some(GoodnessOfFit {
            pearson_chi_square: 120.4,
            pearson_df: 110,
            pearson_sig: 0.235,
            deviance_chi_square: 118.2,
            deviance_df: 110,
            deviance_sig: 0.280,
        }),
        pseudo_r_squared: MultinomialPseudoRSquared {
            cox_and_snell: 0.261,
            nagelkerke: 0.295,
            mcfadden: 0.139,
        },
        likelihood_ratio_tests: vec![
            LikelihoodRatioTestRow {
                effect_name: "Intercept".to_string(),
                model_fitting_criteria_minus_2ll: 295.0,
                chi_square: 14.88,
                df: 2,
                sig: 0.0006,
            },
            LikelihoodRatioTestRow {
                effect_name: "price".to_string(),
                model_fitting_criteria_minus_2ll: 310.5,
                chi_square: 30.38,
                df: 2,
                sig: 0.0001,
            },
        ],
        parameter_estimates: vec![
            ParameterEstimateRow {
                category_value: 1.0,
                category_label: Some("Brand A".to_string()),
                param_name: "Intercept".to_string(),
                b: 1.25,
                std_err: 0.35,
                wald: 12.75,
                df: 1,
                sig: 0.0004,
                exp_b: 3.490,
                ci_lower: 1.76,
                ci_upper: 6.93,
                is_reference_category: false,
            },
            ParameterEstimateRow {
                category_value: 3.0,
                category_label: Some("Brand C".to_string()),
                param_name: "Intercept".to_string(),
                b: 0.0,
                std_err: 0.0,
                wald: 0.0,
                df: 0,
                sig: 0.0,
                exp_b: 1.0,
                ci_lower: 0.0,
                ci_upper: 0.0,
                is_reference_category: true,
            },
        ],
        classification_table: Some(MultinomialClassificationTable {
            categories: vec![1.0, 2.0, 3.0],
            category_labels: vec![
                Some("Brand A".to_string()),
                Some("Brand B".to_string()),
                Some("Brand C".to_string()),
            ],
            rows: vec![
                MultinomialClassificationTableRow {
                    observed_category: 1.0,
                    observed_label: Some("Brand A".to_string()),
                    predicted_counts: vec![35, 10, 5],
                    percent_correct: 70.0,
                },
                MultinomialClassificationTableRow {
                    observed_category: 2.0,
                    observed_label: Some("Brand B".to_string()),
                    predicted_counts: vec![8, 42, 10],
                    percent_correct: 70.0,
                },
                MultinomialClassificationTableRow {
                    observed_category: 3.0,
                    observed_label: Some("Brand C".to_string()),
                    predicted_counts: vec![5, 8, 27],
                    percent_correct: 67.5,
                },
            ],
            overall_percent: 69.3,
        }),
        iterations: 6,
        converged: true,
        warnings: Vec::new(),
    }
}

#[test]
fn test_multinomial_logistic_output_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();
    let res = sample_multinomial_result();

    state.add_multinomial_logistic(res.clone());
    assert_eq!(state.procedures.len(), 1);
    assert_eq!(state.procedures[0].items.len(), 3);
    assert!(matches!(
        state.procedures[0].items[2].kind,
        OutputItemKind::MultinomialLogistic(_)
    ));

    // Headless render
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = multinomial_logistic_to_tsv(&res);
    assert!(tsv.contains("Multinomial Logistic Regression"));
    assert!(tsv.contains("Model Fitting Information"));
    assert!(tsv.contains("Parameter Estimates"));
    assert!(tsv.contains("Brand A"));
    assert!(tsv.contains("3.490"));
}
