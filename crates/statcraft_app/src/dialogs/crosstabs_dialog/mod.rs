pub mod state;
pub mod subdialogs_cells;
pub mod subdialogs_exact_format;
pub mod subdialogs_stats;

pub use state::{CrosstabsDialogAction, CrosstabsDialogState, CrosstabsTargetList};
pub use subdialogs_cells::render_cells_subdialog;
pub use subdialogs_exact_format::{render_exact_subdialog, render_format_subdialog};
pub use subdialogs_stats::render_stats_subdialog;

use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use egui::vec2;
use statcraft_core::DataSet;
use statcraft_i18n::t;

pub fn render_crosstabs_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut CrosstabsDialogState,
) -> CrosstabsDialogAction {
    if !state.is_open {
        return CrosstabsDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return CrosstabsDialogAction::None;
        }
    };

    state.ensure_valid_layer();

    // Render active subdialogs
    if state.show_exact {
        render_exact_subdialog(ctx, state);
    }
    if state.show_stats {
        render_stats_subdialog(ctx, state);
    }
    if state.show_cells {
        render_cells_subdialog(ctx, state);
    }
    if state.show_format {
        render_format_subdialog(ctx, state);
    }
    if state.table_style.is_open {
        crate::dialogs::table_style_dialog::render_table_style_dialog(ctx, &mut state.table_style);
    }
    if state.bootstrap.is_open {
        crate::dialogs::bootstrap_dialog::render_bootstrap_dialog(
            ctx,
            Some(ds),
            &mut state.bootstrap,
        );
    }

    let mut action = CrosstabsDialogAction::None;
    let mut close_dialog = false;

    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary.get(col_idx).and_then(|meta| {
                if !state.is_var_selected(&meta.name) {
                    Some((col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-crosstabs-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // 1. Left Panel: Available Variables
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
                vec2(180.0, 275.0),
                "crosstabs_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.row_selected = None;
                    state.col_selected = None;
                    state.layer_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        match state.active_target {
                            CrosstabsTargetList::Row => state.row_vars.push(name.clone()),
                            CrosstabsTargetList::Col => state.col_vars.push(name.clone()),
                            CrosstabsTargetList::Layer => {
                                state.current_layer_vars_mut().push(name.clone());
                            }
                        }
                        state.source_selected = None;
                    }
                }
                ListEvent::None => {}
            }

            // 2. Transfer Buttons Column
            let can_move_source = state
                .source_selected
                .is_some_and(|i| i < available_vars.len());
            let can_move_row = state.row_selected.is_some_and(|i| i < state.row_vars.len());
            let can_move_col = state.col_selected.is_some_and(|i| i < state.col_vars.len());
            let can_move_layer = state
                .layer_selected
                .is_some_and(|i| i < state.current_layer_vars().len());

            let transfer_row_btn = if state.row_selected.is_some() {
                TransferButton::to_source(can_move_row)
            } else {
                TransferButton::to_target(can_move_source)
            };
            let transfer_col_btn = if state.col_selected.is_some() {
                TransferButton::to_source(can_move_col)
            } else {
                TransferButton::to_target(can_move_source)
            };
            let transfer_layer_btn = if state.layer_selected.is_some() {
                TransferButton::to_source(can_move_layer)
            } else {
                TransferButton::to_target(can_move_source)
            };

            ui.allocate_ui_with_layout(
                vec2(34.0, 275.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(28.0);
                    if transfer_row_btn.show(ui).clicked() {
                        state.active_target = CrosstabsTargetList::Row;
                        if let Some(idx) = state.row_selected {
                            if idx < state.row_vars.len() {
                                state.row_vars.remove(idx);
                                state.row_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.row_vars.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    ui.add_space(62.0);
                    if transfer_col_btn.show(ui).clicked() {
                        state.active_target = CrosstabsTargetList::Col;
                        if let Some(idx) = state.col_selected {
                            if idx < state.col_vars.len() {
                                state.col_vars.remove(idx);
                                state.col_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.col_vars.push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }

                    ui.add_space(68.0);
                    if transfer_layer_btn.show(ui).clicked() {
                        state.active_target = CrosstabsTargetList::Layer;
                        if let Some(idx) = state.layer_selected {
                            let layer_vars = state.current_layer_vars_mut();
                            if idx < layer_vars.len() {
                                layer_vars.remove(idx);
                                state.layer_selected = None;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                state.current_layer_vars_mut().push(name.clone());
                                state.source_selected = None;
                            }
                        }
                    }
                },
            );

            // 3. Center Panel: Row(s), Column(s), and Layer 1 of N
            ui.allocate_ui_with_layout(
                vec2(214.0, 275.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let row_items = build_var_items(ds, &state.row_vars, state.row_selected);
                    match render_variable_list_panel(
                        ui,
                        t!("dialog-crosstabs-rows"),
                        vec2(214.0, 78.0),
                        "crosstabs_row_vars",
                        &row_items,
                    ) {
                        ListEvent::Clicked(idx) => {
                            state.row_selected = Some(idx);
                            state.source_selected = None;
                            state.col_selected = None;
                            state.layer_selected = None;
                            state.active_target = CrosstabsTargetList::Row;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.row_vars.len() {
                                state.row_vars.remove(idx);
                                state.row_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }

                    ui.add_space(4.0);

                    let col_items = build_var_items(ds, &state.col_vars, state.col_selected);
                    match render_variable_list_panel(
                        ui,
                        t!("dialog-crosstabs-cols"),
                        vec2(214.0, 78.0),
                        "crosstabs_col_vars",
                        &col_items,
                    ) {
                        ListEvent::Clicked(idx) => {
                            state.col_selected = Some(idx);
                            state.source_selected = None;
                            state.row_selected = None;
                            state.layer_selected = None;
                            state.active_target = CrosstabsTargetList::Col;
                        }
                        ListEvent::DoubleClicked(idx) => {
                            if idx < state.col_vars.len() {
                                state.col_vars.remove(idx);
                                state.col_selected = None;
                            }
                        }
                        ListEvent::None => {}
                    }

                    ui.add_space(4.0);

                    render_layer_group_box(ui, ds, state);
                },
            );

            // 4. Right Panel: SubdialogSidebar (6 buttons)
            ui.allocate_ui_with_layout(
                vec2(96.0, 275.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(16.0);
                    if SubdialogSidebar::button(ui, t!("dialog-crosstabs-btn-exact"), true)
                        .clicked()
                    {
                        state.exact_backup = Some((
                            state.options.exact_mode.clone(),
                            state.mc_confidence_input.clone(),
                            state.mc_samples_input.clone(),
                            state.exact_time_enabled,
                            state.exact_time_input.clone(),
                        ));
                        state.show_exact = true;
                    }
                    ui.add_space(4.0);
                    if SubdialogSidebar::button(ui, t!("dialog-crosstabs-btn-statistics"), true)
                        .clicked()
                    {
                        state.stats_backup =
                            Some((state.options.clone(), state.cmh_or_input.clone()));
                        state.show_stats = true;
                    }
                    ui.add_space(4.0);
                    if SubdialogSidebar::button(ui, t!("dialog-crosstabs-btn-cells"), true)
                        .clicked()
                    {
                        state.cells_backup =
                            Some((state.options.clone(), state.hide_small_input.clone()));
                        state.show_cells = true;
                    }
                    ui.add_space(4.0);
                    if SubdialogSidebar::button(ui, t!("dialog-btn-format"), true).clicked() {
                        state.format_backup = Some(state.options.clone());
                        state.show_format = true;
                    }
                    ui.add_space(4.0);
                    if SubdialogSidebar::button(ui, t!("dialog-btn-style"), true).clicked() {
                        state.table_style.backup = Some(state.table_style.rules.clone());
                        state.table_style.backup_report = state.table_style.create_style_report;
                        state.table_style.is_open = true;
                    }
                    ui.add_space(4.0);
                    if SubdialogSidebar::button(ui, t!("dialog-btn-bootstrap"), true).clicked() {
                        state.bootstrap.open();
                    }
                },
            );
        });

        ui.add_space(4.0);
        ui.checkbox(
            &mut state.options.display_clustered_bar_charts,
            t!("dialog-crosstabs-clustered-bar"),
        );
        ui.checkbox(
            &mut state.options.suppress_tables,
            t!("dialog-crosstabs-suppress-tables"),
        );

        ui.add_space(6.0);

        let is_ready = !state.row_vars.is_empty() && !state.col_vars.is_empty();
        let bar_action = DialogActionBar::new(is_ready).show(ui);

        let table_styles = state.table_style.rules_to_json();
        let create_style_report = state.table_style.create_style_report;
        let bootstrap = state.bootstrap.to_json();
        let layers = state.non_empty_layers();

        match bar_action {
            ActionBarAction::Ok => {
                action = CrosstabsDialogAction::Run {
                    rows: state.row_vars.clone(),
                    cols: state.col_vars.clone(),
                    layers,
                    options: state.options.clone(),
                    table_styles,
                    create_style_report,
                    bootstrap,
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = CrosstabsDialogAction::Paste {
                    rows: state.row_vars.clone(),
                    cols: state.col_vars.clone(),
                    layers,
                    options: state.options.clone(),
                    table_styles,
                    create_style_report,
                    bootstrap,
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = CrosstabsDialogState {
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

fn build_var_items<'a>(
    ds: &'a DataSet,
    vars: &'a [String],
    selected: Option<usize>,
) -> Vec<VariableListItem<'a>> {
    vars.iter()
        .enumerate()
        .map(|(idx, name)| {
            let meta = ds.dictionary.find_by_name(name);
            let measure = meta
                .map(|m| m.measure)
                .unwrap_or(statcraft_core::Measure::Nominal);
            let label = meta.map(|m| m.label.as_str()).unwrap_or("");
            VariableListItem {
                name,
                display_text: format_var_display(name, label),
                measure,
                is_selected: selected == Some(idx),
            }
        })
        .collect()
}

fn render_layer_group_box(ui: &mut egui::Ui, ds: &DataSet, state: &mut CrosstabsDialogState) {
    let total_layers = state.layers.len();
    let current_num = state.current_layer_idx + 1;
    let header = t!(
        "dialog-crosstabs-layer-title",
        current = current_num,
        total = total_layers
    );

    GroupBox::new(header).show(ui, |ui| {
        ui.horizontal(|ui| {
            let can_prev = state.current_layer_idx > 0;
            let can_next = state.current_layer_idx + 1 < state.layers.len()
                || !state.current_layer_vars().is_empty();

            if ui
                .add_enabled(
                    can_prev,
                    egui::Button::new(t!("dialog-crosstabs-layer-prev")).min_size(vec2(88.0, 22.0)),
                )
                .clicked()
            {
                state.prev_layer();
            }
            if ui
                .add_enabled(
                    can_next,
                    egui::Button::new(t!("dialog-crosstabs-layer-next")).min_size(vec2(88.0, 22.0)),
                )
                .clicked()
            {
                state.next_layer();
            }
        });

        let layer_items = build_var_items(ds, state.current_layer_vars(), state.layer_selected);
        match render_variable_list_panel(
            ui,
            "",
            vec2(198.0, 46.0),
            "crosstabs_layer_vars",
            &layer_items,
        ) {
            ListEvent::Clicked(idx) => {
                state.layer_selected = Some(idx);
                state.source_selected = None;
                state.row_selected = None;
                state.col_selected = None;
                state.active_target = CrosstabsTargetList::Layer;
            }
            ListEvent::DoubleClicked(idx) => {
                let layer_vars = state.current_layer_vars_mut();
                if idx < layer_vars.len() {
                    layer_vars.remove(idx);
                    state.layer_selected = None;
                }
            }
            ListEvent::None => {}
        }

        let has_layers = state.has_any_layer_vars();
        ui.add_enabled(
            has_layers,
            egui::Checkbox::new(
                &mut state.options.show_layers_in_layers,
                t!("dialog-crosstabs-display-layers"),
            ),
        );
    });
}
