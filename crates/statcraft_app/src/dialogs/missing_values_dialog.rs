use super::common::{ActionBarAction, DialogActionBar, DialogShell, GroupBox};
use statcraft_core::MissingValues;
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MissingType {
    #[default]
    None,
    Discrete,
    Range,
}

#[derive(Debug, Clone, Default)]
pub struct MissingValuesDialogState {
    pub is_open: bool,
    pub col_index: usize,
    pub var_name: String,
    pub missing_type: MissingType,
    pub discrete_1: String,
    pub discrete_2: String,
    pub discrete_3: String,
    pub range_low: String,
    pub range_high: String,
    pub range_discrete: String,
}

impl MissingValuesDialogState {
    pub fn open_for_variable(&mut self, col_index: usize, var_name: &str, missing: &MissingValues) {
        self.is_open = true;
        self.col_index = col_index;
        self.var_name = var_name.to_string();
        self.discrete_1.clear();
        self.discrete_2.clear();
        self.discrete_3.clear();
        self.range_low.clear();
        self.range_high.clear();
        self.range_discrete.clear();

        if let Some((low, high)) = missing.range {
            self.missing_type = MissingType::Range;
            self.range_low = if low.fract() == 0.0 {
                format!("{:.0}", low)
            } else {
                low.to_string()
            };
            self.range_high = if high.fract() == 0.0 {
                format!("{:.0}", high)
            } else {
                high.to_string()
            };
            if let Some(d) = missing.discrete.first() {
                self.range_discrete = d.clone();
            }
        } else if !missing.discrete.is_empty() {
            self.missing_type = MissingType::Discrete;
            if let Some(d1) = missing.discrete.first() {
                self.discrete_1 = d1.clone();
            }
            if let Some(d2) = missing.discrete.get(1) {
                self.discrete_2 = d2.clone();
            }
            if let Some(d3) = missing.discrete.get(2) {
                self.discrete_3 = d3.clone();
            }
        } else {
            self.missing_type = MissingType::None;
        }
    }
}

pub enum MissingValuesDialogAction {
    Apply {
        col_index: usize,
        discrete: Vec<String>,
        range: Option<(f64, f64)>,
    },
    None,
}

pub fn render_missing_values_dialog(
    ctx: &egui::Context,
    state: &mut MissingValuesDialogState,
) -> MissingValuesDialogAction {
    if !state.is_open {
        return MissingValuesDialogAction::None;
    }

    let mut action = MissingValuesDialogAction::None;
    let mut open = true;
    let mut close_dialog = false;

    let title = t!(
        "dialog-missing-values-title",
        name = state.var_name.as_str()
    );

    DialogShell::properties(&title).show(ctx, &mut open, |ui| {
        GroupBox::new(t!("dialog-missing-none")).show(ui, |ui| {
            // 1. Radio: No missing values
            ui.radio_value(
                &mut state.missing_type,
                MissingType::None,
                t!("dialog-missing-none"),
            );
        });

        GroupBox::new(t!("dialog-missing-discrete")).show(ui, |ui| {
            // 2. Radio: Discrete missing values
            ui.radio_value(
                &mut state.missing_type,
                MissingType::Discrete,
                t!("dialog-missing-discrete"),
            );

            let is_discrete = state.missing_type == MissingType::Discrete;
            ui.add_enabled_ui(is_discrete, |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [80.0, 20.0],
                        egui::TextEdit::singleline(&mut state.discrete_1),
                    );
                    ui.add_sized(
                        [80.0, 20.0],
                        egui::TextEdit::singleline(&mut state.discrete_2),
                    );
                    ui.add_sized(
                        [80.0, 20.0],
                        egui::TextEdit::singleline(&mut state.discrete_3),
                    );
                });
            });
        });

        GroupBox::new(t!("dialog-missing-range")).show(ui, |ui| {
            // 3. Radio: Range plus one optional discrete
            ui.radio_value(
                &mut state.missing_type,
                MissingType::Range,
                t!("dialog-missing-range"),
            );

            let is_range = state.missing_type == MissingType::Range;
            ui.add_enabled_ui(is_range, |ui| {
                egui::Grid::new("range_missing_grid")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(t!("dialog-missing-low"));
                        ui.add_sized(
                            [90.0, 20.0],
                            egui::TextEdit::singleline(&mut state.range_low),
                        );
                        ui.end_row();

                        ui.label(t!("dialog-missing-high"));
                        ui.add_sized(
                            [90.0, 20.0],
                            egui::TextEdit::singleline(&mut state.range_high),
                        );
                        ui.end_row();

                        ui.label(t!("dialog-missing-discrete-extra"));
                        ui.add_sized(
                            [90.0, 20.0],
                            egui::TextEdit::singleline(&mut state.range_discrete),
                        );
                        ui.end_row();
                    });
            });
        });

        // 4. Bottom Buttons: DialogActionBar (OK, Cancel, Help)
        let bar_action = DialogActionBar::new(true)
            .show_paste(false)
            .show_reset(false)
            .show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                let mut discrete = Vec::new();
                let mut range = None;

                match state.missing_type {
                    MissingType::None => {}
                    MissingType::Discrete => {
                        for d in [&state.discrete_1, &state.discrete_2, &state.discrete_3] {
                            let trimmed = d.trim();
                            if !trimmed.is_empty() {
                                discrete.push(trimmed.to_string());
                            }
                        }
                    }
                    MissingType::Range => {
                        if let (Ok(low), Ok(high)) = (
                            state.range_low.trim().parse::<f64>(),
                            state.range_high.trim().parse::<f64>(),
                        ) {
                            range = Some((low.min(high), low.max(high)));
                        }
                        let trimmed_extra = state.range_discrete.trim();
                        if !trimmed_extra.is_empty() {
                            discrete.push(trimmed_extra.to_string());
                        }
                    }
                }

                action = MissingValuesDialogAction::Apply {
                    col_index: state.col_index,
                    discrete,
                    range,
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
