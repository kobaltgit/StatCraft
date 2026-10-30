use super::{DateTimeDialogAction, DateTimeDialogState, DateTimeDialogTask};
use crate::dialogs::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_algo::{DatePart, DateUnit};
use statcraft_core::DataSet;
use statcraft_i18n::t;

pub fn render_date_time_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut DateTimeDialogState,
) -> DateTimeDialogAction {
    if !state.is_open {
        return DateTimeDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return DateTimeDialogAction::None;
        }
    };

    let mut action = DateTimeDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if state.primary_var.as_deref() != Some(&meta.name)
                    && state.secondary_var.as_deref() != Some(&meta.name)
                {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-datetime-wizard-title")).show(ctx, &mut main_open, |ui| {
        // 1. Task Selector GroupBox
        GroupBox::new(t!("dialog-datetime-task-group")).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.radio_value(
                    &mut state.task,
                    DateTimeDialogTask::Extract,
                    t!("dialog-datetime-task-extract"),
                );
                ui.radio_value(
                    &mut state.task,
                    DateTimeDialogTask::Diff,
                    t!("dialog-datetime-task-diff"),
                );
                ui.radio_value(
                    &mut state.task,
                    DateTimeDialogTask::Add,
                    t!("dialog-datetime-task-add"),
                );
                ui.radio_value(
                    &mut state.task,
                    DateTimeDialogTask::Construct,
                    t!("dialog-datetime-task-construct"),
                );
            });
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            // Left Panel: Available Variables (180x230)
            let source_items: Vec<VariableListItem> = available_vars
                .iter()
                .enumerate()
                .map(|(idx, (_col, name, label, measure))| VariableListItem {
                    name,
                    display_text: format_var_display(name, label),
                    measure: *measure,
                    is_selected: state.available_selected == Some(idx),
                })
                .collect();

            let event = render_variable_list_panel(
                ui,
                t!("dialog-variables-source"),
                vec2(180.0, 230.0),
                "dt_avail_vars",
                &source_items,
            );
            match event {
                ListEvent::Clicked(idx) => {
                    state.available_selected = Some(idx);
                    state.primary_selected = false;
                    state.secondary_selected = false;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_col, name, _, _)) = available_vars.get(idx) {
                        if state.primary_var.is_none() {
                            state.primary_var = Some(name.clone());
                        } else if state.task == DateTimeDialogTask::Diff
                            && state.secondary_var.is_none()
                        {
                            state.secondary_var = Some(name.clone());
                        }
                    }
                    state.available_selected = None;
                }
                ListEvent::None => {}
            }

            // Middle: Transfer Buttons
            let is_moving_left = state.primary_selected || state.secondary_selected;
            let can_move_right = state.available_selected.is_some();
            let transfer_btn = if is_moving_left {
                TransferButton::to_source(true)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 230.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(30.0);
                    if transfer_btn.show(ui).clicked() {
                        if state.primary_selected {
                            state.primary_var = None;
                            state.primary_selected = false;
                        } else if state.secondary_selected {
                            state.secondary_var = None;
                            state.secondary_selected = false;
                        } else if let Some(idx) = state.available_selected {
                            if let Some((_col, name, _, _)) = available_vars.get(idx) {
                                if state.primary_var.is_none() {
                                    state.primary_var = Some(name.clone());
                                } else if state.task == DateTimeDialogTask::Diff
                                    && state.secondary_var.is_none()
                                {
                                    state.secondary_var = Some(name.clone());
                                }
                                state.available_selected = None;
                            }
                        }
                    }
                },
            );

            // Right Panel: Inputs and parameters (300px)
            ui.vertical(|ui| {
                ui.set_width(300.0);

                // Primary Date slot
                ui.label(t!("dialog-datetime-first-date"));
                let prim_text = state.primary_var.as_deref().unwrap_or("");
                let prim_selected = state.primary_selected;
                let prim_resp = ui.selectable_label(
                    prim_selected,
                    format!("[ {} ]", if prim_text.is_empty() { "-" } else { prim_text }),
                );
                if prim_resp.clicked() && state.primary_var.is_some() {
                    state.primary_selected = !state.primary_selected;
                    state.secondary_selected = false;
                    state.available_selected = None;
                }

                // Secondary Date slot (only for Diff)
                if state.task == DateTimeDialogTask::Diff {
                    ui.add_space(4.0);
                    ui.label(t!("dialog-datetime-second-date"));
                    let sec_text = state.secondary_var.as_deref().unwrap_or("");
                    let sec_selected = state.secondary_selected;
                    let sec_resp = ui.selectable_label(
                        sec_selected,
                        format!("[ {} ]", if sec_text.is_empty() { "-" } else { sec_text }),
                    );
                    if sec_resp.clicked() && state.secondary_var.is_some() {
                        state.secondary_selected = !state.secondary_selected;
                        state.primary_selected = false;
                        state.available_selected = None;
                    }
                }

                ui.add_space(6.0);

                // Operation-specific controls
                match state.task {
                    DateTimeDialogTask::Extract => {
                        ui.label(t!("dialog-datetime-part-label"));
                        let part_label = match state.part {
                            DatePart::Year => t!("dialog-datetime-part-year"),
                            DatePart::Month => t!("dialog-datetime-part-month"),
                            DatePart::Day => t!("dialog-datetime-part-day"),
                            DatePart::Hour => t!("dialog-datetime-part-hour"),
                            DatePart::Minute => t!("dialog-datetime-part-minute"),
                            DatePart::Second => t!("dialog-datetime-part-second"),
                            DatePart::DayOfWeek => t!("dialog-datetime-part-dayofweek"),
                            DatePart::Quarter => t!("dialog-datetime-part-quarter"),
                        };
                        egui::ComboBox::from_id_salt("dt_part_cb")
                            .selected_text(part_label)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut state.part,
                                    DatePart::Year,
                                    t!("dialog-datetime-part-year"),
                                );
                                ui.selectable_value(
                                    &mut state.part,
                                    DatePart::Month,
                                    t!("dialog-datetime-part-month"),
                                );
                                ui.selectable_value(
                                    &mut state.part,
                                    DatePart::Day,
                                    t!("dialog-datetime-part-day"),
                                );
                                ui.selectable_value(
                                    &mut state.part,
                                    DatePart::Hour,
                                    t!("dialog-datetime-part-hour"),
                                );
                                ui.selectable_value(
                                    &mut state.part,
                                    DatePart::Minute,
                                    t!("dialog-datetime-part-minute"),
                                );
                                ui.selectable_value(
                                    &mut state.part,
                                    DatePart::Second,
                                    t!("dialog-datetime-part-second"),
                                );
                                ui.selectable_value(
                                    &mut state.part,
                                    DatePart::DayOfWeek,
                                    t!("dialog-datetime-part-dayofweek"),
                                );
                                ui.selectable_value(
                                    &mut state.part,
                                    DatePart::Quarter,
                                    t!("dialog-datetime-part-quarter"),
                                );
                            });
                    }
                    DateTimeDialogTask::Diff => {
                        ui.label(t!("dialog-datetime-unit-label"));
                        let unit_label = match state.unit {
                            DateUnit::Years => t!("dialog-datetime-unit-years"),
                            DateUnit::Months => t!("dialog-datetime-unit-months"),
                            DateUnit::Weeks => t!("dialog-datetime-unit-weeks"),
                            DateUnit::Days => t!("dialog-datetime-unit-days"),
                            DateUnit::Hours => t!("dialog-datetime-unit-hours"),
                            DateUnit::Minutes => t!("dialog-datetime-unit-minutes"),
                            DateUnit::Seconds => t!("dialog-datetime-unit-seconds"),
                        };
                        egui::ComboBox::from_id_salt("dt_unit_diff_cb")
                            .selected_text(unit_label)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Years,
                                    t!("dialog-datetime-unit-years"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Months,
                                    t!("dialog-datetime-unit-months"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Weeks,
                                    t!("dialog-datetime-unit-weeks"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Days,
                                    t!("dialog-datetime-unit-days"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Hours,
                                    t!("dialog-datetime-unit-hours"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Minutes,
                                    t!("dialog-datetime-unit-minutes"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Seconds,
                                    t!("dialog-datetime-unit-seconds"),
                                );
                            });
                    }
                    DateTimeDialogTask::Add => {
                        ui.label(t!("dialog-datetime-duration-amount"));
                        ui.add(egui::DragValue::new(&mut state.duration_amount).speed(1.0));

                        ui.add_space(4.0);
                        ui.label(t!("dialog-datetime-unit-label"));
                        let unit_label = match state.unit {
                            DateUnit::Years => t!("dialog-datetime-unit-years"),
                            DateUnit::Months => t!("dialog-datetime-unit-months"),
                            DateUnit::Weeks => t!("dialog-datetime-unit-weeks"),
                            DateUnit::Days => t!("dialog-datetime-unit-days"),
                            DateUnit::Hours => t!("dialog-datetime-unit-hours"),
                            DateUnit::Minutes => t!("dialog-datetime-unit-minutes"),
                            DateUnit::Seconds => t!("dialog-datetime-unit-seconds"),
                        };
                        egui::ComboBox::from_id_salt("dt_unit_add_cb")
                            .selected_text(unit_label)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Years,
                                    t!("dialog-datetime-unit-years"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Months,
                                    t!("dialog-datetime-unit-months"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Weeks,
                                    t!("dialog-datetime-unit-weeks"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Days,
                                    t!("dialog-datetime-unit-days"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Hours,
                                    t!("dialog-datetime-unit-hours"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Minutes,
                                    t!("dialog-datetime-unit-minutes"),
                                );
                                ui.selectable_value(
                                    &mut state.unit,
                                    DateUnit::Seconds,
                                    t!("dialog-datetime-unit-seconds"),
                                );
                            });
                    }
                    DateTimeDialogTask::Construct => {}
                }

                ui.add_space(8.0);

                // Target variable & label
                ui.label(t!("dialog-datetime-target-var"));
                ui.add(egui::TextEdit::singleline(&mut state.target_var).desired_width(280.0));

                ui.add_space(4.0);
                ui.label(t!("dialog-datetime-target-label"));
                ui.add(egui::TextEdit::singleline(&mut state.target_label).desired_width(280.0));
            });
        });

        ui.add_space(10.0);

        // Validation & Action Bar
        let is_target_name_valid = !state.target_var.trim().is_empty();
        let can_run = state.primary_var.is_some()
            && is_target_name_valid
            && (state.task != DateTimeDialogTask::Diff || state.secondary_var.is_some());

        let act = DialogActionBar::new(can_run).show(ui);
        match act {
            ActionBarAction::Ok => {
                if let Some(ref prim) = state.primary_var {
                    let (op_name, part_s, unit_s, dur_s) = build_action_params(state);
                    let target_lbl = if state.target_label.trim().is_empty() {
                        None
                    } else {
                        Some(state.target_label.trim().to_string())
                    };
                    action = DateTimeDialogAction::Run {
                        operation: op_name,
                        target_var: state.target_var.trim().to_string(),
                        target_label: target_lbl,
                        primary_var: prim.clone(),
                        secondary_var: state.secondary_var.clone(),
                        part: part_s,
                        unit: unit_s,
                        duration_value: dur_s,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Paste => {
                if let Some(ref prim) = state.primary_var {
                    let (op_name, part_s, unit_s, dur_s) = build_action_params(state);
                    let target_lbl = if state.target_label.trim().is_empty() {
                        None
                    } else {
                        Some(state.target_label.trim().to_string())
                    };
                    action = DateTimeDialogAction::Paste {
                        operation: op_name,
                        target_var: state.target_var.trim().to_string(),
                        target_label: target_lbl,
                        primary_var: prim.clone(),
                        secondary_var: state.secondary_var.clone(),
                        part: part_s,
                        unit: unit_s,
                        duration_value: dur_s,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Reset => {
                state.reset();
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

    action
}

fn build_action_params(
    state: &DateTimeDialogState,
) -> (String, Option<String>, Option<String>, Option<f64>) {
    match state.task {
        DateTimeDialogTask::Extract => {
            let p_str = match state.part {
                DatePart::Year => "year",
                DatePart::Month => "month",
                DatePart::Day => "day",
                DatePart::Hour => "hour",
                DatePart::Minute => "minute",
                DatePart::Second => "second",
                DatePart::DayOfWeek => "day_of_week",
                DatePart::Quarter => "quarter",
            };
            ("extract".to_string(), Some(p_str.to_string()), None, None)
        }
        DateTimeDialogTask::Diff => {
            let u_str = match state.unit {
                DateUnit::Years => "years",
                DateUnit::Months => "months",
                DateUnit::Weeks => "weeks",
                DateUnit::Days => "days",
                DateUnit::Hours => "hours",
                DateUnit::Minutes => "minutes",
                DateUnit::Seconds => "seconds",
            };
            ("diff".to_string(), None, Some(u_str.to_string()), None)
        }
        DateTimeDialogTask::Add => {
            let u_str = match state.unit {
                DateUnit::Years => "years",
                DateUnit::Months => "months",
                DateUnit::Weeks => "weeks",
                DateUnit::Days => "days",
                DateUnit::Hours => "hours",
                DateUnit::Minutes => "minutes",
                DateUnit::Seconds => "seconds",
            };
            (
                "add".to_string(),
                None,
                Some(u_str.to_string()),
                Some(state.duration_amount),
            )
        }
        DateTimeDialogTask::Construct => ("construct".to_string(), None, None, None),
    }
}
