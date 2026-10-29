use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::{DataSet, SplitFileMode};
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct SplitFileDialogState {
    pub is_open: bool,
    pub mode: SplitFileMode,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub sort_file: bool,
}

impl Default for SplitFileDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            mode: SplitFileMode::Off,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            sort_file: true,
        }
    }
}

pub enum SplitFileDialogAction {
    Run {
        mode: SplitFileMode,
        variables: Vec<String>,
        sort: bool,
    },
    Paste {
        mode: SplitFileMode,
        variables: Vec<String>,
        sort: bool,
    },
    None,
}

pub fn render_split_file_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut SplitFileDialogState,
) -> SplitFileDialogAction {
    if !state.is_open {
        return SplitFileDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return SplitFileDialogAction::None;
        }
    };

    let mut action = SplitFileDialogAction::None;
    let mut close_dialog = false;

    // Available variables: columns not yet in selected_vars
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
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

    let mut main_open = true;
    DialogShell::standard(t!("dialog-split-file-title")).show(ctx, &mut main_open, |ui| {
        // Mode Selection Radios
        ui.vertical(|ui| {
            ui.radio_value(
                &mut state.mode,
                SplitFileMode::Off,
                t!("dialog-split-mode-off"),
            );
            ui.radio_value(
                &mut state.mode,
                SplitFileMode::CompareGroups,
                t!("dialog-split-mode-compare"),
            );
            ui.radio_value(
                &mut state.mode,
                SplitFileMode::OrganizeOutput,
                t!("dialog-split-mode-separate"),
            );
        });

        ui.add_space(6.0);

        let is_split_active = state.mode != SplitFileMode::Off;

        // Variables Selection (enabled only when mode != Off)
        ui.add_enabled_ui(is_split_active, |ui| {
            ui.horizontal(|ui| {
                // Left Panel: Available Variables (190x210)
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
                    vec2(190.0, 210.0),
                    "split_file_source_vars",
                    &source_items,
                );

                match source_ev {
                    ListEvent::Clicked(idx) => {
                        state.source_selected = Some(idx);
                        state.target_selected = None;
                    }
                    ListEvent::DoubleClicked(idx) => {
                        if let Some((_col, name, _label, _m)) = available_vars.get(idx) {
                            state.selected_vars.push(name.clone());
                            state.source_selected = None;
                            state.target_selected = None;
                        }
                    }
                    ListEvent::None => {}
                }

                // Middle: Transfer Button
                let can_move_right = state.source_selected.is_some()
                    && state.source_selected.unwrap_or(usize::MAX) < available_vars.len();
                let can_move_left = state.target_selected.is_some()
                    && state.target_selected.unwrap_or(usize::MAX) < state.selected_vars.len();

                let transfer_btn = if state.target_selected.is_some() {
                    TransferButton::to_source(can_move_left)
                } else {
                    TransferButton::to_target(can_move_right)
                };

                ui.allocate_ui_with_layout(
                    vec2(36.0, 210.0),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.add_space(80.0);
                        if transfer_btn.show(ui).clicked() {
                            if let Some(src_idx) = state.source_selected {
                                if let Some((_col, name, _label, _m)) = available_vars.get(src_idx)
                                {
                                    state.selected_vars.push(name.clone());
                                    state.source_selected = None;
                                }
                            } else if let Some(tgt_idx) = state.target_selected {
                                if tgt_idx < state.selected_vars.len() {
                                    state.selected_vars.remove(tgt_idx);
                                    state.target_selected = None;
                                }
                            }
                        }
                    },
                );

                // Right Panel: Groups Based on (190x210)
                let target_items: Vec<VariableListItem> = state
                    .selected_vars
                    .iter()
                    .enumerate()
                    .map(|(idx, name)| {
                        let meta = ds
                            .dictionary
                            .find_index(name)
                            .and_then(|i| ds.dictionary.get(i));
                        let label = meta.map(|m| m.label.as_str()).unwrap_or("");
                        let measure = meta
                            .map(|m| m.measure)
                            .unwrap_or(statcraft_core::Measure::Nominal);
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
                    t!("dialog-split-groups-based-on"),
                    vec2(190.0, 210.0),
                    "split_file_target_vars",
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
            });

            ui.add_space(6.0);

            // Sorting Options GroupBox
            GroupBox::new(t!("dialog-split-sort-group")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut state.sort_file,
                        true,
                        t!("dialog-split-sort-by-variables"),
                    );
                    ui.add_space(10.0);
                    ui.radio_value(
                        &mut state.sort_file,
                        false,
                        t!("dialog-split-file-already-sorted"),
                    );
                });
            });
        });

        ui.add_space(8.0);

        // Action Bar Type 1
        let can_run = match state.mode {
            SplitFileMode::Off => true,
            _ => !state.selected_vars.is_empty(),
        };

        let act = DialogActionBar::new(can_run).show(ui);
        match act {
            ActionBarAction::Ok => {
                action = SplitFileDialogAction::Run {
                    mode: state.mode,
                    variables: state.selected_vars.clone(),
                    sort: state.sort_file,
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = SplitFileDialogAction::Paste {
                    mode: state.mode,
                    variables: state.selected_vars.clone(),
                    sort: state.sort_file,
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                state.mode = SplitFileMode::Off;
                state.selected_vars.clear();
                state.source_selected = None;
                state.target_selected = None;
                state.sort_file = true;
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
