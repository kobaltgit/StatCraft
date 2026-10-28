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
pub struct IndependentSamplesTTestDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub grouping_var: Option<String>,
    pub group1_val: String,
    pub group2_val: String,
    pub ci_level_str: String,
    pub is_define_groups_open: bool,
    pub temp_group1: String,
    pub temp_group2: String,
    pub show_options: bool,
}

impl Default for IndependentSamplesTTestDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            grouping_var: None,
            group1_val: "1".to_string(),
            group2_val: "2".to_string(),
            ci_level_str: "95".to_string(),
            is_define_groups_open: false,
            temp_group1: "1".to_string(),
            temp_group2: "2".to_string(),
            show_options: false,
        }
    }
}

pub enum IndependentSamplesTTestDialogAction {
    Run {
        vars: Vec<String>,
        grouping_var: String,
        group1: String,
        group2: String,
        ci: f64,
    },
    Paste {
        vars: Vec<String>,
        grouping_var: String,
        group1: String,
        group2: String,
        ci: f64,
    },
    None,
}

pub fn render_ttest_indep_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut IndependentSamplesTTestDialogState,
) -> IndependentSamplesTTestDialogAction {
    if !state.is_open {
        return IndependentSamplesTTestDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return IndependentSamplesTTestDialogAction::None;
        }
    };

    let mut action = IndependentSamplesTTestDialogAction::None;
    let mut close_dialog = false;

    // Subdialog 1: Define Groups
    super::ttest_indep_define_groups::render_define_groups_subdialog(ctx, state);

    // Subdialog 2: Options
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

    // Build available variables
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_in_selected = state.selected_vars.contains(&meta.name);
                let is_grouping = state
                    .grouping_var
                    .as_ref()
                    .map(|g| g == &meta.name)
                    .unwrap_or(false);
                if !is_in_selected && !is_grouping {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    // Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-ttest-indep-title")).show(ctx, &mut main_open, |ui| {
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
                "ttest_indep_source_vars",
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
            let can_move_right_test = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();
            let can_move_left_test = state.target_selected.is_some()
                && state.target_selected.unwrap() < state.selected_vars.len();
            let can_move_grp = can_move_right_test;

            let transfer_test_btn = if state.target_selected.is_some() {
                TransferButton::to_source(can_move_left_test)
            } else {
                TransferButton::to_target(can_move_right_test)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(50.0);
                    if transfer_test_btn.show(ui).clicked() {
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
                    let transfer_grp_btn = TransferButton::to_target(can_move_grp);
                    if transfer_grp_btn.show(ui).clicked() {
                        if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.grouping_var = Some(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // Center Panel: Test Variables + Grouping Variable
            ui.allocate_ui_with_layout(
                vec2(200.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // Test Variables
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
                        vec2(200.0, 160.0),
                        "ttest_indep_target_vars",
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

                    // Grouping Variable Box
                    ui.label(t!("dialog-ttest-indep-grouping-var"));
                    egui::Frame::canvas(ui.style())
                        .fill(Tokens::BG_PANEL)
                        .stroke(egui::Stroke::new(1.0_f32, Tokens::BORDER))
                        .show(ui, |ui| {
                            ui.set_min_size(vec2(196.0, 32.0));
                            ui.set_max_size(vec2(196.0, 32.0));

                            let grp_display = match &state.grouping_var {
                                Some(g) => {
                                    format!("{}({} {})", g, state.group1_val, state.group2_val)
                                }
                                None => "-".to_string(),
                            };
                            let res = ui.selectable_label(false, grp_display.as_str());
                            if res.double_clicked() {
                                state.grouping_var = None;
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
                    let has_grp = state.grouping_var.is_some();
                    if SubdialogSidebar::button(
                        ui,
                        t!("dialog-ttest-indep-define-groups-btn"),
                        has_grp,
                    )
                    .clicked()
                    {
                        state.temp_group1 = state.group1_val.clone();
                        state.temp_group2 = state.group2_val.clone();
                        state.is_define_groups_open = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-btn-options"), true).clicked() {
                        state.show_options = true;
                    }
                },
            );
        });

        // Bottom Action Bar
        let can_run = !state.selected_vars.is_empty() && state.grouping_var.is_some();
        let parsed_ci_pct = state.ci_level_str.trim().parse::<f64>().unwrap_or(95.0);
        let ci_ratio = (parsed_ci_pct / 100.0).clamp(0.50, 0.999);

        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                if let Some(ref grp_var) = state.grouping_var {
                    close_dialog = true;
                    action = IndependentSamplesTTestDialogAction::Run {
                        vars: state.selected_vars.clone(),
                        grouping_var: grp_var.clone(),
                        group1: state.group1_val.clone(),
                        group2: state.group2_val.clone(),
                        ci: ci_ratio,
                    };
                }
            }
            ActionBarAction::Paste => {
                if let Some(ref grp_var) = state.grouping_var {
                    close_dialog = true;
                    action = IndependentSamplesTTestDialogAction::Paste {
                        vars: state.selected_vars.clone(),
                        grouping_var: grp_var.clone(),
                        group1: state.group1_val.clone(),
                        group2: state.group2_val.clone(),
                        ci: ci_ratio,
                    };
                }
            }
            ActionBarAction::Reset => {
                state.selected_vars.clear();
                state.grouping_var = None;
                state.source_selected = None;
                state.target_selected = None;
                state.group1_val = "1".to_string();
                state.group2_val = "2".to_string();
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
