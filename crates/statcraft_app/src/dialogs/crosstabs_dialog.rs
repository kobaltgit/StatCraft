use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use egui::vec2;
use statcraft_algo::CrosstabOptions;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrosstabsTargetList {
    Row,
    Col,
}

#[derive(Debug, Clone)]
pub struct CrosstabsDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub row_selected: Option<usize>,
    pub col_selected: Option<usize>,
    pub active_target: CrosstabsTargetList,
    pub row_vars: Vec<String>,
    pub col_vars: Vec<String>,
    pub options: CrosstabOptions,
    pub show_cells: bool,
    pub show_stats: bool,
}

impl Default for CrosstabsDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            row_selected: None,
            col_selected: None,
            active_target: CrosstabsTargetList::Row,
            row_vars: Vec::new(),
            col_vars: Vec::new(),
            options: CrosstabOptions::default(),
            show_cells: false,
            show_stats: false,
        }
    }
}

pub enum CrosstabsDialogAction {
    Run {
        rows: Vec<String>,
        cols: Vec<String>,
        options: CrosstabOptions,
    },
    Paste {
        rows: Vec<String>,
        cols: Vec<String>,
        options: CrosstabOptions,
    },
    None,
}

pub fn render_crosstabs_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut CrosstabsDialogState,
) -> CrosstabsDialogAction {
    if !state.is_open {
        return CrosstabsDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return CrosstabsDialogAction::None;
        }
    };

    let mut action = CrosstabsDialogAction::None;
    let mut close_dialog = false;

    // Available variables: filter out row_vars and col_vars
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let in_rows = state.row_vars.contains(&meta.name);
                let in_cols = state.col_vars.contains(&meta.name);
                if !in_rows && !in_cols {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    // 1. Subdialog: Cells
    if state.show_cells {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-crosstabs-cells-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-crosstabs-group-counts")).show(ui, |ui| {
                ui.checkbox(&mut true, t!("dialog-crosstabs-count-observed"));
                ui.checkbox(
                    &mut state.options.expected,
                    t!("dialog-crosstabs-count-expected"),
                );
                ui.checkbox(
                    &mut state.options.residuals,
                    t!("dialog-crosstabs-count-residuals"),
                );
            });
            ui.add_space(6.0);
            GroupBox::new(t!("dialog-crosstabs-group-percentages")).show(ui, |ui| {
                ui.checkbox(&mut state.options.row_pct, t!("dialog-crosstabs-pct-row"));
                ui.checkbox(
                    &mut state.options.col_pct,
                    t!("dialog-crosstabs-pct-column"),
                );
                ui.checkbox(
                    &mut state.options.total_pct,
                    t!("dialog-crosstabs-pct-total"),
                );
            });

            let sub_action = SubdialogActionBar::new(true).show(ui);
            match sub_action {
                SubdialogAction::Continue | SubdialogAction::Cancel => {
                    state.show_cells = false;
                }
                SubdialogAction::Help | SubdialogAction::None => {}
            }
        });
        if !sub_open {
            state.show_cells = false;
        }
    }

    // 2. Subdialog: Statistics
    if state.show_stats {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-crosstabs-stats-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-crosstabs-stats-title")).show(ui, |ui| {
                ui.checkbox(&mut state.options.chisq, t!("dialog-crosstabs-stat-chisq"));
                ui.add_space(4.0);
                ui.checkbox(
                    &mut state.options.phi_cramer,
                    t!("dialog-crosstabs-stat-phi-cramer"),
                );
            });

            let sub_action = SubdialogActionBar::new(true).show(ui);
            match sub_action {
                SubdialogAction::Continue | SubdialogAction::Cancel => {
                    state.show_stats = false;
                }
                SubdialogAction::Help | SubdialogAction::None => {}
            }
        });
        if !sub_open {
            state.show_stats = false;
        }
    }

    // 3. Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-crosstabs-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left Panel: Available Variables
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
                "crosstabs_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.row_selected = None;
                    state.col_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        match state.active_target {
                            CrosstabsTargetList::Row => state.row_vars.push(name.clone()),
                            CrosstabsTargetList::Col => state.col_vars.push(name.clone()),
                        }
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Transfer Buttons Column
            let can_move_source = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();
            let can_move_row =
                state.row_selected.is_some() && state.row_selected.unwrap() < state.row_vars.len();
            let can_move_col =
                state.col_selected.is_some() && state.col_selected.unwrap() < state.col_vars.len();

            let transfer_row_btn = if state.row_selected.is_some() {
                TransferButton::to_source(can_move_row)
            } else {
                TransferButton::to_target(can_move_source)
            };

            let transfer_col_btn = if state.col_selected.is_some() {
                TransferButton::to_source(can_move_col)
            } else {
                TransferButton::to_target(can_move_source)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(40.0);
                    if transfer_row_btn.show(ui).clicked() {
                        state.active_target = CrosstabsTargetList::Row;
                        if let Some(idx) = state.row_selected {
                            if idx < state.row_vars.len() {
                                state.row_vars.remove(idx);
                                state.row_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.row_vars.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    ui.add_space(90.0);
                    if transfer_col_btn.show(ui).clicked() {
                        state.active_target = CrosstabsTargetList::Col;
                        if let Some(idx) = state.col_selected {
                            if idx < state.col_vars.len() {
                                state.col_vars.remove(idx);
                                state.col_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.col_vars.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Center Panel: Row(s) and Column(s) lists
            ui.allocate_ui_with_layout(
                vec2(200.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // Row List
                    let row_items: Vec<VariableListItem> = state
                        .row_vars
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
                                is_selected: state.row_selected == Some(idx),
                            }
                        })
                        .collect();

                    let row_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-crosstabs-rows"),
                        vec2(200.0, 115.0),
                        "crosstabs_row_vars",
                        &row_items,
                    );

                    match row_ev {
                        ListEvent::Clicked(idx) => {
                            state.row_selected = Some(idx);
                            state.source_selected = None;
                            state.col_selected = None;
                            state.active_target = CrosstabsTargetList::Row;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.row_vars.len() {
                                state.row_vars.remove(idx);
                                state.row_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }

                    ui.add_space(8.0);

                    // Col List
                    let col_items: Vec<VariableListItem> = state
                        .col_vars
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
                                is_selected: state.col_selected == Some(idx),
                            }
                        })
                        .collect();

                    let col_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-crosstabs-cols"),
                        vec2(200.0, 115.0),
                        "crosstabs_col_vars",
                        &col_items,
                    );

                    match col_ev {
                        ListEvent::Clicked(idx) => {
                            state.col_selected = Some(idx);
                            state.source_selected = None;
                            state.row_selected = None;
                            state.active_target = CrosstabsTargetList::Col;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.col_vars.len() {
                                state.col_vars.remove(idx);
                                state.col_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }
                },
            );

            // Right Panel: SubdialogSidebar
            ui.allocate_ui_with_layout(
                vec2(90.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    if SubdialogSidebar::button(ui, t!("dialog-crosstabs-btn-cells"), true)
                        .clicked()
                    {
                        state.show_cells = true;
                    }
                    ui.add_space(6.0);
                    if SubdialogSidebar::button(ui, t!("dialog-crosstabs-btn-statistics"), true)
                        .clicked()
                    {
                        state.show_stats = true;
                    }
                },
            );
        });

        ui.add_space(10.0);

        // Bottom Action Bar
        let is_ready = !state.row_vars.is_empty() && !state.col_vars.is_empty();
        let bar_action = DialogActionBar::new(is_ready).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                action = CrosstabsDialogAction::Run {
                    rows: state.row_vars.clone(),
                    cols: state.col_vars.clone(),
                    options: state.options.clone(),
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = CrosstabsDialogAction::Paste {
                    rows: state.row_vars.clone(),
                    cols: state.col_vars.clone(),
                    options: state.options.clone(),
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = CrosstabsDialogState {
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
