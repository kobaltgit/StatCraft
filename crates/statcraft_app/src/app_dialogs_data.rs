use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_compute_dialog, render_missing_values_dialog, render_recode_dialog,
    render_select_cases_dialog, render_sort_cases_dialog, render_split_file_dialog,
    render_value_labels_dialog, render_variable_type_dialog, ComputeDialogAction,
    DialogCasesAction, DialogSelectMode, MissingValuesDialogAction, RecodeDialogAction,
    SelectCasesDialogAction, SortCasesDialogAction, SplitFileDialogAction, ValueLabelsDialogAction,
    VariableTypeDialogAction,
};
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_data_dialogs(&mut self, ctx: &egui::Context) {
        // 1. Value Labels Dialog
        let val_action = render_value_labels_dialog(ctx, &mut self.value_labels_dialog_state);
        if let ValueLabelsDialogAction::Apply { col_index, mapping } = val_action {
            let params = json!({
                "col_index": col_index,
                "labels": mapping
            });
            match self
                .command_bus
                .execute("var.set_value_labels", &mut self.dataset, params)
            {
                Ok(_) => {
                    self.status_message = StatusMessage::Ready;
                }
                Err(e) => {
                    self.status_message = StatusMessage::Error(e.to_string());
                }
            }
        }

        // 2. Missing Values Dialog
        let miss_action = render_missing_values_dialog(ctx, &mut self.missing_values_dialog_state);
        if let MissingValuesDialogAction::Apply {
            col_index,
            discrete,
            range,
        } = miss_action
        {
            let mut params_map = serde_json::Map::new();
            params_map.insert("col_index".to_string(), json!(col_index));
            params_map.insert("discrete".to_string(), json!(discrete));
            if let Some((low, high)) = range {
                params_map.insert("range".to_string(), json!([low, high]));
            }
            match self.command_bus.execute(
                "var.set_missing_values",
                &mut self.dataset,
                serde_json::Value::Object(params_map),
            ) {
                Ok(_) => {
                    self.status_message = StatusMessage::Ready;
                }
                Err(e) => {
                    self.status_message = StatusMessage::Error(e.to_string());
                }
            }
        }

        // 3. Variable Type Dialog
        let type_action = render_variable_type_dialog(ctx, &mut self.variable_type_dialog_state);
        if let VariableTypeDialogAction::Apply {
            col_index,
            var_type,
            width,
            decimals,
        } = type_action
        {
            if let Some(ds) = &mut self.dataset {
                let _ = ds.set_variable_type(col_index, var_type, width, decimals);
            }
        }

        // 4. Compute Variable Dialog
        let compute_action =
            render_compute_dialog(ctx, self.dataset.as_ref(), &mut self.compute_dialog_state);

        match compute_action {
            ComputeDialogAction::Run {
                target,
                expression,
                label,
            } => {
                let params = json!({
                    "target": target,
                    "expression": expression,
                    "label": label,
                });
                match self
                    .command_bus
                    .execute("transform.compute", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        let target_name = result_value["target"]
                            .as_str()
                            .unwrap_or(&target)
                            .to_string();
                        let rows = result_value["rows"].as_u64().unwrap_or(0) as usize;
                        let sps = statcraft_syntax::commands::ComputeVariableCommand::to_sps(
                            &target,
                            &expression,
                            label.as_deref(),
                        );
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message = StatusMessage::ComputeExecuted {
                            target: target_name,
                            rows,
                        };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            ComputeDialogAction::Paste {
                target,
                expression,
                label,
            } => {
                let sps = statcraft_syntax::commands::ComputeVariableCommand::to_sps(
                    &target,
                    &expression,
                    label.as_deref(),
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            ComputeDialogAction::None => {}
        }

        // 5. Recode Dialog
        let recode_action =
            render_recode_dialog(ctx, self.dataset.as_ref(), &mut self.recode_dialog_state);

        match recode_action {
            RecodeDialogAction::RunDiff { pairs, mappings } => {
                let serialized_pairs: Vec<serde_json::Value> = pairs
                    .iter()
                    .map(|p| {
                        json!({
                            "source": p.source,
                            "target": p.target,
                            "label": p.label,
                        })
                    })
                    .collect();

                let params = json!({
                    "pairs": serialized_pairs,
                    "mappings": mappings,
                });

                match self
                    .command_bus
                    .execute("transform.recode.diff", &mut self.dataset, params)
                {
                    Ok(_) => {
                        let sps_pairs: Vec<(&str, &str, Option<&str>)> = pairs
                            .iter()
                            .map(|p| (p.source.as_str(), p.target.as_str(), Some(p.label.as_str())))
                            .collect();
                        let plan = statcraft_algo::RecodePlan::new(mappings);
                        let sps = statcraft_syntax::commands::RecodeIntoDifferentCommand::to_sps(
                            &sps_pairs, &plan,
                        );
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message = StatusMessage::RecodeExecuted { count: pairs.len() };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            RecodeDialogAction::RunSame { vars, mappings } => {
                let params = json!({
                    "vars": vars,
                    "mappings": mappings,
                });

                match self
                    .command_bus
                    .execute("transform.recode.same", &mut self.dataset, params)
                {
                    Ok(_) => {
                        let var_refs: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
                        let plan = statcraft_algo::RecodePlan::new(mappings);
                        let sps = statcraft_syntax::commands::RecodeIntoSameCommand::to_sps(
                            &var_refs, &plan,
                        );
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message = StatusMessage::RecodeExecuted { count: vars.len() };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            RecodeDialogAction::PasteDiff { pairs, mappings } => {
                let sps_pairs: Vec<(&str, &str, Option<&str>)> = pairs
                    .iter()
                    .map(|p| (p.source.as_str(), p.target.as_str(), Some(p.label.as_str())))
                    .collect();
                let plan = statcraft_algo::RecodePlan::new(mappings);
                let sps = statcraft_syntax::commands::RecodeIntoDifferentCommand::to_sps(
                    &sps_pairs, &plan,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            RecodeDialogAction::PasteSame { vars, mappings } => {
                let var_refs: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
                let plan = statcraft_algo::RecodePlan::new(mappings);
                let sps =
                    statcraft_syntax::commands::RecodeIntoSameCommand::to_sps(&var_refs, &plan);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            RecodeDialogAction::None => {}
        }

        // 6. Sort Cases Dialog
        let sort_action = render_sort_cases_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.sort_cases_dialog_state,
        );

        match sort_action {
            SortCasesDialogAction::Run(keys) => {
                let serialized_keys: Vec<serde_json::Value> = keys
                    .iter()
                    .map(|k| {
                        let dir_str = match k.direction {
                            statcraft_algo::SortDirection::Ascending => "ascending",
                            statcraft_algo::SortDirection::Descending => "descending",
                        };
                        json!({
                            "column_name": k.column_name,
                            "direction": dir_str,
                        })
                    })
                    .collect();

                let params = json!({
                    "keys": serialized_keys,
                });

                match self
                    .command_bus
                    .execute("data.sort_cases", &mut self.dataset, params)
                {
                    Ok(_) => {
                        let sps = statcraft_syntax::commands::SortCasesCommand::to_sps(&keys);
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message =
                            StatusMessage::SortCasesExecuted { count: keys.len() };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            SortCasesDialogAction::Paste(keys) => {
                let sps = statcraft_syntax::commands::SortCasesCommand::to_sps(&keys);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            SortCasesDialogAction::None => {}
        }

        // 7. Select Cases Dialog
        let select_action = render_select_cases_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.select_cases_dialog_state,
        );

        match select_action {
            SelectCasesDialogAction::Run {
                mode,
                action,
                condition,
                filter_var,
            } => {
                let mode_str = match mode {
                    DialogSelectMode::All => "all",
                    DialogSelectMode::Condition => "condition",
                    DialogSelectMode::FilterVar => "variable",
                };
                let action_str = match action {
                    DialogCasesAction::Filtered => "filter",
                    DialogCasesAction::Deleted => "delete",
                };

                let params = json!({
                    "mode": mode_str,
                    "action": action_str,
                    "condition": condition,
                    "filter_var": filter_var.clone().unwrap_or_default(),
                });

                let sps_mode = match mode {
                    DialogSelectMode::All => statcraft_syntax::commands::SelectCasesMode::All,
                    DialogSelectMode::Condition => {
                        statcraft_syntax::commands::SelectCasesMode::Condition(condition)
                    }
                    DialogSelectMode::FilterVar => {
                        statcraft_syntax::commands::SelectCasesMode::FilterVar(
                            filter_var.unwrap_or_default(),
                        )
                    }
                };
                let sps_action = match action {
                    DialogCasesAction::Filtered => {
                        statcraft_syntax::commands::SelectCasesAction::Filter
                    }
                    DialogCasesAction::Deleted => {
                        statcraft_syntax::commands::SelectCasesAction::Delete
                    }
                };

                match self
                    .command_bus
                    .execute("data.select_cases", &mut self.dataset, params)
                {
                    Ok(_) => {
                        let sps = statcraft_syntax::commands::SelectCasesCommand::to_sps(
                            &sps_mode,
                            &sps_action,
                        );
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message = StatusMessage::SelectCasesExecuted {
                            mode: mode_str.to_string(),
                        };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            SelectCasesDialogAction::Paste {
                mode,
                action,
                condition,
                filter_var,
            } => {
                let sps_mode = match mode {
                    DialogSelectMode::All => statcraft_syntax::commands::SelectCasesMode::All,
                    DialogSelectMode::Condition => {
                        statcraft_syntax::commands::SelectCasesMode::Condition(condition)
                    }
                    DialogSelectMode::FilterVar => {
                        statcraft_syntax::commands::SelectCasesMode::FilterVar(
                            filter_var.unwrap_or_default(),
                        )
                    }
                };
                let sps_action = match action {
                    DialogCasesAction::Filtered => {
                        statcraft_syntax::commands::SelectCasesAction::Filter
                    }
                    DialogCasesAction::Deleted => {
                        statcraft_syntax::commands::SelectCasesAction::Delete
                    }
                };
                let sps =
                    statcraft_syntax::commands::SelectCasesCommand::to_sps(&sps_mode, &sps_action);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            SelectCasesDialogAction::None => {}
        }

        // 8. Split File Dialog
        let split_action = render_split_file_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.split_file_dialog_state,
        );

        match split_action {
            SplitFileDialogAction::Run {
                mode,
                variables,
                sort,
            } => {
                let mode_str = match mode {
                    statcraft_core::SplitFileMode::Off => "off",
                    statcraft_core::SplitFileMode::CompareGroups => "compare",
                    statcraft_core::SplitFileMode::OrganizeOutput => "separate",
                };

                let params = json!({
                    "mode": mode_str,
                    "variables": variables,
                    "sort": sort,
                });

                match self
                    .command_bus
                    .execute("data.split_file", &mut self.dataset, params)
                {
                    Ok(_) => {
                        let sps = statcraft_syntax::commands::SplitFileCommand::to_sps(
                            mode, &variables, sort,
                        );
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message = StatusMessage::SplitFileExecuted {
                            mode: mode_str.to_string(),
                            count: variables.len(),
                        };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            SplitFileDialogAction::Paste {
                mode,
                variables,
                sort,
            } => {
                let sps =
                    statcraft_syntax::commands::SplitFileCommand::to_sps(mode, &variables, sort);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            SplitFileDialogAction::None => {}
        }
    }
}
