use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_algo::{SortDirection, SortKeySpec};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct SortCasesDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_keys: Vec<SortKeySpec>,
    pub current_direction: SortDirection,
}

impl Default for SortCasesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_keys: Vec::new(),
            current_direction: SortDirection::Ascending,
        }
    }
}

pub enum SortCasesDialogAction {
    Run(Vec<SortKeySpec>),
    Paste(Vec<SortKeySpec>),
    None,
}

pub fn render_sort_cases_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut SortCasesDialogState,
) -> SortCasesDialogAction {
    if !state.is_open {
        return SortCasesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return SortCasesDialogAction::None;
        }
    };

    let mut action = SortCasesDialogAction::None;
    let mut close_dialog = false;

    // Available variables: columns not yet in selected_keys
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if !state
                    .selected_keys
                    .iter()
                    .any(|k| k.column_name == meta.name)
                {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-sort-cases-title")).show(ctx, &mut main_open, |ui| {
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
                "sort_cases_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        state
                            .selected_keys
                            .push(SortKeySpec::new(name.clone(), state.current_direction));
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Middle Column: Transfer Button (36x260)
            let can_move_right = state.source_selected.is_some()
                && state.source_selected.unwrap_or(usize::MAX) < available_vars.len();
            let can_move_left = state.target_selected.is_some()
                && state.target_selected.unwrap_or(usize::MAX) < state.selected_keys.len();

            let transfer_btn = if state.target_selected.is_some() {
                TransferButton::to_source(can_move_left)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(80.0);
                    if transfer_btn.show(ui).clicked() {
                        if let Some(idx) = state.target_selected {
                            if idx < state.selected_keys.len() {
                                state.selected_keys.remove(idx);
                                state.target_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state
                                    .selected_keys
                                    .push(SortKeySpec::new(name.clone(), state.current_direction));
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Right Panel: Target keys, Move Up/Down, Direction radio
            ui.allocate_ui_with_layout(
                vec2(290.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let target_items: Vec<VariableListItem> = state
                        .selected_keys
                        .iter()
                        .enumerate()
                        .map(|(idx, key)| {
                            let meta = ds.dictionary.find_by_name(&key.column_name);
                            let measure = meta
                                .map(|m| m.measure)
                                .unwrap_or(statcraft_core::Measure::Scale);
                            let label = meta.map(|m| m.label.as_str()).unwrap_or("");
                            let dir_suffix = match key.direction {
                                SortDirection::Ascending => "(A)",
                                SortDirection::Descending => "(D)",
                            };
                            let base_disp = format_var_display(&key.column_name, label);
                            let full_disp = format!("{} {}", base_disp, dir_suffix);
                            VariableListItem {
                                name: &key.column_name,
                                display_text: full_disp,
                                measure,
                                is_selected: state.target_selected == Some(idx),
                            }
                        })
                        .collect();

                    let target_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-sort-cases-sort-by"),
                        vec2(280.0, 140.0),
                        "sort_cases_target_keys",
                        &target_items,
                    );

                    match target_ev {
                        ListEvent::Clicked(idx) => {
                            state.target_selected = Some(idx);
                            state.source_selected = None;
                            if let Some(key) = state.selected_keys.get(idx) {
                                state.current_direction = key.direction;
                            }
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.selected_keys.len() {
                                state.selected_keys.remove(idx);
                                state.target_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }

                    // Move Up / Move Down buttons
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let can_move_up = state
                            .target_selected
                            .map(|idx| idx > 0 && idx < state.selected_keys.len())
                            .unwrap_or(false);
                        let can_move_down = state
                            .target_selected
                            .map(|idx| idx + 1 < state.selected_keys.len())
                            .unwrap_or(false);

                        if ui
                            .add_enabled(
                                can_move_up,
                                egui::Button::new(t!("dialog-sort-cases-move-up")),
                            )
                            .clicked()
                        {
                            if let Some(idx) = state.target_selected {
                                if idx > 0 && idx < state.selected_keys.len() {
                                    state.selected_keys.swap(idx, idx - 1);
                                    state.target_selected = Some(idx - 1);
                                }
                            }
                        }

                        if ui
                            .add_enabled(
                                can_move_down,
                                egui::Button::new(t!("dialog-sort-cases-move-down")),
                            )
                            .clicked()
                        {
                            if let Some(idx) = state.target_selected {
                                if idx + 1 < state.selected_keys.len() {
                                    state.selected_keys.swap(idx, idx + 1);
                                    state.target_selected = Some(idx + 1);
                                }
                            }
                        }
                    });

                    // Sort Order GroupBox
                    ui.add_space(6.0);
                    GroupBox::new(t!("dialog-sort-cases-order-group")).show(ui, |ui| {
                        let mut dir = state.current_direction;
                        let changed_asc = ui
                            .radio_value(
                                &mut dir,
                                SortDirection::Ascending,
                                t!("dialog-sort-cases-order-ascending"),
                            )
                            .changed();
                        let changed_desc = ui
                            .radio_value(
                                &mut dir,
                                SortDirection::Descending,
                                t!("dialog-sort-cases-order-descending"),
                            )
                            .changed();

                        if changed_asc || changed_desc {
                            state.current_direction = dir;
                            if let Some(idx) = state.target_selected {
                                if let Some(key) = state.selected_keys.get_mut(idx) {
                                    key.direction = dir;
                                }
                            }
                        }
                    });
                },
            );
        });

        // Bottom Action Bar
        let can_run = !state.selected_keys.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = SortCasesDialogAction::Run(state.selected_keys.clone());
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = SortCasesDialogAction::Paste(state.selected_keys.clone());
            }
            ActionBarAction::Reset => {
                state.selected_keys.clear();
                state.source_selected = None;
                state.target_selected = None;
                state.current_direction = SortDirection::Ascending;
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
