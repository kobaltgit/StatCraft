use super::source_panel::render_source_panel;
use super::types::{AddCasesDialogState, MergeCasesAction, MergeSourceKind};
use crate::dialogs::common::{
    render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell, ListEvent,
    TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::dataset::merge::load_dataset_from_file;
use statcraft_core::workspace::WorkspaceManager;
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;
use std::collections::{HashMap, HashSet};

pub fn render_add_cases_dialog(
    ctx: &egui::Context,
    active_dataset: Option<&DataSet>,
    workspace: Option<&WorkspaceManager>,
    state: &mut AddCasesDialogState,
) -> MergeCasesAction {
    if !state.is_open {
        return MergeCasesAction::None;
    }

    let ds = match active_dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return MergeCasesAction::None;
        }
    };

    let mut action = MergeCasesAction::None;
    let mut is_open = true;

    DialogShell::standard(t!("dialog-merge-cases-title")).show(ctx, &mut is_open, |ui| {
        // 1. Source selector
        let source_changed = render_source_panel(
            ui,
            &mut state.source_kind,
            &mut state.external_file_path,
            &mut state.selected_session_id,
            workspace,
        );

        if source_changed || (state.secondary_dataset.is_none() && has_valid_source(state)) {
            refresh_secondary_cases(ds, workspace, state);
        }

        ui.add_space(6.0);

        // 2. Middle lists (Unpaired <-> Included)
        ui.horizontal(|ui| {
            // Left list: Unpaired variables
            let unpaired_items: Vec<VariableListItem> = state
                .unpaired_vars
                .iter()
                .enumerate()
                .map(|(idx, (name, is_act))| {
                    let suffix = if *is_act { " (*)" } else { " (+)" };
                    let display_text = format!("{}{}", name, suffix);
                    let measure =
                        find_var_measure(name, *is_act, ds, state.secondary_dataset.as_ref());
                    VariableListItem {
                        name,
                        display_text,
                        measure,
                        is_selected: state.unpaired_selected == Some(idx),
                    }
                })
                .collect();

            let left_event = render_variable_list_panel(
                ui,
                t!("dialog-merge-unpaired-vars"),
                vec2(200.0, 240.0),
                "merge_cases_unpaired",
                &unpaired_items,
            );

            match left_event {
                ListEvent::Clicked(idx) => state.unpaired_selected = Some(idx),
                ListEvent::DoubleClicked(idx) => {
                    if idx < state.unpaired_vars.len() {
                        let (var_name, _) = state.unpaired_vars.remove(idx);
                        if !state.included_vars.contains(&var_name) {
                            state.included_vars.push(var_name);
                        }
                        state.unpaired_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Transfer buttons
            ui.vertical(|ui| {
                ui.add_space(80.0);
                let can_move_right = state.unpaired_selected.is_some();
                if TransferButton::to_target(can_move_right).show(ui).clicked() {
                    if let Some(idx) = state.unpaired_selected {
                        if idx < state.unpaired_vars.len() {
                            let (var_name, _) = state.unpaired_vars.remove(idx);
                            if !state.included_vars.contains(&var_name) {
                                state.included_vars.push(var_name);
                            }
                            state.unpaired_selected = None;
                        }
                    }
                }

                ui.add_space(6.0);

                let can_move_left = state.included_selected.is_some();
                if TransferButton::to_source(can_move_left).show(ui).clicked() {
                    if let Some(idx) = state.included_selected {
                        if idx < state.included_vars.len() {
                            let var_name = state.included_vars.remove(idx);
                            state.unpaired_vars.push((var_name, true));
                            state.included_selected = None;
                        }
                    }
                }
            });

            // Right list: Included variables in new active dataset
            let included_items: Vec<VariableListItem> = state
                .included_vars
                .iter()
                .enumerate()
                .map(|(idx, name)| {
                    let measure =
                        find_var_measure(name, true, ds, state.secondary_dataset.as_ref());
                    VariableListItem {
                        name,
                        display_text: name.clone(),
                        measure,
                        is_selected: state.included_selected == Some(idx),
                    }
                })
                .collect();

            let right_event = render_variable_list_panel(
                ui,
                t!("dialog-merge-new-active-vars"),
                vec2(200.0, 240.0),
                "merge_cases_included",
                &included_items,
            );

            match right_event {
                ListEvent::Clicked(idx) => state.included_selected = Some(idx),
                ListEvent::DoubleClicked(idx) => {
                    if idx < state.included_vars.len() {
                        let var_name = state.included_vars.remove(idx);
                        state.unpaired_vars.push((var_name, true));
                        state.included_selected = None;
                    }
                }
                ListEvent::None => {}
            }
        });

        ui.add_space(6.0);

        // 3. Indicate source variable
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

        ui.add_space(8.0);

        // 4. Action bar
        let is_ready = state.secondary_dataset.is_some() && !state.included_vars.is_empty();
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
                    action = MergeCasesAction::Run {
                        file_path: path,
                        in_variable: in_var,
                        renames: HashMap::new(),
                        interleave_by: None,
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
                    action = MergeCasesAction::Paste {
                        file_path: path,
                        in_variable: in_var,
                        renames: HashMap::new(),
                        interleave_by: None,
                    };
                    state.is_open = false;
                } else {
                    state.error_msg = Some(t!("dialog-merge-error-no-source"));
                }
            }
            ActionBarAction::Reset => {
                state.secondary_dataset = None;
                state.unpaired_vars.clear();
                state.included_vars.clear();
                state.unpaired_selected = None;
                state.included_selected = None;
                state.error_msg = None;
                refresh_secondary_cases(ds, workspace, state);
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

fn has_valid_source(state: &AddCasesDialogState) -> bool {
    match state.source_kind {
        MergeSourceKind::OpenSession(sid) => state.selected_session_id.is_some() || sid > 0,
        MergeSourceKind::ExternalFile => !state.external_file_path.trim().is_empty(),
    }
}

fn resolve_file_path(
    state: &AddCasesDialogState,
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

fn refresh_secondary_cases(
    active_ds: &DataSet,
    workspace: Option<&WorkspaceManager>,
    state: &mut AddCasesDialogState,
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
        state.included_vars.clear();
        state.unpaired_vars.clear();

        let mut sec_names: HashSet<String> = HashSet::new();
        for col in &sec.table.columns {
            sec_names.insert(col.name.to_lowercase());
        }

        // Pair active variables that exist and have compatible types in secondary
        for i in 0..active_ds.dictionary.count() {
            if let Some(a_meta) = active_ds.dictionary.get(i) {
                let lower = a_meta.name.to_lowercase();
                if sec_names.contains(&lower) {
                    state.included_vars.push(a_meta.name.clone());
                } else {
                    state.unpaired_vars.push((a_meta.name.clone(), true));
                }
            }
        }

        // Unpaired secondary variables
        let act_names: HashSet<String> = (0..active_ds.dictionary.count())
            .filter_map(|i| active_ds.dictionary.get(i).map(|m| m.name.to_lowercase()))
            .collect();

        for col in &sec.table.columns {
            if !act_names.contains(&col.name.to_lowercase()) {
                state.unpaired_vars.push((col.name.clone(), false));
            }
        }
    }

    state.secondary_dataset = sec_ds;
}

fn find_var_measure(
    name: &str,
    is_act: bool,
    active_ds: &DataSet,
    sec_ds: Option<&DataSet>,
) -> Measure {
    if is_act {
        active_ds
            .dictionary
            .find_by_name(name)
            .map(|m| m.measure)
            .unwrap_or(Measure::Scale)
    } else if let Some(s) = sec_ds {
        s.dictionary
            .find_by_name(name)
            .map(|m| m.measure)
            .unwrap_or(Measure::Scale)
    } else {
        Measure::Scale
    }
}
