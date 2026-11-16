use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_algo::TieMethod;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct CoxRegressionDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub time_var: Option<String>,
    pub status_var: Option<String>,
    pub covariates: Vec<String>,
    pub strata_var: Option<String>,
    pub target_time_selected: bool,
    pub target_status_selected: bool,
    pub target_cov_selected: Option<usize>,
    pub target_strata_selected: bool,
    pub event_val_str: String,
    pub tie_method: TieMethod,
}

impl Default for CoxRegressionDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            time_var: None,
            status_var: None,
            covariates: Vec::new(),
            strata_var: None,
            target_time_selected: false,
            target_status_selected: false,
            target_cov_selected: None,
            target_strata_selected: false,
            event_val_str: "1".to_string(),
            tie_method: TieMethod::Breslow,
        }
    }
}

pub enum CoxRegressionDialogAction {
    Run {
        time_var: String,
        status_var: String,
        event_value: f64,
        covariates: Vec<String>,
        strata_var: Option<String>,
        tie_method: TieMethod,
    },
    Paste {
        time_var: String,
        status_var: String,
        event_value: f64,
        covariates: Vec<String>,
        strata_var: Option<String>,
        tie_method: TieMethod,
    },
    None,
}

pub fn render_cox_regression_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut CoxRegressionDialogState,
) -> CoxRegressionDialogAction {
    if !state.is_open {
        return CoxRegressionDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return CoxRegressionDialogAction::None;
        }
    };

    let mut action = CoxRegressionDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_used = state.time_var.as_ref() == Some(&meta.name)
                    || state.status_var.as_ref() == Some(&meta.name)
                    || state.strata_var.as_ref() == Some(&meta.name)
                    || state.covariates.contains(&meta.name);
                if is_used {
                    None
                } else {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                }
            })
        })
        .collect();

    let shell = DialogShell::standard(t!("dialog-cox-title"));
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
                vec2(170.0, 310.0),
                "cox_source_list",
                &source_items,
            );

            match src_event {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_time_selected = false;
                    state.target_status_selected = false;
                    state.target_cov_selected = None;
                    state.target_strata_selected = false;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        if state.time_var.is_none() {
                            state.time_var = Some(name.clone());
                        } else if state.status_var.is_none() {
                            state.status_var = Some(name.clone());
                        } else {
                            state.covariates.push(name.clone());
                        }
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Middle column: Transfer buttons
            ui.vertical(|ui| {
                ui.add_space(15.0);
                // Time transfer
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
                ui.add_space(2.0);
                if TransferButton::to_source(can_from_time).show(ui).clicked() {
                    state.time_var = None;
                    state.target_time_selected = false;
                }

                ui.add_space(30.0);
                // Status transfer
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
                ui.add_space(2.0);
                if TransferButton::to_source(can_from_status)
                    .show(ui)
                    .clicked()
                {
                    state.status_var = None;
                    state.target_status_selected = false;
                }

                ui.add_space(35.0);
                // Covariates transfer
                let can_to_cov = state.source_selected.is_some();
                let can_from_cov = state.target_cov_selected.is_some();
                if TransferButton::to_target(can_to_cov).show(ui).clicked() {
                    if let Some(src_idx) = state.source_selected {
                        if let Some(item) = source_items.get(src_idx) {
                            state.covariates.push(item.name.to_string());
                            state.source_selected = None;
                        }
                    }
                }
                ui.add_space(2.0);
                if TransferButton::to_source(can_from_cov).show(ui).clicked() {
                    if let Some(cov_idx) = state.target_cov_selected {
                        if cov_idx < state.covariates.len() {
                            state.covariates.remove(cov_idx);
                            state.target_cov_selected = None;
                        }
                    }
                }

                ui.add_space(45.0);
                // Strata transfer
                let can_to_strata = state.source_selected.is_some() && state.strata_var.is_none();
                let can_from_strata = state.target_strata_selected && state.strata_var.is_some();
                if TransferButton::to_target(can_to_strata).show(ui).clicked() {
                    if let Some(src_idx) = state.source_selected {
                        if let Some(item) = source_items.get(src_idx) {
                            state.strata_var = Some(item.name.to_string());
                            state.source_selected = None;
                        }
                    }
                }
                ui.add_space(2.0);
                if TransferButton::to_source(can_from_strata)
                    .show(ui)
                    .clicked()
                {
                    state.strata_var = None;
                    state.target_strata_selected = false;
                }
            });

            // Right column: Target sections
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
                    t!("dialog-km-time-variable"),
                    vec2(210.0, 36.0),
                    "cox_time_list",
                    &time_items,
                );
                if let ListEvent::Clicked(_) = t_event {
                    state.target_time_selected = true;
                    state.target_status_selected = false;
                    state.target_cov_selected = None;
                    state.target_strata_selected = false;
                    state.source_selected = None;
                }

                ui.add_space(4.0);

                // Status variable + Event value in one horizontal block
                ui.horizontal(|ui| {
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
                        t!("dialog-km-status-variable"),
                        vec2(130.0, 36.0),
                        "cox_status_list",
                        &status_items,
                    );
                    if let ListEvent::Clicked(_) = s_event {
                        state.target_status_selected = true;
                        state.target_time_selected = false;
                        state.target_cov_selected = None;
                        state.target_strata_selected = false;
                        state.source_selected = None;
                    }

                    GroupBox::new(t!("dialog-km-event-value-group")).show(ui, |ui| {
                        ui.text_edit_singleline(&mut state.event_val_str);
                    });
                });

                ui.add_space(4.0);

                // Covariates list
                let cov_items: Vec<VariableListItem> = state
                    .covariates
                    .iter()
                    .enumerate()
                    .map(|(idx, v)| VariableListItem {
                        name: v,
                        display_text: format_var_display(
                            v,
                            &ds.dictionary
                                .find_by_name(v)
                                .map(|m| m.label.clone())
                                .unwrap_or_default(),
                        ),
                        measure: statcraft_core::Measure::Scale,
                        is_selected: state.target_cov_selected == Some(idx),
                    })
                    .collect();

                let c_event = render_variable_list_panel(
                    ui,
                    t!("dialog-cox-covariates"),
                    vec2(210.0, 75.0),
                    "cox_cov_list",
                    &cov_items,
                );
                if let ListEvent::Clicked(idx) = c_event {
                    state.target_cov_selected = Some(idx);
                    state.target_time_selected = false;
                    state.target_status_selected = false;
                    state.target_strata_selected = false;
                    state.source_selected = None;
                }

                ui.add_space(4.0);

                // Strata variable
                let strata_items: Vec<VariableListItem> = state
                    .strata_var
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
                            is_selected: state.target_strata_selected,
                        }]
                    })
                    .unwrap_or_default();

                let st_event = render_variable_list_panel(
                    ui,
                    t!("dialog-cox-strata-variable"),
                    vec2(210.0, 36.0),
                    "cox_strata_list",
                    &strata_items,
                );
                if let ListEvent::Clicked(_) = st_event {
                    state.target_strata_selected = true;
                    state.target_time_selected = false;
                    state.target_status_selected = false;
                    state.target_cov_selected = None;
                    state.source_selected = None;
                }

                ui.add_space(4.0);

                // Ties Method
                GroupBox::new(t!("dialog-cox-ties-group")).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut state.tie_method,
                            TieMethod::Breslow,
                            t!("dialog-cox-ties-breslow"),
                        );
                        ui.radio_value(
                            &mut state.tie_method,
                            TieMethod::Efron,
                            t!("dialog-cox-ties-efron"),
                        );
                    });
                });
            });
        });

        ui.add_space(10.0);

        // Validation & Action Bar
        let event_val: Option<f64> = state.event_val_str.parse().ok();
        let is_valid = state.time_var.is_some()
            && state.status_var.is_some()
            && !state.covariates.is_empty()
            && event_val.is_some();

        let bar_action = DialogActionBar::new(is_valid).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                if let (Some(t_var), Some(s_var), Some(ev)) =
                    (&state.time_var, &state.status_var, event_val)
                {
                    action = CoxRegressionDialogAction::Run {
                        time_var: t_var.clone(),
                        status_var: s_var.clone(),
                        event_value: ev,
                        covariates: state.covariates.clone(),
                        strata_var: state.strata_var.clone(),
                        tie_method: state.tie_method,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Paste => {
                if let (Some(t_var), Some(s_var), Some(ev)) =
                    (&state.time_var, &state.status_var, event_val)
                {
                    action = CoxRegressionDialogAction::Paste {
                        time_var: t_var.clone(),
                        status_var: s_var.clone(),
                        event_value: ev,
                        covariates: state.covariates.clone(),
                        strata_var: state.strata_var.clone(),
                        tie_method: state.tie_method,
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Reset => {
                *state = CoxRegressionDialogState {
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
