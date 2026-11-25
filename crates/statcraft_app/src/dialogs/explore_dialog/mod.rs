pub mod options;
pub mod plots;
pub mod state;
pub mod statistics;

pub use state::{
    ExploreBoxplotMode, ExploreDialogAction, ExploreDialogState, ExploreDisplayMode,
    ExploreMissingMode, ExploreSpreadLevelMode, ExploreSpreadPower,
};

use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use crate::theme::Tokens;
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

fn build_target_items<'a>(
    ds: &'a DataSet,
    names: &'a [String],
    selected: Option<usize>,
    default_measure: statcraft_core::Measure,
) -> Vec<VariableListItem<'a>> {
    names
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let meta = ds.dictionary.find_by_name(name);
            let measure = meta.map(|m| m.measure).unwrap_or(default_measure);
            let label = meta.map(|m| m.label.as_str()).unwrap_or("");
            VariableListItem {
                name,
                display_text: format_var_display(name, label),
                measure,
                is_selected: selected == Some(idx),
            }
        })
        .collect()
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

    if state.show_statistics {
        statistics::render_statistics_subdialog(ctx, state);
    }
    if state.show_plots {
        plots::render_plots_subdialog(ctx, state);
    }
    if state.show_options {
        options::render_options_subdialog(ctx, state);
    }
    if state.bootstrap.is_open {
        crate::dialogs::bootstrap_dialog::render_bootstrap_dialog(
            ctx,
            Some(ds),
            &mut state.bootstrap,
        );
    }

    let mut action = ExploreDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let in_deps = state.selected_deps.contains(&meta.name);
                let in_factors = state.selected_factors.contains(&meta.name);
                let is_label = state.label_var.as_deref() == Some(meta.name.as_str());
                if !in_deps && !in_factors && !is_label {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

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
            let can_move_right_source = state
                .source_selected
                .is_some_and(|i| i < available_vars.len());
            let can_move_left_dep = state
                .dep_selected
                .is_some_and(|i| i < state.selected_deps.len());
            let can_move_left_factor = state
                .factor_selected
                .is_some_and(|i| i < state.selected_factors.len());
            let has_label_var = state.label_var.is_some();

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
            let transfer_label_btn = if has_label_var && !can_move_right_source {
                TransferButton::to_source(true)
            } else {
                TransferButton::to_target(can_move_right_source)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(38.0);
                    if transfer_dep_btn.show(ui).clicked() {
                        if let Some(idx) = state.dep_selected {
                            if idx < state.selected_deps.len() {
                                state.selected_deps.remove(idx);
                                state.dep_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.selected_deps.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    ui.add_space(74.0);
                    if transfer_factor_btn.show(ui).clicked() {
                        if let Some(idx) = state.factor_selected {
                            if idx < state.selected_factors.len() {
                                state.selected_factors.remove(idx);
                                state.factor_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.selected_factors.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    ui.add_space(64.0);
                    if transfer_label_btn.show(ui).clicked() {
                        if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.label_var = Some(name.clone());
                                state.source_selected = None;
                            }
                        } else if state.label_var.is_some() {
                            state.label_var = None;
                        }
                    }
                },
            );

            // Center Panel: Dependent List, Factor List, Label Cases by
            ui.allocate_ui_with_layout(
                vec2(200.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let dep_items = build_target_items(
                        ds,
                        &state.selected_deps,
                        state.dep_selected,
                        statcraft_core::Measure::Scale,
                    );
                    let dep_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-explore-dependent-list"),
                        vec2(200.0, 96.0),
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

                    ui.add_space(6.0);

                    let factor_items = build_target_items(
                        ds,
                        &state.selected_factors,
                        state.factor_selected,
                        statcraft_core::Measure::Nominal,
                    );
                    let factor_ev = render_variable_list_panel(
                        ui,
                        t!("dialog-explore-factor-list"),
                        vec2(200.0, 82.0),
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

                    ui.add_space(6.0);

                    ui.label(t!("dialog-explore-label-cases-by"));
                    egui::Frame::canvas(ui.style())
                        .fill(Tokens::bg_panel())
                        .stroke(egui::Stroke::new(1.0_f32, Tokens::border()))
                        .show(ui, |ui| {
                            ui.set_min_size(vec2(196.0, 26.0));
                            ui.set_max_size(vec2(196.0, 26.0));
                            let label_display = match &state.label_var {
                                Some(name) => {
                                    let lbl = ds
                                        .dictionary
                                        .find_by_name(name)
                                        .map(|m| m.label.as_str())
                                        .unwrap_or("");
                                    format_var_display(name, lbl)
                                }
                                None => "-".to_string(),
                            };
                            if ui
                                .selectable_label(false, label_display.as_str())
                                .double_clicked()
                            {
                                state.label_var = None;
                            }
                        });
                },
            );

            // Right Column: Subdialog Sidebar
            ui.allocate_ui_with_layout(
                vec2(94.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(18.0);
                    let stats_enabled = state.display_mode != ExploreDisplayMode::Plots;
                    if SubdialogSidebar::button(ui, t!("dialog-btn-statistics"), stats_enabled)
                        .clicked()
                    {
                        state.show_statistics = true;
                    }
                    let plots_enabled = state.display_mode != ExploreDisplayMode::Statistics;
                    if SubdialogSidebar::button(ui, t!("dialog-explore-plots-btn"), plots_enabled)
                        .clicked()
                    {
                        state.show_plots = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-btn-options"), true).clicked() {
                        state.show_options = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-btn-bootstrap"), true).clicked() {
                        state.bootstrap.open();
                    }
                },
            );
        });

        // Display mode group
        GroupBox::new(t!("dialog-explore-display-group")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut state.display_mode,
                    ExploreDisplayMode::Both,
                    t!("dialog-explore-display-both"),
                );
                ui.add_space(16.0);
                ui.radio_value(
                    &mut state.display_mode,
                    ExploreDisplayMode::Statistics,
                    t!("dialog-explore-display-stats"),
                );
                ui.add_space(16.0);
                ui.radio_value(
                    &mut state.display_mode,
                    ExploreDisplayMode::Plots,
                    t!("dialog-explore-display-plots"),
                );
            });
        });

        // Bottom Action Bar
        let can_run = !state.selected_deps.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                let include_stats = state.display_mode != ExploreDisplayMode::Plots;
                let include_plots = state.display_mode != ExploreDisplayMode::Statistics;
                let (spread_level_mode, spread_level_power) = state.spread_level_params();
                let display_mode = match state.display_mode {
                    ExploreDisplayMode::Both => "both",
                    ExploreDisplayMode::Statistics => "statistics",
                    ExploreDisplayMode::Plots => "plots",
                };
                let boxplot_mode = match state.boxplot_mode {
                    ExploreBoxplotMode::FactorLevelsTogether => "factor_levels_together",
                    ExploreBoxplotMode::DependentsTogether => "dependents_together",
                    ExploreBoxplotMode::None => "none",
                };

                action = ExploreDialogAction::Run {
                    dependent_vars: state.selected_deps.clone(),
                    factor_vars: state.selected_factors.clone(),
                    label_var: state.label_var.clone(),
                    compute_descriptives: include_stats && state.compute_descriptives,
                    compute_m_estimators: include_stats && state.compute_m_estimators,
                    compute_percentiles: include_stats && state.compute_percentiles,
                    compute_normality: include_plots && state.compute_normality,
                    compute_extremes: include_stats && state.compute_extremes,
                    spread_level_mode,
                    spread_level_power,
                    display_mode,
                    boxplot_mode,
                    plot_stem_leaf: include_plots && state.plot_stem_leaf,
                    plot_histogram: include_plots && state.plot_histogram,
                    confidence_level: state.confidence_level(),
                    missing: state.missing_mode.as_str(),
                    bootstrap: state.bootstrap.to_json(),
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = ExploreDialogAction::Paste {
                    options: state.to_explore_options(),
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
