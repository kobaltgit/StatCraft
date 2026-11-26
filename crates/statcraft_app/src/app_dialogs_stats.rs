use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_crosstabs_dialog, render_descriptives_dialog, render_explore_dialog,
    render_frequencies_dialog, render_turf_dialog, CrosstabsDialogAction, DescriptivesDialogAction,
    ExploreDialogAction, FrequenciesDialogAction, TurfDialogAction,
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
            DescriptivesDialogAction::Run {
                vars,
                options,
                table_styles,
                create_style_report,
                bootstrap,
            } => {
                let mut params = json!({
                    "vars": vars,
                    "options": options
                });
                let style_rules = table_styles.as_ref().and_then(|val| {
                    serde_json::from_value::<Vec<statcraft_reporting::styling::TableStyleRule>>(
                        val.clone(),
                    )
                    .ok()
                });
                if let Some(ts) = table_styles {
                    if let Some(obj) = params.as_object_mut() {
                        obj.insert("table_styles".to_string(), ts);
                    }
                }
                let boot_opts = bootstrap.as_ref().and_then(|v| {
                    serde_json::from_value::<statcraft_algo::BootstrapOptions>(v.clone()).ok()
                });
                if let Some(boot) = bootstrap {
                    if let Some(obj) = params.as_object_mut() {
                        obj.insert("bootstrap".to_string(), boot);
                    }
                }
                let sps_log = statcraft_syntax::commands::DescriptivesCommand::to_sps_full(
                    &vars,
                    &options,
                    boot_opts.as_ref(),
                );
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
                            self.output_view_state.add_descriptives_with_log(
                                results,
                                sps_log,
                                style_rules.as_deref(),
                                create_style_report,
                            );
                            self.status_message = StatusMessage::DescriptivesExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            DescriptivesDialogAction::Paste {
                vars,
                options,
                bootstrap,
            } => {
                let boot_opts = bootstrap.and_then(|v| {
                    serde_json::from_value::<statcraft_algo::BootstrapOptions>(v).ok()
                });
                let sps = statcraft_syntax::commands::DescriptivesCommand::to_sps_full(
                    &vars,
                    &options,
                    boot_opts.as_ref(),
                );
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
                bootstrap,
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
                if let Some(b) = bootstrap {
                    params_map.insert("bootstrap".to_string(), b);
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
                bootstrap,
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
                if let Some(b) = bootstrap {
                    params_map.insert("bootstrap".to_string(), b);
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
                label_var,
                compute_descriptives,
                compute_m_estimators,
                compute_percentiles,
                compute_normality,
                compute_extremes,
                spread_level_mode,
                spread_level_power,
                display_mode,
                boxplot_mode,
                plot_stem_leaf,
                plot_histogram,
                confidence_level,
                missing,
                bootstrap,
            } => {
                let mut params_map = serde_json::Map::new();
                params_map.insert("dependent_vars".to_string(), json!(dependent_vars));
                params_map.insert("factor_vars".to_string(), json!(factor_vars));
                if let Some(lbl) = label_var {
                    params_map.insert("label_var".to_string(), json!(lbl));
                }
                params_map.insert(
                    "compute_descriptives".to_string(),
                    json!(compute_descriptives),
                );
                params_map.insert(
                    "compute_m_estimators".to_string(),
                    json!(compute_m_estimators),
                );
                params_map.insert(
                    "compute_percentiles".to_string(),
                    json!(compute_percentiles),
                );
                params_map.insert("compute_normality".to_string(), json!(compute_normality));
                params_map.insert("compute_extremes".to_string(), json!(compute_extremes));
                params_map.insert("spread_level_mode".to_string(), json!(spread_level_mode));
                if let Some(p) = spread_level_power {
                    params_map.insert("spread_level_power".to_string(), json!(p));
                }
                params_map.insert("display_mode".to_string(), json!(display_mode));
                params_map.insert("boxplot_mode".to_string(), json!(boxplot_mode));
                params_map.insert("plot_stem_leaf".to_string(), json!(plot_stem_leaf));
                params_map.insert("plot_histogram".to_string(), json!(plot_histogram));
                params_map.insert("confidence_level".to_string(), json!(confidence_level));
                params_map.insert("missing".to_string(), json!(missing));
                if let Some(b) = bootstrap {
                    params_map.insert("bootstrap".to_string(), b);
                }
                let params = serde_json::Value::Object(params_map);
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
            ExploreDialogAction::Paste { options } => {
                let sps = statcraft_syntax::commands::ExploreCommand::to_sps_full(&options);
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
                layers,
                options,
                table_styles,
                create_style_report,
                bootstrap,
            } => {
                let mut params_map = serde_json::to_value(&options)
                    .ok()
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default();
                params_map.insert("rows".to_string(), json!(rows));
                params_map.insert("cols".to_string(), json!(cols));
                if !layers.is_empty() {
                    params_map.insert("layers".to_string(), json!(layers));
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
                let boot_opts = bootstrap.as_ref().and_then(|v| {
                    serde_json::from_value::<statcraft_algo::BootstrapOptions>(v.clone()).ok()
                });
                if let Some(b) = bootstrap {
                    params_map.insert("bootstrap".to_string(), b);
                }
                let sps_log = statcraft_syntax::commands::CrosstabsCommand::to_sps_with_layers(
                    &rows, &cols, &layers, &options,
                );
                let params = serde_json::Value::Object(params_map);
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
                            self.output_view_state.add_crosstabs_styled(
                                results,
                                options,
                                Some(sps_log),
                                style_rules.as_deref(),
                                create_style_report,
                                boot_opts.as_ref(),
                            );
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
                layers,
                options,
                table_styles: _,
                create_style_report: _,
                bootstrap: _,
            } => {
                let sps = statcraft_syntax::commands::CrosstabsCommand::to_sps_with_layers(
                    &rows, &cols, &layers, &options,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            CrosstabsDialogAction::None => {}
        }

        // 5. TURF Dialog
        let turf_action =
            render_turf_dialog(ctx, self.dataset.as_ref(), &mut self.turf_dialog_state);

        match turf_action {
            TurfDialogAction::Run { options } => {
                let sps_log = statcraft_syntax::commands::TurfCommand::to_sps(&options);
                match serde_json::to_value(&options) {
                    Ok(params) => {
                        match self
                            .command_bus
                            .execute("stat.turf", &mut self.dataset, params)
                        {
                            Ok(result_val) => {
                                match serde_json::from_value::<statcraft_algo::TurfResult>(
                                    result_val,
                                ) {
                                    Ok(result) => {
                                        let count = options.variables.len();
                                        self.output_view_state.add_turf_with_dataset(
                                            result,
                                            options,
                                            sps_log,
                                            self.dataset.as_ref(),
                                        );
                                        self.status_message = StatusMessage::TurfExecuted { count };
                                    }
                                    Err(e) => {
                                        self.status_message = StatusMessage::Error(e.to_string());
                                    }
                                }
                            }
                            Err(e) => {
                                self.status_message = StatusMessage::Error(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            TurfDialogAction::Paste { options } => {
                let sps = statcraft_syntax::commands::TurfCommand::to_sps(&options);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            TurfDialogAction::None => {}
        }
    }
}
