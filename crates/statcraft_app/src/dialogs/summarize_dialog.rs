use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct SummarizeDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub variables: Vec<String>,
    pub group_by: Vec<String>,
    pub target_var_selected: Option<usize>,
    pub target_grp_selected: Option<usize>,
    pub limit_cases_enabled: bool,
    pub limit_cases_str: String,
    pub display_cases: bool,
}

impl Default for SummarizeDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            variables: Vec::new(),
            group_by: Vec::new(),
            target_var_selected: None,
            target_grp_selected: None,
            limit_cases_enabled: true,
            limit_cases_str: "100".to_string(),
            display_cases: true,
        }
    }
}

pub enum SummarizeDialogAction {
    Run {
        variables: Vec<String>,
        group_by: Vec<String>,
        limit_cases: Option<usize>,
        display_cases: bool,
    },
    Paste {
        variables: Vec<String>,
        group_by: Vec<String>,
        limit_cases: Option<usize>,
        display_cases: bool,
    },
    None,
}

pub fn render_summarize_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut SummarizeDialogState,
) -> SummarizeDialogAction {
    if !state.is_open {
        return SummarizeDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return SummarizeDialogAction::None;
        }
    };

    let mut action = SummarizeDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if state.variables.contains(&meta.name) || state.group_by.contains(&meta.name) {
                    None
                } else {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                }
            })
        })
        .collect();

    let shell = DialogShell::standard(t!("dialog-summarize-title"));
    let mut main_open = state.is_open;

    shell.show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left column: Source variables list
            let source_items: Vec<VariableListItem> = available_vars
                .iter()
                .enumerate()
                .map(|(idx, (_, name, label, measure))| VariableListItem {
                    name,
                    display_text: format_var_display(name, label),
                    measure: *measure,
                    is_selected: state.source_selected == Some(idx),
                })
                .collect();

            let src_event = render_variable_list_panel(
                ui,
                t!("dialog-common-variables"),
                vec2(180.0, 240.0),
                "summarize_source_list",
                &source_items,
            );

            match src_event {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_var_selected = None;
                    state.target_grp_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        state.variables.push(name.clone());
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Middle column: Transfer buttons for Variables and Grouping
            ui.vertical(|ui| {
                ui.add_space(40.0);
                let can_transfer_var = state.source_selected.is_some();
                let can_remove_var = state.target_var_selected.is_some();

                if TransferButton::to_target(can_transfer_var)
                    .show(ui)
                    .clicked()
                {
                    if let Some(src_idx) = state.source_selected {
                        if let Some(item) = source_items.get(src_idx) {
                            state.variables.push(item.name.to_string());
                            state.source_selected = None;
                        }
                    }
                }
                ui.add_space(4.0);
                if TransferButton::to_source(can_remove_var).show(ui).clicked() {
                    if let Some(idx) = state.target_var_selected {
                        if idx < state.variables.len() {
                            state.variables.remove(idx);
                            state.target_var_selected = None;
                        }
                    }
                }

                ui.add_space(50.0);
                let can_transfer_grp = state.source_selected.is_some();
                let can_remove_grp = state.target_grp_selected.is_some();

                if TransferButton::to_target(can_transfer_grp)
                    .show(ui)
                    .clicked()
                {
                    if let Some(src_idx) = state.source_selected {
                        if let Some(item) = source_items.get(src_idx) {
                            state.group_by.push(item.name.to_string());
                            state.source_selected = None;
                        }
                    }
                }
                ui.add_space(4.0);
                if TransferButton::to_source(can_remove_grp).show(ui).clicked() {
                    if let Some(idx) = state.target_grp_selected {
                        if idx < state.group_by.len() {
                            state.group_by.remove(idx);
                            state.target_grp_selected = None;
                        }
                    }
                }
            });

            // Right column: Target lists and settings
            ui.vertical(|ui| {
                // Variables target list
                let var_items: Vec<VariableListItem> = state
                    .variables
                    .iter()
                    .enumerate()
                    .map(|(idx, v_name)| {
                        let measure = ds
                            .dictionary
                            .find_by_name(v_name)
                            .map(|m| m.measure)
                            .unwrap_or(statcraft_core::Measure::Scale);
                        let label = ds
                            .dictionary
                            .find_by_name(v_name)
                            .map(|m| m.label.clone())
                            .unwrap_or_default();
                        VariableListItem {
                            name: v_name,
                            display_text: format_var_display(v_name, &label),
                            measure,
                            is_selected: state.target_var_selected == Some(idx),
                        }
                    })
                    .collect();

                let v_event = render_variable_list_panel(
                    ui,
                    t!("dialog-summarize-variables"),
                    vec2(220.0, 95.0),
                    "summarize_var_list",
                    &var_items,
                );

                match v_event {
                    ListEvent::Clicked(idx) => {
                        state.target_var_selected = Some(idx);
                        state.source_selected = None;
                        state.target_grp_selected = None;
                    }
                    ListEvent::DoubleClicked(idx) => {
                        if idx < state.variables.len() {
                            state.variables.remove(idx);
                            state.target_var_selected = None;
                        }
                    }
                    ListEvent::None => {}
                }

                ui.add_space(6.0);

                // GroupBy target list
                let grp_items: Vec<VariableListItem> = state
                    .group_by
                    .iter()
                    .enumerate()
                    .map(|(idx, v_name)| {
                        let measure = ds
                            .dictionary
                            .find_by_name(v_name)
                            .map(|m| m.measure)
                            .unwrap_or(statcraft_core::Measure::Nominal);
                        let label = ds
                            .dictionary
                            .find_by_name(v_name)
                            .map(|m| m.label.clone())
                            .unwrap_or_default();
                        VariableListItem {
                            name: v_name,
                            display_text: format_var_display(v_name, &label),
                            measure,
                            is_selected: state.target_grp_selected == Some(idx),
                        }
                    })
                    .collect();

                let g_event = render_variable_list_panel(
                    ui,
                    t!("dialog-summarize-grouping"),
                    vec2(220.0, 95.0),
                    "summarize_grp_list",
                    &grp_items,
                );

                match g_event {
                    ListEvent::Clicked(idx) => {
                        state.target_grp_selected = Some(idx);
                        state.source_selected = None;
                        state.target_var_selected = None;
                    }
                    ListEvent::DoubleClicked(idx) => {
                        if idx < state.group_by.len() {
                            state.group_by.remove(idx);
                            state.target_grp_selected = None;
                        }
                    }
                    ListEvent::None => {}
                }

                ui.add_space(6.0);

                // Options GroupBox
                GroupBox::new(t!("dialog-summarize-options")).show(ui, |ui| {
                    ui.checkbox(
                        &mut state.display_cases,
                        t!("dialog-summarize-display-cases"),
                    );
                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut state.limit_cases_enabled,
                            t!("dialog-summarize-limit-cases"),
                        );
                        if state.limit_cases_enabled {
                            ui.text_edit_singleline(&mut state.limit_cases_str);
                        }
                    });
                });
            });
        });

        ui.add_space(16.0);

        // Validation & Action Bar
        let limit_val: Option<usize> = if state.limit_cases_enabled {
            state.limit_cases_str.parse().ok()
        } else {
            None
        };
        let is_valid = !state.variables.is_empty();

        let bar_action = DialogActionBar::new(is_valid).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                action = SummarizeDialogAction::Run {
                    variables: state.variables.clone(),
                    group_by: state.group_by.clone(),
                    limit_cases: limit_val,
                    display_cases: state.display_cases,
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = SummarizeDialogAction::Paste {
                    variables: state.variables.clone(),
                    group_by: state.group_by.clone(),
                    limit_cases: limit_val,
                    display_cases: state.display_cases,
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = SummarizeDialogState {
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
