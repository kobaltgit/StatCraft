use super::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use egui::{vec2, Align, Layout, ScrollArea, TextEdit, Ui};
use statcraft_algo::{NewValueSpec, OldValueRule, RecodeMapping};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OldValueType {
    #[default]
    Value,
    SysMissing,
    SystemOrUserMissing,
    Range,
    RangeLowest,
    RangeHighest,
    AllOther,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewValueType {
    #[default]
    Value,
    SysMissing,
    CopyOld,
}

#[derive(Debug, Clone, Default)]
pub struct RecodeOldNewState {
    pub is_open: bool,
    pub old_type: OldValueType,
    pub old_val_str: String,
    pub old_range_low_str: String,
    pub old_range_high_str: String,
    pub new_type: NewValueType,
    pub new_val_str: String,
    pub selected_idx: Option<usize>,
    pub temp_mappings: Vec<RecodeMapping>,
}

impl RecodeOldNewState {
    pub fn open_with(&mut self, existing: &[RecodeMapping]) {
        self.is_open = true;
        self.old_type = OldValueType::Value;
        self.old_val_str.clear();
        self.old_range_low_str.clear();
        self.old_range_high_str.clear();
        self.new_type = NewValueType::Value;
        self.new_val_str.clear();
        self.selected_idx = None;
        self.temp_mappings = existing.to_vec();
    }

    fn parse_old_rule(&self) -> Option<OldValueRule> {
        match self.old_type {
            OldValueType::Value => self
                .old_val_str
                .trim()
                .parse::<f64>()
                .ok()
                .map(OldValueRule::Exact),
            OldValueType::SysMissing => Some(OldValueRule::SysMissing),
            OldValueType::SystemOrUserMissing => Some(OldValueRule::SystemOrUserMissing),
            OldValueType::Range => {
                let low = self.old_range_low_str.trim().parse::<f64>().ok()?;
                let high = self.old_range_high_str.trim().parse::<f64>().ok()?;
                Some(OldValueRule::Range(low, high))
            }
            OldValueType::RangeLowest => {
                let high = self.old_range_high_str.trim().parse::<f64>().ok()?;
                Some(OldValueRule::RangeLowest(high))
            }
            OldValueType::RangeHighest => {
                let low = self.old_range_low_str.trim().parse::<f64>().ok()?;
                Some(OldValueRule::RangeHighest(low))
            }
            OldValueType::AllOther => Some(OldValueRule::AllOther),
        }
    }

    fn parse_new_spec(&self) -> Option<NewValueSpec> {
        match self.new_type {
            NewValueType::Value => self
                .new_val_str
                .trim()
                .parse::<f64>()
                .ok()
                .map(NewValueSpec::Value),
            NewValueType::SysMissing => Some(NewValueSpec::SysMissing),
            NewValueType::CopyOld => Some(NewValueSpec::CopyOld),
        }
    }

    fn load_mapping(&mut self, idx: usize) {
        if let Some(m) = self.temp_mappings.get(idx) {
            match m.old {
                OldValueRule::Exact(v) => {
                    self.old_type = OldValueType::Value;
                    self.old_val_str = v.to_string();
                }
                OldValueRule::SysMissing => {
                    self.old_type = OldValueType::SysMissing;
                }
                OldValueRule::SystemOrUserMissing => {
                    self.old_type = OldValueType::SystemOrUserMissing;
                }
                OldValueRule::Range(low, high) => {
                    self.old_type = OldValueType::Range;
                    self.old_range_low_str = low.to_string();
                    self.old_range_high_str = high.to_string();
                }
                OldValueRule::RangeLowest(high) => {
                    self.old_type = OldValueType::RangeLowest;
                    self.old_range_high_str = high.to_string();
                }
                OldValueRule::RangeHighest(low) => {
                    self.old_type = OldValueType::RangeHighest;
                    self.old_range_low_str = low.to_string();
                }
                OldValueRule::AllOther => {
                    self.old_type = OldValueType::AllOther;
                }
            }

            match m.new {
                NewValueSpec::Value(v) => {
                    self.new_type = NewValueType::Value;
                    self.new_val_str = v.to_string();
                }
                NewValueSpec::SysMissing => {
                    self.new_type = NewValueType::SysMissing;
                }
                NewValueSpec::CopyOld => {
                    self.new_type = NewValueType::CopyOld;
                }
            }
            self.selected_idx = Some(idx);
        }
    }
}

pub enum RecodeOldNewAction {
    Apply(Vec<RecodeMapping>),
    None,
}

pub fn render_recode_old_new_dialog(
    ctx: &egui::Context,
    state: &mut RecodeOldNewState,
) -> RecodeOldNewAction {
    if !state.is_open {
        return RecodeOldNewAction::None;
    }

    let mut action = RecodeOldNewAction::None;
    let mut is_open = state.is_open;
    let mut close = false;

    DialogShell::standard(t!("dialog-recode-old-new-title")).show(ctx, &mut is_open, |ui| {
        // Top section: Old Value and New Value side by side
        ui.horizontal(|ui| {
            // Left: Old Value GroupBox (260 px)
            ui.allocate_ui_with_layout(vec2(265.0, 230.0), Layout::top_down(Align::Min), |ui| {
                GroupBox::new(t!("dialog-recode-old-val-group")).show(ui, |ui| {
                    render_old_value_panel(ui, state);
                });
            });

            ui.add_space(6.0);

            // Right: New Value GroupBox (250 px)
            ui.allocate_ui_with_layout(vec2(255.0, 230.0), Layout::top_down(Align::Min), |ui| {
                GroupBox::new(t!("dialog-recode-new-val-group")).show(ui, |ui| {
                    render_new_value_panel(ui, state);
                });
            });
        });

        ui.add_space(6.0);

        // Lower section: List of mappings
        GroupBox::new(t!("dialog-recode-old-new-mappings-title")).show(ui, |ui| {
            ScrollArea::vertical()
                .max_height(80.0)
                .min_scrolled_height(60.0)
                .show(ui, |ui| {
                    let mut clicked_idx = None;
                    for (idx, m) in state.temp_mappings.iter().enumerate() {
                        let is_selected = state.selected_idx == Some(idx);
                        let text = m.display_label();
                        let resp = ui.selectable_label(is_selected, text);
                        if resp.clicked() {
                            clicked_idx = Some(idx);
                        }
                    }
                    if let Some(idx) = clicked_idx {
                        state.load_mapping(idx);
                    }
                });
        });

        // Bottom action bar
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                action = RecodeOldNewAction::Apply(state.temp_mappings.clone());
                close = true;
            }
            SubdialogAction::Cancel => {
                close = true;
            }
            SubdialogAction::None | SubdialogAction::Help => {}
        }
    });

    state.is_open = is_open && !close;

    action
}

