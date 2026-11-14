use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct KMeansDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub variables: Vec<String>,
    pub target_selected: Option<usize>,
    pub clusters_str: String,
    pub max_iter_str: String,
    pub converge_str: String,
    pub save_cluster: bool,
    pub save_distance: bool,
}

impl Default for KMeansDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            variables: Vec::new(),
            target_selected: None,
            clusters_str: "3".to_string(),
            max_iter_str: "10".to_string(),
            converge_str: "0.0001".to_string(),
            save_cluster: false,
            save_distance: false,
        }
    }
}

pub enum KMeansDialogAction {
    Run {
        variables: Vec<String>,
        clusters: usize,
        max_iterations: usize,
        convergence_criterion: f64,
        save_cluster: bool,
        save_distance: bool,
    },
    Paste {
        variables: Vec<String>,
        clusters: usize,
        max_iterations: usize,
        convergence_criterion: f64,
        save_cluster: bool,
        save_distance: bool,
    },
    None,
}

pub fn render_kmeans_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut KMeansDialogState,
) -> KMeansDialogAction {
    if !state.is_open {
        return KMeansDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return KMeansDialogAction::None;
        }
    };

    let mut action = KMeansDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if state.variables.contains(&meta.name) {
                    None
                } else {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                }
            })
        })
        .collect();

    let shell = DialogShell::standard(t!("dialog-kmeans-title"));
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
                "kmeans_source_list",
                &source_items,
            );

            match src_event {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.target_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        state.variables.push(name.clone());
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // Middle column: Transfer button
            ui.vertical(|ui| {
                ui.add_space(80.0);
                let can_transfer_to_target = state.source_selected.is_some();
                let can_transfer_to_source = state.target_selected.is_some();

                if TransferButton::to_target(can_transfer_to_target)
                    .show(ui)
                    .clicked()
                {
                    if let Some(src_idx) = state.source_selected {
                        if let Some(item) = source_items.get(src_idx) {
                            state.variables.push(item.name.to_string());
                            state.source_selected = None;
                        }
                    }
                }

                ui.add_space(8.0);
                if TransferButton::to_source(can_transfer_to_source)
                    .show(ui)
                    .clicked()
                {
                    if let Some(tgt_idx) = state.target_selected {
                        if tgt_idx < state.variables.len() {
                            state.variables.remove(tgt_idx);
                            state.target_selected = None;
                        }
                    }
                }
            });

            // Right column: Target variables and settings
            ui.vertical(|ui| {
                let target_items: Vec<VariableListItem> = state
                    .variables
                    .iter()
                    .enumerate()
                    .map(|(idx, v_name)| {
                        let measure = ds
                            .dictionary
                            .find_by_name(v_name)
                            .map(|m| m.measure)
                            .unwrap_or(statcraft_core::Measure::Scale);
                        let label = ds
                            .dictionary
                            .find_by_name(v_name)
                            .map(|m| m.label.clone())
                            .unwrap_or_default();
                        VariableListItem {
                            name: v_name,
                            display_text: format_var_display(v_name, &label),
                            measure,
                            is_selected: state.target_selected == Some(idx),
                        }
                    })
                    .collect();

                let tgt_event = render_variable_list_panel(
                    ui,
                    t!("dialog-kmeans-selected-variables"),
                    vec2(220.0, 140.0),
                    "kmeans_target_list",
                    &target_items,
                );

                match tgt_event {
                    ListEvent::Clicked(idx) => {
                        state.target_selected = Some(idx);
                        state.source_selected = None;
                    }
                    ListEvent::DoubleClicked(idx) => {
                        if idx < state.variables.len() {
                            state.variables.remove(idx);
                            state.target_selected = None;
                        }
                    }
                    ListEvent::None => {}
                }

                ui.add_space(8.0);

                // Parameters GroupBox
                GroupBox::new(t!("dialog-kmeans-criteria")).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-kmeans-clusters-count"));
                        ui.text_edit_singleline(&mut state.clusters_str);
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-kmeans-max-iterations"));
                        ui.text_edit_singleline(&mut state.max_iter_str);
                    });
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-kmeans-convergence"));
                        ui.text_edit_singleline(&mut state.converge_str);
                    });
                });

                ui.add_space(8.0);

                // Save GroupBox
                GroupBox::new(t!("dialog-kmeans-save")).show(ui, |ui| {
                    ui.checkbox(&mut state.save_cluster, t!("dialog-kmeans-save-cluster"));
                    ui.checkbox(&mut state.save_distance, t!("dialog-kmeans-save-distance"));
                });
            });
        });

        ui.add_space(16.0);

        // Validation & Action Bar
        let k_val: usize = state.clusters_str.parse().unwrap_or(0);
        let max_iter_val: usize = state.max_iter_str.parse().unwrap_or(0);
        let converge_val: f64 = state.converge_str.parse().unwrap_or(-1.0);
        let is_valid =
            !state.variables.is_empty() && k_val >= 2 && max_iter_val >= 1 && converge_val >= 0.0;

        let bar_action = DialogActionBar::new(is_valid).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                action = KMeansDialogAction::Run {
                    variables: state.variables.clone(),
                    clusters: k_val,
                    max_iterations: max_iter_val,
                    convergence_criterion: converge_val,
                    save_cluster: state.save_cluster,
                    save_distance: state.save_distance,
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = KMeansDialogAction::Paste {
                    variables: state.variables.clone(),
                    clusters: k_val,
                    max_iterations: max_iter_val,
                    convergence_criterion: converge_val,
                    save_cluster: state.save_cluster,
                    save_distance: state.save_distance,
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = KMeansDialogState {
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
