use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use egui::vec2;
use statcraft_algo::{ReliabilityModel, ReliabilityOptions};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct ReliabilityDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub scale_name: String,
    pub model: ReliabilityModel,
    pub item_statistics: bool,
    pub scale_statistics: bool,
    pub scale_if_item_deleted: bool,
    pub show_statistics_subdialog: bool,
}

impl Default for ReliabilityDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            scale_name: "ALL VARIABLES".to_string(),
            model: ReliabilityModel::Alpha,
            item_statistics: true,
            scale_statistics: true,
            scale_if_item_deleted: true,
            show_statistics_subdialog: false,
        }
    }
}

pub enum ReliabilityDialogAction {
    Run {
        variables: Vec<String>,
        scale_name: String,
        model: ReliabilityModel,
        options: ReliabilityOptions,
    },
    Paste {
        variables: Vec<String>,
        scale_name: String,
        model: ReliabilityModel,
        options: ReliabilityOptions,
    },
    None,
}

pub fn render_reliability_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut ReliabilityDialogState,
) -> ReliabilityDialogAction {
    if !state.is_open {
        return ReliabilityDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return ReliabilityDialogAction::None;
        }
    };

    let mut action = ReliabilityDialogAction::None;
    let mut close_dialog = false;

    // Filter available numeric variables
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

    // 1. Statistics Subdialog
    if state.show_statistics_subdialog {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-reliability-stats-title")).show(
            ctx,
            &mut sub_open,
            |ui| {
                GroupBox::new(t!("dialog-reliability-descriptives-for-group")).show(ui, |ui| {
                    ui.checkbox(
                        &mut state.item_statistics,
                        t!("dialog-reliability-stat-item"),
                    );
                    ui.checkbox(
                        &mut state.scale_statistics,
                        t!("dialog-reliability-stat-scale"),
                    );
                    ui.checkbox(
                        &mut state.scale_if_item_deleted,
                        t!("dialog-reliability-stat-scale-if-deleted"),
                    );
                });

                ui.add_space(14.0);

                let sub_action = SubdialogActionBar::new(true).show(ui);
                match sub_action {
                    SubdialogAction::Continue | SubdialogAction::Cancel => {
                        state.show_statistics_subdialog = false;
                    }
                    SubdialogAction::Help | SubdialogAction::None => {}
                }
            },
        );

        if !sub_open {
            state.show_statistics_subdialog = false;
        }
    }

    // 2. Main Reliability Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-reliability-title")).show(ctx, &mut main_open, |ui| {
        // Top section: Source -> Transfer -> Target -> Sidebar
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
                vec2(190.0, 190.0),
                "rel_source_vars",
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

            // Transfer button
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
                vec2(36.0, 190.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(60.0);
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

            // Target Items panel
            let target_items: Vec<VariableListItem> = state
                .selected_vars
                .iter()
                .enumerate()
                .map(|(idx, name)| {
                    let measure = ds
                        .dictionary
                        .find_by_name(name)
                        .map(|m| m.measure)
                        .unwrap_or(statcraft_core::Measure::Scale);
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
                t!("dialog-reliability-items"),
                vec2(190.0, 190.0),
                "rel_target_items",
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

            // Right sidebar: Statistics button
            ui.allocate_ui_with_layout(
                vec2(90.0, 190.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(18.0);
                    if SubdialogSidebar::button(ui, t!("dialog-reliability-btn-stats"), true)
                        .clicked()
                    {
                        state.show_statistics_subdialog = true;
                    }
                },
            );
        });

        ui.add_space(10.0);

        // Middle section: Model selection and Scale label
        ui.horizontal(|ui| {
            ui.label(t!("dialog-reliability-model-label"));
            egui::ComboBox::from_id_salt("rel_model_combo")
                .selected_text(t!("dialog-reliability-model-alpha"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.model,
                        ReliabilityModel::Alpha,
                        t!("dialog-reliability-model-alpha"),
                    );
                });

            ui.add_space(20.0);

            ui.label(t!("dialog-reliability-scale-label"));
            ui.text_edit_singleline(&mut state.scale_name);
        });

        ui.add_space(14.0);

        // Bottom Action Bar
        let can_run = state.selected_vars.len() >= 2;
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = ReliabilityDialogAction::Run {
                    variables: state.selected_vars.clone(),
                    scale_name: state.scale_name.clone(),
                    model: state.model,
                    options: ReliabilityOptions {
                        scale_name: state.scale_name.clone(),
                        model: state.model,
                        item_statistics: state.item_statistics,
                        scale_statistics: state.scale_statistics,
                        scale_if_item_deleted: state.scale_if_item_deleted,
                    },
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = ReliabilityDialogAction::Paste {
                    variables: state.selected_vars.clone(),
                    scale_name: state.scale_name.clone(),
                    model: state.model,
                    options: ReliabilityOptions {
                        scale_name: state.scale_name.clone(),
                        model: state.model,
                        item_statistics: state.item_statistics,
                        scale_statistics: state.scale_statistics,
                        scale_if_item_deleted: state.scale_if_item_deleted,
                    },
                };
            }
            ActionBarAction::Reset => {
                *state = ReliabilityDialogState::default();
                state.is_open = true;
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
