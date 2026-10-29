use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use egui::vec2;
use statcraft_algo::MeansOptions;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone, Default)]
pub struct MeansDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_selected: Option<usize>,
    pub factor_selected: Option<usize>,
    pub selected_deps: Vec<String>,
    pub selected_factors: Vec<String>,
    pub options: MeansOptions,
    pub show_options: bool,
}

pub enum MeansDialogAction {
    Run {
        dependent_vars: Vec<String>,
        factor_vars: Vec<String>,
        options: MeansOptions,
    },
    Paste {
        dependent_vars: Vec<String>,
        factor_vars: Vec<String>,
        options: MeansOptions,
    },
    None,
}

pub fn render_means_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut MeansDialogState,
) -> MeansDialogAction {
    if !state.is_open {
        return MeansDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return MeansDialogAction::None;
        }
    };

    let mut action = MeansDialogAction::None;
    let mut close_dialog = false;

    // Available variables (excluding already selected in deps or factors)
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let in_deps = state.selected_deps.contains(&meta.name);
                let in_factors = state.selected_factors.contains(&meta.name);
                if !in_deps && !in_factors {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    // 1. Subdialog: Options
    if state.show_options {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-means-options-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-means-statistics-group")).show(ui, |ui| {
                ui.checkbox(&mut state.options.cell_mean, t!("dialog-means-stat-mean"));
                ui.checkbox(&mut state.options.cell_count, t!("dialog-means-stat-count"));
                ui.checkbox(
                    &mut state.options.cell_stddev,
                    t!("dialog-means-stat-stddev"),
                );
                ui.checkbox(
                    &mut state.options.cell_variance,
                    t!("dialog-means-stat-variance"),
                );
                ui.checkbox(
                    &mut state.options.cell_median,
                    t!("dialog-means-stat-median"),
                );
                ui.checkbox(&mut state.options.cell_min, t!("dialog-means-stat-min"));
                ui.checkbox(&mut state.options.cell_max, t!("dialog-means-stat-max"));
            });

            ui.add_space(6.0);
            GroupBox::new(t!("dialog-means-statistics-anova-group")).show(ui, |ui| {
                ui.checkbox(&mut state.options.anova, t!("dialog-means-stat-anova"));
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

    // 2. Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-means-title")).show(ctx, &mut main_open, |ui| {
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
                "means_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.dep_selected = None;
                    state.factor_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        state.selected_deps.push(name.clone());
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Transfer Buttons Column
            let can_move_right_source = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();
            let can_move_left_dep = state.dep_selected.is_some()
                && state.dep_selected.unwrap() < state.selected_deps.len();
            let can_move_left_factor = state.factor_selected.is_some()
                && state.factor_selected.unwrap() < state.selected_factors.len();

            let transfer_dep_btn = if state.dep_selected.is_some() {
                TransferButton::to_source(can_move_left_dep)
            } else {
                TransferButton::to_target(can_move_right_source)
            };

            let transfer_factor_btn = if state.factor_selected.is_some() {
                TransferButton::to_source(can_move_left_factor)
            } else {
                TransferButton::to_target(can_move_right_source)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(45.0);
                    if transfer_dep_btn.show(ui).clicked() {
                        if state.dep_selected.is_some() {
                            if let Some(idx) = state.dep_selected {
                                if idx < state.selected_deps.len() {
                                    state.selected_deps.remove(idx);
                                    state.dep_selected = None;
                                }
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.selected_deps.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    ui.add_space(85.0);
                    if transfer_factor_btn.show(ui).clicked() {
                        if state.factor_selected.is_some() {
                            if let Some(idx) = state.factor_selected {
                                if idx < state.selected_factors.len() {
                                    state.selected_factors.remove(idx);
                                    state.factor_selected = None;
                                }
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.selected_factors.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Center Panel: Dependent List and Layer 1/Independent List
            ui.allocate_ui_with_layout(
                vec2(200.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // Dependent List
                    let dep_items: Vec<VariableListItem> = state
                        .selected_deps
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
                                is_selected: state.dep_selected == Some(idx),
                            }
                        })
                        .collect();

                    let dep_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-means-dependent-list"),
                        vec2(200.0, 120.0),
                        "means_target_deps",
                        &dep_items,
                    );

                    match dep_ev {
                        ListEvent::Clicked(idx) => {
                            state.dep_selected = Some(idx);
                            state.source_selected = None;
                            state.factor_selected = None;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.selected_deps.len() {
                                state.selected_deps.remove(idx);
                                state.dep_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }

                    ui.add_space(8.0);

                    // Independent / Factor List
                    let factor_items: Vec<VariableListItem> = state
                        .selected_factors
                        .iter()
                        .enumerate()
                        .map(|(idx, name)| {
                            let measure = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|m| m.measure)
                                .unwrap_or(statcraft_core::Measure::Nominal);
                            let label = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|m| m.label.as_str())
                                .unwrap_or("");
                            VariableListItem {
                                name,
                                display_text: format_var_display(name, label),
                                measure,
                                is_selected: state.factor_selected == Some(idx),
                            }
                        })
                        .collect();

                    let factor_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-means-independent-list"),
                        vec2(200.0, 120.0),
                        "means_target_factors",
                        &factor_items,
                    );

                    match factor_ev {
                        ListEvent::Clicked(idx) => {
                            state.factor_selected = Some(idx);
                            state.source_selected = None;
                            state.dep_selected = None;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.selected_factors.len() {
                                state.selected_factors.remove(idx);
                                state.factor_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }
                },
            );

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
        let can_run = !state.selected_deps.is_empty() && !state.selected_factors.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = MeansDialogAction::Run {
                    dependent_vars: state.selected_deps.clone(),
                    factor_vars: state.selected_factors.clone(),
                    options: state.options.clone(),
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = MeansDialogAction::Paste {
                    dependent_vars: state.selected_deps.clone(),
                    factor_vars: state.selected_factors.clone(),
                    options: state.options.clone(),
                };
            }
            ActionBarAction::Reset => {
                *state = MeansDialogState {
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

    if !main_open || close_dialog {
        state.is_open = false;
    }

    action
}
