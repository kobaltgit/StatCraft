use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use egui::vec2;
use statcraft_algo::{RankDirection, RankType, TiesHandling};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct RankCasesDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub direction: RankDirection,
    pub ties: TiesHandling,
    pub rank_type: RankType,
    pub ntiles_groups: usize,
    pub prefix: String,
    pub show_types: bool,
    pub show_ties: bool,
}

impl Default for RankCasesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            direction: RankDirection::Ascending,
            ties: TiesHandling::Mean,
            rank_type: RankType::SimpleRank,
            ntiles_groups: 4,
            prefix: "R_".to_string(),
            show_types: false,
            show_ties: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RankCasesDialogAction {
    Run {
        variables: Vec<String>,
        direction: RankDirection,
        ties: TiesHandling,
        rank_type: RankType,
        ntiles_groups: usize,
        prefix: String,
    },
    Paste {
        variables: Vec<String>,
        direction: RankDirection,
        ties: TiesHandling,
        rank_type: RankType,
        ntiles_groups: usize,
        prefix: String,
    },
    None,
}

pub fn render_rank_cases_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut RankCasesDialogState,
) -> RankCasesDialogAction {
    if !state.is_open {
        return RankCasesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return RankCasesDialogAction::None;
        }
    };

    let mut action = RankCasesDialogAction::None;
    let mut close_dialog = false;

    // Available variables: columns not currently in selected_vars
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

    // 1. Render Subdialog (Rank Types) if open
    if state.show_types {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-rank-types-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-rank-types-title")).show(ui, |ui| {
                let is_simple = matches!(state.rank_type, RankType::SimpleRank);
                if ui.radio(is_simple, t!("dialog-rank-type-simple")).clicked() {
                    state.rank_type = RankType::SimpleRank;
                }

                let is_frac = matches!(state.rank_type, RankType::FractionalRank);
                if ui
                    .radio(is_frac, t!("dialog-rank-type-fractional"))
                    .clicked()
                {
                    state.rank_type = RankType::FractionalRank;
                }

                let is_pct = matches!(state.rank_type, RankType::FractionalPercent);
                if ui.radio(is_pct, t!("dialog-rank-type-percent")).clicked() {
                    state.rank_type = RankType::FractionalPercent;
                }

                let is_ntiles = matches!(state.rank_type, RankType::Ntiles(_));
                ui.horizontal(|ui| {
                    if ui.radio(is_ntiles, t!("dialog-rank-type-ntiles")).clicked() {
                        state.rank_type = RankType::Ntiles(state.ntiles_groups.max(2));
                    }
                    if is_ntiles {
                        ui.label(t!("dialog-rank-ntiles-groups"));
                        let mut groups = state.ntiles_groups as u32;
                        if ui
                            .add(egui::DragValue::new(&mut groups).range(2..=100))
                            .changed()
                        {
                            state.ntiles_groups = groups as usize;
                            state.rank_type = RankType::Ntiles(state.ntiles_groups);
                        }
                    }
                });

                ui.separator();

                let is_blom = matches!(state.rank_type, RankType::NormalBlom);
                if ui.radio(is_blom, t!("dialog-rank-type-blom")).clicked() {
                    state.rank_type = RankType::NormalBlom;
                }

                let is_tukey = matches!(state.rank_type, RankType::NormalTukey);
                if ui.radio(is_tukey, t!("dialog-rank-type-tukey")).clicked() {
                    state.rank_type = RankType::NormalTukey;
                }

                let is_rankit = matches!(state.rank_type, RankType::NormalRankit);
                if ui.radio(is_rankit, t!("dialog-rank-type-rankit")).clicked() {
                    state.rank_type = RankType::NormalRankit;
                }

                let is_vw = matches!(state.rank_type, RankType::NormalVanderWaerden);
                if ui.radio(is_vw, t!("dialog-rank-type-vw")).clicked() {
                    state.rank_type = RankType::NormalVanderWaerden;
                }
            });

            let sub_action = SubdialogActionBar::new(true).show(ui);
            match sub_action {
                SubdialogAction::Continue | SubdialogAction::Cancel => {
                    state.show_types = false;
                }
                SubdialogAction::Help | SubdialogAction::None => {}
            }
        });

        if !sub_open {
            state.show_types = false;
        }
    }

    // 1b. Render Subdialog (Ties) if open
    if state.show_ties {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-rank-ties-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-rank-ties-group")).show(ui, |ui| {
                ui.radio_value(
                    &mut state.ties,
                    TiesHandling::Mean,
                    t!("dialog-rank-ties-mean"),
                );
                ui.radio_value(
                    &mut state.ties,
                    TiesHandling::Low,
                    t!("dialog-rank-ties-low"),
                );
                ui.radio_value(
                    &mut state.ties,
                    TiesHandling::High,
                    t!("dialog-rank-ties-high"),
                );
            });

            let sub_action = SubdialogActionBar::new(true).show(ui);
            match sub_action {
                SubdialogAction::Continue | SubdialogAction::Cancel => {
                    state.show_ties = false;
                }
                SubdialogAction::Help | SubdialogAction::None => {}
            }
        });

        if !sub_open {
            state.show_ties = false;
        }
    }

    // 2. Render Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-rank-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left Panel: Available Variables (200x260)
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
                vec2(200.0, 260.0),
                "rank_source_vars",
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

            // Transfer Button Column
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
                vec2(36.0, 260.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(80.0);
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

            // Center Panel: Target Variables (200x260)
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
                t!("dialog-variables-target"),
                vec2(200.0, 260.0),
                "rank_target_vars",
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

            // Right Panel: SubdialogSidebar (94x260) at the right edge
            ui.allocate_ui_with_layout(
                vec2(94.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(18.0);
                    if SubdialogSidebar::button(ui, t!("dialog-rank-types-button"), true).clicked()
                    {
                        state.show_types = true;
                    }
                    if SubdialogSidebar::button(ui, t!("dialog-rank-ties-button"), true).clicked() {
                        state.show_ties = true;
                    }
                },
            );
        });

        ui.add_space(8.0);

        // GroupBox: Direction (Assign Rank 1 to Smallest / Largest)
        GroupBox::new(t!("dialog-rank-direction-group")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut state.direction,
                    RankDirection::Ascending,
                    t!("dialog-rank-direction-smallest"),
                );
                ui.add_space(16.0);
                ui.radio_value(
                    &mut state.direction,
                    RankDirection::Descending,
                    t!("dialog-rank-direction-largest"),
                );
            });
        });

        ui.add_space(8.0);

        // Bottom Action Bar: [ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]
        let has_selection = !state.selected_vars.is_empty();
        let bar_action = DialogActionBar::new(has_selection).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                if has_selection {
                    action = RankCasesDialogAction::Run {
                        variables: state.selected_vars.clone(),
                        direction: state.direction,
                        ties: state.ties,
                        rank_type: state.rank_type,
                        ntiles_groups: state.ntiles_groups,
                        prefix: state.prefix.clone(),
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Paste => {
                if has_selection {
                    action = RankCasesDialogAction::Paste {
                        variables: state.selected_vars.clone(),
                        direction: state.direction,
                        ties: state.ties,
                        rank_type: state.rank_type,
                        ntiles_groups: state.ntiles_groups,
                        prefix: state.prefix.clone(),
                    };
                    close_dialog = true;
                }
            }
            ActionBarAction::Reset => {
                *state = RankCasesDialogState {
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
