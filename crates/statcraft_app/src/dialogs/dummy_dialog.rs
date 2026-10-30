use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::{vec2, Layout, TextEdit};
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct DummyDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub root_prefix: String,
    pub omit_reference: bool,
}

impl Default for DummyDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            root_prefix: "d_".to_string(),
            omit_reference: false,
        }
    }
}

impl DummyDialogState {
    pub fn reset(&mut self) {
        self.source_selected = None;
        self.target_selected = None;
        self.selected_vars.clear();
        self.root_prefix = "d_".to_string();
        self.omit_reference = false;
    }
}

pub enum DummyDialogAction {
    Run {
        vars: Vec<String>,
        root_prefix: String,
        omit_reference: bool,
    },
    Paste {
        vars: Vec<String>,
        root_prefix: String,
        omit_reference: bool,
    },
    None,
}

pub fn render_dummy_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut DummyDialogState,
) -> DummyDialogAction {
    if !state.is_open {
        return DummyDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return DummyDialogAction::None;
        }
    };

    let mut action = DummyDialogAction::None;
    let mut is_open = state.is_open;
    let mut close = false;

    // Available variables: excluding those already in state.selected_vars
    let available_vars: Vec<(usize, String, String, Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if !state.selected_vars.contains(&meta.name) {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    DialogShell::standard(t!("dialog-dummy-title")).show(ctx, &mut is_open, |ui| {
        ui.horizontal(|ui| {
            // Left: Available variables list (210 x 260)
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
                vec2(210.0, 260.0),
                "dummy_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        state.selected_vars.push(name.clone());
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Center: Transfer Button
            let can_move_right = match state.source_selected {
                Some(idx) => idx < available_vars.len(),
                None => false,
            };
            let can_move_left = match state.target_selected {
                Some(idx) => idx < state.selected_vars.len(),
                None => false,
            };

            let transfer_btn = if state.target_selected.is_some() {
                TransferButton::to_source(can_move_left)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(80.0);
                    if transfer_btn.show(ui).clicked() {
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
                },
            );

            // Right: Selected Categorical Variables (270 x 260)
            ui.vertical(|ui| {
                let target_items: Vec<VariableListItem> = state
                    .selected_vars
                    .iter()
                    .enumerate()
                    .map(|(idx, name)| {
                        let measure = ds
                            .dictionary
                            .find_by_name(name)
                            .map(|m| m.measure)
                            .unwrap_or(Measure::Nominal);
                        let label = ds
                            .dictionary
                            .find_by_name(name)
                            .map(|m| m.label.as_str())
                            .unwrap_or("");
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
                    t!("dialog-dummy-source-vars"),
                    vec2(270.0, 170.0),
                    "dummy_target_vars",
                    &target_items,
                );

                match target_ev {
                    ListEvent::Clicked(idx) => {
                        state.target_selected = Some(idx);
                        state.source_selected = None;
                    }
                    ListEvent::DoubleClicked(idx) => {
                        if idx < state.selected_vars.len() {
                            state.selected_vars.remove(idx);
                            state.target_selected = None;
                        }
                    }
                    ListEvent::None => {}
                }

                ui.add_space(8.0);

                // Options block: Root prefix & omit reference
                GroupBox::new(t!("dialog-dummy-root-prefix")).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(TextEdit::singleline(&mut state.root_prefix).desired_width(120.0));
                    });
                });

                ui.add_space(4.0);
                ui.checkbox(&mut state.omit_reference, t!("dialog-dummy-omit-reference"));
            });
        });

        ui.add_space(10.0);

        // Bottom Action Bar
        let can_run = !state.selected_vars.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close = true;
                action = DummyDialogAction::Run {
                    vars: state.selected_vars.clone(),
                    root_prefix: state.root_prefix.trim().to_string(),
                    omit_reference: state.omit_reference,
                };
            }
            ActionBarAction::Paste => {
                close = true;
                action = DummyDialogAction::Paste {
                    vars: state.selected_vars.clone(),
                    root_prefix: state.root_prefix.trim().to_string(),
                    omit_reference: state.omit_reference,
                };
            }
            ActionBarAction::Reset => {
                state.reset();
            }
            ActionBarAction::Cancel => {
                close = true;
            }
            ActionBarAction::Help | ActionBarAction::None => {}
        }
    });

    state.is_open = is_open && !close;
    action
}
