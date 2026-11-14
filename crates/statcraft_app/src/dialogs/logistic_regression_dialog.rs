use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct LogisticRegressionDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_var: Option<String>,
    pub covariates: Vec<String>,
    pub target_cov_selected: Option<usize>,
    pub target_dep_selected: bool,
    pub hosmer_lemeshow: bool,
    pub cutoff_str: String,
    pub ci_str: String,
}

impl Default for LogisticRegressionDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            dep_var: None,
            covariates: Vec::new(),
            target_cov_selected: None,
            target_dep_selected: false,
            hosmer_lemeshow: true,
            cutoff_str: "0.5".to_string(),
            ci_str: "95".to_string(),
        }
    }
}

pub enum LogisticRegressionDialogAction {
    Run {
        dep_var: String,
        covariates: Vec<String>,
        cutoff: f64,
        ci: f64,
        hosmer_lemeshow: bool,
    },
    Paste {
        dep_var: String,
        covariates: Vec<String>,
        cutoff: f64,
        ci: f64,
        hosmer_lemeshow: bool,
    },
    None,
}

pub fn render_logistic_regression_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut LogisticRegressionDialogState,
) -> LogisticRegressionDialogAction {
    if !state.is_open {
        return LogisticRegressionDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return LogisticRegressionDialogAction::None;
        }
    };

    let mut action = LogisticRegressionDialogAction::None;
    let mut close_dialog = false;

    // Filter available variables
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_dep = state.dep_var.as_ref() == Some(&meta.name);
                let is_cov = state.covariates.contains(&meta.name);
                if !is_dep && !is_cov {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-logistic-regression-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left Panel: Source Variables (190x240)
            let source_items: Vec<VariableListItem> = available_vars
                .iter()
                .enumerate()
                .map(|(idx, (_col, name, label, measure))| VariableListItem {
                    name,
                    display_text: format_var_display(name, label),
                    measure: *measure,
                    is_selected: state.source_selected == Some(idx),
                })
                .collect();

            let source_ev = render_variable_list_panel(
                ui,
                t!("dialog-variables-source"),
                vec2(190.0, 240.0),
                "logistic_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_cov_selected = None;
                    state.target_dep_selected = false;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        if state.dep_var.is_none() {
                            state.dep_var = Some(name.clone());
                        } else {
                            state.covariates.push(name.clone());
                        }
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Transfer Buttons Column
            let can_move_right_source = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();

            let can_move_left_dep = state.target_dep_selected && state.dep_var.is_some();
            let can_move_left_cov = state.target_cov_selected.is_some()
                && state.target_cov_selected.unwrap() < state.covariates.len();

            let transfer_dep_btn = if state.target_dep_selected {
                TransferButton::to_source(can_move_left_dep)
            } else {
                TransferButton::to_target(can_move_right_source)
            };

            let transfer_cov_btn = if state.target_cov_selected.is_some() {
                TransferButton::to_source(can_move_left_cov)
            } else {
                TransferButton::to_target(can_move_right_source)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 240.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(20.0);
                    if transfer_dep_btn.show(ui).clicked() {
                        if state.target_dep_selected {
                            state.dep_var = None;
                            state.target_dep_selected = false;
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.dep_var = Some(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    ui.add_space(60.0);
                    if transfer_cov_btn.show(ui).clicked() {
                        if let Some(idx) = state.target_cov_selected {
                            if idx < state.covariates.len() {
                                state.covariates.remove(idx);
                                state.target_cov_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.covariates.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Right Column: Dependent variable box + Covariates list (210x240)
            ui.allocate_ui_with_layout(
                vec2(210.0, 240.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // 1. Dependent Variable Slot
                    ui.label(t!("dialog-logistic-dependent"));
                    let dep_text = if let Some(ref name) = state.dep_var {
                        let label = ds
                            .dictionary
                            .find_by_name(name)
                            .map(|m| m.label.clone())
                            .unwrap_or_default();
                        format_var_display(name, &label)
                    } else {
                        format!("({})", t!("dialog-logistic-placeholder-dep"))
                    };

                    let dep_rect = ui
                        .selectable_label(state.target_dep_selected, dep_text)
                        .clicked();
                    if dep_rect {
                        state.target_dep_selected = !state.target_dep_selected;
                        state.target_cov_selected = None;
                        state.source_selected = None;
                    }

                    ui.add_space(8.0);

                    // 2. Covariates List
                    let cov_items: Vec<VariableListItem> = state
                        .covariates
                        .iter()
                        .enumerate()
                        .map(|(idx, name)| {
                            let measure = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|m| m.measure)
                                .unwrap_or(statcraft_core::Measure::Scale);
                            let label = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|m| m.label.clone())
                                .unwrap_or_default();
                            VariableListItem {
                                name,
                                display_text: format_var_display(name, &label),
                                measure,
                                is_selected: state.target_cov_selected == Some(idx),
                            }
                        })
                        .collect();

                    let cov_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-logistic-covariates"),
                        vec2(210.0, 160.0),
                        "logistic_covariates_vars",
                        &cov_items,
                    );

                    match cov_ev {
                        ListEvent::Clicked(idx) => {
                            state.target_cov_selected = Some(idx);
                            state.target_dep_selected = false;
                            state.source_selected = None;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.covariates.len() {
                                state.covariates.remove(idx);
                                state.target_cov_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }
                },
            );
        });

        ui.add_space(8.0);

        // Options Section
        GroupBox::new(t!("dialog-logistic-options")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut state.hosmer_lemeshow,
                    t!("dialog-logistic-hosmer-lemeshow"),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(t!("dialog-logistic-cutoff"));
                ui.add(egui::TextEdit::singleline(&mut state.cutoff_str).desired_width(45.0));

                ui.add_space(16.0);
                ui.label(t!("dialog-logistic-ci"));
                ui.add(egui::TextEdit::singleline(&mut state.ci_str).desired_width(40.0));
                ui.label("%");
            });
        });

        ui.add_space(10.0);

        // Validation & Action Bar
        let cutoff_val: f64 = state.cutoff_str.parse().unwrap_or(0.5);
        let ci_val: f64 = state
            .ci_str
            .parse::<f64>()
            .map(|p| p / 100.0)
            .unwrap_or(0.95);
        let is_valid = state.dep_var.is_some()
            && !state.covariates.is_empty()
            && cutoff_val > 0.0
            && cutoff_val < 1.0
            && ci_val > 0.0
            && ci_val < 1.0;

        let bar_action = DialogActionBar::new(is_valid).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                if let Some(dep) = &state.dep_var {
                    action = LogisticRegressionDialogAction::Run {
                        dep_var: dep.clone(),
                        covariates: state.covariates.clone(),
                        cutoff: cutoff_val,
                        ci: ci_val,
                        hosmer_lemeshow: state.hosmer_lemeshow,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Paste => {
                if let Some(dep) = &state.dep_var {
                    action = LogisticRegressionDialogAction::Paste {
                        dep_var: dep.clone(),
                        covariates: state.covariates.clone(),
                        cutoff: cutoff_val,
                        ci: ci_val,
                        hosmer_lemeshow: state.hosmer_lemeshow,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Reset => {
                *state = LogisticRegressionDialogState {
                    is_open: true,
                    ..Default::default()
                };
            }
            ActionBarAction::Cancel => {
                close_dialog = true;
            }
            ActionBarAction::Help | ActionBarAction::None => {}
        }
    });

    if close_dialog || !main_open {
        state.is_open = false;
    }

    action
}
