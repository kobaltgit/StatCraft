use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use egui::vec2;
use statcraft_algo::{BivariateCorrelationsOptions, MissingHandling, SignificanceTest};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone, Default)]
pub struct CorrelationsDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub options: BivariateCorrelationsOptions,
    pub show_options: bool,
}

pub enum CorrelationsDialogAction {
    Run {
        vars: Vec<String>,
        options: BivariateCorrelationsOptions,
    },
    Paste {
        vars: Vec<String>,
        options: BivariateCorrelationsOptions,
    },
    None,
}

pub fn render_correlations_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut CorrelationsDialogState,
) -> CorrelationsDialogAction {
    if !state.is_open {
        return CorrelationsDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return CorrelationsDialogAction::None;
        }
    };

    let mut action = CorrelationsDialogAction::None;
    let mut close_dialog = false;

    // Filter available variables
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

    // 1. Subdialog: Options
    if state.show_options {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-correlations-options-title")).show(
            ctx,
            &mut sub_open,
            |ui| {
                GroupBox::new(t!("dialog-correlations-stats-group")).show(ui, |ui| {
                    ui.checkbox(
                        &mut state.options.descriptives,
                        t!("dialog-correlations-stat-means"),
                    );
                    ui.checkbox(
                        &mut state.options.cross_prod_cov,
                        t!("dialog-correlations-stat-cov"),
                    );
                });

                ui.add_space(8.0);

                GroupBox::new(t!("dialog-correlations-missing-group")).show(ui, |ui| {
                    ui.radio_value(
                        &mut state.options.missing,
                        MissingHandling::Pairwise,
                        t!("dialog-correlations-missing-pairwise"),
                    );
                    ui.radio_value(
                        &mut state.options.missing,
                        MissingHandling::Listwise,
                        t!("dialog-correlations-missing-listwise"),
                    );
                });

                ui.add_space(10.0);

                let sub_action = SubdialogActionBar::new(true).show(ui);
                match sub_action {
                    SubdialogAction::Continue | SubdialogAction::Cancel => {
                        state.show_options = false;
                    }
                    SubdialogAction::Help | SubdialogAction::None => {}
                }
            },
        );

        if !sub_open {
            state.show_options = false;
        }
    }

    // 2. Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-correlations-title")).show(ctx, &mut main_open, |ui| {
        // Top section: Source -> Transfer -> Target -> Sidebar
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
                "corr_source_vars",
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
                && state.source_selected.unwrap() < available_vars.len();
            let can_move_left = state.target_selected.is_some()
                && state.target_selected.unwrap() < state.selected_vars.len();

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

            // Target variables panel
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
                t!("dialog-correlations-vars"),
                vec2(190.0, 190.0),
                "corr_target_vars",
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

            // Right sidebar: Options button
            ui.allocate_ui_with_layout(
                vec2(90.0, 190.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(18.0);
                    if SubdialogSidebar::button(ui, t!("dialog-correlations-btn-options"), true)
                        .clicked()
                    {
                        state.show_options = true;
                    }
                },
            );
        });

        ui.add_space(8.0);

        // Coefficients GroupBox & Test of Significance GroupBox
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                vec2(245.0, 95.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    GroupBox::new(t!("dialog-correlations-coeff-group")).show(ui, |ui| {
                        ui.checkbox(
                            &mut state.options.pearson,
                            t!("dialog-correlations-pearson"),
                        );
                        ui.checkbox(
                            &mut state.options.kendall,
                            t!("dialog-correlations-kendall"),
                        );
                        ui.checkbox(
                            &mut state.options.spearman,
                            t!("dialog-correlations-spearman"),
                        );
                    });
                },
            );

            ui.add_space(8.0);

            ui.allocate_ui_with_layout(
                vec2(265.0, 95.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    GroupBox::new(t!("dialog-correlations-sig-test-group")).show(ui, |ui| {
                        ui.radio_value(
                            &mut state.options.significance,
                            SignificanceTest::TwoTailed,
                            t!("dialog-correlations-two-tailed"),
                        );
                        ui.radio_value(
                            &mut state.options.significance,
                            SignificanceTest::OneTailed,
                            t!("dialog-correlations-one-tailed"),
                        );
                        ui.add_space(4.0);
                        ui.checkbox(
                            &mut state.options.flag_significant,
                            t!("dialog-correlations-flag-sig"),
                        );
                    });
                },
            );
        });

        ui.add_space(6.0);

        // Bottom Action Bar
        let at_least_one_coeff =
            state.options.pearson || state.options.kendall || state.options.spearman;
        let can_run = state.selected_vars.len() >= 2 && at_least_one_coeff;
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = CorrelationsDialogAction::Run {
                    vars: state.selected_vars.clone(),
                    options: state.options.clone(),
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = CorrelationsDialogAction::Paste {
                    vars: state.selected_vars.clone(),
                    options: state.options.clone(),
                };
            }
            ActionBarAction::Reset => {
                state.selected_vars.clear();
                state.source_selected = None;
                state.target_selected = None;
                state.options = BivariateCorrelationsOptions::default();
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
