use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_algo::ShiftDirection;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct ShiftValuesDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub source_var: Option<String>,
    pub target_selected: bool,
    pub target_var: String,
    pub target_label: String,
    pub direction: ShiftDirection,
    pub step: usize,
}

impl Default for ShiftValuesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            source_var: None,
            target_selected: false,
            target_var: String::new(),
            target_label: String::new(),
            direction: ShiftDirection::Lag,
            step: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShiftValuesDialogAction {
    Run {
        source_var: String,
        target_var: String,
        direction: ShiftDirection,
        step: usize,
        target_label: Option<String>,
    },
    Paste {
        source_var: String,
        target_var: String,
        direction: ShiftDirection,
        step: usize,
        target_label: Option<String>,
    },
    None,
}

pub fn render_shift_values_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut ShiftValuesDialogState,
) -> ShiftValuesDialogAction {
    if !state.is_open {
        return ShiftValuesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return ShiftValuesDialogAction::None;
        }
    };

    let mut action = ShiftValuesDialogAction::None;
    let mut close_dialog = false;

    // Available variables: columns not currently selected as source_var
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if state.source_var.as_deref() != Some(&meta.name) {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-shift-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left Panel: Available Variables (200x290)
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
                vec2(200.0, 290.0),
                "shift_values_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_selected = false;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_col, name, _label, _m)) = available_vars.get(idx) {
                        state.source_var = Some(name.clone());
                        if state.target_var.is_empty() {
                            state.target_var = format!("{}_lag1", name);
                        }
                        state.source_selected = None;
                        state.target_selected = false;
                    }
                }
                ListEvent::None => {}
            }

            // Middle: Transfer Button
            let can_move_right = state.source_selected.is_some()
                && state.source_selected.unwrap_or(usize::MAX) < available_vars.len();
            let can_move_left = state.target_selected && state.source_var.is_some();

            let transfer_btn = if state.target_selected {
                TransferButton::to_source(can_move_left)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 290.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(35.0);
                    if transfer_btn.show(ui).clicked() {
                        if state.target_selected {
                            state.source_var = None;
                            state.target_selected = false;
                        } else if let Some(src_idx) = state.source_selected {
                            if let Some((_col, name, _label, _m)) = available_vars.get(src_idx) {
                                state.source_var = Some(name.clone());
                                if state.target_var.is_empty() {
                                    state.target_var = format!("{}_lag1", name);
                                }
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Right Panel: Selected Source, Target Details, Direction & Steps
            ui.vertical(|ui| {
                ui.set_width(280.0);

                // Source variable target box
                let target_meta = state.source_var.as_deref().and_then(|name| {
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

                let target_ev = render_variable_list_panel(
                    ui,
                    t!("dialog-shift-source-var"),
                    vec2(280.0, 68.0),
                    "shift_values_target_var_box",
                    &target_items,
                );

                match target_ev {
                    ListEvent::Clicked(_) => {
                        state.target_selected = true;
                        state.source_selected = None;
                    }
                    ListEvent::DoubleClicked(_) => {
                        state.source_var = None;
                        state.target_selected = false;
                    }
                    ListEvent::None => {}
                }

                ui.add_space(6.0);

                // Target variable name & label
                ui.label(t!("dialog-shift-target-var"));
                ui.add(egui::TextEdit::singleline(&mut state.target_var).desired_width(280.0));

                ui.add_space(4.0);
                ui.label(t!("dialog-shift-target-label"));
                ui.add(egui::TextEdit::singleline(&mut state.target_label).desired_width(280.0));

                ui.add_space(8.0);

                // Direction GroupBox
                GroupBox::new(t!("dialog-shift-direction-group")).show(ui, |ui| {
                    ui.radio_value(
                        &mut state.direction,
                        ShiftDirection::Lag,
                        t!("dialog-shift-direction-lag"),
                    );
                    ui.add_space(2.0);
                    ui.radio_value(
                        &mut state.direction,
                        ShiftDirection::Lead,
                        t!("dialog-shift-direction-lead"),
                    );
                });

                ui.add_space(6.0);

                // Step count
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-shift-step"));
                    ui.add(
                        egui::DragValue::new(&mut state.step)
                            .range(1..=100_000)
                            .speed(1.0),
                    );
                });
            });
        });

        ui.add_space(10.0);

        // Action Bar Type 1
        let is_target_name_valid = !state.target_var.trim().is_empty();
        let can_run = state.source_var.is_some() && is_target_name_valid && state.step >= 1;

        let act = DialogActionBar::new(can_run).show(ui);
        match act {
            ActionBarAction::Ok => {
                if let Some(ref src) = state.source_var {
                    let target_lbl = if state.target_label.trim().is_empty() {
                        None
                    } else {
                        Some(state.target_label.trim().to_string())
                    };
                    action = ShiftValuesDialogAction::Run {
                        source_var: src.clone(),
                        target_var: state.target_var.trim().to_string(),
                        direction: state.direction,
                        step: state.step,
                        target_label: target_lbl,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Paste => {
                if let Some(ref src) = state.source_var {
                    let target_lbl = if state.target_label.trim().is_empty() {
                        None
                    } else {
                        Some(state.target_label.trim().to_string())
                    };
                    action = ShiftValuesDialogAction::Paste {
                        source_var: src.clone(),
                        target_var: state.target_var.trim().to_string(),
                        direction: state.direction,
                        step: state.step,
                        target_label: target_lbl,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Reset => {
                state.source_selected = None;
                state.source_var = None;
                state.target_selected = false;
                state.target_var.clear();
                state.target_label.clear();
                state.direction = ShiftDirection::Lag;
                state.step = 1;
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
