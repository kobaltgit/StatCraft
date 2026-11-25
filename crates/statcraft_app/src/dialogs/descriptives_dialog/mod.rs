pub mod options;
pub mod state;

pub use options::render_descriptives_options_subdialog;
pub use state::{DescriptivesDialogAction, DescriptivesDialogState};

use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

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

    // 1. Render Subdialogs if open
    if state.show_options {
        render_descriptives_options_subdialog(ctx, state);
    }
    if state.table_style.is_open {
        crate::dialogs::table_style_dialog::render_table_style_dialog(ctx, &mut state.table_style);
    }
    if state.bootstrap.is_open {
        crate::dialogs::bootstrap_dialog::render_bootstrap_dialog(
            ctx,
            Some(ds),
            &mut state.bootstrap,
        );
    }

    // 2. Render Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-descriptives-title")).show(ctx, &mut main_open, |ui| {
        // Main 3-column + sidebar layout
        ui.horizontal(|ui| {
            // Left Panel: Available Variables (200x240)
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
                vec2(200.0, 240.0),
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
            let can_move_right = state
                .source_selected
                .is_some_and(|idx| idx < available_vars.len());
            let can_move_left = state
                .target_selected
                .is_some_and(|idx| idx < state.selected_vars.len());

            let transfer_btn = if state.target_selected.is_some() {
                TransferButton::to_source(can_move_left)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 240.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(75.0);
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

            // Center Panel: Target Variables (200x240)
            let target_items: Vec<VariableListItem> = state
                .selected_vars
                .iter()
                .enumerate()
                .map(|(idx, name)| {
                    let meta = ds.dictionary.find_by_name(name);
                    let measure = meta
                        .map(|m| m.measure)
                        .unwrap_or(statcraft_core::Measure::Scale);
                    let label = meta.map(|m| m.label.as_str()).unwrap_or("");
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
                vec2(200.0, 240.0),
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
                vec2(94.0, 240.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(18.0);
                    if SubdialogSidebar::button(ui, t!("dialog-btn-options"), true).clicked() {
                        state.show_options = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-btn-style"), true).clicked() {
                        state.table_style.backup = Some(state.table_style.rules.clone());
                        state.table_style.backup_report = state.table_style.create_style_report;
                        state.table_style.is_open = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-btn-bootstrap"), true).clicked() {
                        state.bootstrap.open();
                    }
                },
            );
        });

        ui.add_space(8.0);
        ui.checkbox(
            &mut state.options.save_standardized,
            t!("dialog-desc-save-standardized"),
        );

        // Bottom Action Bar
        let can_run = !state.selected_vars.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);
        let table_styles = state.table_style.rules_to_json();
        let create_style_report = state.table_style.create_style_report;
        let bootstrap = state.bootstrap.to_json();

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = DescriptivesDialogAction::Run {
                    vars: state.selected_vars.clone(),
                    options: state.options.clone(),
                    table_styles,
                    create_style_report,
                    bootstrap,
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = DescriptivesDialogAction::Paste {
                    vars: state.selected_vars.clone(),
                    options: state.options.clone(),
                    bootstrap,
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
