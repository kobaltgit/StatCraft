use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct OneSampleTTestDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub test_value_str: String,
    pub ci_level_str: String,
    pub show_options: bool,
}

impl Default for OneSampleTTestDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            test_value_str: "0".to_string(),
            ci_level_str: "95".to_string(),
            show_options: false,
        }
    }
}

pub enum OneSampleTTestDialogAction {
    Run {
        vars: Vec<String>,
        test_value: f64,
        ci: f64,
    },
    Paste {
        vars: Vec<String>,
        test_value: f64,
        ci: f64,
    },
    None,
}

pub fn render_ttest_onesample_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut OneSampleTTestDialogState,
) -> OneSampleTTestDialogAction {
    if !state.is_open {
        return OneSampleTTestDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return OneSampleTTestDialogAction::None;
        }
    };

    let mut action = OneSampleTTestDialogAction::None;
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

    // 1. Subdialog: Options
    if state.show_options {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-anova-options-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-anova-options-title")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-ttest-ci-percentage"));
                    ui.add(egui::TextEdit::singleline(&mut state.ci_level_str).desired_width(50.0));
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
    DialogShell::standard(t!("dialog-ttest-onesample-title")).show(ctx, &mut main_open, |ui| {
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
                "onesample_source_vars",
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

            // Center Panel: Target Variables + Test Value (200x260)
            ui.allocate_ui_with_layout(
                vec2(200.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
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
                        t!("dialog-ttest-test-variables"),
                        vec2(200.0, 210.0),
                        "onesample_target_vars",
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
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-ttest-test-value"));
                        ui.add(
                            egui::TextEdit::singleline(&mut state.test_value_str)
                                .desired_width(70.0),
                        );
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

        // Parse values
        let parsed_test_val = state.test_value_str.trim().parse::<f64>().unwrap_or(0.0);
        let parsed_ci_pct = state.ci_level_str.trim().parse::<f64>().unwrap_or(95.0);
        let ci_ratio = (parsed_ci_pct / 100.0).clamp(0.50, 0.999);

        // Bottom Action Bar
        let can_run = !state.selected_vars.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = OneSampleTTestDialogAction::Run {
                    vars: state.selected_vars.clone(),
                    test_value: parsed_test_val,
                    ci: ci_ratio,
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = OneSampleTTestDialogAction::Paste {
                    vars: state.selected_vars.clone(),
                    test_value: parsed_test_val,
                    ci: ci_ratio,
                };
            }
            ActionBarAction::Reset => {
                state.selected_vars.clear();
                state.source_selected = None;
                state.target_selected = None;
                state.test_value_str = "0".to_string();
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
