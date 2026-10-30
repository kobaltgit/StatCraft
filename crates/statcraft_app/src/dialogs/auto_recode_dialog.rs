use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use crate::theme::Tokens;
use egui::{vec2, Align, Button, Layout, ScrollArea, Stroke, TextEdit, Ui};
use statcraft_algo::AutoRecodeOrder;
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoRecodePair {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Default)]
pub struct AutoRecodeDialogState {
    pub is_open: bool,
    pub selected_available_idx: Option<usize>,
    pub pairs: Vec<AutoRecodePair>,
    pub selected_pair_idx: Option<usize>,
    pub new_name: String,
    pub order: AutoRecodeOrder,
}

impl AutoRecodeDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_available_idx = None;
        self.pairs.clear();
        self.selected_pair_idx = None;
        self.new_name.clear();
        self.order = AutoRecodeOrder::LowestValue;
    }

    pub fn reset(&mut self) {
        self.selected_available_idx = None;
        self.pairs.clear();
        self.selected_pair_idx = None;
        self.new_name.clear();
        self.order = AutoRecodeOrder::LowestValue;
    }
}

pub enum AutoRecodeDialogAction {
    Run {
        pairs: Vec<AutoRecodePair>,
        order: AutoRecodeOrder,
    },
    Paste {
        pairs: Vec<AutoRecodePair>,
        order: AutoRecodeOrder,
    },
    None,
}

