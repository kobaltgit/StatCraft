use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, TransferButton, VariableListItem,
};
use egui::{vec2, TextEdit};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogSelectMode {
    #[default]
    All,
    Condition,
    FilterVar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogCasesAction {
    #[default]
    Filtered,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct SelectCasesDialogState {
    pub is_open: bool,
    pub mode: DialogSelectMode,
    pub action: DialogCasesAction,
    pub source_selected: Option<usize>,
    pub filter_var: Option<String>,
    pub condition: String,
    pub temp_condition: String,
    pub condition_subdialog_open: bool,
}

impl Default for SelectCasesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            mode: DialogSelectMode::All,
            action: DialogCasesAction::Filtered,
            source_selected: None,
            filter_var: None,
            condition: String::new(),
            temp_condition: String::new(),
            condition_subdialog_open: false,
        }
    }
}

pub enum SelectCasesDialogAction {
    Run {
        mode: DialogSelectMode,
        action: DialogCasesAction,
        condition: String,
        filter_var: Option<String>,
    },
    Paste {
        mode: DialogSelectMode,
        action: DialogCasesAction,
        condition: String,
        filter_var: Option<String>,
    },
    None,
}

pub fn render_select_cases_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut SelectCasesDialogState,
) -> SelectCasesDialogAction {
    if !state.is_open {
        return SelectCasesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return SelectCasesDialogAction::None;
        }
    };

    let mut action_result = SelectCasesDialogAction::None;
    let mut close_dialog = false;

    // Available variables
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary
                .get(col_idx)
                .map(|meta| (col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-select-cases-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left Column: Variables list (180x260)
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
                vec2(180.0, 260.0),
                "select_cases_vars_list",
                &source_items,
            );

            if let ListEvent::Clicked(idx) = source_ev {
                state.source_selected = Some(idx);
            }

            // Transfer button to assign filter variable
            let can_transfer = state.source_selected.is_some()
                && state.source_selected.unwrap_or(usize::MAX) < available_vars.len();

            ui.allocate_ui_with_layout(
                vec2(36.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(100.0);
                    let btn = TransferButton::to_target(can_transfer);
                    if btn.show(ui).clicked() {
                        if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.filter_var = Some(name.clone());
                                state.mode = DialogSelectMode::FilterVar;
                            }
                        }
                    }
                },
            );

            // Right Area: Selection Mode and Actions
            ui.allocate_ui_with_layout(
                vec2(310.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // Group 1: Select criteria
                    GroupBox::new(t!("dialog-select-cases-select-group")).show(ui, |ui| {
                        ui.radio_value(
                            &mut state.mode,
                            DialogSelectMode::All,
                            t!("dialog-select-cases-all"),
                        );

                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut state.mode,
                                DialogSelectMode::Condition,
                                t!("dialog-select-cases-if"),
                            );
                            if ui.button(t!("dialog-select-cases-if-btn")).clicked() {
                                state.mode = DialogSelectMode::Condition;
                                state.temp_condition = state.condition.clone();
                                state.condition_subdialog_open = true;
                            }
                        });

                        if state.mode == DialogSelectMode::Condition && !state.condition.is_empty()
                        {
                            ui.label(format!(
                                "{}: {}",
                                t!("dialog-select-cases-condition-label"),
                                state.condition
                            ));
                        }

                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut state.mode,
                                DialogSelectMode::FilterVar,
                                t!("dialog-select-cases-use-filter-var"),
                            );
                            let none_str = t!("dialog-select-cases-none");
                            let var_display = state.filter_var.as_deref().unwrap_or(&none_str);
                            ui.label(format!("[{}]", var_display));
                        });
                    });

                    ui.add_space(10.0);

                    // Group 2: Unselected cases action
                    GroupBox::new(t!("dialog-select-cases-unselected-group")).show(ui, |ui| {
                        ui.radio_value(
                            &mut state.action,
                            DialogCasesAction::Filtered,
                            t!("dialog-select-cases-action-filter"),
                        );
                        ui.radio_value(
                            &mut state.action,
                            DialogCasesAction::Deleted,
                            t!("dialog-select-cases-action-delete"),
                        );
                    });
                },
            );
        });

        ui.add_space(14.0);

        // Dialog Bottom Action Bar
        let ok_enabled = match state.mode {
            DialogSelectMode::All => true,
            DialogSelectMode::Condition => !state.condition.trim().is_empty(),
            DialogSelectMode::FilterVar => state.filter_var.is_some(),
        };

        let action = DialogActionBar::new(ok_enabled).show(ui);
        match action {
            ActionBarAction::Ok => {
                action_result = SelectCasesDialogAction::Run {
                    mode: state.mode,
                    action: state.action,
                    condition: state.condition.clone(),
                    filter_var: state.filter_var.clone(),
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action_result = SelectCasesDialogAction::Paste {
                    mode: state.mode,
                    action: state.action,
                    condition: state.condition.clone(),
                    filter_var: state.filter_var.clone(),
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = SelectCasesDialogState {
                    is_open: true,
                    ..Default::default()
                };
            }
            ActionBarAction::Cancel => {
                close_dialog = true;
            }
            ActionBarAction::Help => {}
            ActionBarAction::None => {}
        }
    });

    if close_dialog || !main_open {
        state.is_open = false;
    }

    // Condition subdialog
    if state.condition_subdialog_open {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-select-cases-condition-title")).show(
            ctx,
            &mut sub_open,
            |ui| {
                ui.label(t!("dialog-select-cases-enter-condition"));
                ui.add(
                    TextEdit::singleline(&mut state.temp_condition)
                        .desired_width(320.0)
                        .hint_text(t!("dialog-select-cases-condition-hint")),
                );

                ui.add_space(10.0);
                ui.label(t!("dialog-select-cases-keypad"));

                ui.horizontal(|ui| {
                    if ui.button("+").clicked() {
                        state.temp_condition.push_str(" + ");
                    }
                    if ui.button("-").clicked() {
                        state.temp_condition.push_str(" - ");
                    }
                    if ui.button("*").clicked() {
                        state.temp_condition.push_str(" * ");
                    }
                    if ui.button("/").clicked() {
                        state.temp_condition.push_str(" / ");
                    }
                    if ui.button("**").clicked() {
                        state.temp_condition.push_str(" ** ");
                    }
                    if ui.button("(").clicked() {
                        state.temp_condition.push('(');
                    }
                    if ui.button(")").clicked() {
                        state.temp_condition.push(')');
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("=").clicked() {
                        state.temp_condition.push_str(" = ");
                    }
                    if ui.button("~=").clicked() {
                        state.temp_condition.push_str(" ~= ");
                    }
                    if ui.button("<").clicked() {
                        state.temp_condition.push_str(" < ");
                    }
                    if ui.button("<=").clicked() {
                        state.temp_condition.push_str(" <= ");
                    }
                    if ui.button(">").clicked() {
                        state.temp_condition.push_str(" > ");
                    }
                    if ui.button(">=").clicked() {
                        state.temp_condition.push_str(" >= ");
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("&").clicked() {
                        state.temp_condition.push_str(" & ");
                    }
                    if ui.button("|").clicked() {
                        state.temp_condition.push_str(" | ");
                    }
                    if ui.button("~").clicked() {
                        state.temp_condition.push('~');
                    }
                });

                ui.add_space(14.0);
                let can_continue = !state.temp_condition.trim().is_empty();
                let sub_action = SubdialogActionBar::new(can_continue).show(ui);
                match sub_action {
                    SubdialogAction::Continue => {
                        state.condition = state.temp_condition.trim().to_string();
                        state.condition_subdialog_open = false;
                    }
                    SubdialogAction::Cancel => {
                        state.condition_subdialog_open = false;
                    }
                    SubdialogAction::Help => {}
                    SubdialogAction::None => {}
                }
            },
        );

        if !sub_open {
            state.condition_subdialog_open = false;
        }
    }

    action_result
}
