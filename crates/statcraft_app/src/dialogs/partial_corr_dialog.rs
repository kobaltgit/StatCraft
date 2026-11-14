use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct PartialCorrDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_var_selected: Option<usize>,
    pub target_control_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub selected_controls: Vec<String>,
    pub two_tailed: bool,
    pub show_zero_order: bool,
}

impl Default for PartialCorrDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_var_selected: None,
            target_control_selected: None,
            selected_vars: Vec::new(),
            selected_controls: Vec::new(),
            two_tailed: true,
            show_zero_order: false,
        }
    }
}

pub enum PartialCorrDialogAction {
    Run {
        vars: Vec<String>,
        controls: Vec<String>,
        two_tailed: bool,
        show_zero_order: bool,
    },
    Paste {
        vars: Vec<String>,
        controls: Vec<String>,
        two_tailed: bool,
        show_zero_order: bool,
    },
    None,
}

pub fn render_partial_corr_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut PartialCorrDialogState,
) -> PartialCorrDialogAction {
    if !state.is_open {
        return PartialCorrDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return PartialCorrDialogAction::None;
        }
    };

    let mut action = PartialCorrDialogAction::None;
    let mut close_dialog = false;

    // Filter available variables
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let in_vars = state.selected_vars.contains(&meta.name);
                let in_ctrls = state.selected_controls.contains(&meta.name);
                if !in_vars && !in_ctrls {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-partial-corr-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Source variables panel (190x240)
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
                "partial_corr_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_var_selected = None;
                    state.target_control_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        state.selected_vars.push(name.clone());
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Transfer buttons column
            let can_move_right_var = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();
            let can_move_left_var = state.target_var_selected.is_some()
                && state.target_var_selected.unwrap() < state.selected_vars.len();

            let can_move_right_ctrl = can_move_right_var;
            let can_move_left_ctrl = state.target_control_selected.is_some()
                && state.target_control_selected.unwrap() < state.selected_controls.len();

            let transfer_var_btn = if state.target_var_selected.is_some() {
                TransferButton::to_source(can_move_left_var)
            } else {
                TransferButton::to_target(can_move_right_var)
            };

            let transfer_ctrl_btn = if state.target_control_selected.is_some() {
                TransferButton::to_source(can_move_left_ctrl)
            } else {
                TransferButton::to_target(can_move_right_ctrl)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 240.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(40.0);
                    if transfer_var_btn.show(ui).clicked() {
                        if let Some(idx) = state.target_var_selected {
                            if idx < state.selected_vars.len() {
                                state.selected_vars.remove(idx);
                                state.target_var_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.selected_vars.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    ui.add_space(80.0);
                    if transfer_ctrl_btn.show(ui).clicked() {
                        if let Some(idx) = state.target_control_selected {
                            if idx < state.selected_controls.len() {
                                state.selected_controls.remove(idx);
                                state.target_control_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.selected_controls.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Target panels column: Variables list + Controls list
            ui.allocate_ui_with_layout(
                vec2(210.0, 240.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // 1. Variables list
                    let var_items: Vec<VariableListItem> = state
                        .selected_vars
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
                                is_selected: state.target_var_selected == Some(idx),
                            }
                        })
                        .collect();

                    let var_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-partial-corr-variables"),
                        vec2(210.0, 110.0),
                        "partial_corr_target_vars",
                        &var_items,
                    );

                    match var_ev {
                        ListEvent::Clicked(idx) => {
                            state.target_var_selected = Some(idx);
                            state.target_control_selected = None;
                            state.source_selected = None;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.selected_vars.len() {
                                state.selected_vars.remove(idx);
                                state.target_var_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }

                    ui.add_space(8.0);

                    // 2. Controlling for list
                    let ctrl_items: Vec<VariableListItem> = state
                        .selected_controls
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
                                is_selected: state.target_control_selected == Some(idx),
                            }
                        })
                        .collect();

                    let ctrl_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-partial-corr-controlling"),
                        vec2(210.0, 110.0),
                        "partial_corr_target_ctrls",
                        &ctrl_items,
                    );

                    match ctrl_ev {
                        ListEvent::Clicked(idx) => {
                            state.target_control_selected = Some(idx);
                            state.target_var_selected = None;
                            state.source_selected = None;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.selected_controls.len() {
                                state.selected_controls.remove(idx);
                                state.target_control_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }
                },
            );
        });

        ui.add_space(10.0);

        // Bottom settings: Test of Significance & Zero-Order Checkbox
        ui.horizontal(|ui| {
            GroupBox::new(t!("dialog-partial-corr-test-sig")).show(ui, |ui| {
                ui.radio_value(
                    &mut state.two_tailed,
                    true,
                    t!("dialog-partial-corr-two-tailed"),
                );
                ui.radio_value(
                    &mut state.two_tailed,
                    false,
                    t!("dialog-partial-corr-one-tailed"),
                );
            });

            ui.add_space(20.0);

            ui.vertical(|ui| {
                ui.add_space(12.0);
                ui.checkbox(
                    &mut state.show_zero_order,
                    t!("dialog-partial-corr-zero-order"),
                );
            });
        });

        ui.add_space(10.0);

        // Action Bar
        let is_valid = state.selected_vars.len() >= 2 && !state.selected_controls.is_empty();
        let bar_action = DialogActionBar::new(is_valid).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                action = PartialCorrDialogAction::Run {
                    vars: state.selected_vars.clone(),
                    controls: state.selected_controls.clone(),
                    two_tailed: state.two_tailed,
                    show_zero_order: state.show_zero_order,
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = PartialCorrDialogAction::Paste {
                    vars: state.selected_vars.clone(),
                    controls: state.selected_controls.clone(),
                    two_tailed: state.two_tailed,
                    show_zero_order: state.show_zero_order,
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = PartialCorrDialogState {
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
