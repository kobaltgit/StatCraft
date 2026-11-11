use super::common::{DialogShell, SubdialogAction, SubdialogActionBar};
use crate::widgets::paint_measure_icon;
use egui::{pos2, vec2, Context, Rect, ScrollArea, TextEdit};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoToTab {
    Case,
    Variable,
}

#[derive(Debug, Clone)]
pub struct GoToDialogState {
    pub is_open: bool,
    pub active_tab: GoToTab,
    pub case_number_text: String,
    pub var_filter_text: String,
    pub selected_var_index: Option<usize>,
}

impl Default for GoToDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: GoToTab::Case,
            case_number_text: String::new(),
            var_filter_text: String::new(),
            selected_var_index: None,
        }
    }
}

impl GoToDialogState {
    pub fn open_case(&mut self, current_case: Option<usize>) {
        self.is_open = true;
        self.active_tab = GoToTab::Case;
        if let Some(c) = current_case {
            self.case_number_text = format!("{}", c + 1);
        }
    }

    pub fn open_variable(&mut self, current_var: Option<usize>) {
        self.is_open = true;
        self.active_tab = GoToTab::Variable;
        self.selected_var_index = current_var;
        self.var_filter_text.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoToAction {
    None,
    GoToCase(usize),
    GoToVariable(usize),
    Close,
}

pub fn render_goto_dialog(
    ctx: &Context,
    dataset: Option<&DataSet>,
    state: &mut GoToDialogState,
) -> GoToAction {
    if !state.is_open {
        return GoToAction::None;
    }

    let mut action = GoToAction::None;
    let mut is_open = state.is_open;

    let total_rows = dataset.map(|d| d.row_count()).unwrap_or(0);
    let total_cols = dataset.map(|d| d.col_count()).unwrap_or(0);

    let parsed_case = state
        .case_number_text
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|&k| k >= 1 && k <= total_rows);

    let can_go = match state.active_tab {
        GoToTab::Case => parsed_case.is_some(),
        GoToTab::Variable => state
            .selected_var_index
            .map(|i| i < total_cols)
            .unwrap_or(false),
    };

    DialogShell::subdialog(t!("dialog-goto-title")).show(ctx, &mut is_open, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut state.active_tab,
                GoToTab::Case,
                t!("dialog-goto-tab-case"),
            );
            ui.selectable_value(
                &mut state.active_tab,
                GoToTab::Variable,
                t!("dialog-goto-tab-variable"),
            );
        });

        ui.separator();
        ui.add_space(6.0);

        match state.active_tab {
            GoToTab::Case => {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-goto-case-number"));
                    let resp = ui.add(
                        TextEdit::singleline(&mut state.case_number_text).desired_width(100.0),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Some(case_num) = parsed_case {
                            action = GoToAction::GoToCase(case_num - 1);
                            state.is_open = false;
                        }
                    }
                    ui.label(t!("dialog-goto-case-range", total = total_rows));
                });
            }
            GoToTab::Variable => {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-goto-var-filter"));
                    ui.add(TextEdit::singleline(&mut state.var_filter_text).desired_width(180.0));
                });

                ui.add_space(6.0);

                if let Some(ds) = dataset {
                    ScrollArea::vertical()
                        .max_height(140.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let filter = state.var_filter_text.trim().to_ascii_lowercase();
                            for col_idx in 0..ds.col_count() {
                                if let Some(meta) = ds.dictionary.get(col_idx) {
                                    if !filter.is_empty()
                                        && !meta.name.to_ascii_lowercase().contains(&filter)
                                        && !meta.label.to_ascii_lowercase().contains(&filter)
                                    {
                                        continue;
                                    }

                                    let is_selected = state.selected_var_index == Some(col_idx);
                                    let mut item_rect = Rect::NOTHING;
                                    ui.horizontal(|ui| {
                                        let (rect, _) = ui.allocate_exact_size(
                                            vec2(16.0, 16.0),
                                            egui::Sense::hover(),
                                        );
                                        let icon_rect = Rect::from_min_size(
                                            pos2(rect.min.x, rect.min.y),
                                            vec2(16.0, 16.0),
                                        );
                                        paint_measure_icon(ui.painter(), icon_rect, meta.measure);

                                        let display_text = if meta.label.is_empty() {
                                            meta.name.clone()
                                        } else {
                                            format!("{} [{}]", meta.label, meta.name)
                                        };

                                        let resp = ui.selectable_label(is_selected, display_text);
                                        item_rect = resp.rect;
                                        if resp.clicked() {
                                            state.selected_var_index = Some(col_idx);
                                        }
                                        if resp.double_clicked() {
                                            state.selected_var_index = Some(col_idx);
                                            action = GoToAction::GoToVariable(col_idx);
                                            state.is_open = false;
                                        }
                                    });
                                }
                            }
                        });
                }
            }
        }

        ui.add_space(8.0);
        let sub_action = SubdialogActionBar::new(can_go).show(ui);
        match sub_action {
            SubdialogAction::Continue => match state.active_tab {
                GoToTab::Case => {
                    if let Some(case_num) = parsed_case {
                        action = GoToAction::GoToCase(case_num - 1);
                        state.is_open = false;
                    }
                }
                GoToTab::Variable => {
                    if let Some(col_idx) = state.selected_var_index {
                        action = GoToAction::GoToVariable(col_idx);
                        state.is_open = false;
                    }
                }
            },
            SubdialogAction::Cancel => {
                action = GoToAction::Close;
                state.is_open = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !is_open {
        state.is_open = false;
        if action == GoToAction::None {
            action = GoToAction::Close;
        }
    }

    action
}