fn render_old_value_panel(ui: &mut Ui, state: &mut RecodeOldNewState) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut state.old_type,
                OldValueType::Value,
                t!("dialog-recode-old-val"),
            );
            ui.add(TextEdit::singleline(&mut state.old_val_str).desired_width(65.0));
        });

        ui.radio_value(
            &mut state.old_type,
            OldValueType::SysMissing,
            t!("dialog-recode-old-sysmis"),
        );

        ui.radio_value(
            &mut state.old_type,
            OldValueType::SystemOrUserMissing,
            t!("dialog-recode-old-missing"),
        );

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut state.old_type,
                OldValueType::Range,
                t!("dialog-recode-old-range"),
            );
            ui.add(TextEdit::singleline(&mut state.old_range_low_str).desired_width(40.0));
            ui.label(t!("dialog-recode-through"));
            ui.add(TextEdit::singleline(&mut state.old_range_high_str).desired_width(40.0));
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut state.old_type,
                OldValueType::RangeLowest,
                t!("dialog-recode-old-range-lowest"),
            );
            ui.add(TextEdit::singleline(&mut state.old_range_high_str).desired_width(50.0));
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut state.old_type,
                OldValueType::RangeHighest,
                t!("dialog-recode-old-range-highest"),
            );
            ui.add(TextEdit::singleline(&mut state.old_range_low_str).desired_width(50.0));
        });

        ui.radio_value(
            &mut state.old_type,
            OldValueType::AllOther,
            t!("dialog-recode-old-all-other"),
        );
    });
}

fn render_new_value_panel(ui: &mut Ui, state: &mut RecodeOldNewState) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut state.new_type,
                NewValueType::Value,
                t!("dialog-recode-new-val"),
            );
            ui.add(TextEdit::singleline(&mut state.new_val_str).desired_width(65.0));
        });

        ui.radio_value(
            &mut state.new_type,
            NewValueType::SysMissing,
            t!("dialog-recode-new-sysmis"),
        );

        ui.radio_value(
            &mut state.new_type,
            NewValueType::CopyOld,
            t!("dialog-recode-new-copy"),
        );

        ui.add_space(14.0);

        // Action buttons [ Add ] [ Change ] [ Remove ]
        let can_add = state.parse_old_rule().is_some() && state.parse_new_spec().is_some();
        let can_change = state.selected_idx.is_some() && can_add;
        let can_remove = state.selected_idx.is_some();

        ui.horizontal(|ui| {
            if ui
                .add_enabled(can_add, egui::Button::new(t!("dialog-recode-btn-add")))
                .clicked()
            {
                if let (Some(old), Some(new)) = (state.parse_old_rule(), state.parse_new_spec()) {
                    state.temp_mappings.push(RecodeMapping::new(old, new));
                    state.selected_idx = None;
                }
            }

            if ui
                .add_enabled(
                    can_change,
                    egui::Button::new(t!("dialog-recode-btn-change")),
                )
                .clicked()
            {
                if let (Some(idx), Some(old), Some(new)) = (
                    state.selected_idx,
                    state.parse_old_rule(),
                    state.parse_new_spec(),
                ) {
                    if idx < state.temp_mappings.len() {
                        state.temp_mappings[idx] = RecodeMapping::new(old, new);
                    }
                }
            }

            if ui
                .add_enabled(
                    can_remove,
                    egui::Button::new(t!("dialog-recode-btn-remove")),
                )
                .clicked()
            {
                if let Some(idx) = state.selected_idx {
                    if idx < state.temp_mappings.len() {
                        state.temp_mappings.remove(idx);
                        state.selected_idx = None;
                    }
                }
            }
        });
    });
}
