use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    ListEvent, TransferButton, VariableListItem,
};
use crate::theme::Tokens;
use crate::widgets::paint_measure_icon;
use egui::{vec2, Align, Layout, Sense, Stroke};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone, Default)]
pub struct TransposeDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub name_var_selected: bool,
    pub selected_vars: Vec<String>,
    pub name_var: Option<String>,
}

pub enum TransposeDialogAction {
    Run {
        variables: Option<Vec<String>>,
        name_var: Option<String>,
    },
    Paste {
        variables: Option<Vec<String>>,
        name_var: Option<String>,
    },
    None,
}

pub fn render_transpose_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut TransposeDialogState,
) -> TransposeDialogAction {
    if !state.is_open {
        return TransposeDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return TransposeDialogAction::None;
        }
    };

    let mut action = TransposeDialogAction::None;
    let mut close_dialog = false;

    // Available variables: columns not in selected_vars and not name_var
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let in_selected = state.selected_vars.contains(&meta.name);
                let is_name_var = state
                    .name_var
                    .as_ref()
                    .map(|n| n == &meta.name)
                    .unwrap_or(false);
                if !in_selected && !is_name_var {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-transpose-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left Panel: Available Variables (190x260)
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
                "transpose_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_selected = None;
                    state.name_var_selected = false;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        state.selected_vars.push(name.clone());
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Middle Column: Transfer Buttons (36x260)
            let can_move_vars = if let Some(idx) = state.target_selected {
                idx < state.selected_vars.len()
            } else if let Some(idx) = state.source_selected {
                idx < available_vars.len()
            } else {
                false
            };

            let can_move_name = if state.name_var_selected {
                state.name_var.is_some()
            } else if let Some(idx) = state.source_selected {
                idx < available_vars.len()
            } else {
                false
            };

            let transfer_vars_btn = if state.target_selected.is_some() {
                TransferButton::to_source(can_move_vars)
            } else {
                TransferButton::to_target(can_move_vars)
            };

            let transfer_name_btn = if state.name_var_selected {
                TransferButton::to_source(can_move_name)
            } else {
                TransferButton::to_target(can_move_name)
            };

            ui.allocate_ui_with_layout(vec2(36.0, 260.0), Layout::top_down(Align::Center), |ui| {
                ui.add_space(50.0);
                if transfer_vars_btn.show(ui).clicked() {
                    if let Some(idx) = state.target_selected {
                        if idx < state.selected_vars.len() {
                            state.selected_vars.remove(idx);
                            state.target_selected = None;
                        }
                    } else if let Some(idx) = state.source_selected {
                        if let Some((_, name, _, _)) = available_vars.get(idx) {
                            state.selected_vars.push(name.clone());
                            state.source_selected = None;
                        }
                    }
                }

                ui.add_space(80.0);
                if transfer_name_btn.show(ui).clicked() {
                    if state.name_var_selected {
                        state.name_var = None;
                        state.name_var_selected = false;
                    } else if let Some(idx) = state.source_selected {
                        if let Some((_, name, _, _)) = available_vars.get(idx) {
                            state.name_var = Some(name.clone());
                            state.source_selected = None;
                        }
                    }
                }
            });

            // Right Panel: Target Variables + Name Variable (280x260)
            ui.allocate_ui_with_layout(vec2(280.0, 260.0), Layout::top_down(Align::Min), |ui| {
                // 1. Variable(s) List Panel (280x150)
                let target_items: Vec<VariableListItem> = state
                    .selected_vars
                    .iter()
                    .enumerate()
                    .map(|(idx, name)| {
                        let meta = ds.dictionary.find_by_name(name);
                        let measure = meta
                            .map(|m| m.measure)
                            .unwrap_or(statcraft_core::Measure::Scale);
                        let label = meta.map(|m| m.label.as_str()).unwrap_or("");
                        VariableListItem {
                            name,
                            display_text: format_var_display(name, label),
                            measure,
                            is_selected: state.target_selected == Some(idx),
                        }
                    })
                    .collect();

                let target_ev = render_variable_list_panel(
                    ui,
                    t!("dialog-transpose-variables"),
                    vec2(280.0, 150.0),
                    "transpose_target_vars",
                    &target_items,
                );

                match target_ev {
                    ListEvent::Clicked(idx) => {
                        state.target_selected = Some(idx);
                        state.source_selected = None;
                        state.name_var_selected = false;
                    }
                    ListEvent::DoubleClicked(idx) => {
                        if idx < state.selected_vars.len() {
                            state.selected_vars.remove(idx);
                            state.target_selected = None;
                        }
                    }
                    ListEvent::None => {}
                }

                ui.add_space(10.0);

                // 2. Name Variable Box
                ui.label(t!("dialog-transpose-name-var"));
                egui::Frame::canvas(ui.style())
                    .fill(Tokens::bg_panel())
                    .stroke(Stroke::new(1.0_f32, Tokens::border()))
                    .show(ui, |ui| {
                        let box_size = vec2(276.0, 40.0);
                        ui.set_min_size(box_size);
                        ui.set_max_size(box_size);

                        ui.horizontal_centered(|ui| {
                            ui.add_space(4.0);
                            if let Some(ref nv) = state.name_var {
                                let meta = ds.dictionary.find_by_name(nv);
                                let measure = meta
                                    .map(|m| m.measure)
                                    .unwrap_or(statcraft_core::Measure::Scale);
                                let label = meta.map(|m| m.label.as_str()).unwrap_or("");
                                let disp = format_var_display(nv, label);

                                let icon_size = vec2(14.0, 14.0);
                                let (rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
                                paint_measure_icon(ui.painter(), rect, measure);

                                let res = ui.selectable_label(state.name_var_selected, disp);
                                if res.clicked() {
                                    state.name_var_selected = !state.name_var_selected;
                                    state.source_selected = None;
                                    state.target_selected = None;
                                }
                                if res.double_clicked() {
                                    state.name_var = None;
                                    state.name_var_selected = false;
                                }
                            } else {
                                ui.colored_label(Tokens::text_muted(), "-");
                            }
                        });
                    });
            });
        });

        // Bottom Action Bar
        let can_run = ds.col_count() > 0 && ds.row_count() > 0;
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                let vars = if state.selected_vars.is_empty() {
                    None
                } else {
                    Some(state.selected_vars.clone())
                };
                action = TransposeDialogAction::Run {
                    variables: vars,
                    name_var: state.name_var.clone(),
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                let vars = if state.selected_vars.is_empty() {
                    None
                } else {
                    Some(state.selected_vars.clone())
                };
                action = TransposeDialogAction::Paste {
                    variables: vars,
                    name_var: state.name_var.clone(),
                };
            }
            ActionBarAction::Reset => {
                state.selected_vars.clear();
                state.name_var = None;
                state.source_selected = None;
                state.target_selected = None;
                state.name_var_selected = false;
            }
            ActionBarAction::Cancel => {
                close_dialog = true;
            }
            ActionBarAction::Help | ActionBarAction::None => {}
        }
    });

    if !main_open || close_dialog {
        state.is_open = false;
    }

    action
}
