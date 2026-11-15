use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_codebook_dialog, render_kmeans_dialog, render_summarize_dialog,
    render_survival_km_dialog, render_survival_lifetables_dialog, CodebookDialogAction,
    KMeansDialogAction, KaplanMeierDialogAction, LifeTablesDialogAction, SummarizeDialogAction,
};
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_pkg3_dialogs(&mut self, ctx: &egui::Context) {
        // 1. K-Means Cluster Analysis (QUICK CLUSTER)
        let kmeans_action =
            render_kmeans_dialog(ctx, self.dataset.as_ref(), &mut self.kmeans_dialog_state);

        match kmeans_action {
            KMeansDialogAction::Run {
                variables,
                clusters,
                max_iterations,
                convergence_criterion,
                save_cluster,
                save_distance,
            } => {
                let params = json!({
                    "variables": variables,
                    "k": clusters,
                    "max_iterations": max_iterations,
                    "convergence_criterion": convergence_criterion,
                    "save_cluster": save_cluster,
                    "save_distance": save_distance,
                });
                match self
                    .command_bus
                    .execute("stat.cluster.kmeans", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) =
                            serde_json::from_value::<statcraft_algo::KMeansResult>(result_value)
                        {
                            let cases = result.valid_cases_count;
                            let k = result.k;
                            self.output_view_state.add_kmeans(result);
                            self.status_message =
                                StatusMessage::KMeansExecuted { clusters: k, cases };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            KMeansDialogAction::Paste {
                variables,
                clusters,
                max_iterations,
                convergence_criterion,
                save_cluster,
                save_distance,
            } => {
                let sps = statcraft_syntax::commands::KMeansClusterCommand::to_sps(
                    &variables,
                    clusters,
                    max_iterations,
                    convergence_criterion,
                    save_cluster,
                    save_distance,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            KMeansDialogAction::None => {}
        }

        // 2. Codebook Report (CODEBOOK)
        let codebook_action =
            render_codebook_dialog(ctx, self.dataset.as_ref(), &mut self.codebook_dialog_state);

        match codebook_action {
            CodebookDialogAction::Run {
                variables,
                max_categories,
                include_counts,
                include_stats,
            } => {
                let params = json!({
                    "variables": variables,
                    "max_categories": max_categories,
                    "include_counts": include_counts,
                    "include_stats": include_stats,
                });
                match self
                    .command_bus
                    .execute("stat.reports.codebook", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) =
                            serde_json::from_value::<statcraft_algo::CodebookResult>(result_value)
                        {
                            let count = result.cards.len();
                            self.output_view_state.add_codebook(result);
                            self.status_message = StatusMessage::CodebookExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            CodebookDialogAction::Paste {
                variables,
                max_categories,
                ..
            } => {
                let sps =
                    statcraft_syntax::commands::CodebookCommand::to_sps(&variables, max_categories);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            CodebookDialogAction::None => {}
        }

        // 3. Summarize Report (SUMMARIZE)
        let summarize_action =
            render_summarize_dialog(ctx, self.dataset.as_ref(), &mut self.summarize_dialog_state);

        match summarize_action {
            SummarizeDialogAction::Run {
                variables,
                group_by,
                limit_cases,
                display_cases,
            } => {
                let params = json!({
                    "variables": variables,
                    "group_by": group_by,
                    "limit_cases": limit_cases,
                    "display_cases": display_cases,
                });
                match self
                    .command_bus
                    .execute("stat.reports.summarize", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) =
                            serde_json::from_value::<statcraft_algo::SummarizeResult>(result_value)
                        {
                            let count = result.total_cases;
                            self.output_view_state.add_summarize(result);
                            self.status_message = StatusMessage::SummarizeExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            SummarizeDialogAction::Paste {
                variables,
                group_by,
                limit_cases,
                ..
            } => {
                let sps = statcraft_syntax::commands::SummarizeCommand::to_sps(
                    &variables,
                    &group_by,
                    limit_cases,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            SummarizeDialogAction::None => {}
        }

        // 4. Kaplan-Meier Survival Analysis (KM)
        let km_action = render_survival_km_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.survival_km_dialog_state,
        );

        match km_action {
            KaplanMeierDialogAction::Run {
                time_var,
                status_var,
                event_value,
                factor_var,
            } => {
                let time_name = time_var.clone();
                let params = json!({
                    "time_var": time_var,
                    "status_var": status_var,
                    "event_value": event_value,
                    "factor_var": factor_var,
                });
                match self
                    .command_bus
                    .execute("stat.survival.km", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) =
                            serde_json::from_value::<statcraft_algo::KMResult>(result_value)
                        {
                            self.output_view_state.add_kaplan_meier(result);
                            self.status_message = StatusMessage::KaplanMeierExecuted {
                                time_var: time_name,
                            };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            KaplanMeierDialogAction::Paste {
                time_var,
                status_var,
                event_value,
                factor_var,
            } => {
                let sps = statcraft_syntax::commands::KaplanMeierCommand::to_sps(
                    &time_var,
                    &status_var,
                    event_value,
                    factor_var.as_deref(),
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            KaplanMeierDialogAction::None => {}
        }

        // 5. Life Tables Survival Analysis (SURVIVAL)
        let lifetables_action = render_survival_lifetables_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.survival_lifetables_dialog_state,
        );

        match lifetables_action {
            LifeTablesDialogAction::Run {
                time_var,
                status_var,
                event_value,
                interval_end,
                interval_width,
            } => {
                let time_name = time_var.clone();
                let params = json!({
                    "time_var": time_var,
                    "status_var": status_var,
                    "event_value": event_value,
                    "interval_end": interval_end,
                    "interval_width": interval_width,
                });
                match self.command_bus.execute(
                    "stat.survival.lifetables",
                    &mut self.dataset,
                    params,
                ) {
                    Ok(result_value) => {
                        if let Ok(result) =
                            serde_json::from_value::<statcraft_algo::LifeTableResult>(result_value)
                        {
                            self.output_view_state.add_life_tables(result);
                            self.status_message = StatusMessage::LifeTablesExecuted {
                                time_var: time_name,
                            };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            LifeTablesDialogAction::Paste {
                time_var,
                status_var,
                event_value,
                interval_end,
                interval_width,
            } => {
                let sps = statcraft_syntax::commands::LifeTablesCommand::to_sps(
                    &time_var,
                    &status_var,
                    event_value,
                    interval_end,
                    interval_width,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            LifeTablesDialogAction::None => {}
        }
    }
}
