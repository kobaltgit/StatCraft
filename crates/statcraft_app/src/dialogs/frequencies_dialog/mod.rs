pub mod charts_subdialog;
pub mod format_subdialog;
pub mod state;
pub mod statistics_subdialog;

pub use charts_subdialog::*;
pub use format_subdialog::*;
pub use state::*;
pub use statistics_subdialog::*;

use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

pub fn render_frequencies_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut FrequenciesDialogState,
) -> FrequenciesDialogAction {
    if !state.is_open {
        return FrequenciesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return FrequenciesDialogAction::None;
        }
    };

    // Subdialogs
    if state.show_statistics {
        render_statistics_subdialog(ctx, state);
    }
    if state.show_charts {
        render_charts_subdialog(ctx, state);
    }
    if state.show_format {
        render_format_subdialog(ctx, state);
    }
    if state.table_style.is_open {
        crate::dialogs::table_style_dialog::render_table_style_dialog(ctx, &mut state.table_style);
    }

    let mut action = FrequenciesDialogAction::None;
    let mut close_dialog = false;

    // Available variables
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
    DialogShell::standard(t!("dialog-frequencies-title")).show(ctx, &mut main_open, |ui| {
        // Main 3-column + sidebar
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
                "freq_source_vars",
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
                "freq_target_vars",
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
                    if SubdialogSidebar::button(ui, t!("dialog-btn-statistics"), true).clicked() {
                        state.stats_backup = Some(state.stats.clone());
                        state.show_statistics = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-btn-charts"), true).clicked() {
                        state.charts_backup = Some(state.charts.clone());
                        state.show_charts = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-btn-format"), true).clicked() {
                        state.format_backup = Some(state.format.clone());
                        state.show_format = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-btn-style"), true).clicked() {
                        state.table_style.backup = Some(state.table_style.rules.clone());
                        state.table_style.backup_report = state.table_style.create_style_report;
                        state.table_style.is_open = true;
                    }
                    let _ = SubdialogSidebar::button(ui, t!("dialog-btn-bootstrap"), false);
                },
            );
        });

        ui.add_space(8.0);
        ui.checkbox(
            &mut state.display_tables,
            t!("dialog-frequencies-display-tables"),
        );
        ui.add_space(2.0);
        ui.checkbox(&mut state.apa_tables, t!("dialog-freq-apa-tables"));

        // Bottom Action Bar
        let can_run = !state.selected_vars.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        let order = if state.format.order != statcraft_algo::CategoryOrder::AValue {
            Some(state.format.order.as_str().to_string())
        } else {
            None
        };
        let multiple_vars_mode =
            if state.format.multiple_vars_mode != statcraft_algo::MultipleVarsMode::Compare {
                Some(state.format.multiple_vars_mode.as_str().to_string())
            } else {
                None
            };
        let suppress_tables_limit = state.format.suppress_tables_limit();

        let table_styles = state.table_style.rules_to_json();
        let create_style_report = state.table_style.create_style_report;

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = FrequenciesDialogAction::Run {
                    vars: state.selected_vars.clone(),
                    display_tables: state.display_tables,
                    apa_tables: state.apa_tables,
                    statistics: state.stats.to_statistics_json(),
                    percentiles: state.stats.to_percentiles_json(),
                    group_midpoints: state.stats.group_midpoints,
                    charts: state.charts.to_json(),
                    order,
                    multiple_vars_mode,
                    suppress_tables_limit,
                    table_styles,
                    create_style_report,
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = FrequenciesDialogAction::Paste {
                    vars: state.selected_vars.clone(),
                    display_tables: state.display_tables,
                    apa_tables: state.apa_tables,
                    statistics: state.stats.to_statistics_json(),
                    percentiles: state.stats.to_percentiles_json(),
                    group_midpoints: state.stats.group_midpoints,
                    charts: state.charts.to_json(),
                    order,
                    multiple_vars_mode,
                    suppress_tables_limit,
                    table_styles,
                    create_style_report,
                };
            }
            ActionBarAction::Reset => {
                state.reset();
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
