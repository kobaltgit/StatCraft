use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_auto_recode_dialog, render_count_values_dialog, render_date_time_dialog,
    render_dummy_dialog, render_optimal_binning_dialog, render_random_seed_dialog,
    render_replace_missing_dialog, render_shift_values_dialog, AutoRecodeDialogAction,
    CountValuesDialogAction, DateTimeDialogAction, DummyDialogAction, OptimalBinningDialogAction,
    RandomSeedDialogAction, ReplaceMissingDialogAction, ShiftValuesDialogAction,
};
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_transform_dialogs(&mut self, ctx: &egui::Context) {
        // 1. Shift Values Dialog (Lead / Lag)
        let shift_action = render_shift_values_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.shift_values_dialog_state,
        );

        match shift_action {
            ShiftValuesDialogAction::Run {
                source_var,
                target_var,
                direction,
                step,
                target_label,
            } => {
                let dir_str = match direction {
                    statcraft_algo::ShiftDirection::Lag => "lag",
                    statcraft_algo::ShiftDirection::Lead => "lead",
                };
                let mut params_map = serde_json::Map::new();
                params_map.insert("source_var".to_string(), json!(source_var));
                params_map.insert("target_var".to_string(), json!(target_var));
                params_map.insert("direction".to_string(), json!(dir_str));
                params_map.insert("step".to_string(), json!(step));
                if let Some(lbl) = &target_label {
                    params_map.insert("target_label".to_string(), json!(lbl));
                }

                let sps = statcraft_syntax::commands::ShiftValuesCommand::to_sps(
                    &source_var,
                    &target_var,
                    direction,
                    step,
                    target_label.as_deref(),
                );

                match self.command_bus.execute(
                    "transform.shift_values",
                    &mut self.dataset,
                    serde_json::Value::Object(params_map),
                ) {
                    Ok(_) => {
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        let rows = self.dataset.as_ref().map(|d| d.row_count()).unwrap_or(0);
                        self.status_message = StatusMessage::ShiftValuesExecuted {
                            target: target_var,
                            rows,
                        };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            ShiftValuesDialogAction::Paste {
                source_var,
                target_var,
                direction,
                step,
                target_label,
            } => {
                let sps = statcraft_syntax::commands::ShiftValuesCommand::to_sps(
                    &source_var,
                    &target_var,
                    direction,
                    step,
                    target_label.as_deref(),
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            ShiftValuesDialogAction::None => {}
        }

        // 2. Automatic Recode Dialog
        let auto_recode_action = render_auto_recode_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.auto_recode_dialog_state,
        );

        match auto_recode_action {
            AutoRecodeDialogAction::Run { pairs, order } => {
                let order_str = match order {
                    statcraft_algo::AutoRecodeOrder::LowestValue => "lowest",
                    statcraft_algo::AutoRecodeOrder::HighestValue => "highest",
                };
                let pair_objs: Vec<serde_json::Value> = pairs
                    .iter()
                    .map(|p| {
                        json!({
                            "source_var": p.source,
                            "target_var": p.target,
                        })
                    })
                    .collect();
                let params = json!({
                    "pairs": pair_objs,
                    "order": order_str
                });

                let pair_refs: Vec<(&str, &str)> = pairs
                    .iter()
                    .map(|p| (p.source.as_str(), p.target.as_str()))
                    .collect();
                let sps = statcraft_syntax::commands::AutoRecodeCommand::to_sps(&pair_refs, order);

                match self
                    .command_bus
                    .execute("transform.auto_recode", &mut self.dataset, params)
                {
                    Ok(_) => {
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message =
                            StatusMessage::AutoRecodeExecuted { count: pairs.len() };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            AutoRecodeDialogAction::Paste { pairs, order } => {
                let pair_refs: Vec<(&str, &str)> = pairs
                    .iter()
                    .map(|p| (p.source.as_str(), p.target.as_str()))
                    .collect();
                let sps = statcraft_syntax::commands::AutoRecodeCommand::to_sps(&pair_refs, order);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            AutoRecodeDialogAction::None => {}
        }

        // 3. Create Dummy Variables Dialog
        let dummy_action =
            render_dummy_dialog(ctx, self.dataset.as_ref(), &mut self.dummy_dialog_state);

        match dummy_action {
            DummyDialogAction::Run {
                vars,
                root_prefix,
                omit_reference,
            } => {
                let var_refs: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
                let sps = statcraft_syntax::commands::CreateDummiesCommand::to_sps(
                    &var_refs,
                    Some(&root_prefix),
                    omit_reference,
                );
                let params = json!({
                    "variables": vars,
                    "root_prefix": root_prefix,
                    "omit_reference": omit_reference
                });

                match self.command_bus.execute(
                    "transform.create_dummies",
                    &mut self.dataset,
                    params,
                ) {
                    Ok(res) => {
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        let count = res["created_count"].as_u64().unwrap_or(0) as usize;
                        self.status_message = StatusMessage::CreateDummiesExecuted { count };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            DummyDialogAction::Paste {
                vars,
                root_prefix,
                omit_reference,
            } => {
                let var_refs: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
                let sps = statcraft_syntax::commands::CreateDummiesCommand::to_sps(
                    &var_refs,
                    Some(&root_prefix),
                    omit_reference,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            DummyDialogAction::None => {}
        }

        // 4. Set Random Number Seed Dialog
        let seed_action = render_random_seed_dialog(ctx, &mut self.random_seed_dialog_state);

        match seed_action {
            RandomSeedDialogAction::Run { seed } => {
                let sps = statcraft_syntax::commands::RandomSeedCommand::to_sps(seed);
                let params = json!({ "seed": seed });

                match self
                    .command_bus
                    .execute("transform.random_seed", &mut self.dataset, params)
                {
                    Ok(_) => {
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message = StatusMessage::RandomSeedExecuted { seed };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            RandomSeedDialogAction::Paste { seed } => {
                let sps = statcraft_syntax::commands::RandomSeedCommand::to_sps(seed);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            RandomSeedDialogAction::None => {}
        }

        // 5. Count Values within Cases Dialog
        let count_action = render_count_values_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.count_values_dialog_state,
        );

        match count_action {
            CountValuesDialogAction::Run {
                target,
                label,
                variables,
                criteria,
            } => {
                let sps = statcraft_syntax::commands::CountValuesCommand::to_sps(
                    &target,
                    &variables,
                    &criteria,
                    label.as_deref(),
                );
                let mut params_map = serde_json::Map::new();
                params_map.insert("target_var".to_string(), json!(target));
                params_map.insert("variables".to_string(), json!(variables));
                params_map.insert("criteria".to_string(), json!(criteria));
                if let Some(lbl) = &label {
                    params_map.insert("target_label".to_string(), json!(lbl));
                }

                match self.command_bus.execute(
                    "transform.count_values",
                    &mut self.dataset,
                    serde_json::Value::Object(params_map),
                ) {
                    Ok(_) => {
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        let rows = self.dataset.as_ref().map(|d| d.row_count()).unwrap_or(0);
                        self.status_message = StatusMessage::CountValuesExecuted {
                            target,
                            variables: variables.len(),
                            rows,
                        };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            CountValuesDialogAction::Paste {
                target,
                label,
                variables,
                criteria,
            } => {
                let sps = statcraft_syntax::commands::CountValuesCommand::to_sps(
                    &target,
                    &variables,
                    &criteria,
                    label.as_deref(),
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            CountValuesDialogAction::None => {}
        }

        // 6. Replace Missing Values Dialog
        let missing_action = render_replace_missing_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.replace_missing_dialog_state,
        );

        match missing_action {
            ReplaceMissingDialogAction::Run { specs } => {
                let sps = statcraft_syntax::commands::ReplaceMissingCommand::to_sps(&specs);
                let params = json!({ "variables": specs });

                match self.command_bus.execute(
                    "transform.replace_missing",
                    &mut self.dataset,
                    params,
                ) {
                    Ok(_) => {
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message =
                            StatusMessage::ReplaceMissingExecuted { count: specs.len() };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            ReplaceMissingDialogAction::Paste { specs } => {
                let sps = statcraft_syntax::commands::ReplaceMissingCommand::to_sps(&specs);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            ReplaceMissingDialogAction::None => {}
        }

        // 7. Date and Time Wizard Dialog
        let dt_action =
            render_date_time_dialog(ctx, self.dataset.as_ref(), &mut self.date_time_dialog_state);

        match dt_action {
            DateTimeDialogAction::Run {
                operation,
                target_var,
                target_label,
                primary_var,
                secondary_var,
                part,
                unit,
                duration_value,
            } => {
                let sps = statcraft_syntax::commands::DateTimeWizardCommand::to_sps(
                    &operation,
                    &target_var,
                    &primary_var,
                    secondary_var.as_deref(),
                    part.as_deref(),
                    unit.as_deref(),
                    duration_value,
                    target_label.as_deref(),
                );

                let mut params_map = serde_json::Map::new();
                params_map.insert("operation".to_string(), json!(operation));
                params_map.insert("target_var".to_string(), json!(target_var));
                params_map.insert("primary_var".to_string(), json!(primary_var));
                if let Some(lbl) = &target_label {
                    params_map.insert("target_label".to_string(), json!(lbl));
                }
                if let Some(sec) = &secondary_var {
                    params_map.insert("secondary_var".to_string(), json!(sec));
                }
                if let Some(p) = &part {
                    params_map.insert("part".to_string(), json!(p));
                }
                if let Some(u) = &unit {
                    params_map.insert("unit".to_string(), json!(u));
                }
                if let Some(dur) = duration_value {
                    params_map.insert("duration_value".to_string(), json!(dur));
                }

                match self.command_bus.execute(
                    "transform.date_time_wizard",
                    &mut self.dataset,
                    serde_json::Value::Object(params_map),
                ) {
                    Ok(_) => {
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message =
                            StatusMessage::DateTimeExecuted { target: target_var };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            DateTimeDialogAction::Paste {
                operation,
                target_var,
                target_label,
                primary_var,
                secondary_var,
                part,
                unit,
                duration_value,
            } => {
                let sps = statcraft_syntax::commands::DateTimeWizardCommand::to_sps(
                    &operation,
                    &target_var,
                    &primary_var,
                    secondary_var.as_deref(),
                    part.as_deref(),
                    unit.as_deref(),
                    duration_value,
                    target_label.as_deref(),
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            DateTimeDialogAction::None => {}
        }

        // 8. Optimal Binning Dialog
        let optbin_action = render_optimal_binning_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.optimal_binning_dialog_state,
        );

        match optbin_action {
            OptimalBinningDialogAction::Run {
                variables,
                target_var,
                max_bins,
                prefix,
            } => {
                let vars_ref: Vec<&str> = variables.iter().map(|s| s.as_str()).collect();
                let sps = statcraft_syntax::commands::OptimalBinningCommand::to_sps(
                    &vars_ref,
                    &target_var,
                    max_bins,
                );

                let params = json!({
                    "variables": variables,
                    "target_var": target_var,
                    "max_bins": max_bins,
                    "prefix": prefix,
                });

                match self.command_bus.execute(
                    "transform.optimal_binning",
                    &mut self.dataset,
                    params,
                ) {
                    Ok(res) => {
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        let count = res.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        self.status_message = StatusMessage::OptimalBinningExecuted { count };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            OptimalBinningDialogAction::Paste {
                variables,
                target_var,
                max_bins,
                prefix: _,
            } => {
                let vars_ref: Vec<&str> = variables.iter().map(|s| s.as_str()).collect();
                let sps = statcraft_syntax::commands::OptimalBinningCommand::to_sps(
                    &vars_ref,
                    &target_var,
                    max_bins,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            OptimalBinningDialogAction::None => {}
        }
    }
}
