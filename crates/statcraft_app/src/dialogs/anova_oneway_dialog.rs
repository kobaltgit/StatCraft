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
pub struct OneWayAnovaDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub factor_var: Option<String>,
    pub descriptives: bool,
    pub homogeneity: bool,
    pub ci_level_str: String,
    pub show_options: bool,
}

impl Default for OneWayAnovaDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            factor_var: None,
            descriptives: true,
            homogeneity: true,
            ci_level_str: "95".to_string(),
            show_options: false,
        }
    }
}

pub enum OneWayAnovaDialogAction {
    Run {
        dependent_vars: Vec<String>,
        factor_var: String,
        descriptives: bool,
        homogeneity: bool,
        ci: f64,
    },
    Paste {
        dependent_vars: Vec<String>,
        factor_var: String,
        descriptives: bool,
        homogeneity: bool,
        ci: f64,
    },
    None,
}

pub fn render_anova_oneway_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut OneWayAnovaDialogState,
) -> OneWayAnovaDialogAction {
    if !state.is_open {
        return OneWayAnovaDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return OneWayAnovaDialogAction::None;
        }
    };

    let mut action = OneWayAnovaDialogAction::None;
    let mut close_dialog = false;

    // Build available variables
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_in_selected = state.selected_vars.contains(&meta.name);
                let is_factor = state
                    .factor_var
                    .as_ref()
                    .map(|f| f == &meta.name)
                    .unwrap_or(false);
                if !is_in_selected && !is_factor {
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
        DialogShell::subdialog(t!("dialog-anova-options-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-anova-options-title")).show(ui, |ui| {
                ui.checkbox(&mut state.descriptives, t!("dialog-anova-descriptives"));
                ui.add_space(4.0);
                ui.checkbox(&mut state.homogeneity, t!("dialog-anova-homogeneity"));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-anova-ci-level"));
                    ui.add(egui::TextEdit::singleline(&mut state.ci_level_str).desired_width(38.0));
                    ui.label("%");
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

    // 2. Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-anova-oneway-title")).show(ctx, &mut main_open, |ui| {
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
                "anova_source_vars",
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

            // Transfer Buttons Column
            let can_move_right_dep = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();
            let can_move_left_dep = state.target_selected.is_some()
                && state.target_selected.unwrap() < state.selected_vars.len();
            let can_move_factor = can_move_right_dep;

            let transfer_dep_btn = if state.target_selected.is_some() {
                TransferButton::to_source(can_move_left_dep)
            } else {
                TransferButton::to_target(can_move_right_dep)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(50.0);
                    if transfer_dep_btn.show(ui).clicked() {
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

                    ui.add_space(80.0);
                    let transfer_factor_btn = TransferButton::to_target(can_move_factor);
                    if transfer_factor_btn.show(ui).clicked() {
                        if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.factor_var = Some(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Center Panel: Dependent List + Factor
            ui.allocate_ui_with_layout(
                vec2(200.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // Dependent Variables List
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
                        t!("dialog-anova-dependent-list"),
                        vec2(200.0, 160.0),
                        "anova_target_vars",
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

                    ui.add_space(8.0);

                    // Factor Variable Box
                    ui.label(t!("dialog-anova-factor"));
                    egui::Frame::canvas(ui.style())
                        .fill(Tokens::BG_PANEL)
                        .stroke(egui::Stroke::new(1.0_f32, Tokens::BORDER))
                        .show(ui, |ui| {
                            ui.set_min_size(vec2(196.0, 32.0));
                            ui.set_max_size(vec2(196.0, 32.0));

                            ui.horizontal(|ui| {
                                ui.add_space(4.0);
                                if let Some(f_name) = &state.factor_var {
                                    let res = ui.selectable_label(false, f_name);
                                    if res.double_clicked() {
                                        state.factor_var = None;
                                    }
                                } else {
                                    ui.label(egui::RichText::new("-").color(Tokens::TEXT_MUTED));
                                }
                            });
                        });
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
        let can_run = !state.selected_vars.is_empty() && state.factor_var.is_some();
        let ci_val = state
            .ci_level_str
            .parse::<f64>()
            .map(|v| (v / 100.0).clamp(0.01, 0.999))
            .unwrap_or(0.95);

        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                if let Some(fact) = &state.factor_var {
                    close_dialog = true;
                    action = OneWayAnovaDialogAction::Run {
                        dependent_vars: state.selected_vars.clone(),
                        factor_var: fact.clone(),
                        descriptives: state.descriptives,
                        homogeneity: state.homogeneity,
                        ci: ci_val,
                    };
                }
            }
            ActionBarAction::Paste => {
                if let Some(fact) = &state.factor_var {
                    close_dialog = true;
                    action = OneWayAnovaDialogAction::Paste {
                        dependent_vars: state.selected_vars.clone(),
                        factor_var: fact.clone(),
                        descriptives: state.descriptives,
                        homogeneity: state.homogeneity,
                        ci: ci_val,
                    };
                }
            }
            ActionBarAction::Reset => {
                *state = OneWayAnovaDialogState {
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
