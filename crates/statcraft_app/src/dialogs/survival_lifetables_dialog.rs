use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct LifeTablesDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub time_var: Option<String>,
    pub status_var: Option<String>,
    pub target_time_selected: bool,
    pub target_status_selected: bool,
    pub event_val_str: String,
    pub end_str: String,
    pub width_str: String,
}

impl Default for LifeTablesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            time_var: None,
            status_var: None,
            target_time_selected: false,
            target_status_selected: false,
            event_val_str: "1".to_string(),
            end_str: "60".to_string(),
            width_str: "10".to_string(),
        }
    }
}

pub enum LifeTablesDialogAction {
    Run {
        time_var: String,
        status_var: String,
        event_value: f64,
        interval_end: f64,
        interval_width: f64,
    },
    Paste {
        time_var: String,
        status_var: String,
        event_value: f64,
        interval_end: f64,
        interval_width: f64,
    },
    None,
}

pub fn render_survival_lifetables_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut LifeTablesDialogState,
) -> LifeTablesDialogAction {
    if !state.is_open {
        return LifeTablesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return LifeTablesDialogAction::None;
        }
    };

    let mut action = LifeTablesDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_used = state.time_var.as_ref() == Some(&meta.name)
                    || state.status_var.as_ref() == Some(&meta.name);
                if is_used {
                    None
                } else {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                }
            })
        })
        .collect();

    let shell = DialogShell::standard(t!("dialog-lifetables-title"));
    let mut main_open = state.is_open;

    shell.show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left column: Source variables list
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
                t!("dialog-common-variables"),
                vec2(180.0, 240.0),
                "lifetables_source_list",
                &source_items,
            );

            match src_event {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_time_selected = false;
                    state.target_status_selected = false;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        if state.time_var.is_none() {
                            state.time_var = Some(name.clone());
                        } else if state.status_var.is_none() {
                            state.status_var = Some(name.clone());
                        }
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Middle column: Transfer buttons
            ui.vertical(|ui| {
                ui.add_space(30.0);
                let can_to_time = state.source_selected.is_some() && state.time_var.is_none();
                let can_from_time = state.target_time_selected && state.time_var.is_some();
                if TransferButton::to_target(can_to_time).show(ui).clicked() {
                    if let Some(src_idx) = state.source_selected {
                        if let Some(item) = source_items.get(src_idx) {
                            state.time_var = Some(item.name.to_string());
                            state.source_selected = None;
                        }
                    }
                }
                ui.add_space(4.0);
                if TransferButton::to_source(can_from_time).show(ui).clicked() {
                    state.time_var = None;
                    state.target_time_selected = false;
                }

                ui.add_space(50.0);
                let can_to_status = state.source_selected.is_some() && state.status_var.is_none();
                let can_from_status = state.target_status_selected && state.status_var.is_some();
                if TransferButton::to_target(can_to_status).show(ui).clicked() {
                    if let Some(src_idx) = state.source_selected {
                        if let Some(item) = source_items.get(src_idx) {
                            state.status_var = Some(item.name.to_string());
                            state.source_selected = None;
                        }
                    }
                }
                ui.add_space(4.0);
                if TransferButton::to_source(can_from_status)
                    .show(ui)
                    .clicked()
                {
                    state.status_var = None;
                    state.target_status_selected = false;
                }
            });

            // Right column: Target selections & Interval GroupBox
            ui.vertical(|ui| {
                // Time variable
                let time_items: Vec<VariableListItem> = state
                    .time_var
                    .as_ref()
                    .map(|v| {
                        vec![VariableListItem {
                            name: v,
                            display_text: format_var_display(
                                v,
                                &ds.dictionary
                                    .find_by_name(v)
                                    .map(|m| m.label.clone())
                                    .unwrap_or_default(),
                            ),
                            measure: statcraft_core::Measure::Scale,
                            is_selected: state.target_time_selected,
                        }]
                    })
                    .unwrap_or_default();

                let t_event = render_variable_list_panel(
                    ui,
                    t!("dialog-lifetables-time-variable"),
                    vec2(220.0, 55.0),
                    "lifetables_time_list",
                    &time_items,
                );
                if let ListEvent::Clicked(_) = t_event {
                    state.target_time_selected = true;
                    state.target_status_selected = false;
                    state.source_selected = None;
                }

                ui.add_space(6.0);

                // Status variable
                let status_items: Vec<VariableListItem> = state
                    .status_var
                    .as_ref()
                    .map(|v| {
                        vec![VariableListItem {
                            name: v,
                            display_text: format_var_display(
                                v,
                                &ds.dictionary
                                    .find_by_name(v)
                                    .map(|m| m.label.clone())
                                    .unwrap_or_default(),
                            ),
                            measure: statcraft_core::Measure::Nominal,
                            is_selected: state.target_status_selected,
                        }]
                    })
                    .unwrap_or_default();

                let s_event = render_variable_list_panel(
                    ui,
                    t!("dialog-lifetables-status-variable"),
                    vec2(220.0, 55.0),
                    "lifetables_status_list",
                    &status_items,
                );
                if let ListEvent::Clicked(_) = s_event {
                    state.target_status_selected = true;
                    state.target_time_selected = false;
                    state.source_selected = None;
                }

                ui.add_space(8.0);

                // Intervals GroupBox
                GroupBox::new(t!("dialog-lifetables-intervals-group")).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-lifetables-event-value"));
                        ui.text_edit_singleline(&mut state.event_val_str);
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-lifetables-thru-end"));
                        ui.text_edit_singleline(&mut state.end_str);
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-lifetables-by-width"));
                        ui.text_edit_singleline(&mut state.width_str);
                    });
                });
            });
        });

        ui.add_space(16.0);

        // Validation & Action Bar
        let event_val: Option<f64> = state.event_val_str.parse().ok();
        let end_val: Option<f64> = state.end_str.parse().ok();
        let width_val: Option<f64> = state.width_str.parse().ok();

        let is_valid = state.time_var.is_some()
            && state.status_var.is_some()
            && event_val.is_some()
            && end_val.map(|e| e > 0.0).unwrap_or(false)
            && width_val.map(|w| w > 0.0).unwrap_or(false);

        let bar_action = DialogActionBar::new(is_valid).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                if let (Some(t_var), Some(s_var), Some(ev), Some(end), Some(w)) = (
                    &state.time_var,
                    &state.status_var,
                    event_val,
                    end_val,
                    width_val,
                ) {
                    action = LifeTablesDialogAction::Run {
                        time_var: t_var.clone(),
                        status_var: s_var.clone(),
                        event_value: ev,
                        interval_end: end,
                        interval_width: w,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Paste => {
                if let (Some(t_var), Some(s_var), Some(ev), Some(end), Some(w)) = (
                    &state.time_var,
                    &state.status_var,
                    event_val,
                    end_val,
                    width_val,
                ) {
                    action = LifeTablesDialogAction::Paste {
                        time_var: t_var.clone(),
                        status_var: s_var.clone(),
                        event_value: ev,
                        interval_end: end,
                        interval_width: w,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Reset => {
                *state = LifeTablesDialogState {
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

    if close_dialog || !main_open {
        state.is_open = false;
    }

    action
}
