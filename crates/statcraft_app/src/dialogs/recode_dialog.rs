use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogSidebar, TransferButton, VariableListItem,
};
use super::recode_old_new_dialog::{
    render_recode_old_new_dialog, RecodeOldNewAction, RecodeOldNewState,
};
use crate::theme::Tokens;
use egui::{vec2, Align, Button, Layout, ScrollArea, Stroke, TextEdit, Ui};
use statcraft_algo::RecodeMapping;
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecodeMode {
    #[default]
    Different,
    Same,
}

#[derive(Debug, Clone, Default)]
pub struct RecodePair {
    pub source: String,
    pub target: String,
    pub label: String,
}

#[derive(Debug, Clone, Default)]
pub struct RecodeDialogState {
    pub is_open: bool,
    pub mode: RecodeMode,
    pub selected_available_idx: Option<usize>,
    pub pairs: Vec<RecodePair>,
    pub selected_pair_idx: Option<usize>,
    pub output_name: String,
    pub output_label: String,
    pub selected_vars: Vec<String>,
    pub selected_var_idx: Option<usize>,
    pub mappings: Vec<RecodeMapping>,
    pub old_new_state: RecodeOldNewState,
}

impl RecodeDialogState {
    pub fn open_diff(&mut self) {
        self.is_open = true;
        self.mode = RecodeMode::Different;
        self.selected_available_idx = None;
        self.pairs.clear();
        self.selected_pair_idx = None;
        self.output_name.clear();
        self.output_label.clear();
        self.mappings.clear();
        self.old_new_state.is_open = false;
    }

    pub fn open_same(&mut self) {
        self.is_open = true;
        self.mode = RecodeMode::Same;
        self.selected_available_idx = None;
        self.selected_vars.clear();
        self.selected_var_idx = None;
        self.mappings.clear();
        self.old_new_state.is_open = false;
    }

    pub fn reset(&mut self) {
        self.selected_available_idx = None;
        self.pairs.clear();
        self.selected_pair_idx = None;
        self.output_name.clear();
        self.output_label.clear();
        self.selected_vars.clear();
        self.selected_var_idx = None;
        self.mappings.clear();
    }
}

pub enum RecodeDialogAction {
    RunDiff {
        pairs: Vec<RecodePair>,
        mappings: Vec<RecodeMapping>,
    },
    RunSame {
        vars: Vec<String>,
        mappings: Vec<RecodeMapping>,
    },
    PasteDiff {
        pairs: Vec<RecodePair>,
        mappings: Vec<RecodeMapping>,
    },
    PasteSame {
        vars: Vec<String>,
        mappings: Vec<RecodeMapping>,
    },
    None,
}

