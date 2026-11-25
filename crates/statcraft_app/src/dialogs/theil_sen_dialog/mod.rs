pub mod state;
pub mod subdialogs;

pub use state::{TheilSenDialogAction, TheilSenDialogState};

use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use crate::theme::Tokens;
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

pub fn render_theil_sen_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut TheilSenDialogState,
) -> TheilSenDialogAction {
    if !state.is_open {
        return TheilSenDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return TheilSenDialogAction::None;
        }
    };

    if state.show_options {
        subdialogs::render_options_subdialog(ctx, state);
    }
    if state.show_plots {
        subdialogs::render_plots_subdialog(ctx, state);
    }
    if state.show_save {
        subdialogs::render_save_subdialog(ctx, state);
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

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_dep = state.dep_var.as_deref() == Some(meta.name.as_str());
                let is_indep = state.indep_var.as_deref() == Some(meta.name.as_str());
                if !is_dep && !is_indep && meta.var_type.is_numeric() {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut action = TheilSenDialogAction::None;
    let mut close_dialog = false;
    let mut main_open = true;

    DialogShell::standard(t!("dialog-theil-sen-title")).show(ctx, &mut main_open, |ui| {
        let ok_enabled = state.dep_var.is_some() && state.indep_var.is_some();

        ui.horizontal(|ui| {
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
                t!("dialog-variables-source"),
                vec2(180.0, 290.0),
                "theil_sen_source_vars",
                &source_items,
            );

            match src_event {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_dep_selected = false;
                    state.target_indep_selected = false;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        if state.dep_var.is_none() {
                            state.dep_var = Some(name.clone());
                        } else if state.indep_var.is_none() {
                            state.indep_var = Some(name.clone());
                        }
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            ui.add_space(6.0);

            ui.vertical(|ui| {
                ui.label(t!("dialog-regression-label-dependent"));
                ui.horizontal(|ui| {
                    let to_dep = state.source_selected.is_some() && state.dep_var.is_none();
                    let from_dep = state.target_dep_selected && state.dep_var.is_some();
                    let btn = if from_dep {
                        TransferButton::to_source(true)
                    } else {
                        TransferButton::to_target(to_dep)
                    };

                    if btn.show(ui).clicked() {
                        if to_dep {
                            if let Some(idx) = state.source_selected {
                                if let Some((_, name, _, _)) = available_vars.get(idx) {
                                    state.dep_var = Some(name.clone());
                                    state.source_selected = None;
                                }
                            }
                        } else if from_dep {
                            state.dep_var = None;
                            state.target_dep_selected = false;
                        }
                    }

                    render_single_var_box(
                        ui,
                        ds,
                        &state.dep_var,
                        &mut state.target_dep_selected,
                        &mut state.source_selected,
                        &mut state.target_indep_selected,
                        t!("dialog-regression-placeholder-dep"),
                    );
                });

                ui.add_space(12.0);

                ui.label(t!("dialog-theil-sen-label-independent"));
                ui.horizontal(|ui| {
                    let to_indep = state.source_selected.is_some() && state.indep_var.is_none();
                    let from_indep = state.target_indep_selected && state.indep_var.is_some();
                    let btn = if from_indep {
                        TransferButton::to_source(true)
                    } else {
                        TransferButton::to_target(to_indep)
                    };

                    if btn.show(ui).clicked() {
                        if to_indep {
                            if let Some(idx) = state.source_selected {
                                if let Some((_, name, _, _)) = available_vars.get(idx) {
                                    state.indep_var = Some(name.clone());
                                    state.source_selected = None;
                                }
                            }
                        } else if from_indep {
                            state.indep_var = None;
                            state.target_indep_selected = false;
                        }
                    }

                    render_single_var_box(
                        ui,
                        ds,
                        &state.indep_var,
                        &mut state.target_indep_selected,
                        &mut state.source_selected,
                        &mut state.target_dep_selected,
                        t!("dialog-theil-sen-placeholder-indep"),
                    );
                });
            });

            ui.add_space(6.0);

            ui.allocate_ui_with_layout(
                vec2(90.0, 290.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(20.0);
                    if SubdialogSidebar::button(ui, t!("dialog-theil-sen-btn-options"), true)
                        .clicked()
                    {
                        state.show_options = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-theil-sen-btn-plots"), true)
                        .clicked()
                    {
                        state.show_plots = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-theil-sen-btn-save"), true).clicked()
                    {
                        state.show_save = true;
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

        ui.add_space(10.0);

        let bar_action = DialogActionBar::new(ok_enabled).show(ui);
        let parsed_ci = state
            .ci_level_str
            .parse::<f64>()
            .map(|p| p / 100.0)
            .unwrap_or(0.95)
            .clamp(0.5, 0.9999);
        let bootstrap = state.bootstrap.to_json();

        match bar_action {
            ActionBarAction::Ok => {
                if let (Some(dep), Some(indep)) = (&state.dep_var, &state.indep_var) {
                    action = TheilSenDialogAction::Run {
                        dependent_var: dep.clone(),
                        independent_var: indep.clone(),
                        ci: parsed_ci,
                        mann_kendall: state.mann_kendall,
                        include_scatter_plot: state.include_scatter_plot,
                        include_residual_plot: state.include_residual_plot,
                        save_predicted: state.save_predicted,
                        save_residuals: state.save_residuals,
                        table_styles: state.table_style.rules.clone(),
                        bootstrap,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Paste => {
                if let (Some(dep), Some(indep)) = (&state.dep_var, &state.indep_var) {
                    action = TheilSenDialogAction::Paste {
                        dependent_var: dep.clone(),
                        independent_var: indep.clone(),
                        ci: parsed_ci,
                        mann_kendall: state.mann_kendall,
                        include_scatter_plot: state.include_scatter_plot,
                        include_residual_plot: state.include_residual_plot,
                        save_predicted: state.save_predicted,
                        save_residuals: state.save_residuals,
                        bootstrap,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Reset => {
                *state = TheilSenDialogState::default();
                state.is_open = true;
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

fn render_single_var_box(
    ui: &mut egui::Ui,
    ds: &DataSet,
    var_name_opt: &Option<String>,
    this_selected: &mut bool,
    source_selected: &mut Option<usize>,
    other_selected: &mut bool,
    placeholder: String,
) {
    let box_size = vec2(170.0, 26.0);
    egui::Frame::group(ui.style())
        .fill(Tokens::bg_panel())
        .stroke(egui::Stroke::new(1.0_f32, Tokens::border()))
        .show(ui, |ui| {
            ui.set_min_size(box_size);
            ui.set_max_size(box_size);
            if let Some(ref var_name) = var_name_opt {
                let label = ds
                    .dictionary
                    .find_by_name(var_name)
                    .map(|m| m.label.as_str())
                    .unwrap_or("");
                let label_str = format_var_display(var_name, label);
                if ui.selectable_label(*this_selected, label_str).clicked() {
                    *this_selected = !*this_selected;
                    *source_selected = None;
                    *other_selected = false;
                }
            } else {
                ui.colored_label(Tokens::text_muted(), placeholder);
            }
        });
}
