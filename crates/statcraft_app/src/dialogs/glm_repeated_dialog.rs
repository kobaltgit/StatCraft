use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct RepeatedGlmDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub factor_name: String,
    pub measures: Vec<String>,
    pub target_measure_selected: Option<usize>,
}

impl Default for RepeatedGlmDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            factor_name: "time".to_string(),
            measures: Vec::new(),
            target_measure_selected: None,
        }
    }
}

pub enum RepeatedGlmDialogAction {
    Run {
        factor_name: String,
        measures: Vec<String>,
    },
    Paste {
        factor_name: String,
        measures: Vec<String>,
    },
    None,
}

pub fn render_repeated_glm_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut RepeatedGlmDialogState,
) -> RepeatedGlmDialogAction {
    if !state.is_open {
        return RepeatedGlmDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return RepeatedGlmDialogAction::None;
        }
    };

    let mut action = RepeatedGlmDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if !state.measures.contains(&meta.name) {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-repeated-glm-title")).show(ctx, &mut main_open, |ui| {
        // Factor definition row
        ui.horizontal(|ui| {
            ui.label(t!("dialog-glm-within-factor-name"));
            ui.text_edit_singleline(&mut state.factor_name);
        });

        ui.add_space(8.0);

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
                vec2(200.0, 240.0),
                "repeated_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_measure_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        state.measures.push(name.clone());
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Transfer Buttons and Targets
            let can_move_right = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();
            let can_move_left = state.target_measure_selected.is_some()
                && state.target_measure_selected.unwrap() < state.measures.len();

            let trans_btn = if state.target_measure_selected.is_some() {
                TransferButton::to_source(can_move_left)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 240.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(80.0);
                    if trans_btn.show(ui).clicked() {
                        if let Some(t_idx) = state.target_measure_selected {
                            if t_idx < state.measures.len() {
                                state.measures.remove(t_idx);
                                state.target_measure_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.measures.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Target repeated measures list
            let measure_items: Vec<VariableListItem> = state
                .measures
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
                        is_selected: state.target_measure_selected == Some(idx),
                    }
                })
                .collect();

            let target_ev = render_variable_list_panel(
                ui,
                t!("dialog-glm-repeated-measures-list"),
                vec2(200.0, 240.0),
                "repeated_measures_target",
                &measure_items,
            );

            if let ListEvent::Clicked(idx) = target_ev {
                state.target_measure_selected = Some(idx);
                state.source_selected = None;
            }
        });

        ui.add_space(8.0);

        // Action Bar
        let is_ok_enabled = state.measures.len() >= 2 && !state.factor_name.trim().is_empty();
        let bar_action = DialogActionBar::new(is_ok_enabled).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                action = RepeatedGlmDialogAction::Run {
                    factor_name: state.factor_name.clone(),
                    measures: state.measures.clone(),
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = RepeatedGlmDialogAction::Paste {
                    factor_name: state.factor_name.clone(),
                    measures: state.measures.clone(),
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = RepeatedGlmDialogState {
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
