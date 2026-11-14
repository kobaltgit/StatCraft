use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_algo::CurveModelType;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct CurveFitDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_var: Option<String>,
    pub indep_var: Option<String>,
    pub target_dep_selected: bool,
    pub target_indep_selected: bool,
    pub linear: bool,
    pub logarithmic: bool,
    pub inverse: bool,
    pub quadratic: bool,
    pub power: bool,
    pub exponential: bool,
    pub include_constant: bool,
}

impl Default for CurveFitDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            dep_var: None,
            indep_var: None,
            target_dep_selected: false,
            target_indep_selected: false,
            linear: true,
            logarithmic: false,
            inverse: false,
            quadratic: false,
            power: false,
            exponential: false,
            include_constant: true,
        }
    }
}

impl CurveFitDialogState {
    pub fn selected_models(&self) -> Vec<CurveModelType> {
        let mut models = Vec::new();
        if self.linear {
            models.push(CurveModelType::Linear);
        }
        if self.logarithmic {
            models.push(CurveModelType::Logarithmic);
        }
        if self.inverse {
            models.push(CurveModelType::Inverse);
        }
        if self.quadratic {
            models.push(CurveModelType::Quadratic);
        }
        if self.power {
            models.push(CurveModelType::Power);
        }
        if self.exponential {
            models.push(CurveModelType::Exponential);
        }
        models
    }
}

pub enum CurveFitDialogAction {
    Run {
        dep_var: String,
        indep_var: String,
        models: Vec<CurveModelType>,
        include_constant: bool,
    },
    Paste {
        dep_var: String,
        indep_var: String,
        models: Vec<CurveModelType>,
        include_constant: bool,
    },
    None,
}

pub fn render_curvefit_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut CurveFitDialogState,
) -> CurveFitDialogAction {
    if !state.is_open {
        return CurveFitDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return CurveFitDialogAction::None;
        }
    };

    let mut action = CurveFitDialogAction::None;
    let mut close_dialog = false;

    // Filter available variables
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_dep = state.dep_var.as_ref() == Some(&meta.name);
                let is_indep = state.indep_var.as_ref() == Some(&meta.name);
                if !is_dep && !is_indep {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-curvefit-title")).show(ctx, &mut main_open, |ui| {
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
                "curvefit_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_dep_selected = false;
                    state.target_indep_selected = false;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        if state.dep_var.is_none() {
                            state.dep_var = Some(name.clone());
                        } else if state.indep_var.is_none() {
                            state.indep_var = Some(name.clone());
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
            let can_move_left_indep = state.target_indep_selected && state.indep_var.is_some();

            let transfer_dep_btn = if state.target_dep_selected {
                TransferButton::to_source(can_move_left_dep)
            } else {
                TransferButton::to_target(can_move_right_source)
            };

            let transfer_indep_btn = if state.target_indep_selected {
                TransferButton::to_source(can_move_left_indep)
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
                    if transfer_indep_btn.show(ui).clicked() {
                        if state.target_indep_selected {
                            state.indep_var = None;
                            state.target_indep_selected = false;
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.indep_var = Some(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Right Column: Target Slots + Models Box
            ui.allocate_ui_with_layout(
                vec2(210.0, 240.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // 1. Dependent Variable Slot
                    ui.label(t!("dialog-curvefit-dependent"));
                    let dep_text = if let Some(ref name) = state.dep_var {
                        let label = ds
                            .dictionary
                            .find_by_name(name)
                            .map(|m| m.label.clone())
                            .unwrap_or_default();
                        format_var_display(name, &label)
                    } else {
                        format!("({})", t!("dialog-curvefit-placeholder-dep"))
                    };

                    let dep_clicked = ui
                        .selectable_label(state.target_dep_selected, dep_text)
                        .clicked();
                    if dep_clicked {
                        state.target_dep_selected = !state.target_dep_selected;
                        state.target_indep_selected = false;
                        state.source_selected = None;
                    }

                    ui.add_space(8.0);

                    // 2. Independent Variable Slot
                    ui.label(t!("dialog-curvefit-independent"));
                    let indep_text = if let Some(ref name) = state.indep_var {
                        let label = ds
                            .dictionary
                            .find_by_name(name)
                            .map(|m| m.label.clone())
                            .unwrap_or_default();
                        format_var_display(name, &label)
                    } else {
                        format!("({})", t!("dialog-curvefit-placeholder-indep"))
                    };

                    let indep_clicked = ui
                        .selectable_label(state.target_indep_selected, indep_text)
                        .clicked();
                    if indep_clicked {
                        state.target_indep_selected = !state.target_indep_selected;
                        state.target_dep_selected = false;
                        state.source_selected = None;
                    }

                    ui.add_space(12.0);

                    // 3. Models Checkboxes
                    GroupBox::new(t!("dialog-curvefit-models")).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.checkbox(&mut state.linear, t!("dialog-curvefit-linear"));
                                ui.checkbox(
                                    &mut state.logarithmic,
                                    t!("dialog-curvefit-logarithmic"),
                                );
                                ui.checkbox(&mut state.inverse, t!("dialog-curvefit-inverse"));
                            });
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.checkbox(&mut state.quadratic, t!("dialog-curvefit-quadratic"));
                                ui.checkbox(&mut state.power, t!("dialog-curvefit-power"));
                                ui.checkbox(
                                    &mut state.exponential,
                                    t!("dialog-curvefit-exponential"),
                                );
                            });
                        });
                    });
                },
            );
        });

        ui.add_space(8.0);

        ui.checkbox(
            &mut state.include_constant,
            t!("dialog-curvefit-include-constant"),
        );

        ui.add_space(10.0);

        // Validation & Action Bar
        let models = state.selected_models();
        let is_valid = state.dep_var.is_some() && state.indep_var.is_some() && !models.is_empty();

        let bar_action = DialogActionBar::new(is_valid).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                if let (Some(dep), Some(indep)) = (&state.dep_var, &state.indep_var) {
                    action = CurveFitDialogAction::Run {
                        dep_var: dep.clone(),
                        indep_var: indep.clone(),
                        models,
                        include_constant: state.include_constant,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Paste => {
                if let (Some(dep), Some(indep)) = (&state.dep_var, &state.indep_var) {
                    action = CurveFitDialogAction::Paste {
                        dep_var: dep.clone(),
                        indep_var: indep.clone(),
                        models,
                        include_constant: state.include_constant,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Reset => {
                *state = CurveFitDialogState {
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
