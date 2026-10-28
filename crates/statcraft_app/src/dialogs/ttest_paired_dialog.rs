use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, SubdialogAction, SubdialogActionBar, SubdialogSidebar, TransferButton,
    VariableListItem,
};
use crate::theme::Tokens;
use egui::{vec2, Stroke};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct PairedSamplesTTestDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub active_slot: usize, // 0 for var1, 1 for var2
    pub current_var1: Option<String>,
    pub current_var2: Option<String>,
    pub paired_vars: Vec<(String, String)>,
    pub pair_selected: Option<usize>,
    pub ci_level_str: String,
    pub show_options: bool,
}

impl Default for PairedSamplesTTestDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            active_slot: 0,
            current_var1: None,
            current_var2: None,
            paired_vars: Vec::new(),
            pair_selected: None,
            ci_level_str: "95".to_string(),
            show_options: false,
        }
    }
}

pub enum PairedSamplesTTestDialogAction {
    Run {
        pairs: Vec<(String, String)>,
        ci: f64,
    },
    Paste {
        pairs: Vec<(String, String)>,
        ci: f64,
    },
    None,
}

pub fn render_ttest_paired_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut PairedSamplesTTestDialogState,
) -> PairedSamplesTTestDialogAction {
    if !state.is_open {
        return PairedSamplesTTestDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return PairedSamplesTTestDialogAction::None;
        }
    };

    let mut action = PairedSamplesTTestDialogAction::None;
    let mut close_dialog = false;

    // Available variables list
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary
                .get(col_idx)
                .map(|meta| (col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    // 1. Subdialog: Options
    if state.show_options {
        let mut sub_open = true;
        DialogShell::subdialog(t!("dialog-anova-options-title")).show(ctx, &mut sub_open, |ui| {
            GroupBox::new(t!("dialog-anova-options-title")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-ttest-ci-percentage"));
                    ui.add(egui::TextEdit::singleline(&mut state.ci_level_str).desired_width(50.0));
                });
            });

            let sub_action = SubdialogActionBar::new(true).show(ui);
            match sub_action {
                SubdialogAction::Continue | SubdialogAction::Cancel => {
                    state.show_options = false;
                }
                SubdialogAction::Help | SubdialogAction::None => {}
            }
        });

        if !sub_open {
            state.show_options = false;
        }
    }

    // 2. Main Dialog
    let mut main_open = true;
    DialogShell::standard(t!("dialog-ttest-paired-title")).show(ctx, &mut main_open, |ui| {
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
                "ttest_paired_source_vars",
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
            let can_move_right = state.source_selected.is_some()
                && state.source_selected.unwrap() < available_vars.len();
            let can_move_left = state.pair_selected.is_some()
                && state.pair_selected.unwrap() < state.paired_vars.len();

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
                        if state.pair_selected.is_some() {
                            if let Some(idx) = state.pair_selected {
                                if idx < state.paired_vars.len() {
                                    let removed = state.paired_vars.remove(idx);
                                    state.current_var1 = Some(removed.0);
                                    state.current_var2 = Some(removed.1);
                                    state.pair_selected = None;
                                    state.active_slot = 0;
                                }
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
                vec2(200.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.label(t!("dialog-ttest-paired-pairs-list"));
                    egui::Frame::canvas(ui.style())
                        .fill(Tokens::BG_PANEL)
                        .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
                        .show(ui, |ui| {
                            ui.set_min_size(vec2(196.0, 160.0));
                            ui.set_max_size(vec2(196.0, 160.0));

                            egui::ScrollArea::vertical()
                                .id_salt("ttest_paired_pairs_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for (idx, (v1, v2)) in state.paired_vars.iter().enumerate() {
                                        let is_sel = state.pair_selected == Some(idx);
                                        let prefix = t!(
                                            "dialog-ttest-paired-pair-prefix",
                                            num = (idx + 1).to_string()
                                        );
                                        let display_str = format!("{} {} - {}", prefix, v1, v2);
                                        let res = ui.selectable_label(is_sel, display_str);
                                        if res.clicked() {
                                            state.pair_selected = Some(idx);
                                            state.source_selected = None;
                                        }
                                        if res.double_clicked() {
                                            let removed = state.paired_vars.remove(idx);
                                            state.current_var1 = Some(removed.0);
                                            state.current_var2 = Some(removed.1);
                                            state.pair_selected = None;
                                            state.active_slot = 0;
                                            break;
                                        }
                                    }
                                });
                        });

                    ui.add_space(8.0);

                    // Current selections box
                    ui.label(t!("dialog-ttest-paired-current-selections"));
                    ui.horizontal(|ui| {
                        let is_s1 = state.active_slot == 0;
                        let stroke1 = if is_s1 {
                            Stroke::new(1.0_f32, Tokens::ACCENT)
                        } else {
                            Stroke::new(1.0_f32, Tokens::BORDER)
                        };
                        egui::Frame::canvas(ui.style())
                            .fill(Tokens::BG_PANEL)
                            .stroke(stroke1)
                            .show(ui, |ui| {
                                ui.set_min_size(vec2(94.0, 24.0));
                                ui.set_max_size(vec2(94.0, 24.0));
                                let v1_str = match &state.current_var1 {
                                    Some(v) => format!("1: {}", v),
                                    None => "1: -".to_string(),
                                };
                                if ui.selectable_label(is_s1, v1_str.as_str()).clicked() {
                                    state.active_slot = 0;
                                }
                            });

                        ui.add_space(4.0);

                        let is_s2 = state.active_slot == 1;
                        let stroke2 = if is_s2 {
                            Stroke::new(1.0_f32, Tokens::ACCENT)
                        } else {
                            Stroke::new(1.0_f32, Tokens::BORDER)
                        };
                        egui::Frame::canvas(ui.style())
                            .fill(Tokens::BG_PANEL)
                            .stroke(stroke2)
                            .show(ui, |ui| {
                                ui.set_min_size(vec2(94.0, 24.0));
                                ui.set_max_size(vec2(94.0, 24.0));
                                let v2_str = match &state.current_var2 {
                                    Some(v) => format!("2: {}", v),
                                    None => "2: -".to_string(),
                                };
                                if ui.selectable_label(is_s2, v2_str.as_str()).clicked() {
                                    state.active_slot = 1;
                                }
                            });
                    });
                },
            );

            // Right Column: Subdialog Sidebar
            ui.allocate_ui_with_layout(
                vec2(94.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add_space(18.0);
                    if SubdialogSidebar::button(ui, t!("dialog-btn-options"), true).clicked() {
                        state.show_options = true;
                    }
                },
            );
        });

        // Bottom Action Bar
        let has_pairs = !state.paired_vars.is_empty();
        let parsed_ci_pct = state.ci_level_str.trim().parse::<f64>().unwrap_or(95.0);
        let ci_ratio = (parsed_ci_pct / 100.0).clamp(0.01, 0.9999);

        let bar_action = DialogActionBar::new(has_pairs).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = PairedSamplesTTestDialogAction::Run {
                    pairs: state.paired_vars.clone(),
                    ci: ci_ratio,
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = PairedSamplesTTestDialogAction::Paste {
                    pairs: state.paired_vars.clone(),
                    ci: ci_ratio,
                };
            }
            ActionBarAction::Reset => {
                state.source_selected = None;
                state.active_slot = 0;
                state.current_var1 = None;
                state.current_var2 = None;
                state.paired_vars.clear();
                state.pair_selected = None;
                state.ci_level_str = "95".to_string();
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
