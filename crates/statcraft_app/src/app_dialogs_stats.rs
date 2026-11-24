use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_crosstabs_dialog, render_descriptives_dialog, render_explore_dialog,
    render_frequencies_dialog, CrosstabsDialogAction, DescriptivesDialogAction,
    ExploreDialogAction, FrequenciesDialogAction,
};
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_stats_dialogs(&mut self, ctx: &egui::Context) {
        // 1. Descriptives Dialog
        let desc_action = render_descriptives_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.descriptives_dialog_state,
        );

        match desc_action {
            DescriptivesDialogAction::Run { vars, options } => {
                let params = json!({
                    "vars": vars,
                    "options": options
                });
                match self
                    .command_bus
                    .execute("stat.descriptives", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::DescriptivesResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state.add_descriptives(results);
                            self.status_message = StatusMessage::DescriptivesExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            DescriptivesDialogAction::Paste { vars } => {
                let sps = statcraft_syntax::commands::DescriptivesCommand::to_sps(&vars);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            DescriptivesDialogAction::None => {}
        }

        // 2. Frequencies Dialog
        let freq_action = render_frequencies_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.frequencies_dialog_state,
        );

        match freq_action {
            FrequenciesDialogAction::Run {
                vars,
                display_tables,
                apa_tables: _,
                statistics,
                percentiles,
                group_midpoints,
                charts,
                order,
                multiple_vars_mode,
                suppress_tables_limit,
                table_styles,
                create_style_report,
            } => {
                let mut params_map = serde_json::Map::new();
                params_map.insert("vars".to_string(), json!(vars));
                params_map.insert("display_tables".to_string(), json!(display_tables));
                if let Some(s) = statistics {
                    params_map.insert("statistics".to_string(), s);
                }
                if let Some(p) = percentiles {
                    params_map.insert("percentiles".to_string(), p);
                }
                if group_midpoints {
                    params_map.insert("group_midpoints".to_string(), json!(true));
                }
                if let Some(c) = charts {
                    params_map.insert("charts".to_string(), c);
                }
                if let Some(o) = order {
                    params_map.insert("order".to_string(), json!(o));
                }
                if let Some(m) = multiple_vars_mode {
                    params_map.insert("multiple_vars_mode".to_string(), json!(m));
                }
                if let Some(l) = suppress_tables_limit {
                    params_map.insert("suppress_tables_limit".to_string(), json!(l));
                }
                let style_rules = table_styles.as_ref().and_then(|val| {
                    serde_json::from_value::<Vec<statcraft_reporting::styling::TableStyleRule>>(
                        val.clone(),
                    )
                    .ok()
                });
                if let Some(ts) = table_styles {
                    params_map.insert("table_styles".to_string(), ts);
                }
                if create_style_report {
                    params_map.insert("create_style_report".to_string(), json!(true));
                }
                let params = serde_json::Value::Object(params_map);
                let sps_log =
                    statcraft_syntax::commands::FrequenciesCommand::to_sps_from_params(&params);
                match self
                    .command_bus
                    .execute("stat.frequencies", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::FrequencyResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            if display_tables {
                                self.output_view_state.add_frequencies_styled(
                                    results,
                                    sps_log,
                                    style_rules.as_deref(),
                                    create_style_report,
                                );
                            }
                            self.status_message = StatusMessage::FrequenciesExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            FrequenciesDialogAction::Paste {
                vars,
                display_tables,
                apa_tables: _,
                statistics,
                percentiles,
                group_midpoints,
                charts,
                order,
                multiple_vars_mode,
                suppress_tables_limit,
                table_styles,
                create_style_report,
            } => {
                let mut params_map = serde_json::Map::new();
                params_map.insert("vars".to_string(), json!(vars));
                params_map.insert("display_tables".to_string(), json!(display_tables));
                if let Some(s) = statistics {
                    params_map.insert("statistics".to_string(), s);
                }
                if let Some(p) = percentiles {
                    params_map.insert("percentiles".to_string(), p);
                }
                if group_midpoints {
                    params_map.insert("group_midpoints".to_string(), json!(true));
                }
                if let Some(c) = charts {
                    params_map.insert("charts".to_string(), c);
                }
                if let Some(o) = order {
                    params_map.insert("order".to_string(), json!(o));
                }
                if let Some(m) = multiple_vars_mode {
                    params_map.insert("multiple_vars_mode".to_string(), json!(m));
                }
                if let Some(l) = suppress_tables_limit {
                    params_map.insert("suppress_tables_limit".to_string(), json!(l));
                }
                if let Some(ts) = table_styles {
                    params_map.insert("table_styles".to_string(), ts);
                }
                if create_style_report {
                    params_map.insert("create_style_report".to_string(), json!(true));
                }
                let params = serde_json::Value::Object(params_map);
                let sps =
                    statcraft_syntax::commands::FrequenciesCommand::to_sps_from_params(&params);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            FrequenciesDialogAction::None => {}
        }

        // 3. Explore Dialog
        let explore_action =
            render_explore_dialog(ctx, self.dataset.as_ref(), &mut self.explore_dialog_state);

        match explore_action {
            ExploreDialogAction::Run {
                dependent_vars,
                factor_vars,
                compute_descriptives,
                compute_normality,
                compute_extremes,
                confidence_level,
                missing,
            } => {
                let params = json!({
                    "dependent_vars": dependent_vars,
                    "factor_vars": factor_vars,
                    "compute_descriptives": compute_descriptives,
                    "compute_normality": compute_normality,
                    "compute_extremes": compute_extremes,
                    "confidence_level": confidence_level,
                    "missing": missing,
                });
                match self
                    .command_bus
                    .execute("stat.explore", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(res) =
                            serde_json::from_value::<statcraft_algo::ExploreResult>(result_value)
                        {
                            self.output_view_state.add_explore(res);
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            ExploreDialogAction::Paste {
                dependent_vars,
                factor_vars,
            } => {
                let sps = statcraft_syntax::commands::ExploreCommand::to_sps(
                    &dependent_vars,
                    &factor_vars,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            ExploreDialogAction::None => {}
        }

        // 4. Crosstabs Dialog
        let crosstabs_action =
            render_crosstabs_dialog(ctx, self.dataset.as_ref(), &mut self.crosstabs_dialog_state);

        match crosstabs_action {
            CrosstabsDialogAction::Run {
                rows,
                cols,
                options,
            } => {
                let params = json!({
                    "rows": rows,
                    "cols": cols,
                    "expected": options.expected,
                    "row_pct": options.row_pct,
                    "col_pct": options.col_pct,
                    "total_pct": options.total_pct,
                    "residuals": options.residuals,
                    "chisq": options.chisq,
                    "phi_cramer": options.phi_cramer,
                });
                match self
                    .command_bus
                    .execute("stat.crosstabs", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::CrosstabResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state.add_crosstabs(results, options);
                            self.status_message = StatusMessage::CrosstabsExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            CrosstabsDialogAction::Paste {
                rows,
                cols,
                options,
            } => {
                let sps =
                    statcraft_syntax::commands::CrosstabsCommand::to_sps(&rows, &cols, &options);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            CrosstabsDialogAction::None => {}
        }
    }
}
