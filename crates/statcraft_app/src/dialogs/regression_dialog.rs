use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use crate::theme::Tokens;
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct LinearRegressionDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_var: Option<String>,
    pub indep_vars: Vec<String>,
    pub target_indep_selected: Option<usize>,
    pub target_dep_selected: bool,
    pub ci_level_str: String,
    pub collinearity: bool,
    pub show_statistics: bool,
}

impl Default for LinearRegressionDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            dep_var: None,
            indep_vars: Vec::new(),
            target_indep_selected: None,
            target_dep_selected: false,
            ci_level_str: "95".to_string(),
            collinearity: true,
            show_statistics: false,
        }
    }
}

pub enum LinearRegressionDialogAction {
    Run {
        dependent_var: String,
        independent_vars: Vec<String>,
        ci: f64,
        collinearity: bool,
    },
    Paste {
        dependent_var: String,
        independent_vars: Vec<String>,
        ci: f64,
        collinearity: bool,
    },
    None,
}

pub fn render_linear_regression_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut LinearRegressionDialogState,
) -> LinearRegressionDialogAction {
    if !state.is_open {
        return LinearRegressionDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return LinearRegressionDialogAction::None;
        }
    };

    let mut action = LinearRegressionDialogAction::None;
    let mut close_dialog = false;

    // Filter available variables
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_dep = state.dep_var.as_ref() == Some(&meta.name);
                let is_indep = state.indep_vars.contains(&meta.name);
                if !is_dep && !is_indep {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    // 1. Subdialog: Statistics...
    if state.show_statistics {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-regression-statistics-title")).show(
            ctx,
            &mut sub_open,
            |ui| {
                GroupBox::new(t!("dialog-regression-group-regression-coeffs")).show(ui, |ui| {
                    ui.checkbox(
                        &mut state.collinearity,
                        t!("dialog-regression-stat-collinearity"),
                    );
                });

                ui.add_space(8.0);

                GroupBox::new(t!("dialog-regression-group-confidence-intervals")).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-regression-label-ci-level"));
                        ui.add(
                            egui::TextEdit::singleline(&mut state.ci_level_str).desired_width(45.0),
                        );
                        ui.label("%");
                    });
                });

                ui.add_space(12.0);

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

    // 2. Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-regression-linear-title")).show(ctx, &mut main_open, |ui| {
        let ok_enabled = state.dep_var.is_some() && !state.indep_vars.is_empty();

        ui.horizontal(|ui| {
            // Column 1: Source variables list
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
                "regr_source_vars",
                &source_items,
            );

            match src_event {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_dep_selected = false;
                    state.target_indep_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        if state.dep_var.is_none() {
                            state.dep_var = Some(name.clone());
                        } else {
                            state.indep_vars.push(name.clone());
                        }
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            ui.add_space(6.0);

            // Column 2: Target lists and transfer buttons
            ui.vertical(|ui| {
                // Row 2A: Dependent Variable
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

                    let dep_box_size = vec2(170.0, 26.0);
                    egui::Frame::group(ui.style())
                        .fill(Tokens::BG_PANEL)
                        .stroke(egui::Stroke::new(1.0_f32, Tokens::BORDER))
                        .show(ui, |ui| {
                            ui.set_min_size(dep_box_size);
                            ui.set_max_size(dep_box_size);
                            if let Some(ref dep_name) = state.dep_var {
                                let mut is_sel = state.target_dep_selected;
                                let label = ds
                                    .dictionary
                                    .find_by_name(dep_name)
                                    .map(|m| m.label.as_str())
                                    .unwrap_or("");
                                let label_str = format_var_display(dep_name, label);
                                if ui.selectable_label(is_sel, label_str).clicked() {
                                    is_sel = !is_sel;
                                    state.target_dep_selected = is_sel;
                                    state.source_selected = None;
                                    state.target_indep_selected = None;
                                }
                            } else {
                                ui.colored_label(
                                    Tokens::TEXT_MUTED,
                                    t!("dialog-regression-placeholder-dep"),
                                );
                            }
                        });
                });

                ui.add_space(8.0);

                // Row 2B: Independent Variables
                ui.label(t!("dialog-regression-label-independents"));
                ui.horizontal(|ui| {
                    let to_indep = state.source_selected.is_some();
                    let from_indep = state.target_indep_selected.is_some();
                    let btn = if from_indep {
                        TransferButton::to_source(true)
                    } else {
                        TransferButton::to_target(to_indep)
                    };

                    if btn.show(ui).clicked() {
                        if to_indep {
                            if let Some(idx) = state.source_selected {
                                if let Some((_, name, _, _)) = available_vars.get(idx) {
                                    state.indep_vars.push(name.clone());
                                    state.source_selected = None;
                                }
                            }
                        } else if from_indep {
                            if let Some(sel_idx) = state.target_indep_selected {
                                if sel_idx < state.indep_vars.len() {
                                    state.indep_vars.remove(sel_idx);
                                    state.target_indep_selected = None;
                                }
                            }
                        }
                    }

                    let indep_items: Vec<VariableListItem> = state
                        .indep_vars
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
                                is_selected: state.target_indep_selected == Some(idx),
                            }
                        })
                        .collect();

                    let target_event = render_variable_list_panel(
                        ui,
                        t!("dialog-regression-label-independents"),
                        vec2(170.0, 185.0),
                        "regr_indep_vars",
                        &indep_items,
                    );

                    match target_event {
                        ListEvent::Clicked(idx) => {
                            state.target_indep_selected = Some(idx);
                            state.source_selected = None;
                            state.target_dep_selected = false;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.indep_vars.len() {
                                state.indep_vars.remove(idx);
                                state.target_indep_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-regression-label-method"));
                    ui.label(t!("dialog-regression-method-enter"));
                });
            });

            ui.add_space(6.0);

            // Column 3: Subdialog buttons sidebar
            ui.allocate_ui_with_layout(
                vec2(90.0, 290.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(20.0);
                    if SubdialogSidebar::button(ui, t!("dialog-regression-btn-statistics"), true)
                        .clicked()
                    {
                        state.show_statistics = true;
                    }
                },
            );
        });

        ui.add_space(10.0);

        // Bottom action bar
        let bar_action = DialogActionBar::new(ok_enabled).show(ui);
        let parsed_ci = state
            .ci_level_str
            .parse::<f64>()
            .map(|p| p / 100.0)
            .unwrap_or(0.95)
            .clamp(0.5, 0.9999);

        match bar_action {
            ActionBarAction::Ok => {
                if let Some(dep) = &state.dep_var {
                    action = LinearRegressionDialogAction::Run {
                        dependent_var: dep.clone(),
                        independent_vars: state.indep_vars.clone(),
                        ci: parsed_ci,
                        collinearity: state.collinearity,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Paste => {
                if let Some(dep) = &state.dep_var {
                    action = LinearRegressionDialogAction::Paste {
                        dependent_var: dep.clone(),
                        independent_vars: state.indep_vars.clone(),
                        ci: parsed_ci,
                        collinearity: state.collinearity,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Reset => {
                *state = LinearRegressionDialogState::default();
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
