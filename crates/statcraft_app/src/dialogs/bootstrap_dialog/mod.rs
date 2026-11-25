pub mod state;

pub use state::*;

use super::common::{
    format_var_display, render_variable_list_panel, DialogShell, DialogSize, GroupBox, ListEvent,
    SubdialogAction, SubdialogActionBar, TransferButton, VariableListItem,
};
use egui::{vec2, TextEdit};
use statcraft_algo::BootstrapCiMethod;
use statcraft_core::DataSet;
use statcraft_i18n::t;

pub fn render_bootstrap_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut BootstrapDialogState,
) {
    if !state.is_open {
        return;
    }

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = match dataset {
        Some(ds) => (0..ds.col_count())
            .filter_map(|col_idx| {
                ds.dictionary.get(col_idx).and_then(|meta| {
                    if !state.strata_vars.contains(&meta.name) {
                        Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                    } else {
                        None
                    }
                })
            })
            .collect(),
        None => Vec::new(),
    };

    let mut win_open = true;
    DialogShell::new(
        t!("dialog-bootstrap-title"),
        DialogSize::Custom(vec2(440.0, 480.0)),
    )
    .show(ctx, &mut win_open, |ui| {
        ui.checkbox(&mut state.enabled, t!("dialog-bootstrap-perform"));
        ui.add_space(6.0);

        ui.add_enabled_ui(state.enabled, |ui| {
            ui.horizontal(|ui| {
                ui.label(t!("dialog-bootstrap-samples"));
                ui.add(TextEdit::singleline(&mut state.samples_input).desired_width(80.0));
            });

            ui.add_space(4.0);
            ui.checkbox(&mut state.use_seed, t!("dialog-bootstrap-set-seed"));
            ui.add_enabled_ui(state.use_seed, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    ui.label(t!("dialog-bootstrap-seed-value"));
                    ui.add(TextEdit::singleline(&mut state.seed_input).desired_width(100.0));
                });
            });

            GroupBox::new(t!("dialog-bootstrap-ci-group")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-bootstrap-ci-level"));
                    ui.add(TextEdit::singleline(&mut state.ci_level_input).desired_width(60.0));
                });
                ui.add_space(2.0);
                ui.radio_value(
                    &mut state.ci_method,
                    BootstrapCiMethod::Percentile,
                    t!("dialog-bootstrap-ci-percentile"),
                );
                ui.radio_value(
                    &mut state.ci_method,
                    BootstrapCiMethod::Bca,
                    t!("dialog-bootstrap-ci-bca"),
                );
            });

            GroupBox::new(t!("dialog-bootstrap-sampling-group")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut state.sampling_method,
                        BootstrapSamplingUi::Simple,
                        t!("dialog-bootstrap-sampling-simple"),
                    );
                    ui.add_space(16.0);
                    ui.radio_value(
                        &mut state.sampling_method,
                        BootstrapSamplingUi::Stratified,
                        t!("dialog-bootstrap-sampling-stratified"),
                    );
                });

                ui.add_space(4.0);
                let is_stratified = state.sampling_method == BootstrapSamplingUi::Stratified;
                ui.add_enabled_ui(is_stratified, |ui| {
                    render_strata_picker(ui, dataset, &available_vars, state);
                });
            });
        });

        let action = SubdialogActionBar::new(state.can_continue()).show(ui);
        match action {
            SubdialogAction::Continue => state.commit(),
            SubdialogAction::Cancel => state.cancel(),
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !win_open {
        state.cancel();
    }
}

fn render_strata_picker(
    ui: &mut egui::Ui,
    dataset: Option<&DataSet>,
    available_vars: &[(usize, String, String, statcraft_core::Measure)],
    state: &mut BootstrapDialogState,
) {
    ui.horizontal(|ui| {
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
            vec2(170.0, 120.0),
            "boot_strata_source",
            &source_items,
        );

        match source_ev {
            ListEvent::Clicked(idx) => {
                state.source_selected = Some(idx);
                state.target_selected = None;
            }
            ListEvent::DoubleClicked(idx) => {
                if let Some((_, name, _, _)) = available_vars.get(idx) {
                    state.strata_vars.push(name.clone());
                    state.source_selected = None;
                }
            }
            ListEvent::None => {}
        }

        let can_move_right = state
            .source_selected
            .is_some_and(|i| i < available_vars.len());
        let can_move_left = state
            .target_selected
            .is_some_and(|i| i < state.strata_vars.len());

        let transfer_btn = if state.target_selected.is_some() {
            TransferButton::to_source(can_move_left)
        } else {
            TransferButton::to_target(can_move_right)
        };

        ui.allocate_ui_with_layout(
            vec2(32.0, 120.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space(40.0);
                if transfer_btn.show(ui).clicked() {
                    if let Some(idx) = state.target_selected {
                        if idx < state.strata_vars.len() {
                            state.strata_vars.remove(idx);
                            state.target_selected = None;
                        }
                    } else if let Some(idx) = state.source_selected {
                        if let Some((_, name, _, _)) = available_vars.get(idx) {
                            state.strata_vars.push(name.clone());
                            state.source_selected = None;
                        }
                    }
                }
            },
        );

        let target_items: Vec<VariableListItem> = state
            .strata_vars
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let meta = dataset.and_then(|ds| ds.dictionary.find_by_name(name));
                let measure = meta
                    .map(|m| m.measure)
                    .unwrap_or(statcraft_core::Measure::Nominal);
                let label = meta.map(|m| m.label.as_str()).unwrap_or("");
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
            t!("dialog-bootstrap-strata-vars"),
            vec2(170.0, 120.0),
            "boot_strata_target",
            &target_items,
        );

        match target_ev {
            ListEvent::Clicked(idx) => {
                state.target_selected = Some(idx);
                state.source_selected = None;
            }
            ListEvent::DoubleClicked(idx) => {
                if idx < state.strata_vars.len() {
                    state.strata_vars.remove(idx);
                    state.target_selected = None;
                }
            }
            ListEvent::None => {}
        }
    });
}
