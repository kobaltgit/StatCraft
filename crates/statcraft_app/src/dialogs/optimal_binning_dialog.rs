use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::{vec2, DragValue, Layout, TextEdit};
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct OptimalBinningDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub guide_var: Option<String>,
    pub max_bins: usize,
    pub prefix: String,
}

impl Default for OptimalBinningDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            guide_var: None,
            max_bins: 5,
            prefix: "OB_".to_string(),
        }
    }
}

impl OptimalBinningDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn reset(&mut self) {
        self.source_selected = None;
        self.target_selected = None;
        self.selected_vars.clear();
        self.guide_var = None;
        self.max_bins = 5;
        self.prefix = "OB_".to_string();
    }
}

pub enum OptimalBinningDialogAction {
    Run {
        variables: Vec<String>,
        target_var: String,
        max_bins: usize,
        prefix: String,
    },
    Paste {
        variables: Vec<String>,
        target_var: String,
        max_bins: usize,
        prefix: String,
    },
    None,
}

pub fn render_optimal_binning_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut OptimalBinningDialogState,
) -> OptimalBinningDialogAction {
    if !state.is_open {
        return OptimalBinningDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return OptimalBinningDialogAction::None;
        }
    };

    let mut action = OptimalBinningDialogAction::None;
    let mut is_open = state.is_open;
    let mut close = false;

    let available_vars: Vec<(usize, String, String, Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_in_selected = state.selected_vars.contains(&meta.name);
                let is_guide = state.guide_var.as_deref() == Some(&meta.name);
                if !is_in_selected && !is_guide {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    DialogShell::standard(t!("dialog-optbin-title")).show(ctx, &mut is_open, |ui| {
        ui.horizontal(|ui| {
            // Left panel: Available variables
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
                "optbin_source_vars",
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

            // Transfer Buttons Column
            let can_move_right = match state.source_selected {
                Some(idx) => idx < available_vars.len(),
                None => false,
            };
            let can_move_left = match state.target_selected {
                Some(idx) => idx < state.selected_vars.len(),
                None => false,
            };

            let transfer_vars_btn = if state.target_selected.is_some() {
                TransferButton::to_source(can_move_left)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(40.0);
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

                    ui.add_space(60.0);
                    let transfer_guide_btn = TransferButton::to_target(can_move_right);
                    if transfer_guide_btn.show(ui).clicked() {
                        if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.guide_var = Some(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Right: Variables to Bin + Guide Variable + Options
            ui.vertical(|ui| {
                // 1. Variables to Bin list
                let target_items: Vec<VariableListItem> = state
                    .selected_vars
                    .iter()
                    .enumerate()
                    .map(|(idx, name)| {
                        let measure = ds
                            .dictionary
                            .find_by_name(name)
                            .map(|m| m.measure)
                            .unwrap_or(Measure::Scale);
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
                    t!("dialog-optbin-variables"),
                    vec2(280.0, 120.0),
                    "optbin_target_vars",
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

                ui.add_space(6.0);

                // 2. Guide Variable Slot
                GroupBox::new(t!("dialog-optbin-target")).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if let Some(ref gvar) = state.guide_var {
                            let label = ds
                                .dictionary
                                .find_by_name(gvar)
                                .map(|m| m.label.as_str())
                                .unwrap_or("");
                            ui.label(format_var_display(gvar, label));
                            if ui.small_button("x").clicked() {
                                state.guide_var = None;
                            }
                        } else {
                            ui.weak(t!("dialog-optbin-error-target"));
                        }
                    });
                });

                ui.add_space(6.0);

                // 3. Parameters (max_bins & prefix)
                GroupBox::new(t!("dialog-optbin-options")).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-optbin-max-bins"));
                        ui.add(DragValue::new(&mut state.max_bins).range(2..=20));
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-optbin-prefix"));
                        ui.add(TextEdit::singleline(&mut state.prefix).desired_width(70.0));
                    });
                });
            });
        });

        ui.add_space(10.0);

        // Bottom Action Bar
        let can_run = !state.selected_vars.is_empty() && state.guide_var.is_some();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                if let Some(ref gvar) = state.guide_var {
                    close = true;
                    action = OptimalBinningDialogAction::Run {
                        variables: state.selected_vars.clone(),
                        target_var: gvar.clone(),
                        max_bins: state.max_bins,
                        prefix: state.prefix.trim().to_string(),
                    };
                }
            }
            ActionBarAction::Paste => {
                if let Some(ref gvar) = state.guide_var {
                    close = true;
                    action = OptimalBinningDialogAction::Paste {
                        variables: state.selected_vars.clone(),
                        target_var: gvar.clone(),
                        max_bins: state.max_bins,
                        prefix: state.prefix.trim().to_string(),
                    };
                }
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
