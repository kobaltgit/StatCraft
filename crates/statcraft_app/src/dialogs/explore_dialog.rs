use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct ExploreDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_selected: Option<usize>,
    pub factor_selected: Option<usize>,
    pub selected_deps: Vec<String>,
    pub selected_factors: Vec<String>,
    pub compute_normality: bool,
    pub compute_descriptives: bool,
    pub compute_extremes: bool,
    pub missing_listwise: bool,
    pub ci_level_str: String,
    pub show_statistics: bool,
    pub show_plots: bool,
    pub show_options: bool,
}

impl Default for ExploreDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            dep_selected: None,
            factor_selected: None,
            selected_deps: Vec::new(),
            selected_factors: Vec::new(),
            compute_normality: true,
            compute_descriptives: true,
            compute_extremes: true,
            missing_listwise: true,
            ci_level_str: "95".to_string(),
            show_statistics: false,
            show_plots: false,
            show_options: false,
        }
    }
}

pub enum ExploreDialogAction {
    Run {
        dependent_vars: Vec<String>,
        factor_vars: Vec<String>,
        compute_descriptives: bool,
        compute_normality: bool,
        compute_extremes: bool,
        confidence_level: f64,
        missing: &'static str,
    },
    Paste {
        dependent_vars: Vec<String>,
        factor_vars: Vec<String>,
    },
    None,
}

pub fn render_explore_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut ExploreDialogState,
) -> ExploreDialogAction {
    if !state.is_open {
        return ExploreDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return ExploreDialogAction::None;
        }
    };

    let mut action = ExploreDialogAction::None;
    let mut close_dialog = false;

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

    // 1. Subdialog: Statistics
    if state.show_statistics {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-explore-statistics-title")).show(
            ctx,
            &mut sub_open,
            |ui| {
                GroupBox::new(t!("dialog-explore-statistics")).show(ui, |ui| {
                    ui.checkbox(
                        &mut state.compute_descriptives,
                        t!("dialog-explore-descriptives"),
                    );
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(t!("dialog-explore-confidence-interval"));
                        ui.add(
                            egui::TextEdit::singleline(&mut state.ci_level_str).desired_width(38.0),
                        );
                        ui.label("%");
                    });
                    ui.add_space(6.0);
                    ui.checkbox(&mut state.compute_extremes, t!("dialog-explore-extremes"));
                });

                let sub_action = SubdialogActionBar::new(true).show(ui);
                match sub_action {
                    SubdialogAction::Continue | SubdialogAction::Cancel => {
                        state.show_statistics = false;
                    }
                    SubdialogAction::Help | SubdialogAction::None => {}
                }
            },
        );
        if !sub_open {
            state.show_statistics = false;
        }
    }

    // 2. Subdialog: Plots
    if state.show_plots {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-explore-plots")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-explore-plots")).show(ui, |ui| {
                ui.checkbox(
                    &mut state.compute_normality,
                    t!("dialog-explore-normality-tests"),
                );
            });

            let sub_action = SubdialogActionBar::new(true).show(ui);
            match sub_action {
                SubdialogAction::Continue | SubdialogAction::Cancel => {
                    state.show_plots = false;
                }
                SubdialogAction::Help | SubdialogAction::None => {}
            }
        });
        if !sub_open {
            state.show_plots = false;
        }
    }

    // 3. Subdialog: Options
    if state.show_options {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-explore-options-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-explore-missing-values")).show(ui, |ui| {
                ui.radio_value(
                    &mut state.missing_listwise,
                    true,
                    t!("dialog-explore-exclude-listwise"),
                );
                ui.radio_value(
                    &mut state.missing_listwise,
                    false,
                    t!("dialog-explore-exclude-pairwise"),
                );
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

    // 4. Main Explore Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-explore-title")).show(ctx, &mut main_open, |ui| {
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
                "explore_source_vars",
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

            // Center Panel: Dependent List and Factor List
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
                        t!("dialog-explore-dependent-list"),
                        vec2(200.0, 120.0),
                        "explore_target_deps",
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

                    // Factor List
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
                        t!("dialog-explore-factor-list"),
                        vec2(200.0, 120.0),
                        "explore_target_factors",
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
                    if SubdialogSidebar::button(ui, t!("dialog-btn-statistics"), true).clicked() {
                        state.show_statistics = true;
                    }
                    ui.add_space(6.0);
                    if SubdialogSidebar::button(ui, t!("dialog-explore-plots"), true).clicked() {
                        state.show_plots = true;
                    }
                    ui.add_space(6.0);
                    if SubdialogSidebar::button(ui, t!("dialog-btn-options"), true).clicked() {
                        state.show_options = true;
                    }
                },
            );
        });

        // Bottom Action Bar
        let can_run = !state.selected_deps.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                let ci = state
                    .ci_level_str
                    .parse::<f64>()
                    .map(|p| p / 100.0)
                    .unwrap_or(0.95);
                let missing = if state.missing_listwise {
                    "listwise"
                } else {
                    "pairwise"
                };

                action = ExploreDialogAction::Run {
                    dependent_vars: state.selected_deps.clone(),
                    factor_vars: state.selected_factors.clone(),
                    compute_descriptives: state.compute_descriptives,
                    compute_normality: state.compute_normality,
                    compute_extremes: state.compute_extremes,
                    confidence_level: ci,
                    missing,
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = ExploreDialogAction::Paste {
                    dependent_vars: state.selected_deps.clone(),
                    factor_vars: state.selected_factors.clone(),
                };
            }
            ActionBarAction::Reset => {
                state.selected_deps.clear();
                state.selected_factors.clear();
                state.source_selected = None;
                state.dep_selected = None;
                state.factor_selected = None;
                state.compute_normality = true;
                state.compute_descriptives = true;
                state.compute_extremes = true;
                state.missing_listwise = true;
                state.ci_level_str = "95".to_string();
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
