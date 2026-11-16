mod subdialogs;

use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;
use subdialogs::{render_ref_category_subdialog, render_stats_subdialog};

#[derive(Debug, Clone)]
pub struct MultinomialLogisticDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_var: Option<String>,
    pub factors: Vec<String>,
    pub covariates: Vec<String>,
    pub target_dep_selected: bool,
    pub target_factor_selected: Option<usize>,
    pub target_cov_selected: Option<usize>,
    pub reference_category: String, // "last", "first", "custom"
    pub custom_ref_val: String,
    pub ci_str: String,
    pub include_constant: bool,
    pub print_classification: bool,
    pub print_goodness_of_fit: bool,
    pub print_lrt: bool,
    pub ref_dialog_open: bool,
    pub stats_dialog_open: bool,
}

impl Default for MultinomialLogisticDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            dep_var: None,
            factors: Vec::new(),
            covariates: Vec::new(),
            target_dep_selected: false,
            target_factor_selected: None,
            target_cov_selected: None,
            reference_category: "last".to_string(),
            custom_ref_val: String::new(),
            ci_str: "95".to_string(),
            include_constant: true,
            print_classification: true,
            print_goodness_of_fit: true,
            print_lrt: true,
            ref_dialog_open: false,
            stats_dialog_open: false,
        }
    }
}

pub enum MultinomialLogisticDialogAction {
    Run {
        dep_var: String,
        factors: Vec<String>,
        covariates: Vec<String>,
        reference_category: String,
        reference_value: Option<f64>,
        confidence_level: f64,
        include_constant: bool,
        print_classification: bool,
        print_goodness_of_fit: bool,
        print_lrt: bool,
    },
    Paste {
        dep_var: String,
        factors: Vec<String>,
        covariates: Vec<String>,
        reference_category: String,
        reference_value: Option<f64>,
        confidence_level: f64,
        include_constant: bool,
        print_classification: bool,
        print_goodness_of_fit: bool,
        print_lrt: bool,
    },
    None,
}

