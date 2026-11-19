use super::source_panel::render_source_panel;
use super::types::{AddVarsDialogState, MergeSourceKind, MergeVarsAction};
use crate::dialogs::common::{
    render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell, ListEvent,
    TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::dataset::merge::{load_dataset_from_file, MergeMethod};
use statcraft_core::workspace::WorkspaceManager;
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;
use std::collections::{HashMap, HashSet};

pub fn render_add_vars_dialog(
    ctx: &egui::Context,
    active_dataset: Option<&DataSet>,
    workspace: Option<&WorkspaceManager>,
    state: &mut AddVarsDialogState,
) -> MergeVarsAction {
    if !state.is_open {
        return MergeVarsAction::None;
    }

    let ds = match active_dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return MergeVarsAction::None;
        }
    };

    let mut action = MergeVarsAction::None;
    let mut is_open = true;

    DialogShell::standard(t!("dialog-merge-vars-title")).show(ctx, &mut is_open, |ui| {
        // 1. Source selector
        let source_changed = render_source_panel(
            ui,
            &mut state.source_kind,
            &mut state.external_file_path,
            &mut state.selected_session_id,
            workspace,
        );

        if source_changed || (state.secondary_dataset.is_none() && has_valid_source(state)) {
            refresh_secondary_vars(ds, workspace, state);
        }

        ui.add_space(4.0);

        // 2. Merge Method radio buttons
        ui.horizontal(|ui| {
            ui.label(t!("dialog-merge-method-group"));
            ui.radio_value(
                &mut state.method,
                MergeMethod::OneToOneWithKeys,
                t!("dialog-merge-method-1to1"),
            );
            ui.radio_value(
                &mut state.method,
                MergeMethod::OneToManyTable,
                t!("dialog-merge-method-table"),
            );
            ui.radio_value(
                &mut state.method,
                MergeMethod::Sequential,
                t!("dialog-merge-method-seq"),
            );
        });

        if state.method != MergeMethod::Sequential {
            ui.checkbox(&mut state.auto_sort_keys, t!("dialog-merge-sort-keys"));
        }

        ui.add_space(4.0);

        // 3. Middle lists
        ui.horizontal(|ui| {
            // Left list: Excluded variables
            let excluded_items: Vec<VariableListItem> = state
                .excluded_vars
                .iter()
                .enumerate()
                .map(|(idx, name)| {
                    let measure = find_var_measure(name, ds, state.secondary_dataset.as_ref());
                    VariableListItem {
                        name,
                        display_text: name.clone(),
                        measure,
                        is_selected: state.excluded_selected == Some(idx),
                    }
                })
                .collect();

            let left_event = render_variable_list_panel(
                ui,
                t!("dialog-merge-excluded-vars"),
                vec2(190.0, 230.0),
                "merge_vars_excluded",
                &excluded_items,
            );

            match left_event {
                ListEvent::Clicked(idx) => state.excluded_selected = Some(idx),
                ListEvent::DoubleClicked(idx) => {
                    if idx < state.excluded_vars.len() {
                        let var_name = state.excluded_vars.remove(idx);
                        if !state.new_active_vars.contains(&var_name) {
                            state.new_active_vars.push(var_name);
                        }
                        state.excluded_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Transfer buttons
            ui.vertical(|ui| {
                ui.add_space(40.0);
                let can_move_right = state.excluded_selected.is_some();
                if TransferButton::to_target(can_move_right).show(ui).clicked() {
                    if let Some(idx) = state.excluded_selected {
                        if idx < state.excluded_vars.len() {
                            let var_name = state.excluded_vars.remove(idx);
                            if !state.new_active_vars.contains(&var_name) {
                                state.new_active_vars.push(var_name);
                            }
                            state.excluded_selected = None;
                        }
                    }
                }

                ui.add_space(4.0);

                let can_move_left = state.new_active_selected.is_some();
                if TransferButton::to_source(can_move_left).show(ui).clicked() {
                    if let Some(idx) = state.new_active_selected {
                        if idx < state.new_active_vars.len() {
                            let var_name = state.new_active_vars.remove(idx);
                            state.excluded_vars.push(var_name);
                            state.new_active_selected = None;
                        }
                    }
                }

                if state.method != MergeMethod::Sequential {
                    ui.add_space(12.0);
                    let can_key =
                        state.new_active_selected.is_some() || state.excluded_selected.is_some();
                    if ui
                        .add_enabled(
                            can_key,
                            egui::Button::new(t!("dialog-merge-move-to-keys-btn")),
                        )
                        .clicked()
                    {
                        if let Some(idx) = state.new_active_selected {
                            if idx < state.new_active_vars.len() {
                                let var_name = state.new_active_vars.remove(idx);
                                if !state.key_vars.contains(&var_name) {
                                    state.key_vars.push(var_name);
                                }
                                state.new_active_selected = None;
                            }
                        } else if let Some(idx) = state.excluded_selected {
                            if idx < state.excluded_vars.len() {
                                let var_name = state.excluded_vars.remove(idx);
                                if !state.key_vars.contains(&var_name) {
                                    state.key_vars.push(var_name);
                                }
                                state.excluded_selected = None;
                            }
                        }
                    }
                }
            });

            // Right column: New Active Vars (Top) + Key Vars (Bottom)
            ui.vertical(|ui| {
                let right_h = if state.method != MergeMethod::Sequential {
                    130.0
                } else {
                    230.0
                };
                let active_items: Vec<VariableListItem> = state
                    .new_active_vars
                    .iter()
                    .enumerate()
                    .map(|(idx, name)| {
                        let measure = find_var_measure(name, ds, state.secondary_dataset.as_ref());
                        VariableListItem {
                            name,
                            display_text: name.clone(),
                            measure,
                            is_selected: state.new_active_selected == Some(idx),
                        }
                    })
                    .collect();

                let act_event = render_variable_list_panel(
                    ui,
                    t!("dialog-merge-new-active-vars"),
                    vec2(200.0, right_h),
                    "merge_vars_new_active",
                    &active_items,
                );

                match act_event {
                    ListEvent::Clicked(idx) => state.new_active_selected = Some(idx),
                    ListEvent::DoubleClicked(idx) => {
                        if idx < state.new_active_vars.len() {
                            let var_name = state.new_active_vars.remove(idx);
                            state.excluded_vars.push(var_name);
                            state.new_active_selected = None;
                        }
                    }
                    ListEvent::None => {}
                }

                if state.method != MergeMethod::Sequential {
                    ui.add_space(4.0);
                    let key_items: Vec<VariableListItem> = state
                        .key_vars
                        .iter()
                        .enumerate()
                        .map(|(idx, name)| {
                            let measure =
                                find_var_measure(name, ds, state.secondary_dataset.as_ref());
                            VariableListItem {
                                name,
                                display_text: name.clone(),
                                measure,
                                is_selected: state.key_selected == Some(idx),
                            }
                        })
                        .collect();

                    let key_event = render_variable_list_panel(
                        ui,
                        t!("dialog-merge-key-vars"),
                        vec2(200.0, 85.0),
                        "merge_vars_keys",
                        &key_items,
                    );

                    match key_event {
                        ListEvent::Clicked(idx) => state.key_selected = Some(idx),
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.key_vars.len() {
                                let var_name = state.key_vars.remove(idx);
                                state.new_active_vars.push(var_name);
                                state.key_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }
                }
            });
        });

        ui.add_space(4.0);

        // 4. Source indicator checkbox
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut state.in_variable_enabled,
                t!("dialog-merge-indicate-source"),
            );
            if state.in_variable_enabled {
                ui.add(
                    egui::TextEdit::singleline(&mut state.in_variable_name)
                        .desired_width(120.0)
                        .hint_text(t!("dialog-merge-source-var-name")),
                );
            }
        });

        if let Some(ref err) = state.error_msg {
            ui.add_space(4.0);
            ui.colored_label(egui::Color32::from_rgb(220, 50, 50), err);
        }

        ui.add_space(6.0);

        // 5. Action bar
        let is_ready = state.secondary_dataset.is_some()
            && (state.method == MergeMethod::Sequential || !state.key_vars.is_empty());
        let target_file_path = resolve_file_path(state, workspace);

        let action_bar = DialogActionBar::new(is_ready).show(ui);

        match action_bar {
            ActionBarAction::Ok => {
                if let Some(path) = target_file_path {
                    let in_var =
                        if state.in_variable_enabled && !state.in_variable_name.trim().is_empty() {
                            Some(state.in_variable_name.trim().to_string())
                        } else {
                            None
                        };
                    action = MergeVarsAction::Run {
                        file_path: path,
                        method: state.method,
                        keys: state.key_vars.clone(),
                        renames: HashMap::new(),
                        drop_second: state.excluded_vars.clone(),
                        in_second: in_var,
                    };
                    state.is_open = false;
                } else {
                    state.error_msg = Some(t!("dialog-merge-error-no-source"));
                }
            }
            ActionBarAction::Paste => {
                if let Some(path) = target_file_path {
                    let in_var =
                        if state.in_variable_enabled && !state.in_variable_name.trim().is_empty() {
                            Some(state.in_variable_name.trim().to_string())
                        } else {
                            None
                        };
                    action = MergeVarsAction::Paste {
                        file_path: path,
                        method: state.method,
                        keys: state.key_vars.clone(),
                        renames: HashMap::new(),
                        drop_second: state.excluded_vars.clone(),
                        in_second: in_var,
                    };
                    state.is_open = false;
                } else {
                    state.error_msg = Some(t!("dialog-merge-error-no-source"));
                }
            }
            ActionBarAction::Reset => {
                state.secondary_dataset = None;
                state.excluded_vars.clear();
                state.new_active_vars.clear();
                state.key_vars.clear();
                state.excluded_selected = None;
                state.new_active_selected = None;
                state.key_selected = None;
                state.error_msg = None;
                refresh_secondary_vars(ds, workspace, state);
            }
            ActionBarAction::Cancel => {
                state.is_open = false;
            }
            ActionBarAction::Help | ActionBarAction::None => {}
        }
    });

    if !is_open {
        state.is_open = false;
    }

    action
}

