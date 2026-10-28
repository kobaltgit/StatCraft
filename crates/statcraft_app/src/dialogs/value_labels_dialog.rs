use super::common::{ActionBarAction, DialogActionBar, DialogShell, GroupBox};
use crate::theme::Tokens;
use egui::{vec2, ScrollArea, Stroke};
use statcraft_i18n::t;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct ValueLabelsDialogState {
    pub is_open: bool,
    pub col_index: usize,
    pub var_name: String,
    pub current_value: String,
    pub current_label: String,
    pub selected_item: Option<String>,
    pub mapping: BTreeMap<String, String>,
}

impl ValueLabelsDialogState {
    pub fn open_for_variable(
        &mut self,
        col_index: usize,
        var_name: &str,
        existing: &BTreeMap<String, String>,
    ) {
        self.is_open = true;
        self.col_index = col_index;
        self.var_name = var_name.to_string();
        self.current_value.clear();
        self.current_label.clear();
        self.selected_item = None;
        self.mapping = existing.clone();
    }
}

pub enum ValueLabelsDialogAction {
    Apply {
        col_index: usize,
        mapping: BTreeMap<String, String>,
    },
    None,
}

pub fn render_value_labels_dialog(
    ctx: &egui::Context,
    state: &mut ValueLabelsDialogState,
) -> ValueLabelsDialogAction {
    if !state.is_open {
        return ValueLabelsDialogAction::None;
    }

    let mut action = ValueLabelsDialogAction::None;
    let mut open = true;
    let mut close_dialog = false;

    let title = t!("dialog-value-labels-title", name = state.var_name.as_str());

    DialogShell::properties(&title).show(ctx, &mut open, |ui| {
        GroupBox::new(t!("dialog-value-labels-list-title")).show(ui, |ui| {
            // 1. Input fields: Value & Label
            egui::Grid::new("value_labels_input_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label(t!("dialog-value-labels-value"));
                    ui.add_sized(
                        [240.0, 20.0],
                        egui::TextEdit::singleline(&mut state.current_value),
                    );
                    ui.end_row();

                    ui.label(t!("dialog-value-labels-label"));
                    ui.add_sized(
                        [240.0, 20.0],
                        egui::TextEdit::singleline(&mut state.current_label),
                    );
                    ui.end_row();
                });

            ui.add_space(8.0);

            // 2. Action buttons: [ Add ] [ Change ] [ Remove ]
            let can_add =
                !state.current_value.trim().is_empty() && !state.current_label.trim().is_empty();
            let can_change = state.selected_item.is_some()
                && !state.current_value.trim().is_empty()
                && !state.current_label.trim().is_empty();
            let can_remove = state.selected_item.is_some();

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        can_add,
                        egui::Button::new(t!("dialog-value-labels-btn-add")),
                    )
                    .clicked()
                {
                    state.mapping.insert(
                        state.current_value.trim().to_string(),
                        state.current_label.trim().to_string(),
                    );
                    state.current_value.clear();
                    state.current_label.clear();
                    state.selected_item = None;
                }

                if ui
                    .add_enabled(
                        can_change,
                        egui::Button::new(t!("dialog-value-labels-btn-change")),
                    )
                    .clicked()
                {
                    if let Some(old_val) = &state.selected_item {
                        state.mapping.remove(old_val);
                    }
                    state.mapping.insert(
                        state.current_value.trim().to_string(),
                        state.current_label.trim().to_string(),
                    );
                    state.selected_item = Some(state.current_value.trim().to_string());
                }

                if ui
                    .add_enabled(
                        can_remove,
                        egui::Button::new(t!("dialog-value-labels-btn-remove")),
                    )
                    .clicked()
                {
                    if let Some(sel) = &state.selected_item {
                        state.mapping.remove(sel);
                        state.selected_item = None;
                        state.current_value.clear();
                        state.current_label.clear();
                    }
                }
            });
        });

        // 3. Scrollable List of existing labels
        ui.label(t!("dialog-value-labels-list-title"));

        egui::Frame::canvas(ui.style())
            .fill(Tokens::BG_PANEL)
            .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
            .show(ui, |ui| {
                ui.set_min_size(vec2(410.0, 110.0));
                ui.set_max_size(vec2(410.0, 110.0));

                ScrollArea::vertical()
                    .id_salt("value_labels_list")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let keys: Vec<String> = state.mapping.keys().cloned().collect();
                        for key in keys {
                            let label = state.mapping.get(&key).map(|s| s.as_str()).unwrap_or("");
                            let is_selected = state.selected_item.as_deref() == Some(&key);
                            let display_text = format!("{} = \"{}\"", key, label);

                            let res = ui.selectable_label(is_selected, display_text);
                            if res.clicked() {
                                state.selected_item = Some(key.clone());
                                state.current_value = key;
                                state.current_label = label.to_string();
                            }
                        }
                    });
            });

        // 4. Bottom Buttons: DialogActionBar
        let bar_action = DialogActionBar::new(true)
            .show_paste(false)
            .show_reset(false)
            .show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                action = ValueLabelsDialogAction::Apply {
                    col_index: state.col_index,
                    mapping: state.mapping.clone(),
                };
                close_dialog = true;
            }
            ActionBarAction::Cancel => {
                close_dialog = true;
            }
            ActionBarAction::Help
            | ActionBarAction::Paste
            | ActionBarAction::Reset
            | ActionBarAction::None => {}
        }
    });

    if !open || close_dialog {
        state.is_open = false;
    }
    action
}
