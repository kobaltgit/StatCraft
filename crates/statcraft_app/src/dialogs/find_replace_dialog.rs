use super::common::{DialogShell, GroupBox};
use crate::theme::Tokens;
use egui::{vec2, Button, Context, TextEdit};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindReplaceMode {
    Find,
    Replace,
}

#[derive(Debug, Clone)]
pub struct FindReplaceDialogState {
    pub is_open: bool,
    pub mode: FindReplaceMode,
    pub find_text: String,
    pub replace_text: String,
    pub match_case: bool,
    pub match_entire_cell: bool,
    pub search_down: bool,
    pub target_col: Option<usize>,
    pub last_match: Option<(usize, usize)>,
    pub status_message: Option<String>,
}

impl Default for FindReplaceDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            mode: FindReplaceMode::Find,
            find_text: String::new(),
            replace_text: String::new(),
            match_case: false,
            match_entire_cell: false,
            search_down: true,
            target_col: None,
            last_match: None,
            status_message: None,
        }
    }
}

impl FindReplaceDialogState {
    pub fn open_find(&mut self) {
        self.is_open = true;
        self.mode = FindReplaceMode::Find;
        self.status_message = None;
    }

    pub fn open_replace(&mut self) {
        self.is_open = true;
        self.mode = FindReplaceMode::Replace;
        self.status_message = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindReplaceAction {
    None,
    FindNext {
        find_text: String,
        match_case: bool,
        match_entire_cell: bool,
        search_down: bool,
        target_col: Option<usize>,
    },
    Replace {
        find_text: String,
        replace_text: String,
        match_case: bool,
        match_entire_cell: bool,
        search_down: bool,
        target_col: Option<usize>,
    },
    ReplaceAll {
        find_text: String,
        replace_text: String,
        match_case: bool,
        match_entire_cell: bool,
        target_col: Option<usize>,
    },
    Close,
}

pub fn render_find_replace_dialog(
    ctx: &Context,
    dataset: Option<&DataSet>,
    state: &mut FindReplaceDialogState,
) -> FindReplaceAction {
    if !state.is_open {
        return FindReplaceAction::None;
    }

    let mut action = FindReplaceAction::None;
    let mut is_open = state.is_open;

    DialogShell::subdialog(t!("dialog-find-replace-title")).show(ctx, &mut is_open, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut state.mode,
                FindReplaceMode::Find,
                t!("dialog-find-tab-find"),
            );
            ui.selectable_value(
                &mut state.mode,
                FindReplaceMode::Replace,
                t!("dialog-find-tab-replace"),
            );
        });

        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(t!("dialog-find-what"));
            ui.add(TextEdit::singleline(&mut state.find_text).desired_width(220.0));
        });

        if state.mode == FindReplaceMode::Replace {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(t!("dialog-find-replace-with"));
                ui.add(TextEdit::singleline(&mut state.replace_text).desired_width(220.0));
            });
        }

        ui.add_space(6.0);

        GroupBox::new(t!("dialog-find-direction")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.radio_value(&mut state.search_down, true, t!("dialog-find-dir-down"));
                ui.radio_value(&mut state.search_down, false, t!("dialog-find-dir-up"));
            });
            ui.checkbox(&mut state.match_case, t!("dialog-find-match-case"));
            ui.checkbox(&mut state.match_entire_cell, t!("dialog-find-match-entire"));
        });

        if let Some(ds) = dataset {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(t!("dialog-find-column"));
                let current_label = match state.target_col {
                    Some(col_idx) => ds
                        .dictionary
                        .get(col_idx)
                        .map(|v| v.name.clone())
                        .unwrap_or_else(|| t!("dialog-find-col-all")),
                    None => t!("dialog-find-col-all"),
                };

                egui::ComboBox::from_id_salt("find_col_combo")
                    .selected_text(current_label)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.target_col, None, t!("dialog-find-col-all"));
                        for col_idx in 0..ds.col_count() {
                            if let Some(meta) = ds.dictionary.get(col_idx) {
                                ui.selectable_value(
                                    &mut state.target_col,
                                    Some(col_idx),
                                    &meta.name,
                                );
                            }
                        }
                    });
            });
        }

        if let Some(ref msg) = state.status_message {
            ui.add_space(4.0);
            ui.colored_label(Tokens::text_muted(), msg);
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        let can_search = !state.find_text.trim().is_empty();

        ui.horizontal(|ui| {
            let btn_size = vec2(80.0, 24.0);

            if ui
                .add_enabled(
                    can_search,
                    Button::new(t!("dialog-find-btn-find-next")).min_size(btn_size),
                )
                .clicked()
            {
                action = FindReplaceAction::FindNext {
                    find_text: state.find_text.clone(),
                    match_case: state.match_case,
                    match_entire_cell: state.match_entire_cell,
                    search_down: state.search_down,
                    target_col: state.target_col,
                };
            }

            if state.mode == FindReplaceMode::Replace {
                if ui
                    .add_enabled(
                        can_search,
                        Button::new(t!("dialog-find-btn-replace")).min_size(btn_size),
                    )
                    .clicked()
                {
                    action = FindReplaceAction::Replace {
                        find_text: state.find_text.clone(),
                        replace_text: state.replace_text.clone(),
                        match_case: state.match_case,
                        match_entire_cell: state.match_entire_cell,
                        search_down: state.search_down,
                        target_col: state.target_col,
                    };
                }

                if ui
                    .add_enabled(
                        can_search,
                        Button::new(t!("dialog-find-btn-replace-all")).min_size(btn_size),
                    )
                    .clicked()
                {
                    action = FindReplaceAction::ReplaceAll {
                        find_text: state.find_text.clone(),
                        replace_text: state.replace_text.clone(),
                        match_case: state.match_case,
                        match_entire_cell: state.match_entire_cell,
                        target_col: state.target_col,
                    };
                }
            }

            if ui
                .add(Button::new(t!("dialog-find-btn-close")).min_size(btn_size))
                .clicked()
            {
                action = FindReplaceAction::Close;
                state.is_open = false;
            }
        });
    });

    if !is_open {
        state.is_open = false;
        if action == FindReplaceAction::None {
            action = FindReplaceAction::Close;
        }
    }

    action
}
