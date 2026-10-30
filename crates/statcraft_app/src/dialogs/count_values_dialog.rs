use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use super::count_values_define_dialog::{
    render_count_values_define_dialog, CountValuesDefineAction, CountValuesDefineState,
};
use crate::theme::Tokens;
use egui::{vec2, Align, Layout, ScrollArea, Stroke, TextEdit};
use statcraft_algo::ValueCriterion;
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;

#[derive(Debug, Clone, Default)]
pub struct CountValuesDialogState {
    pub is_open: bool,
    pub target_var: String,
    pub target_label: String,
    pub selected_available_idx: Option<usize>,
    pub selected_variable_idx: Option<usize>,
    pub variables: Vec<String>,
    pub criteria: Vec<ValueCriterion>,
    pub define_values_state: CountValuesDefineState,
}

impl CountValuesDialogState {
    pub fn reset(&mut self) {
        self.target_var.clear();
        self.target_label.clear();
        self.selected_available_idx = None;
        self.selected_variable_idx = None;
        self.variables.clear();
        self.criteria.clear();
        self.define_values_state = CountValuesDefineState::default();
    }
}

pub enum CountValuesDialogAction {
    Run {
        target: String,
        label: Option<String>,
        variables: Vec<String>,
        criteria: Vec<ValueCriterion>,
    },
    Paste {
        target: String,
        label: Option<String>,
        variables: Vec<String>,
        criteria: Vec<ValueCriterion>,
    },
    None,
}

