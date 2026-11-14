use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_multivariate_glm_dialog, render_repeated_glm_dialog, render_univariate_glm_dialog,
    MultivariateGlmDialogAction, RepeatedGlmDialogAction, UnivariateGlmDialogAction,
};
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_glm_dialogs(&mut self, ctx: &egui::Context) {
        // 1. Univariate GLM (UNIANOVA)
        let uni_action = render_univariate_glm_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.univariate_glm_dialog_state,
        );

        match uni_action {
            UnivariateGlmDialogAction::Run {
                dependent,
                fixed_factors,
                covariates,
                intercept,
                descriptives,
                homogeneity_levene,
            } => {
                let params = json!({
                    "dependent": dependent,
                    "fixed_factors": fixed_factors,
                    "covariates": covariates,
                    "intercept": intercept,
                    "descriptives": descriptives,
                    "homogeneity_levene": homogeneity_levene,
                });
                match self
                    .command_bus
                    .execute("stat.glm.univariate", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::glm::UnivariateGlmResult,
                        >(result_value)
                        {
                            let dep = result.dependent.clone();
                            self.output_view_state.add_univariate_glm(result);
                            self.status_message =
                                StatusMessage::UnivariateGlmExecuted { dep_var: dep };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            UnivariateGlmDialogAction::Paste {
                dependent,
                fixed_factors,
                covariates,
                intercept,
            } => {
                let sps = statcraft_syntax::commands::UnivariateGlmCommand::to_sps(
                    &dependent,
                    &fixed_factors,
                    &covariates,
                    intercept,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            UnivariateGlmDialogAction::None => {}
        }

        // 2. Multivariate GLM (MANOVA)
        let multi_action = render_multivariate_glm_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.multivariate_glm_dialog_state,
        );

        match multi_action {
            MultivariateGlmDialogAction::Run {
                dependents,
                factors,
                covariates,
                intercept,
            } => {
                let params = json!({
                    "dependents": dependents,
                    "factors": factors,
                    "covariates": covariates,
                    "intercept": intercept,
                });
                match self
                    .command_bus
                    .execute("stat.glm.multivariate", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::glm::MultivariateGlmResult,
                        >(result_value)
                        {
                            let count = result.dependents.len();
                            self.output_view_state.add_multivariate_glm(result);
                            self.status_message = StatusMessage::MultivariateGlmExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            MultivariateGlmDialogAction::Paste {
                dependents,
                factors,
                covariates,
                intercept,
            } => {
                let sps = statcraft_syntax::commands::MultivariateGlmCommand::to_sps(
                    &dependents,
                    &factors,
                    &covariates,
                    intercept,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            MultivariateGlmDialogAction::None => {}
        }

        // 3. Repeated Measures GLM
        let repeated_action = render_repeated_glm_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.repeated_glm_dialog_state,
        );

        match repeated_action {
            RepeatedGlmDialogAction::Run {
                factor_name,
                measures,
            } => {
                let params = json!({
                    "factor_name": factor_name,
                    "measures": measures,
                });
                match self
                    .command_bus
                    .execute("stat.glm.repeated", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::glm::RepeatedGlmResult,
                        >(result_value)
                        {
                            let factor = result.factor_name.clone();
                            let count = result.measure_variables.len();
                            self.output_view_state.add_repeated_glm(result);
                            self.status_message = StatusMessage::RepeatedGlmExecuted {
                                factor,
                                measures_count: count,
                            };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            RepeatedGlmDialogAction::Paste {
                factor_name,
                measures,
            } => {
                let sps =
                    statcraft_syntax::commands::RepeatedGlmCommand::to_sps(&factor_name, &measures);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            RepeatedGlmDialogAction::None => {}
        }
    }
}
