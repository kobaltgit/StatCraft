use super::app::{StatCraftApp, StatusMessage};
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_deming_regression_dialog(&mut self, ctx: &egui::Context) {
        let deming_action = crate::dialogs::render_deming_regression_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.deming_regression_dialog_state,
        );
        match deming_action {
            crate::dialogs::DemingRegressionDialogAction::Run {
                dependent_var,
                independent_var,
                lambda,
                ci,
                include_scatter_plot,
                include_identity_line,
                include_residual_plot,
                save_predicted,
                save_residuals,
                table_styles,
                bootstrap,
            } => {
                let mut params = json!({
                    "dependent_var": dependent_var,
                    "independent_var": independent_var,
                    "lambda": lambda,
                    "ci": ci,
                    "include_scatter_plot": include_scatter_plot,
                    "include_identity_line": include_identity_line,
                    "include_residual_plot": include_residual_plot,
                    "save_predicted": save_predicted,
                    "save_residuals": save_residuals,
                });
                if let Some(boot) = bootstrap {
                    if let Some(obj) = params.as_object_mut() {
                        obj.insert("bootstrap".to_string(), boot);
                    }
                }
                match self.command_bus.execute(
                    "stat.regression.orthogonal",
                    &mut self.dataset,
                    params,
                ) {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::DemingRegressionResult,
                        >(result_value)
                        {
                            let dep_var = result.dep_var.clone();
                            let dep_label =
                                crate::views::output_view::tables_common::resolve_var_label(
                                    self.dataset.as_ref(),
                                    &result.dep_var,
                                );
                            let indep_label =
                                crate::views::output_view::tables_common::resolve_var_label(
                                    self.dataset.as_ref(),
                                    &result.indep_var,
                                );
                            self.output_view_state.add_deming_regression(
                                result,
                                dep_label,
                                indep_label,
                                include_scatter_plot,
                                include_identity_line,
                                include_residual_plot,
                                table_styles,
                            );
                            self.status_message = StatusMessage::RegressionExecuted {
                                dep_var,
                                indep_count: 1,
                            };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            crate::dialogs::DemingRegressionDialogAction::Paste {
                dependent_var,
                independent_var,
                lambda,
                ci,
                include_scatter_plot,
                include_identity_line,
                include_residual_plot,
                save_predicted,
                save_residuals,
                bootstrap,
            } => {
                let boot_opts = bootstrap.and_then(|v| {
                    serde_json::from_value::<statcraft_algo::BootstrapOptions>(v).ok()
                });
                let sps =
                    statcraft_syntax::commands::DemingRegressionCommand::to_sps_with_bootstrap(
                        &dependent_var,
                        &independent_var,
                        lambda,
                        ci,
                        include_scatter_plot,
                        include_identity_line,
                        include_residual_plot,
                        save_predicted,
                        save_residuals,
                        boot_opts.as_ref(),
                    );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            crate::dialogs::DemingRegressionDialogAction::None => {}
        }
    }
}
