pub mod subdialog_partition;
pub mod subdialog_screening;
pub mod subdialog_transform;

use crate::dialogs::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_algo::{
    AdpBinningOptions, AdpCategoricalOptions, AdpInteractionOptions, AdpObjective, AdpOptions,
    AdpOutlierOptions, AdpPartitionOptions, AdpRescaleMethod, AdpScreeningOptions, OutlierMethod,
    OutlierTreatment,
};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct AdpDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub inputs_selected: Option<usize>,
    pub weight_selected: Option<usize>,

    pub target_var: Option<String>,
    pub input_vars: Vec<String>,
    pub weight_var: Option<String>,
    pub objective: AdpObjective,

    // Subdialog visibility
    pub show_screening: bool,
    pub show_transform: bool,
    pub show_partition: bool,

    // Screening subdialog settings
    pub max_missing_pct_text: String,
    pub max_category_pct_text: String,
    pub max_unique_pct_text: String,
    pub outlier_method_idx: usize,
    pub outlier_threshold_text: String,
    pub outlier_action_idx: usize,

    // Transform subdialog settings
    pub rescale_method_idx: usize,
    pub merge_sparse: bool,
    pub bin_continuous: bool,
    pub create_interactions: bool,

    // Partition subdialog settings
    pub partition_enabled: bool,
    pub train_pct_text: String,
    pub test_pct_text: String,
    pub validate_pct_text: String,
    pub seed_text: String,
}

impl Default for AdpDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            inputs_selected: None,
            weight_selected: None,

            target_var: None,
            input_vars: Vec::new(),
            weight_var: None,
            objective: AdpObjective::BalanceSpeedAccuracy,

            show_screening: false,
            show_transform: false,
            show_partition: false,

            max_missing_pct_text: "50.0".to_string(),
            max_category_pct_text: "95.0".to_string(),
            max_unique_pct_text: "95.0".to_string(),
            outlier_method_idx: 1, // Z-Score
            outlier_threshold_text: "3.0".to_string(),
            outlier_action_idx: 0, // Winsorize

            rescale_method_idx: 0, // None
            merge_sparse: true,
            bin_continuous: false,
            create_interactions: false,

            partition_enabled: false,
            train_pct_text: "70".to_string(),
            test_pct_text: "20".to_string(),
            validate_pct_text: "10".to_string(),
            seed_text: "2026".to_string(),
        }
    }
}

impl AdpDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn build_options(&self) -> AdpOptions {
        let max_missing_pct = self
            .max_missing_pct_text
            .trim()
            .parse::<f64>()
            .unwrap_or(50.0);
        let max_category_pct = self
            .max_category_pct_text
            .trim()
            .parse::<f64>()
            .unwrap_or(95.0);
        let max_unique_pct = self
            .max_unique_pct_text
            .trim()
            .parse::<f64>()
            .unwrap_or(95.0);

        let outlier_thresh = self
            .outlier_threshold_text
            .trim()
            .parse::<f64>()
            .unwrap_or(3.0);
        let outlier_method = match self.outlier_method_idx {
            0 => OutlierMethod::None,
            2 => OutlierMethod::Iqr {
                multiplier: outlier_thresh,
            },
            _ => OutlierMethod::ZScore {
                threshold: outlier_thresh,
            },
        };

        let outlier_treatment = match self.outlier_action_idx {
            1 => OutlierTreatment::SetSysmis,
            _ => OutlierTreatment::Winsorize,
        };

        let rescale = match self.rescale_method_idx {
            1 => AdpRescaleMethod::ZScore,
            2 => AdpRescaleMethod::MinMax { min: 0.0, max: 1.0 },
            _ => AdpRescaleMethod::None,
        };

        let train_pct = self.train_pct_text.trim().parse::<f64>().unwrap_or(70.0);
        let test_pct = self.test_pct_text.trim().parse::<f64>().unwrap_or(20.0);
        let validate_pct = self.validate_pct_text.trim().parse::<f64>().unwrap_or(10.0);
        let seed = self.seed_text.trim().parse::<u64>().ok();

        AdpOptions {
            objective: self.objective,
            target_variable: self.target_var.clone(),
            input_variables: self.input_vars.clone(),
            weight_variable: self.weight_var.clone(),
            screening: AdpScreeningOptions {
                max_missing_pct,
                max_category_pct,
                max_unique_pct,
            },
            outliers: AdpOutlierOptions {
                method: outlier_method,
                treatment: outlier_treatment,
            },
            rescale,
            categorical: AdpCategoricalOptions {
                merge_sparse: self.merge_sparse,
                sparse_threshold_pct: 5.0,
            },
            binning: AdpBinningOptions {
                bin_continuous_inputs: self.bin_continuous,
                max_bins: 5,
            },
            interactions: AdpInteractionOptions {
                create_interactions: self.create_interactions,
                max_pairs: 5,
            },
            partition: AdpPartitionOptions {
                enabled: self.partition_enabled,
                train_pct,
                test_pct,
                validate_pct,
                seed,
            },
        }
    }
}

