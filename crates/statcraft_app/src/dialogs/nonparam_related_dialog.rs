use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use crate::theme::Tokens;
use egui::{vec2, Stroke};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct NonparamRelatedDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub active_slot: usize, // 0 for var1, 1 for var2
    pub current_var1: Option<String>,
    pub current_var2: Option<String>,
    pub paired_vars: Vec<(String, String)>,
    pub pair_selected: Option<usize>,
    pub test_wilcoxon: bool,
    pub test_sign: bool,
}

impl Default for NonparamRelatedDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            active_slot: 0,
            current_var1: None,
            current_var2: None,
            paired_vars: Vec::new(),
            pair_selected: None,
            test_wilcoxon: true,
            test_sign: false,
        }
    }
}

pub enum NonparamRelatedDialogAction {
    Run {
        pairs: Vec<(String, String)>,
        wilcoxon: bool,
        sign: bool,
    },
    Paste {
        pairs: Vec<(String, String)>,
        wilcoxon: bool,
        sign: bool,
    },
    None,
}

pub fn render_nonparam_related_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut NonparamRelatedDialogState,
) -> NonparamRelatedDialogAction {
    if !state.is_open {
        return NonparamRelatedDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return NonparamRelatedDialogAction::None;
        }
    };

    let mut action = NonparamRelatedDialogAction::None;
    let mut close_dialog = false;

    // Available variables list
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary
                .get(col_idx)
                .map(|meta| (col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    let mut main_open = true;
    DialogShell::standard(t!("dialog-npar-two-related-title")).show(ctx, &mut main_open, |ui| {
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
                "npar_related_source_vars",
                &source_items,
            );

            match source_ev {
                ListEvent::Clicked(idx) => {
                    state.source_selected = Some(idx);
                    state.pair_selected = None;
                }
                ListEvent::DoubleClicked(idx) => {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        if state.active_slot == 0 {
                            state.current_var1 = Some(name.clone());
                            state.active_slot = 1;
                        } else {
                            state.current_var2 = Some(name.clone());
                            if let (Some(v1), Some(v2)) = (&state.current_var1, &state.current_var2)
                            {
                                state.paired_vars.push((v1.clone(), v2.clone()));
                                state.current_var1 = None;
                                state.current_var2 = None;
                                state.active_slot = 0;
                            }
                        }
                    }
                }
                ListEvent::None => {}
            }

            // Transfer Buttons Column
            let can_move_right = match state.source_selected {
                Some(idx) => idx < available_vars.len(),
                None => false,
            };
            let can_move_left = match state.pair_selected {
                Some(idx) => idx < state.paired_vars.len(),
                None => false,
            };

            let transfer_btn = if state.pair_selected.is_some() {
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
                        if let Some(idx) = state.pair_selected {
                            if idx < state.paired_vars.len() {
                                let removed = state.paired_vars.remove(idx);
                                state.current_var1 = Some(removed.0);
                                state.current_var2 = Some(removed.1);
                                state.pair_selected = None;
                                state.active_slot = 0;
                            }
                        } else if let Some(idx) = state.source_selected {
                            if let Some((_, name, _, _)) = available_vars.get(idx) {
                                if state.active_slot == 0 {
                                    state.current_var1 = Some(name.clone());
                                    state.active_slot = 1;
                                } else {
                                    state.current_var2 = Some(name.clone());
                                    if let (Some(v1), Some(v2)) =
                                        (&state.current_var1, &state.current_var2)
                                    {
                                        state.paired_vars.push((v1.clone(), v2.clone()));
                                        state.current_var1 = None;
                                        state.current_var2 = None;
                                        state.active_slot = 0;
                                    }
                                }
                            }
                        }
                    }
                },
            );

            // Center Panel: Paired Variables List + Current Selections Box
            ui.allocate_ui_with_layout(
                vec2(280.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.label(t!("dialog-npar-related-test-pairs"));
                    egui::Frame::canvas(ui.style())
                        .fill(Tokens::BG_PANEL)
                        .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
                        .show(ui, |ui| {
                            ui.set_min_size(vec2(276.0, 150.0));
                            ui.set_max_size(vec2(276.0, 150.0));

                            egui::ScrollArea::vertical()
                                .id_salt("npar_related_pairs_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for (i, (v1, v2)) in state.paired_vars.iter().enumerate() {
                                        let is_sel = state.pair_selected == Some(i);
                                        let label_text = format!("{}. {} - {}", i + 1, v1, v2);
                                        let resp = ui.selectable_label(is_sel, label_text);
                                        if resp.clicked() {
                                            state.pair_selected = Some(i);
                                            state.source_selected = None;
                                        }
                                    }
                                });
                        });

                    ui.add_space(6.0);

                    // Current Selections Box
                    GroupBox::new(t!("dialog-npar-related-current-selections")).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let slot1_active = state.active_slot == 0;
                            let slot1_text = format!(
                                "{}: {}",
                                t!("dialog-npar-related-var1"),
                                state.current_var1.as_deref().unwrap_or("-")
                            );
                            if ui.selectable_label(slot1_active, slot1_text).clicked() {
                                state.active_slot = 0;
                            }

                            let slot2_active = state.active_slot == 1;
                            let slot2_text = format!(
                                "{}: {}",
                                t!("dialog-npar-related-var2"),
                                state.current_var2.as_deref().unwrap_or("-")
                            );
                            if ui.selectable_label(slot2_active, slot2_text).clicked() {
                                state.active_slot = 1;
                            }
                        });
                    });
                },
            );
        });

        ui.add_space(8.0);

        // Test Type GroupBox
        GroupBox::new(t!("dialog-npar-test-type")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.test_wilcoxon, t!("dialog-npar-related-wilcoxon"));
                ui.add_space(20.0);
                ui.checkbox(&mut state.test_sign, t!("dialog-npar-related-sign"));
            });
        });

        ui.add_space(8.0);

        // Action Bar
        let can_run = !state.paired_vars.is_empty() && (state.test_wilcoxon || state.test_sign);
        let action_bar = DialogActionBar::new(can_run).show(ui);

        match action_bar {
            ActionBarAction::Ok => {
                action = NonparamRelatedDialogAction::Run {
                    pairs: state.paired_vars.clone(),
                    wilcoxon: state.test_wilcoxon,
                    sign: state.test_sign,
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = NonparamRelatedDialogAction::Paste {
                    pairs: state.paired_vars.clone(),
                    wilcoxon: state.test_wilcoxon,
                    sign: state.test_sign,
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                *state = NonparamRelatedDialogState {
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
