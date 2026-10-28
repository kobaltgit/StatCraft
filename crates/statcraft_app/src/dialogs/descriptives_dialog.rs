use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use egui::vec2;
use statcraft_algo::DescriptivesOptions;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct DescriptivesDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub options: DescriptivesOptions,
    pub show_options: bool,
}

impl Default for DescriptivesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            options: DescriptivesOptions::spps_default(),
            show_options: false,
        }
    }
}

pub enum DescriptivesDialogAction {
    Run {
        vars: Vec<String>,
        options: DescriptivesOptions,
    },
    Paste {
        vars: Vec<String>,
    },
    None,
}

pub fn render_descriptives_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut DescriptivesDialogState,
) -> DescriptivesDialogAction {
    if !state.is_open {
        return DescriptivesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return DescriptivesDialogAction::None;
        }
    };

    let mut action = DescriptivesDialogAction::None;
    let mut close_dialog = false;

    // Build available variables (excluding already selected)
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

    // 1. Render Subdialog (Options) if open
    if state.show_options {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-desc-options-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-stats-options")).show(ui, |ui| {
                ui.columns(3, |columns| {
                    columns[0].checkbox(&mut state.options.mean, t!("stat-mean"));
                    columns[0].checkbox(&mut state.options.std_dev, t!("stat-std-dev"));
                    columns[0].checkbox(&mut state.options.variance, t!("stat-variance"));

                    columns[1].checkbox(&mut state.options.min, t!("stat-min"));
                    columns[1].checkbox(&mut state.options.max, t!("stat-max"));
                    columns[1].checkbox(&mut state.options.range, t!("stat-range"));

                    columns[2].checkbox(&mut state.options.se_mean, t!("stat-se-mean"));
                    columns[2].checkbox(&mut state.options.skewness, t!("stat-skewness"));
                    columns[2].checkbox(&mut state.options.kurtosis, t!("stat-kurtosis"));
                });
            });

            let sub_action = SubdialogActionBar::new(true).show(ui);
            match sub_action {
                SubdialogAction::Continue | SubdialogAction::Cancel => {
                    state.show_options = false;
                }
                SubdialogAction::Help | SubdialogAction::None => {}
            }
        });

        if !sub_open {
            state.show_options = false;
        }
    }

    // 2. Render Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-descriptives-title")).show(ctx, &mut main_open, |ui| {
        // Main 3-column + sidebar layout
        ui.horizontal(|ui| {
            // Left Panel: Available Variables (200x260)
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
                "desc_source_vars",
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

            // Transfer Button Column
            let can_move_right = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();
            let can_move_left = state.target_selected.is_some()
                && state.target_selected.unwrap() < state.selected_vars.len();

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
                        if state.target_selected.is_some() {
                            if let Some(idx) = state.target_selected {
                                if idx < state.selected_vars.len() {
                                    state.selected_vars.remove(idx);
                                    state.target_selected = None;
                                }
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

            // Center Panel: Target Variables (200x260)
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
                t!("dialog-variables-target"),
                vec2(200.0, 260.0),
                "desc_target_vars",
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

            // Right Column: Subdialog Sidebar
            ui.allocate_ui_with_layout(
                vec2(94.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(18.0);
                    if SubdialogSidebar::button(ui, t!("dialog-btn-options"), true).clicked() {
                        state.show_options = true;
                    }
                },
            );
        });

        // Bottom Action Bar
        let can_run = !state.selected_vars.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = DescriptivesDialogAction::Run {
                    vars: state.selected_vars.clone(),
                    options: state.options.clone(),
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = DescriptivesDialogAction::Paste {
                    vars: state.selected_vars.clone(),
                };
            }
            ActionBarAction::Reset => {
                state.selected_vars.clear();
                state.source_selected = None;
                state.target_selected = None;
                state.options = DescriptivesOptions::spps_default();
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
