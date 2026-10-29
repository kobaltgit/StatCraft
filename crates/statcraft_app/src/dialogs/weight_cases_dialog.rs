use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone, Default)]
pub struct WeightCasesDialogState {
    pub is_open: bool,
    pub weight_active: bool,
    pub source_selected: Option<usize>,
    pub selected_var: Option<String>,
    pub target_selected: bool,
}

pub enum WeightCasesDialogAction {
    Run { weight_var: Option<String> },
    Paste { weight_var: Option<String> },
    None,
}

pub fn render_weight_cases_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut WeightCasesDialogState,
) -> WeightCasesDialogAction {
    if !state.is_open {
        return WeightCasesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return WeightCasesDialogAction::None;
        }
    };

    let mut action = WeightCasesDialogAction::None;
    let mut close_dialog = false;

    // Available numeric variables: columns not selected as weight_var and numeric type
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if !meta.var_type.is_numeric() {
                    return None;
                }
                if state.selected_var.as_deref() != Some(&meta.name) {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-weight-cases-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left Panel: Available Numeric Variables (200x260)
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
                vec2(200.0, 260.0),
                "weight_cases_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_selected = false;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_col, name, _label, _m)) = available_vars.get(idx) {
                        state.selected_var = Some(name.clone());
                        state.weight_active = true;
                        state.source_selected = None;
                        state.target_selected = false;
                    }
                }
                ListEvent::None => {}
            }

            // Middle: Transfer Button
            let can_move_right = state.weight_active
                && state.source_selected.is_some()
                && state.source_selected.unwrap_or(usize::MAX) < available_vars.len();
            let can_move_left = state.target_selected && state.selected_var.is_some();

            let transfer_btn = if state.target_selected {
                TransferButton::to_source(can_move_left)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(110.0);
                    if transfer_btn.show(ui).clicked() {
                        if state.target_selected {
                            state.selected_var = None;
                            state.target_selected = false;
                        } else if let Some(src_idx) = state.source_selected {
                            if let Some((_col, name, _label, _m)) = available_vars.get(src_idx) {
                                state.selected_var = Some(name.clone());
                                state.weight_active = true;
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Right Panel: Options & Frequency Variable Target Box
            ui.vertical(|ui| {
                ui.radio_value(
                    &mut state.weight_active,
                    false,
                    t!("dialog-weight-do-not-weight"),
                );
                ui.add_space(4.0);

                ui.radio_value(&mut state.weight_active, true, t!("dialog-weight-cases-by"));
                ui.add_space(4.0);

                // Target box for frequency variable
                let target_meta = state.selected_var.as_deref().and_then(|name| {
                    let col_idx = ds.dictionary.find_index(name)?;
                    let meta = ds.dictionary.get(col_idx)?;
                    Some((name.to_string(), meta.label.clone(), meta.measure))
                });

                let target_items: Vec<VariableListItem> = match &target_meta {
                    Some((name, label, measure)) => vec![VariableListItem {
                        name,
                        display_text: format_var_display(name, label),
                        measure: *measure,
                        is_selected: state.target_selected,
                    }],
                    None => Vec::new(),
                };

                ui.add_enabled_ui(state.weight_active, |ui| {
                    let target_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-weight-frequency-var"),
                        vec2(220.0, 110.0),
                        "weight_cases_target_var",
                        &target_items,
                    );

                    match target_ev {
                        ListEvent::Clicked(_) => {
                            state.target_selected = true;
                            state.source_selected = None;
                        }
                        ListEvent::DoubleClicked(_) => {
                            state.selected_var = None;
                            state.target_selected = false;
                        }
                        ListEvent::None => {}
                    }
                });

                ui.add_space(10.0);

                // Current Status GroupBox
                GroupBox::new(t!("dialog-weight-current-status")).show(ui, |ui| {
                    ui.set_width(220.0);
                    if let Some(curr_var) = ds.weight_var() {
                        ui.label(t!("dialog-weight-status-on", var = curr_var));
                    } else {
                        ui.label(t!("dialog-weight-status-off"));
                    }
                });
            });
        });

        ui.add_space(10.0);

        // Action Bar Type 1
        let can_run = if state.weight_active {
            state.selected_var.is_some()
        } else {
            true
        };

        let act = DialogActionBar::new(can_run).show(ui);
        match act {
            ActionBarAction::Ok => {
                let var = if state.weight_active {
                    state.selected_var.clone()
                } else {
                    None
                };
                action = WeightCasesDialogAction::Run { weight_var: var };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                let var = if state.weight_active {
                    state.selected_var.clone()
                } else {
                    None
                };
                action = WeightCasesDialogAction::Paste { weight_var: var };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                state.weight_active = false;
                state.selected_var = None;
                state.source_selected = None;
                state.target_selected = false;
            }
            ActionBarAction::Cancel => {
                close_dialog = true;
            }
            ActionBarAction::Help => {}
            ActionBarAction::None => {}
        }
    });

    if close_dialog || !main_open {
        state.is_open = false;
    }

    action
}