pub fn render_recode_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut RecodeDialogState,
) -> RecodeDialogAction {
    // 1. Render subdialog if active
    let old_new_act = render_recode_old_new_dialog(ctx, &mut state.old_new_state);
    if let RecodeOldNewAction::Apply(new_rules) = old_new_act {
        state.mappings = new_rules;
    }

    if !state.is_open {
        return RecodeDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return RecodeDialogAction::None;
        }
    };

    let title = match state.mode {
        RecodeMode::Different => t!("dialog-recode-diff-title"),
        RecodeMode::Same => t!("dialog-recode-same-title"),
    };

    let mut action = RecodeDialogAction::None;
    let mut is_open = state.is_open;
    let mut close = false;

    // Available variables: (index, name, label, measure)
    let available_vars: Vec<(usize, String, String, Measure)> = (0..ds.col_count())
        .filter_map(|idx| {
            ds.dictionary
                .get(idx)
                .map(|meta| (idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    DialogShell::standard(title).show(ctx, &mut is_open, |ui| {
        match state.mode {
            RecodeMode::Different => {
                render_diff_content(ui, state, &available_vars);
            }
            RecodeMode::Same => {
                render_same_content(ui, state, &available_vars);
            }
        }

        ui.add_space(8.0);

        // Action Bar
        let can_run = match state.mode {
            RecodeMode::Different => {
                !state.pairs.is_empty()
                    && state.pairs.iter().all(|p| !p.target.trim().is_empty())
                    && !state.mappings.is_empty()
            }
            RecodeMode::Same => !state.selected_vars.is_empty() && !state.mappings.is_empty(),
        };

        let bar_action = DialogActionBar::new(can_run).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                action = match state.mode {
                    RecodeMode::Different => RecodeDialogAction::RunDiff {
                        pairs: state.pairs.clone(),
                        mappings: state.mappings.clone(),
                    },
                    RecodeMode::Same => RecodeDialogAction::RunSame {
                        vars: state.selected_vars.clone(),
                        mappings: state.mappings.clone(),
                    },
                };
                close = true;
            }
            ActionBarAction::Paste => {
                action = match state.mode {
                    RecodeMode::Different => RecodeDialogAction::PasteDiff {
                        pairs: state.pairs.clone(),
                        mappings: state.mappings.clone(),
                    },
                    RecodeMode::Same => RecodeDialogAction::PasteSame {
                        vars: state.selected_vars.clone(),
                        mappings: state.mappings.clone(),
                    },
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

fn render_diff_content(
    ui: &mut Ui,
    state: &mut RecodeDialogState,
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
            "recode_diff_source_vars",
            &source_items,
        );

        match source_ev {
            ListEvent::Clicked(idx) => {
                state.selected_available_idx = Some(idx);
                state.selected_pair_idx = None;
            }
            ListEvent::DoubleClicked(idx) => {
                if let Some((_, name, _, _)) = available_vars.get(idx) {
                    let default_tgt = format!("{}_cat", name);
                    state.pairs.push(RecodePair {
                        source: name.clone(),
                        target: default_tgt.clone(),
                        label: String::new(),
                    });
                    state.selected_pair_idx = Some(state.pairs.len() - 1);
                    state.output_name = default_tgt;
                    state.output_label.clear();
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
                            state.output_name.clear();
                            state.output_label.clear();
                        }
                    }
                } else if let Some(idx) = state.selected_available_idx {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        let default_tgt = format!("{}_cat", name);
                        state.pairs.push(RecodePair {
                            source: name.clone(),
                            target: default_tgt.clone(),
                            label: String::new(),
                        });
                        state.selected_pair_idx = Some(state.pairs.len() - 1);
                        state.output_name = default_tgt;
                        state.output_label.clear();
                    }
                }
            }
        });

        // Col 3: Pairs list + Output Variable properties (200 px)
        ui.allocate_ui_with_layout(vec2(200.0, 280.0), Layout::top_down(Align::Min), |ui| {
            ui.label(t!("dialog-recode-vars-output-vars"));
            egui::Frame::canvas(ui.style())
                .fill(Tokens::BG_PANEL)
                .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
                .show(ui, |ui| {
                    let list_sz = vec2(196.0, 110.0);
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
                                state.output_name = p.target.clone();
                                state.output_label = p.label.clone();
                            }
                        }
                    });
                });

            ui.add_space(6.0);

            GroupBox::new(t!("dialog-recode-output-var-group")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-recode-output-name"));
                    ui.add(TextEdit::singleline(&mut state.output_name).desired_width(85.0));
                });
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-recode-output-label"));
                    ui.add(TextEdit::singleline(&mut state.output_label).desired_width(110.0));
                });

                let can_change =
                    state.selected_pair_idx.is_some() && !state.output_name.trim().is_empty();
                if ui
                    .add_enabled(can_change, Button::new(t!("dialog-recode-btn-change")))
                    .clicked()
                {
                    if let Some(idx) = state.selected_pair_idx {
                        if let Some(p) = state.pairs.get_mut(idx) {
                            p.target = state.output_name.trim().to_string();
                            p.label = state.output_label.trim().to_string();
                        }
                    }
                }
            });
        });

        // Col 4: Right sidebar (100 px)
        ui.allocate_ui_with_layout(vec2(100.0, 280.0), Layout::top_down(Align::Min), |ui| {
            ui.add_space(18.0);
            let btn_text = format!(
                "{} ({})",
                t!("dialog-recode-btn-old-new"),
                state.mappings.len()
            );
            if SubdialogSidebar::button(ui, btn_text, true).clicked() {
                state.old_new_state.open_with(&state.mappings);
            }
        });
    });
}

