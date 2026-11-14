use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct MultivariateGlmDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_vars: Vec<String>,
    pub target_dep_selected: Option<usize>,
    pub factors: Vec<String>,
    pub target_factor_selected: Option<usize>,
    pub covariates: Vec<String>,
    pub target_cov_selected: Option<usize>,
    pub intercept: bool,
}

impl Default for MultivariateGlmDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            dep_vars: Vec::new(),
            target_dep_selected: None,
            factors: Vec::new(),
            target_factor_selected: None,
            covariates: Vec::new(),
            target_cov_selected: None,
            intercept: true,
        }
    }
}

pub enum MultivariateGlmDialogAction {
    Run {
        dependents: Vec<String>,
        factors: Vec<String>,
        covariates: Vec<String>,
        intercept: bool,
    },
    Paste {
        dependents: Vec<String>,
        factors: Vec<String>,
        covariates: Vec<String>,
        intercept: bool,
    },
    None,
}

pub fn render_multivariate_glm_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut MultivariateGlmDialogState,
) -> MultivariateGlmDialogAction {
    if !state.is_open {
        return MultivariateGlmDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return MultivariateGlmDialogAction::None;
        }
    };

    let mut action = MultivariateGlmDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let in_dep = state.dep_vars.contains(&meta.name);
                let in_factors = state.factors.contains(&meta.name);
                let in_covs = state.covariates.contains(&meta.name);
                if !in_dep && !in_factors && !in_covs {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-multivariate-glm-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Source variables panel
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
                vec2(190.0, 260.0),
                "manova_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_dep_selected = None;
                    state.target_factor_selected = None;
                    state.target_cov_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        state.dep_vars.push(name.clone());
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Transfer Buttons and Targets
            let can_move_right = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();

            let can_move_left_dep = state.target_dep_selected.is_some()
                && state.target_dep_selected.unwrap() < state.dep_vars.len();
            let can_move_left_factor = state.target_factor_selected.is_some()
                && state.target_factor_selected.unwrap() < state.factors.len();
            let can_move_left_cov = state.target_cov_selected.is_some()
                && state.target_cov_selected.unwrap() < state.covariates.len();

            ui.vertical(|ui| {
                // Dependent Variables Row
                ui.horizontal(|ui| {
                    let dep_btn = if state.target_dep_selected.is_some() {
                        TransferButton::to_source(can_move_left_dep)
                    } else {
                        TransferButton::to_target(can_move_right)
                    };

                    if dep_btn.show(ui).clicked() {
                        if let Some(t_idx) = state.target_dep_selected {
                            if t_idx < state.dep_vars.len() {
                                state.dep_vars.remove(t_idx);
                                state.target_dep_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.dep_vars.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    let dep_items: Vec<VariableListItem> = state
                        .dep_vars
                        .iter()
                        .enumerate()
                        .map(|(idx, name)| {
                            let label = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|v| v.label.clone())
                                .unwrap_or_default();
                            let measure = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|v| v.measure)
                                .unwrap_or(statcraft_core::Measure::Scale);
                            VariableListItem {
                                name,
                                display_text: format_var_display(name, &label),
                                measure,
                                is_selected: state.target_dep_selected == Some(idx),
                            }
                        })
                        .collect();

                    let dep_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-glm-dependent-vars"),
                        vec2(180.0, 80.0),
                        "manova_dep_target",
                        &dep_items,
                    );
                    if let ListEvent::Clicked(idx) = dep_ev {
                        state.target_dep_selected = Some(idx);
                        state.source_selected = None;
                        state.target_factor_selected = None;
                        state.target_cov_selected = None;
                    }
                });

                ui.add_space(4.0);

                // Factors Row
                ui.horizontal(|ui| {
                    let factor_btn = if state.target_factor_selected.is_some() {
                        TransferButton::to_source(can_move_left_factor)
                    } else {
                        TransferButton::to_target(can_move_right)
                    };

                    if factor_btn.show(ui).clicked() {
                        if let Some(t_idx) = state.target_factor_selected {
                            if t_idx < state.factors.len() {
                                state.factors.remove(t_idx);
                                state.target_factor_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.factors.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    let factor_items: Vec<VariableListItem> = state
                        .factors
                        .iter()
                        .enumerate()
                        .map(|(idx, name)| {
                            let label = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|v| v.label.clone())
                                .unwrap_or_default();
                            let measure = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|v| v.measure)
                                .unwrap_or(statcraft_core::Measure::Nominal);
                            VariableListItem {
                                name,
                                display_text: format_var_display(name, &label),
                                measure,
                                is_selected: state.target_factor_selected == Some(idx),
                            }
                        })
                        .collect();

                    let factor_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-glm-fixed-factors"),
                        vec2(180.0, 80.0),
                        "manova_factors_target",
                        &factor_items,
                    );
                    if let ListEvent::Clicked(idx) = factor_ev {
                        state.target_factor_selected = Some(idx);
                        state.source_selected = None;
                        state.target_dep_selected = None;
                        state.target_cov_selected = None;
                    }
                });

                ui.add_space(4.0);

                // Covariates Row
                ui.horizontal(|ui| {
                    let cov_btn = if state.target_cov_selected.is_some() {
                        TransferButton::to_source(can_move_left_cov)
                    } else {
                        TransferButton::to_target(can_move_right)
                    };

                    if cov_btn.show(ui).clicked() {
                        if let Some(t_idx) = state.target_cov_selected {
                            if t_idx < state.covariates.len() {
                                state.covariates.remove(t_idx);
                                state.target_cov_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.covariates.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    let cov_items: Vec<VariableListItem> = state
                        .covariates
                        .iter()
                        .enumerate()
                        .map(|(idx, name)| {
                            let label = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|v| v.label.clone())
                                .unwrap_or_default();
                            let measure = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|v| v.measure)
                                .unwrap_or(statcraft_core::Measure::Scale);
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
                        t!("dialog-glm-covariates"),
                        vec2(180.0, 80.0),
                        "manova_cov_target",
                        &cov_items,
                    );
                    if let ListEvent::Clicked(idx) = cov_ev {
                        state.target_cov_selected = Some(idx);
                        state.source_selected = None;
                        state.target_dep_selected = None;
                        state.target_factor_selected = None;
                    }
                });
            });

            // Options GroupBox on the right
            ui.vertical(|ui| {
                GroupBox::new(t!("dialog-options-group")).show(ui, |ui| {
                    ui.checkbox(&mut state.intercept, t!("dialog-glm-intercept"));
                });
            });
        });

        ui.add_space(8.0);

        // Action Bar
        let is_ok_enabled = state.dep_vars.len() >= 2
            && (!state.factors.is_empty() || !state.covariates.is_empty());
        let bar_action = DialogActionBar::new(is_ok_enabled).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                action = MultivariateGlmDialogAction::Run {
                    dependents: state.dep_vars.clone(),
                    factors: state.factors.clone(),
                    covariates: state.covariates.clone(),
                    intercept: state.intercept,
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = MultivariateGlmDialogAction::Paste {
                    dependents: state.dep_vars.clone(),
                    factors: state.factors.clone(),
                    covariates: state.covariates.clone(),
                    intercept: state.intercept,
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = MultivariateGlmDialogState {
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
