pub mod state;
pub mod subdialogs;

pub use state::{TurfDialogAction, TurfDialogState};
pub use subdialogs::{render_advanced_subdialog, render_weights_subdialog};

use crate::dialogs::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    DialogSize, GroupBox, ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

pub fn render_turf_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut TurfDialogState,
) -> TurfDialogAction {
    if !state.is_open {
        return TurfDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return TurfDialogAction::None;
        }
    };

    // Render child subdialogs if open
    render_weights_subdialog(ctx, state);
    render_advanced_subdialog(ctx, state);

    let mut action = TurfDialogAction::None;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if state.selected_vars.contains(&meta.name) {
                    None
                } else {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                }
            })
        })
        .collect();

    let mut main_open = state.is_open;
    let shell = DialogShell::new(t!("dialog-turf-title"), DialogSize::Compute);

    shell.show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left Column: Source variable list + Limit size GroupBox
            ui.allocate_ui_with_layout(
                vec2(190.0, 345.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
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

                    let src_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-turf-vars-source"),
                        vec2(180.0, 200.0),
                        "turf_source_list",
                        &source_items,
                    );

                    match src_ev {
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

                    ui.add_space(6.0);

                    GroupBox::new(t!("dialog-turf-limit-group")).show(ui, |ui| {
                        ui.checkbox(
                            &mut state.limit_exceeded_abort,
                            t!("dialog-turf-limit-abort"),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(t!("dialog-turf-limit-max-ops"));
                            ui.add(
                                egui::TextEdit::singleline(&mut state.max_operations_text)
                                    .desired_width(68.0),
                            );
                        });
                    });
                },
            );

            // Center Column: Transfer buttons
            ui.vertical(|ui| {
                ui.add_space(50.0);
                let can_to_target = state.source_selected.is_some();
                let can_to_source = state.target_selected.is_some();

                if TransferButton::to_target(can_to_target).show(ui).clicked() {
                    if let Some(src_idx) = state.source_selected {
                        if let Some((_, name, _, _)) = available_vars.get(src_idx) {
                            state.selected_vars.push(name.clone());
                            state.source_selected = None;
                        }
                    }
                }

                ui.add_space(8.0);
                if TransferButton::to_source(can_to_source).show(ui).clicked() {
                    if let Some(tgt_idx) = state.target_selected {
                        if tgt_idx < state.selected_vars.len() {
                            state.selected_vars.remove(tgt_idx);
                            state.target_selected = None;
                        }
                    }
                }
            });

            // Target Column: Target variables + numeric fields + checkboxes
            ui.allocate_ui_with_layout(
                vec2(265.0, 345.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let target_items: Vec<VariableListItem> = state
                        .selected_vars
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
                                is_selected: state.target_selected == Some(idx),
                            }
                        })
                        .collect();

                    let tgt_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-turf-vars-target"),
                        vec2(255.0, 100.0),
                        "turf_target_list",
                        &target_items,
                    );

                    match tgt_ev {
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

                    ui.add_space(6.0);

                    egui::Grid::new("turf_params_grid")
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .max_col_width(205.0)
                        .show(ui, |ui| {
                            ui.label(t!("dialog-turf-max-combinations"));
                            ui.add(
                                egui::TextEdit::singleline(&mut state.max_combinations_text)
                                    .desired_width(50.0),
                            );
                            ui.end_row();

                            ui.label(t!("dialog-turf-num-to-display"));
                            ui.add(
                                egui::TextEdit::singleline(&mut state.num_to_display_text)
                                    .desired_width(50.0),
                            );
                            ui.end_row();

                            ui.label(t!("dialog-turf-min-positive-pct"));
                            ui.add(
                                egui::TextEdit::singleline(&mut state.min_positive_pct_text)
                                    .desired_width(50.0),
                            );
                            ui.end_row();

                            ui.label(t!("dialog-turf-positive-values"));
                            ui.add(
                                egui::TextEdit::singleline(&mut state.positive_values_text)
                                    .desired_width(50.0),
                            );
                            ui.end_row();

                            ui.label(t!("dialog-turf-always-required"));
                            ui.add(
                                egui::TextEdit::singleline(&mut state.always_required_text)
                                    .desired_width(50.0),
                            );
                            ui.end_row();

                            ui.label(t!("dialog-turf-removal-cycles"));
                            ui.add(
                                egui::TextEdit::singleline(&mut state.removal_cycles_text)
                                    .desired_width(50.0),
                            );
                            ui.end_row();
                        });

                    ui.add_space(6.0);
                    ui.checkbox(
                        &mut state.reach_and_frequency_chart,
                        t!("dialog-turf-reach-frequency-chart"),
                    );
                    ui.checkbox(
                        &mut state.show_only_max_combo,
                        t!("dialog-turf-show-only-max-combo"),
                    );
                },
            );

            // Right Column: SubdialogSidebar
            ui.allocate_ui_with_layout(
                vec2(105.0, 275.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(18.0);
                    if SubdialogSidebar::button(ui, t!("dialog-turf-btn-weights"), true).clicked() {
                        state.open_weights();
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-turf-btn-advanced"), true).clicked()
                    {
                        state.open_advanced();
                    }
                },
            );
        });

        let can_run = state.can_execute();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                if let Ok(options) = state.build_options() {
                    action = TurfDialogAction::Run { options };
                    state.is_open = false;
                }
            }
            ActionBarAction::Paste => {
                if let Ok(options) = state.build_options() {
                    action = TurfDialogAction::Paste { options };
                    state.is_open = false;
                }
            }
            ActionBarAction::Reset => {
                state.reset();
            }
            ActionBarAction::Cancel => {
                state.is_open = false;
            }
            ActionBarAction::Help | ActionBarAction::None => {}
        }
    });

    if !main_open {
        state.is_open = false;
    }

    action
}