pub fn render_auto_recode_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut AutoRecodeDialogState,
) -> AutoRecodeDialogAction {
    if !state.is_open {
        return AutoRecodeDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return AutoRecodeDialogAction::None;
        }
    };

    let mut action = AutoRecodeDialogAction::None;
    let mut is_open = state.is_open;
    let mut close = false;

    let available_vars: Vec<(usize, String, String, Measure)> = (0..ds.col_count())
        .filter_map(|idx| {
            ds.dictionary
                .get(idx)
                .map(|meta| (idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    DialogShell::standard(t!("dialog-autorecode-title")).show(ctx, &mut is_open, |ui| {
        render_content(ui, state, &available_vars);

        ui.add_space(8.0);

        let can_run =
            !state.pairs.is_empty() && state.pairs.iter().all(|p| !p.target.trim().is_empty());

        let bar_action = DialogActionBar::new(can_run).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                action = AutoRecodeDialogAction::Run {
                    pairs: state.pairs.clone(),
                    order: state.order,
                };
                close = true;
            }
            ActionBarAction::Paste => {
                action = AutoRecodeDialogAction::Paste {
                    pairs: state.pairs.clone(),
                    order: state.order,
                };
                close = true;
            }
            ActionBarAction::Reset => {
                state.reset();
            }
            ActionBarAction::Cancel => {
                close = true;
            }
            ActionBarAction::None | ActionBarAction::Help => {}
        }
    });

    state.is_open = is_open && !close;
    action
}

fn render_content(
    ui: &mut Ui,
    state: &mut AutoRecodeDialogState,
    available_vars: &[(usize, String, String, Measure)],
) {
    ui.horizontal(|ui| {
        // Col 1: Available variables list (180 px)
        let source_items: Vec<VariableListItem> = available_vars
            .iter()
            .enumerate()
            .map(|(idx, (_col, name, label, measure))| VariableListItem {
                name,
                display_text: format_var_display(name, label),
                measure: *measure,
                is_selected: state.selected_available_idx == Some(idx),
            })
            .collect();

        let source_ev = render_variable_list_panel(
            ui,
            t!("dialog-variables-source"),
            vec2(180.0, 280.0),
            "autorecode_source_vars",
            &source_items,
        );

        match source_ev {
            ListEvent::Clicked(idx) => {
                state.selected_available_idx = Some(idx);
                state.selected_pair_idx = None;
            }
            ListEvent::DoubleClicked(idx) => {
                if let Some((_, name, _, _)) = available_vars.get(idx) {
                    let default_tgt = format!("{}_code", name);
                    state.pairs.push(AutoRecodePair {
                        source: name.clone(),
                        target: default_tgt.clone(),
                    });
                    state.selected_pair_idx = Some(state.pairs.len() - 1);
                    state.new_name = default_tgt;
                    state.selected_available_idx = None;
                }
            }
            ListEvent::None => {}
        }

        // Col 2: Transfer button (36 px)
        let can_move_right = state.selected_available_idx.is_some();
        let can_move_left = state.selected_pair_idx.is_some();

        let transfer_btn = if state.selected_pair_idx.is_some() {
            TransferButton::to_source(can_move_left)
        } else {
            TransferButton::to_target(can_move_right)
        };

        ui.allocate_ui_with_layout(vec2(36.0, 280.0), Layout::top_down(Align::Center), |ui| {
            ui.add_space(60.0);
            if transfer_btn.show(ui).clicked() {
                if state.selected_pair_idx.is_some() {
                    if let Some(idx) = state.selected_pair_idx {
                        if idx < state.pairs.len() {
                            state.pairs.remove(idx);
                            state.selected_pair_idx = None;
                            state.new_name.clear();
                        }
                    }
                } else if let Some(idx) = state.selected_available_idx {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        let default_tgt = format!("{}_code", name);
                        state.pairs.push(AutoRecodePair {
                            source: name.clone(),
                            target: default_tgt.clone(),
                        });
                        state.selected_pair_idx = Some(state.pairs.len() - 1);
                        state.new_name = default_tgt;
                    }
                }
            }
        });

        // Col 3: Pairs list + New Name + Recode Starting from (280 px)
        ui.allocate_ui_with_layout(vec2(280.0, 280.0), Layout::top_down(Align::Min), |ui| {
            ui.label(t!("dialog-autorecode-variables"));
            egui::Frame::canvas(ui.style())
                .fill(Tokens::BG_PANEL)
                .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
                .show(ui, |ui| {
                    let list_sz = vec2(276.0, 100.0);
                    ui.set_min_size(list_sz);
                    ui.set_max_size(list_sz);
                    ScrollArea::vertical().show(ui, |ui| {
                        let mut clicked_idx = None;
                        for (idx, pair) in state.pairs.iter().enumerate() {
                            let is_sel = state.selected_pair_idx == Some(idx);
                            let text = format!("{} -> {}", pair.source, pair.target);
                            if ui.selectable_label(is_sel, text).clicked() {
                                clicked_idx = Some(idx);
                            }
                        }
                        if let Some(idx) = clicked_idx {
                            state.selected_pair_idx = Some(idx);
                            state.selected_available_idx = None;
                            if let Some(p) = state.pairs.get(idx) {
                                state.new_name = p.target.clone();
                            }
                        }
                    });
                });

            ui.add_space(6.0);

            // New name input block
            GroupBox::new(t!("dialog-autorecode-new-name")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(TextEdit::singleline(&mut state.new_name).desired_width(140.0));
                    let can_change =
                        state.selected_pair_idx.is_some() && !state.new_name.trim().is_empty();
                    if ui
                        .add_enabled(can_change, Button::new(t!("dialog-autorecode-change")))
                        .clicked()
                    {
                        if let Some(idx) = state.selected_pair_idx {
                            if let Some(p) = state.pairs.get_mut(idx) {
                                p.target = state.new_name.trim().to_string();
                            }
                        }
                    }
                });
            });

            ui.add_space(6.0);

            // Order group block
            GroupBox::new(t!("dialog-autorecode-order-group")).show(ui, |ui| {
                ui.radio_value(
                    &mut state.order,
                    AutoRecodeOrder::LowestValue,
                    t!("dialog-autorecode-order-lowest"),
                );
                ui.radio_value(
                    &mut state.order,
                    AutoRecodeOrder::HighestValue,
                    t!("dialog-autorecode-order-highest"),
                );
            });
        });
    });
}