fn render_same_content(
    ui: &mut Ui,
    state: &mut RecodeDialogState,
    available_vars: &[(usize, String, String, Measure)],
) {
    ui.horizontal(|ui| {
        // Col 1: Available variables list (190 px)
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
            vec2(190.0, 280.0),
            "recode_same_source_vars",
            &source_items,
        );

        match source_ev {
            ListEvent::Clicked(idx) => {
                state.selected_available_idx = Some(idx);
                state.selected_var_idx = None;
            }
            ListEvent::DoubleClicked(idx) => {
                if let Some((_, name, _, _)) = available_vars.get(idx) {
                    if !state.selected_vars.contains(name) {
                        state.selected_vars.push(name.clone());
                    }
                    state.selected_available_idx = None;
                }
            }
            ListEvent::None => {}
        }

        // Col 2: Transfer button (36 px)
        let can_move_right = state.selected_available_idx.is_some();
        let can_move_left = state.selected_var_idx.is_some();

        let transfer_btn = if state.selected_var_idx.is_some() {
            TransferButton::to_source(can_move_left)
        } else {
            TransferButton::to_target(can_move_right)
        };

        ui.allocate_ui_with_layout(vec2(36.0, 280.0), Layout::top_down(Align::Center), |ui| {
            ui.add_space(60.0);
            if transfer_btn.show(ui).clicked() {
                if state.selected_var_idx.is_some() {
                    if let Some(idx) = state.selected_var_idx {
                        if idx < state.selected_vars.len() {
                            state.selected_vars.remove(idx);
                            state.selected_var_idx = None;
                        }
                    }
                } else if let Some(idx) = state.selected_available_idx {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        if !state.selected_vars.contains(name) {
                            state.selected_vars.push(name.clone());
                        }
                    }
                }
            }
        });

        // Col 3: Selected variables list (190 px)
        ui.allocate_ui_with_layout(vec2(190.0, 280.0), Layout::top_down(Align::Min), |ui| {
            ui.label(t!("dialog-recode-vars-list"));
            egui::Frame::canvas(ui.style())
                .fill(Tokens::BG_PANEL)
                .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
                .show(ui, |ui| {
                    let list_sz = vec2(186.0, 250.0);
                    ui.set_min_size(list_sz);
                    ui.set_max_size(list_sz);
                    ScrollArea::vertical().show(ui, |ui| {
                        let mut clicked_idx = None;
                        for (idx, name) in state.selected_vars.iter().enumerate() {
                            let is_sel = state.selected_var_idx == Some(idx);
                            if ui.selectable_label(is_sel, name).clicked() {
                                clicked_idx = Some(idx);
                            }
                        }
                        if let Some(idx) = clicked_idx {
                            state.selected_var_idx = Some(idx);
                            state.selected_available_idx = None;
                        }
                    });
                });
        });

        // Col 4: Right sidebar (100 px)
        ui.allocate_ui_with_layout(vec2(100.0, 280.0), Layout::top_down(Align::Min), |ui| {
            ui.add_space(18.0);
            let btn_text = format!(
                "{} ({})",
                t!("dialog-recode-btn-old-new"),
                state.mappings.len()
            );
            if SubdialogSidebar::button(ui, btn_text, true).clicked() {
                state.old_new_state.open_with(&state.mappings);
            }
        });
    });
}
