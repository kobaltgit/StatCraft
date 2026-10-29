use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use egui::vec2;
use statcraft_algo::{FactorExtractionCriteria, FactorExtractionMethod, FactorOptions};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct FactorDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub extraction_method: FactorExtractionMethod,
    pub extract_by_mineigen: bool,
    pub mineigen_str: String,
    pub nfactors_str: String,
    pub kmo_bartlett: bool,
    pub show_descriptives_subdialog: bool,
    pub show_extraction_subdialog: bool,
}

impl Default for FactorDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            extraction_method: FactorExtractionMethod::PC,
            extract_by_mineigen: true,
            mineigen_str: "1.0".to_string(),
            nfactors_str: "2".to_string(),
            kmo_bartlett: true,
            show_descriptives_subdialog: false,
            show_extraction_subdialog: false,
        }
    }
}

pub enum FactorDialogAction {
    Run {
        variables: Vec<String>,
        options: FactorOptions,
    },
    Paste {
        variables: Vec<String>,
        options: FactorOptions,
    },
    None,
}

impl FactorDialogState {
    pub fn build_options(&self) -> FactorOptions {
        let criteria = if self.extract_by_mineigen {
            let cutoff = self.mineigen_str.parse::<f64>().unwrap_or(1.0);
            FactorExtractionCriteria::Mineigen(cutoff)
        } else {
            let k = self.nfactors_str.parse::<usize>().unwrap_or(2).max(1);
            FactorExtractionCriteria::Factors(k)
        };

        FactorOptions {
            extraction: self.extraction_method,
            criteria,
            kmo_bartlett: self.kmo_bartlett,
        }
    }
}

pub fn render_factor_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut FactorDialogState,
) -> FactorDialogAction {
    if !state.is_open {
        return FactorDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return FactorDialogAction::None;
        }
    };

    let mut action = FactorDialogAction::None;
    let mut close_dialog = false;

    // Filter available numeric variables
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

    // 1. Descriptives Subdialog
    if state.show_descriptives_subdialog {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-factor-descriptives-title")).show(
            ctx,
            &mut sub_open,
            |ui| {
                GroupBox::new(t!("dialog-factor-correlation-matrix-group")).show(ui, |ui| {
                    ui.checkbox(
                        &mut state.kmo_bartlett,
                        t!("dialog-factor-kmo-and-bartlett"),
                    );
                });

                ui.add_space(14.0);

                let sub_action = SubdialogActionBar::new(true).show(ui);
                match sub_action {
                    SubdialogAction::Continue | SubdialogAction::Cancel => {
                        state.show_descriptives_subdialog = false;
                    }
                    SubdialogAction::Help | SubdialogAction::None => {}
                }
            },
        );
        if !sub_open {
            state.show_descriptives_subdialog = false;
        }
    }

    // 2. Extraction Subdialog
    if state.show_extraction_subdialog {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-factor-extraction-title")).show(
            ctx,
            &mut sub_open,
            |ui| {
                GroupBox::new(t!("dialog-factor-method-group")).show(ui, |ui| {
                    ui.label(t!("dialog-factor-method-pca"));
                });

                ui.add_space(8.0);

                GroupBox::new(t!("dialog-factor-extract-group")).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut state.extract_by_mineigen,
                            true,
                            t!("dialog-factor-based-on-eigen"),
                        );
                        ui.add_space(8.0);
                        ui.add_enabled(
                            state.extract_by_mineigen,
                            egui::TextEdit::singleline(&mut state.mineigen_str).desired_width(50.0),
                        );
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut state.extract_by_mineigen,
                            false,
                            t!("dialog-factor-number-of-factors"),
                        );
                        ui.add_space(8.0);
                        ui.add_enabled(
                            !state.extract_by_mineigen,
                            egui::TextEdit::singleline(&mut state.nfactors_str).desired_width(50.0),
                        );
                    });
                });

                ui.add_space(14.0);

                let sub_action = SubdialogActionBar::new(true).show(ui);
                match sub_action {
                    SubdialogAction::Continue | SubdialogAction::Cancel => {
                        state.show_extraction_subdialog = false;
                    }
                    SubdialogAction::Help | SubdialogAction::None => {}
                }
            },
        );
        if !sub_open {
            state.show_extraction_subdialog = false;
        }
    }

    // Main Factor Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-factor-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Source variables panel
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
                vec2(190.0, 190.0),
                "factor_source_vars",
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

            // Transfer button
            let can_move_right = state.source_selected.is_some()
                && state.source_selected.unwrap_or(usize::MAX) < available_vars.len();
            let can_move_left = state.target_selected.is_some()
                && state.target_selected.unwrap_or(usize::MAX) < state.selected_vars.len();

            let transfer_btn = if state.target_selected.is_some() {
                TransferButton::to_source(can_move_left)
            } else {
                TransferButton::to_target(can_move_right)
            };

            ui.allocate_ui_with_layout(
                vec2(36.0, 190.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(60.0);
                    if transfer_btn.show(ui).clicked() {
                        if let Some(idx) = state.target_selected {
                            if idx < state.selected_vars.len() {
                                state.selected_vars.remove(idx);
                                state.target_selected = None;
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

            // Target Items panel
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
                t!("dialog-factor-variables-label"),
                vec2(190.0, 190.0),
                "factor_target_vars",
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

            // Right sidebar buttons
            ui.allocate_ui_with_layout(
                vec2(90.0, 190.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(18.0);
                    if SubdialogSidebar::button(ui, t!("dialog-factor-descriptives-btn"), true)
                        .clicked()
                    {
                        state.show_descriptives_subdialog = true;
                    }
                    ui.add_space(6.0);
                    if SubdialogSidebar::button(ui, t!("dialog-factor-extraction-btn"), true)
                        .clicked()
                    {
                        state.show_extraction_subdialog = true;
                    }
                },
            );
        });

        ui.add_space(14.0);

        // Bottom Action Bar
        let can_run = state.selected_vars.len() >= 2;
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                let options = state.build_options();
                action = FactorDialogAction::Run {
                    variables: state.selected_vars.clone(),
                    options,
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                let options = state.build_options();
                action = FactorDialogAction::Paste {
                    variables: state.selected_vars.clone(),
                    options,
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = FactorDialogState {
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

    if !main_open || close_dialog {
        state.is_open = false;
    }

    action
}