pub fn render_count_values_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut CountValuesDialogState,
) -> CountValuesDialogAction {
    // 1. Render subdialog if active
    let def_action = render_count_values_define_dialog(ctx, &mut state.define_values_state);
    if let CountValuesDefineAction::Apply(crit) = def_action {
        state.criteria = crit;
    }

    if !state.is_open {
        return CountValuesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return CountValuesDialogAction::None;
        }
    };

    let mut action = CountValuesDialogAction::None;
    let mut is_open = state.is_open;
    let mut close = false;

    // Available variables that are not yet selected
    let available_vars: Vec<(usize, String, String, Measure)> = (0..ds.col_count())
        .filter_map(|idx| {
            ds.dictionary.get(idx).and_then(|meta| {
                if state.variables.contains(&meta.name) {
                    None
                } else {
                    Some((idx, meta.name.clone(), meta.label.clone(), meta.measure))
                }
            })
        })
        .collect();

    DialogShell::standard(t!("dialog-count-title")).show(ctx, &mut is_open, |ui| {
        // Top section: Target Variable and Target Label
        ui.horizontal(|ui| {
            ui.label(t!("dialog-count-target-var"));
            ui.add(
                TextEdit::singleline(&mut state.target_var)
                    .desired_width(130.0)
                    .hint_text("VAR_NAME"),
            );

            ui.add_space(8.0);

            ui.label(t!("dialog-count-target-label"));
            ui.add(
                TextEdit::singleline(&mut state.target_label)
                    .desired_width(180.0)
                    .hint_text("Description"),
            );
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Middle section: Available | Transfer | Selected | Sidebar
        ui.horizontal(|ui| {
            // Col 1: Available variables list (185 px)
            let items: Vec<VariableListItem> = available_vars
                .iter()
                .enumerate()
                .map(|(i, (_, name, label, measure))| VariableListItem {
                    name,
                    display_text: format_var_display(name, label),
                    measure: *measure,
                    is_selected: state.selected_available_idx == Some(i),
                })
                .collect();

            let source_ev = render_variable_list_panel(
                ui,
                t!("dialog-variables-source"),
                vec2(185.0, 270.0),
                "count_values_available_list",
                &items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.selected_available_idx = Some(idx);
                    state.selected_variable_idx = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        if !state.variables.contains(name) {
                            state.variables.push(name.clone());
                        }
                        state.selected_available_idx = None;
                    }
                }
                ListEvent::None => {}
            }

            // Col 2: Transfer button (36 px)
            let can_move_right = state.selected_available_idx.is_some();
            let can_move_left = state.selected_variable_idx.is_some();

            let transfer_btn = if state.selected_variable_idx.is_some() {
                TransferButton::to_source(can_move_left)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(vec2(36.0, 270.0), Layout::top_down(Align::Center), |ui| {
                ui.add_space(60.0);
                if transfer_btn.show(ui).clicked() {
                    if let Some(idx) = state.selected_variable_idx {
                        if idx < state.variables.len() {
                            state.variables.remove(idx);
                            state.selected_variable_idx = None;
                        }
                    } else if let Some(idx) = state.selected_available_idx {
                        if let Some((_, name, _, _)) = available_vars.get(idx) {
                            if !state.variables.contains(name) {
                                state.variables.push(name.clone());
                            }
                            state.selected_available_idx = None;
                        }
                    }
                }
            });

            // Col 3: Selected variables list (185 px)
            ui.allocate_ui_with_layout(vec2(185.0, 270.0), Layout::top_down(Align::Min), |ui| {
                ui.label(t!("dialog-count-variables"));
                egui::Frame::canvas(ui.style())
                    .fill(Tokens::BG_PANEL)
                    .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
                    .show(ui, |ui| {
                        let list_sz = vec2(181.0, 245.0);
                        ui.set_min_size(list_sz);
                        ui.set_max_size(list_sz);
                        ScrollArea::vertical().show(ui, |ui| {
                            let mut clicked_idx = None;
                            for (idx, name) in state.variables.iter().enumerate() {
                                let is_sel = state.selected_variable_idx == Some(idx);
                                if ui.selectable_label(is_sel, name).clicked() {
                                    clicked_idx = Some(idx);
                                }
                            }
                            if let Some(idx) = clicked_idx {
                                state.selected_variable_idx = Some(idx);
                                state.selected_available_idx = None;
                            }
                        });
                    });
            });

            // Col 4: Right sidebar (110 px) with Define Values button
            ui.allocate_ui_with_layout(vec2(110.0, 270.0), Layout::top_down(Align::Min), |ui| {
                ui.add_space(18.0);
                let btn_text = format!(
                    "{} ({})",
                    t!("dialog-count-define-values"),
                    state.criteria.len()
                );
                if SubdialogSidebar::button(ui, btn_text, true).clicked() {
                    state.define_values_state.open_with(&state.criteria);
                }

                ui.add_space(12.0);
                GroupBox::new(t!("dialog-count-values-to-count")).show(ui, |ui| {
                    ScrollArea::vertical()
                        .max_height(170.0)
                        .min_scrolled_height(100.0)
                        .show(ui, |ui| {
                            if state.criteria.is_empty() {
                                ui.label("-");
                            } else {
                                for c in &state.criteria {
                                    ui.label(c.display_label());
                                }
                            }
                        });
                });
            });
        });

        ui.add_space(8.0);

        // Bottom action bar
        let can_run = !state.target_var.trim().is_empty()
            && !state.variables.is_empty()
            && !state.criteria.is_empty();

        let bar_action = DialogActionBar::new(can_run).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                let lbl = if state.target_label.trim().is_empty() {
                    None
                } else {
                    Some(state.target_label.trim().to_string())
                };
                action = CountValuesDialogAction::Run {
                    target: state.target_var.trim().to_string(),
                    label: lbl,
                    variables: state.variables.clone(),
                    criteria: state.criteria.clone(),
                };
                close = true;
            }
            ActionBarAction::Paste => {
                let lbl = if state.target_label.trim().is_empty() {
                    None
                } else {
                    Some(state.target_label.trim().to_string())
                };
                action = CountValuesDialogAction::Paste {
                    target: state.target_var.trim().to_string(),
                    label: lbl,
                    variables: state.variables.clone(),
                    criteria: state.criteria.clone(),
                };
                close = true;
            }
            ActionBarAction::Reset => {
                state.reset();
            }
            ActionBarAction::Cancel => {
                close = true;
            }
            ActionBarAction::None | ActionBarAction::Help => {}
        }
    });

    state.is_open = is_open && !close;

    action
}
