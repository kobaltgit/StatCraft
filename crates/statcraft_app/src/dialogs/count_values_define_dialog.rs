use super::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use egui::{vec2, Align, Layout, ScrollArea, TextEdit, Ui};
use statcraft_algo::ValueCriterion;
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DefineCriterionType {
    #[default]
    Value,
    SystemMissing,
    Range,
    RangeLowest,
    RangeHighest,
}

#[derive(Debug, Clone, Default)]
pub struct CountValuesDefineState {
    pub is_open: bool,
    pub criterion_type: DefineCriterionType,
    pub val_str: String,
    pub range_low_str: String,
    pub range_high_str: String,
    pub selected_idx: Option<usize>,
    pub temp_criteria: Vec<ValueCriterion>,
}

impl CountValuesDefineState {
    pub fn open_with(&mut self, existing: &[ValueCriterion]) {
        self.is_open = true;
        self.criterion_type = DefineCriterionType::Value;
        self.val_str.clear();
        self.range_low_str.clear();
        self.range_high_str.clear();
        self.selected_idx = None;
        self.temp_criteria = existing.to_vec();
    }

    fn parse_current_criterion(&self) -> Option<ValueCriterion> {
        match self.criterion_type {
            DefineCriterionType::Value => {
                let trimmed = self.val_str.trim();
                if trimmed.is_empty() {
                    return None;
                }
                if let Ok(num) = trimmed.parse::<f64>() {
                    Some(ValueCriterion::ExactNumber(num))
                } else {
                    Some(ValueCriterion::ExactString(trimmed.to_string()))
                }
            }
            DefineCriterionType::SystemMissing => Some(ValueCriterion::SystemMissing),
            DefineCriterionType::Range => {
                let low = self.range_low_str.trim().parse::<f64>().ok()?;
                let high = self.range_high_str.trim().parse::<f64>().ok()?;
                Some(ValueCriterion::Range { low, high })
            }
            DefineCriterionType::RangeLowest => {
                let high = self.range_high_str.trim().parse::<f64>().ok()?;
                Some(ValueCriterion::LowestThru(high))
            }
            DefineCriterionType::RangeHighest => {
                let low = self.range_low_str.trim().parse::<f64>().ok()?;
                Some(ValueCriterion::ThruHighest(low))
            }
        }
    }

    fn load_criterion(&mut self, idx: usize) {
        if let Some(c) = self.temp_criteria.get(idx) {
            match c {
                ValueCriterion::ExactNumber(n) => {
                    self.criterion_type = DefineCriterionType::Value;
                    self.val_str = n.to_string();
                }
                ValueCriterion::ExactString(s) => {
                    self.criterion_type = DefineCriterionType::Value;
                    self.val_str = s.clone();
                }
                ValueCriterion::SystemMissing => {
                    self.criterion_type = DefineCriterionType::SystemMissing;
                }
                ValueCriterion::Range { low, high } => {
                    self.criterion_type = DefineCriterionType::Range;
                    self.range_low_str = low.to_string();
                    self.range_high_str = high.to_string();
                }
                ValueCriterion::LowestThru(high) => {
                    self.criterion_type = DefineCriterionType::RangeLowest;
                    self.range_high_str = high.to_string();
                }
                ValueCriterion::ThruHighest(low) => {
                    self.criterion_type = DefineCriterionType::RangeHighest;
                    self.range_low_str = low.to_string();
                }
            }
            self.selected_idx = Some(idx);
        }
    }
}

pub enum CountValuesDefineAction {
    Apply(Vec<ValueCriterion>),
    None,
}