pub fn render_multinomial_logistic_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut MultinomialLogisticDialogState,
) -> MultinomialLogisticDialogAction {
    if !state.is_open {
        return MultinomialLogisticDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return MultinomialLogisticDialogAction::None;
        }
    };

    if state.ref_dialog_open {
        render_ref_category_subdialog(ctx, state);
    }
    if state.stats_dialog_open {
        render_stats_subdialog(ctx, state);
    }

    let mut action = MultinomialLogisticDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                let is_dep = state.dep_var.as_ref() == Some(&meta.name);
                let is_factor = state.factors.contains(&meta.name);
                let is_cov = state.covariates.contains(&meta.name);
                if !is_dep && !is_factor && !is_cov {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-multinomial-logistic-title")).show(
        ctx,
        &mut main_open,
        |ui| {
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
                    vec2(180.0, 240.0),
                    "multinomial_source_vars",
                    &source_items,
                );

                match source_ev {
                    ListEvent::Clicked(idx) => {
                        state.source_selected = Some(idx);
                        state.target_dep_selected = false;
                        state.target_factor_selected = None;
                        state.target_cov_selected = None;
                    }
                    ListEvent::DoubleClicked(idx) => {
                        if let Some((_, name, _, _)) = available_vars.get(idx) {
                            if state.dep_var.is_none() {
                                state.dep_var = Some(name.clone());
                            } else {
                                state.covariates.push(name.clone());
                            }
                            state.source_selected = None;
                        }
                    }
                    ListEvent::None => {}
                }

                let can_right = state.source_selected.is_some()
                    && state.source_selected.unwrap() < available_vars.len();
                let can_left_dep = state.target_dep_selected && state.dep_var.is_some();
                let can_left_fac = state.target_factor_selected.is_some()
                    && state.target_factor_selected.unwrap() < state.factors.len();
                let can_left_cov = state.target_cov_selected.is_some()
                    && state.target_cov_selected.unwrap() < state.covariates.len();

                let transfer_dep_btn = if state.target_dep_selected {
                    TransferButton::to_source(can_left_dep)
                } else {
                    TransferButton::to_target(can_right)
                };
                let transfer_fac_btn = if state.target_factor_selected.is_some() {
                    TransferButton::to_source(can_left_fac)
                } else {
                    TransferButton::to_target(can_right)
                };
                let transfer_cov_btn = if state.target_cov_selected.is_some() {
                    TransferButton::to_source(can_left_cov)
                } else {
                    TransferButton::to_target(can_right)
                };

                ui.allocate_ui_with_layout(
                    vec2(36.0, 240.0),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.add_space(14.0);
                        if transfer_dep_btn.show(ui).clicked() {
                            if state.target_dep_selected {
                                state.dep_var = None;
                                state.target_dep_selected = false;
                            } else if let Some(idx) = state.source_selected {
                                if let Some((_, name, _, _)) = available_vars.get(idx) {
                                    state.dep_var = Some(name.clone());
                                    state.source_selected = None;
                                }
                            }
                        }

                        ui.add_space(42.0);
                        if transfer_fac_btn.show(ui).clicked() {
                            if let Some(idx) = state.target_factor_selected {
                                if idx < state.factors.len() {
                                    state.factors.remove(idx);
                                    state.target_factor_selected = None;
                                }
                            } else if let Some(idx) = state.source_selected {
                                if let Some((_, name, _, _)) = available_vars.get(idx) {
                                    state.factors.push(name.clone());
                                    state.source_selected = None;
                                }
                            }
                        }

                        ui.add_space(48.0);
                        if transfer_cov_btn.show(ui).clicked() {
                            if let Some(idx) = state.target_cov_selected {
                                if idx < state.covariates.len() {
                                    state.covariates.remove(idx);
                                    state.target_cov_selected = None;
                                }
                            } else if let Some(idx) = state.source_selected {
                                if let Some((_, name, _, _)) = available_vars.get(idx) {
                                    state.covariates.push(name.clone());
                                    state.source_selected = None;
                                }
                            }
                        }
                    },
                );

                // Right Targets Column
                ui.allocate_ui_with_layout(
                    vec2(210.0, 240.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label(t!("dialog-multinomial-dependent"));
                            if ui
                                .small_button(t!("dialog-multinomial-btn-ref-cat"))
                                .clicked()
                            {
                                state.ref_dialog_open = true;
                            }
                        });

                        let dep_text = if let Some(ref name) = state.dep_var {
                            let label = ds
                                .dictionary
                                .find_by_name(name)
                                .map(|m| m.label.clone())
                                .unwrap_or_default();
                            format_var_display(name, &label)
                        } else {
                            "".to_string()
                        };

                        let dep_resp = ui.selectable_label(state.target_dep_selected, &dep_text);
                        if dep_resp.clicked() {
                            state.target_dep_selected = !state.target_dep_selected;
                            state.source_selected = None;
                            state.target_factor_selected = None;
                            state.target_cov_selected = None;
                        }

                        ui.add_space(6.0);

                        // Factors List
                        let factor_items: Vec<VariableListItem> = state
                            .factors
                            .iter()
                            .enumerate()
                            .map(|(idx, name)| {
                                let (label, measure) = ds
                                    .dictionary
                                    .find_by_name(name)
                                    .map(|m| (m.label.clone(), m.measure))
                                    .unwrap_or_else(|| {
                                        (String::new(), statcraft_core::Measure::Nominal)
                                    });
                                VariableListItem {
                                    name,
                                    display_text: format_var_display(name, &label),
                                    measure,
                                    is_selected: state.target_factor_selected == Some(idx),
                                }
                            })
                            .collect();

                        let fac_ev = render_variable_list_panel(
                            ui,
                            t!("dialog-multinomial-factors"),
                            vec2(210.0, 68.0),
                            "multinomial_factor_vars",
                            &factor_items,
                        );
                        if let ListEvent::Clicked(idx) = fac_ev {
                            state.target_factor_selected = Some(idx);
                            state.source_selected = None;
                            state.target_dep_selected = false;
                            state.target_cov_selected = None;
                        }

                        ui.add_space(6.0);

                        // Covariates List
                        let cov_items: Vec<VariableListItem> = state
                            .covariates
                            .iter()
                            .enumerate()
                            .map(|(idx, name)| {
                                let (label, measure) = ds
                                    .dictionary
                                    .find_by_name(name)
                                    .map(|m| (m.label.clone(), m.measure))
                                    .unwrap_or_else(|| {
                                        (String::new(), statcraft_core::Measure::Scale)
                                    });
                                VariableListItem {
                                    name,
                                    display_text: format_var_display(name, &label),
                                    measure,
                                    is_selected: state.target_cov_selected == Some(idx),
                                }
                            })
                            .collect();

                        let cov_ev = render_variable_list_panel(
                            ui,
                            t!("dialog-multinomial-covariates"),
                            vec2(210.0, 68.0),
                            "multinomial_cov_vars",
                            &cov_items,
                        );
                        if let ListEvent::Clicked(idx) = cov_ev {
                            state.target_cov_selected = Some(idx);
                            state.source_selected = None;
                            state.target_dep_selected = false;
                            state.target_factor_selected = None;
                        }
                    },
                );

                // Subdialog Sidebar on right
                ui.vertical(|ui| {
                    ui.add_space(20.0);
                    if SubdialogSidebar::button(ui, t!("dialog-multinomial-btn-statistics"), true)
                        .clicked()
                    {
                        state.stats_dialog_open = true;
                    }
                });
            });

            ui.add_space(10.0);

            // Action Bar (OK, Paste, Reset, Cancel, Help)
            let can_run = state.dep_var.is_some()
                && (!state.factors.is_empty() || !state.covariates.is_empty());
            let bar = DialogActionBar::new(can_run);
            match bar.show(ui) {
                ActionBarAction::Ok => {
                    if let Some(ref dep) = state.dep_var {
                        let ci = state.ci_str.parse::<f64>().unwrap_or(95.0) / 100.0;
                        let ref_val = state.custom_ref_val.parse::<f64>().ok();
                        action = MultinomialLogisticDialogAction::Run {
                            dep_var: dep.clone(),
                            factors: state.factors.clone(),
                            covariates: state.covariates.clone(),
                            reference_category: state.reference_category.clone(),
                            reference_value: ref_val,
                            confidence_level: ci,
                            include_constant: state.include_constant,
                            print_classification: state.print_classification,
                            print_goodness_of_fit: state.print_goodness_of_fit,
                            print_lrt: state.print_lrt,
                        };
                        close_dialog = true;
                    }
                }
                ActionBarAction::Paste => {
                    if let Some(ref dep) = state.dep_var {
                        let ci = state.ci_str.parse::<f64>().unwrap_or(95.0) / 100.0;
                        let ref_val = state.custom_ref_val.parse::<f64>().ok();
                        action = MultinomialLogisticDialogAction::Paste {
                            dep_var: dep.clone(),
                            factors: state.factors.clone(),
                            covariates: state.covariates.clone(),
                            reference_category: state.reference_category.clone(),
                            reference_value: ref_val,
                            confidence_level: ci,
                            include_constant: state.include_constant,
                            print_classification: state.print_classification,
                            print_goodness_of_fit: state.print_goodness_of_fit,
                            print_lrt: state.print_lrt,
                        };
                        close_dialog = true;
                    }
                }
                ActionBarAction::Reset => {
                    *state = MultinomialLogisticDialogState {
                        is_open: true,
                        ..Default::default()
                    };
                }
                ActionBarAction::Cancel => {
                    close_dialog = true;
                }
                _ => {}
            }
        },
    );

    if close_dialog || !main_open {
        state.is_open = false;
    }

    action
}
