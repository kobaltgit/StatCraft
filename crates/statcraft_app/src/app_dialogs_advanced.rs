use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_curvefit_dialog, render_logistic_regression_dialog, render_multinomial_logistic_dialog,
    render_partial_corr_dialog, CurveFitDialogAction, LogisticRegressionDialogAction,
    MultinomialLogisticDialogAction, PartialCorrDialogAction,
};
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_advanced_regression_dialogs(&mut self, ctx: &egui::Context) {
        // 1. Partial Correlations Dialog
        let partial_action = render_partial_corr_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.partial_corr_dialog_state,
        );

        match partial_action {
            PartialCorrDialogAction::Run {
                vars,
                controls,
                two_tailed,
                show_zero_order,
            } => {
                let params = json!({
                    "vars": vars,
                    "controls": controls,
                    "two_tailed": two_tailed,
                    "missing": "listwise",
                    "zero_order": show_zero_order,
                });
                match self.command_bus.execute(
                    "stat.correlations.partial",
                    &mut self.dataset,
                    params,
                ) {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::PartialCorrResult,
                        >(result_value)
                        {
                            let vars_str = result.variables.join(", ");
                            let controls_str = result.controls.join(", ");
                            self.output_view_state
                                .add_partial_correlations(result, two_tailed, "listwise");
                            self.status_message = StatusMessage::PartialCorrExecuted {
                                vars: vars_str,
                                controls: controls_str,
                            };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            PartialCorrDialogAction::Paste {
                vars,
                controls,
                two_tailed,
                ..
            } => {
                let sps = statcraft_syntax::commands::PartialCorrCommand::to_sps(
                    &vars, &controls, two_tailed, "listwise",
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            PartialCorrDialogAction::None => {}
        }

        // 2. Binary Logistic Regression Dialog
        let logistic_action = render_logistic_regression_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.logistic_regression_dialog_state,
        );

        match logistic_action {
            LogisticRegressionDialogAction::Run {
                dep_var,
                covariates,
                cutoff,
                ci,
                hosmer_lemeshow,
            } => {
                let params = json!({
                    "dependent_var": dep_var,
                    "covariates": covariates,
                    "cutoff": cutoff,
                    "ci": ci,
                    "hosmer_lemeshow": hosmer_lemeshow,
                    "include_constant": true,
                });
                match self.command_bus.execute(
                    "stat.regression.logistic",
                    &mut self.dataset,
                    params,
                ) {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::LogisticRegressionResult,
                        >(result_value)
                        {
                            let dep = result.dependent_var.clone();
                            let count = result.covariates.len();
                            self.output_view_state.add_logistic_regression(result);
                            self.status_message = StatusMessage::LogisticRegressionExecuted {
                                dep_var: dep,
                                covariates_count: count,
                            };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            LogisticRegressionDialogAction::Paste {
                dep_var,
                covariates,
                cutoff,
                ci,
                hosmer_lemeshow,
            } => {
                let sps = statcraft_syntax::commands::LogisticRegressionCommand::to_sps(
                    &dep_var,
                    &covariates,
                    cutoff,
                    ci,
                    hosmer_lemeshow,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            LogisticRegressionDialogAction::None => {}
        }

        // 3. Multinomial Logistic Regression Dialog
        let multinomial_action = render_multinomial_logistic_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.multinomial_logistic_dialog_state,
        );

        match multinomial_action {
            MultinomialLogisticDialogAction::Run {
                dep_var,
                factors,
                covariates,
                reference_category,
                reference_value,
                confidence_level,
                include_constant,
                print_classification,
                print_goodness_of_fit,
                print_lrt,
            } => {
                let params = json!({
                    "dependent_var": dep_var,
                    "factors": factors,
                    "covariates": covariates,
                    "reference_category": reference_category,
                    "reference_value": reference_value,
                    "confidence_level": confidence_level,
                    "include_constant": include_constant,
                    "print_classification": print_classification,
                    "print_goodness_of_fit": print_goodness_of_fit,
                    "print_lrt": print_lrt,
                });
                match self.command_bus.execute(
                    "stat.regression.multinomial",
                    &mut self.dataset,
                    params,
                ) {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::MultinomialLogisticResult,
                        >(result_value)
                        {
                            let dep = result.dependent_var.clone();
                            let count = result.factors.len() + result.covariates.len();
                            self.output_view_state.add_multinomial_logistic(result);
                            self.status_message = StatusMessage::MultinomialLogisticExecuted {
                                dep_var: dep,
                                predictors_count: count,
                            };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            MultinomialLogisticDialogAction::Paste {
                dep_var,
                factors,
                covariates,
                reference_category,
                reference_value,
                confidence_level,
                ..
            } => {
                let ref_str = match reference_category.as_str() {
                    "first" => "FIRST".to_string(),
                    "custom" => {
                        if let Some(v) = reference_value {
                            format!("{v}")
                        } else {
                            "LAST".to_string()
                        }
                    }
                    _ => "LAST".to_string(),
                };
                let sps = statcraft_syntax::commands::MultinomialLogisticCommand::to_sps(
                    &dep_var,
                    &factors,
                    &covariates,
                    &ref_str,
                    confidence_level,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            MultinomialLogisticDialogAction::None => {}
        }

        // 3. Curve Estimation Dialog
        let curve_action =
            render_curvefit_dialog(ctx, self.dataset.as_ref(), &mut self.curvefit_dialog_state);

        match curve_action {
            CurveFitDialogAction::Run {
                dep_var,
                indep_var,
                models,
                include_constant,
            } => {
                let model_names: Vec<String> =
                    models.iter().map(|m| m.name().to_lowercase()).collect();
                let params = json!({
                    "dependent_var": dep_var,
                    "independent_var": indep_var,
                    "models": model_names,
                    "include_constant": include_constant,
                });
                match self
                    .command_bus
                    .execute("stat.regression.curve", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) =
                            serde_json::from_value::<statcraft_algo::CurveFitResult>(result_value)
                        {
                            let dep = result.dependent_var.clone();
                            let indep = result.independent_var.clone();
                            self.output_view_state.add_curve_fit(result);
                            self.status_message = StatusMessage::CurveFitExecuted {
                                dep_var: dep,
                                indep_var: indep,
                            };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            CurveFitDialogAction::Paste {
                dep_var,
                indep_var,
                models,
                include_constant,
            } => {
                let sps = statcraft_syntax::commands::CurveFitCommand::to_sps(
                    &dep_var,
                    &indep_var,
                    &models,
                    include_constant,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            CurveFitDialogAction::None => {}
        }
    }
}