pub fn render_count_values_define_dialog(
    ctx: &egui::Context,
    state: &mut CountValuesDefineState,
) -> CountValuesDefineAction {
    if !state.is_open {
        return CountValuesDefineAction::None;
    }

    let mut action = CountValuesDefineAction::None;
    let mut is_open = state.is_open;
    let mut close = false;

    DialogShell::standard(t!("dialog-count-values-title")).show(ctx, &mut is_open, |ui| {
        ui.horizontal(|ui| {
            // Left: Criteria inputs panel (270 px)
            ui.allocate_ui_with_layout(vec2(275.0, 320.0), Layout::top_down(Align::Min), |ui| {
                GroupBox::new(t!("dialog-count-val-value")).show(ui, |ui| {
                    render_criteria_options(ui, state);
                });
            });

            // Center: Add / Change / Remove buttons (70 px)
            ui.allocate_ui_with_layout(vec2(75.0, 320.0), Layout::top_down(Align::Center), |ui| {
                ui.add_space(50.0);
                let can_add = state.parse_current_criterion().is_some();
                if ui
                    .add_enabled(can_add, egui::Button::new(t!("dialog-count-btn-add")))
                    .clicked()
                {
                    if let Some(crit) = state.parse_current_criterion() {
                        state.temp_criteria.push(crit);
                        state.selected_idx = None;
                    }
                }

                ui.add_space(8.0);
                let can_change = state.selected_idx.is_some() && can_add;
                if ui
                    .add_enabled(can_change, egui::Button::new(t!("dialog-count-btn-change")))
                    .clicked()
                {
                    if let (Some(idx), Some(crit)) =
                        (state.selected_idx, state.parse_current_criterion())
                    {
                        if idx < state.temp_criteria.len() {
                            state.temp_criteria[idx] = crit;
                        }
                    }
                }

                ui.add_space(8.0);
                let can_remove = state.selected_idx.is_some();
                if ui
                    .add_enabled(can_remove, egui::Button::new(t!("dialog-count-btn-remove")))
                    .clicked()
                {
                    if let Some(idx) = state.selected_idx {
                        if idx < state.temp_criteria.len() {
                            state.temp_criteria.remove(idx);
                            state.selected_idx = None;
                        }
                    }
                }
            });

            // Right: Values list (175 px)
            ui.allocate_ui_with_layout(vec2(175.0, 320.0), Layout::top_down(Align::Min), |ui| {
                GroupBox::new(t!("dialog-count-values-to-count")).show(ui, |ui| {
                    ScrollArea::vertical()
                        .max_height(280.0)
                        .min_scrolled_height(200.0)
                        .show(ui, |ui| {
                            let mut clicked_idx = None;
                            for (idx, c) in state.temp_criteria.iter().enumerate() {
                                let is_selected = state.selected_idx == Some(idx);
                                let text = c.display_label();
                                let resp = ui.selectable_label(is_selected, text);
                                if resp.clicked() {
                                    clicked_idx = Some(idx);
                                }
                            }
                            if let Some(idx) = clicked_idx {
                                state.load_criterion(idx);
                            }
                        });
                });
            });
        });

        ui.add_space(10.0);

        // Bottom action bar
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                action = CountValuesDefineAction::Apply(state.temp_criteria.clone());
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

fn render_criteria_options(ui: &mut Ui, state: &mut CountValuesDefineState) {
    ui.vertical(|ui| {
        // 1. Exact Value
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut state.criterion_type,
                DefineCriterionType::Value,
                t!("dialog-count-val-value"),
            );
            let val_enabled = state.criterion_type == DefineCriterionType::Value;
            ui.add_enabled(
                val_enabled,
                TextEdit::singleline(&mut state.val_str).desired_width(120.0),
            );
        });

        ui.add_space(8.0);

        // 2. System-missing
        ui.radio_value(
            &mut state.criterion_type,
            DefineCriterionType::SystemMissing,
            t!("dialog-count-val-sysmis"),
        );

        ui.add_space(8.0);

        // 3. Range [low] through [high]
        ui.radio_value(
            &mut state.criterion_type,
            DefineCriterionType::Range,
            t!("dialog-count-val-range"),
        );
        let range_enabled = state.criterion_type == DefineCriterionType::Range;
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.add_enabled(
                range_enabled,
                TextEdit::singleline(&mut state.range_low_str).desired_width(60.0),
            );
            ui.label(t!("dialog-count-through"));
            ui.add_enabled(
                range_enabled,
                TextEdit::singleline(&mut state.range_high_str).desired_width(60.0),
            );
        });

        ui.add_space(8.0);

        // 4. Range, LOWEST through [high]
        ui.radio_value(
            &mut state.criterion_type,
            DefineCriterionType::RangeLowest,
            t!("dialog-count-val-range-lowest"),
        );
        let lowest_enabled = state.criterion_type == DefineCriterionType::RangeLowest;
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.add_enabled(
                lowest_enabled,
                TextEdit::singleline(&mut state.range_high_str).desired_width(70.0),
            );
        });

        ui.add_space(8.0);

        // 5. Range, [low] through HIGHEST
        ui.radio_value(
            &mut state.criterion_type,
            DefineCriterionType::RangeHighest,
            t!("dialog-count-val-range-highest"),
        );
        let highest_enabled = state.criterion_type == DefineCriterionType::RangeHighest;
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.add_enabled(
                highest_enabled,
                TextEdit::singleline(&mut state.range_low_str).desired_width(70.0),
            );
        });
    });
}