pub enum AdpDialogAction {
    Run(AdpOptions),
    Paste(AdpOptions),
    None,
}

pub fn render_adp_dialog(
    ctx: &egui::Context,
    state: &mut AdpDialogState,
    dataset: Option<&DataSet>,
) -> AdpDialogAction {
    if !state.is_open {
        return AdpDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return AdpDialogAction::None;
        }
    };

    // Subdialogs
    subdialog_screening::render_screening_subdialog(ctx, state);
    subdialog_transform::render_transform_subdialog(ctx, state);
    subdialog_partition::render_partition_subdialog(ctx, state);

    let mut action = AdpDialogAction::None;
    let mut close_dialog = false;

    // Available variables excluding target, inputs, and weight
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_used = state.input_vars.contains(&meta.name)
                    || state.target_var.as_ref() == Some(&meta.name)
                    || state.weight_var.as_ref() == Some(&meta.name);
                if !is_used {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-adp-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left Panel: Available Variables (190 x 280)
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
                vec2(190.0, 280.0),
                "adp_source_vars",
                &source_items,
            );

            if let ListEvent::Clicked(idx) = source_ev {
                state.source_selected = Some(idx);
                state.inputs_selected = None;
            }

            // Center Panel: Target, Inputs, Weight & Transfers (210 x 280)
            ui.allocate_ui_with_layout(
                vec2(210.0, 280.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // Target Section
                    ui.horizontal(|ui| {
                        let can_move_target =
                            state.source_selected.is_some() && state.target_var.is_none();
                        let target_btn = if state.target_var.is_some() {
                            TransferButton::to_source(true)
                        } else {
                            TransferButton::to_target(can_move_target)
                        };
                        if target_btn.show(ui).clicked() {
                            if state.target_var.is_some() {
                                state.target_var = None;
                            } else if let Some(idx) = state.source_selected {
                                if let Some((_, name, _, _)) = available_vars.get(idx) {
                                    state.target_var = Some(name.clone());
                                    state.source_selected = None;
                                }
                            }
                        }
                        ui.vertical(|ui| {
                            ui.label(t!("dialog-adp-target-label"));
                            let tgt_display = state.target_var.as_deref().unwrap_or("-");
                            ui.label(egui::RichText::new(tgt_display).strong());
                        });
                    });

                    ui.add_space(8.0);

                    // Inputs Section
                    ui.horizontal(|ui| {
                        let can_move_input = state.source_selected.is_some();
                        let input_btn = if state.inputs_selected.is_some() {
                            TransferButton::to_source(true)
                        } else {
                            TransferButton::to_target(can_move_input)
                        };

                        if input_btn.show(ui).clicked() {
                            if let Some(idx) = state.inputs_selected {
                                if idx < state.input_vars.len() {
                                    state.input_vars.remove(idx);
                                    state.inputs_selected = None;
                                }
                            } else if let Some(idx) = state.source_selected {
                                if let Some((_, name, _, _)) = available_vars.get(idx) {
                                    state.input_vars.push(name.clone());
                                    state.source_selected = None;
                                }
                            }
                        }

                        let input_items: Vec<VariableListItem> = state
                            .input_vars
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
                                    is_selected: state.inputs_selected == Some(idx),
                                }
                            })
                            .collect();

                        let input_ev = render_variable_list_panel(
                            ui,
                            t!("dialog-adp-inputs-label"),
                            vec2(160.0, 140.0),
                            "adp_input_vars",
                            &input_items,
                        );

                        if let ListEvent::Clicked(idx) = input_ev {
                            state.inputs_selected = Some(idx);
                            state.source_selected = None;
                        }
                    });

                    ui.add_space(8.0);

                    // Objective Selector
                    GroupBox::new(t!("dialog-adp-group-objective")).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut state.objective,
                                AdpObjective::BalanceSpeedAccuracy,
                                t!("output-adp-obj-balance"),
                            );
                            ui.radio_value(
                                &mut state.objective,
                                AdpObjective::OptimizeAccuracy,
                                t!("output-adp-obj-accuracy"),
                            );
                        });
                    });
                },
            );

            // Right Sidebar (94 x 280)
            ui.allocate_ui_with_layout(
                vec2(94.0, 280.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(14.0);
                    if SubdialogSidebar::button(ui, t!("dialog-adp-btn-screening"), true).clicked()
                    {
                        state.show_screening = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-adp-btn-transform"), true).clicked()
                    {
                        state.show_transform = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-adp-btn-partition"), true).clicked()
                    {
                        state.show_partition = true;
                    }
                },
            );
        });

        // Bottom Action Bar
        let can_run = !state.input_vars.is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = AdpDialogAction::Run(state.build_options());
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = AdpDialogAction::Paste(state.build_options());
            }
            ActionBarAction::Reset => {
                *state = AdpDialogState::default();
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