fn has_valid_source(state: &AddVarsDialogState) -> bool {
    match state.source_kind {
        MergeSourceKind::OpenSession(sid) => state.selected_session_id.is_some() || sid > 0,
        MergeSourceKind::ExternalFile => !state.external_file_path.trim().is_empty(),
    }
}

fn resolve_file_path(
    state: &AddVarsDialogState,
    workspace: Option<&WorkspaceManager>,
) -> Option<String> {
    match state.source_kind {
        MergeSourceKind::OpenSession(sid) => workspace.and_then(|ws| {
            ws.sessions().iter().find(|s| s.id == sid).and_then(|s| {
                s.file_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .or_else(|| Some(s.name.clone()))
            })
        }),
        MergeSourceKind::ExternalFile => {
            let p = state.external_file_path.trim().to_string();
            if p.is_empty() {
                None
            } else {
                Some(p)
            }
        }
    }
}

fn refresh_secondary_vars(
    active_ds: &DataSet,
    workspace: Option<&WorkspaceManager>,
    state: &mut AddVarsDialogState,
) {
    state.error_msg = None;
    let sec_ds = match state.source_kind {
        MergeSourceKind::OpenSession(sid) => workspace
            .and_then(|ws| ws.sessions().iter().find(|s| s.id == sid))
            .map(|s| s.dataset.clone()),
        MergeSourceKind::ExternalFile => {
            if state.external_file_path.trim().is_empty() {
                None
            } else {
                match load_dataset_from_file(&state.external_file_path) {
                    Ok(d) => Some(d),
                    Err(e) => {
                        state.error_msg = Some(e.to_string());
                        None
                    }
                }
            }
        }
    };

    if let Some(ref sec) = sec_ds {
        state.excluded_vars.clear();
        state.new_active_vars.clear();
        state.key_vars.clear();

        // Active dataset variables go into new_active_vars
        for i in 0..active_ds.dictionary.count() {
            if let Some(meta) = active_ds.dictionary.get(i) {
                state.new_active_vars.push(meta.name.clone());
            }
        }

        // Secondary variables: if collision, into excluded_vars; else into new_active_vars
        let act_names: HashSet<String> = active_ds
            .table
            .columns
            .iter()
            .map(|c| c.name.to_lowercase())
            .collect();

        for col in &sec.table.columns {
            let lower = col.name.to_lowercase();
            if act_names.contains(&lower) {
                // Potential key or collision
                if state.key_vars.is_empty() {
                    state.key_vars.push(col.name.clone());
                } else {
                    state.excluded_vars.push(col.name.clone());
                }
            } else {
                state.new_active_vars.push(col.name.clone());
            }
        }
    }

    state.secondary_dataset = sec_ds;
}

fn find_var_measure(name: &str, active_ds: &DataSet, sec_ds: Option<&DataSet>) -> Measure {
    if let Some(meta) = active_ds.dictionary.find_by_name(name) {
        meta.measure
    } else if let Some(s) = sec_ds {
        s.dictionary
            .find_by_name(name)
            .map(|m| m.measure)
            .unwrap_or(Measure::Scale)
    } else {
        Measure::Scale
    }
}
