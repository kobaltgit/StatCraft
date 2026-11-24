use super::state::TableStyleState;
use crate::dialogs::common::{DialogShell, DialogSize, SubdialogAction, SubdialogActionBar};
use egui::{vec2, ComboBox, Context, RichText, Ui};
use statcraft_i18n::t;
use statcraft_reporting::styling::CellCondition;

pub fn render_condition_subdialog(ctx: &Context, state: &mut TableStyleState) {
    if state.editing_condition_idx.is_none() {
        return;
    }

    let mut is_open = true;
    let shell = DialogShell::new(
        t!("dialog-table-style-condition-title"),
        DialogSize::Custom(vec2(430.0, 360.0)),
    );

    shell.show(ctx, &mut is_open, |ui| {
        render_condition_content(ui, state);
    });

    if !is_open {
        state.editing_condition_idx = None;
    }
}

fn render_condition_content(ui: &mut Ui, state: &mut TableStyleState) {
    ui.add_space(4.0);

    let target_val_str = state
        .editing_condition_idx
        .and_then(|idx| state.rules.get(idx))
        .map(|r| super::value_target_display(&r.value_target))
        .unwrap_or_else(|| t!("dialog-table-style-val-all"));

    ui.horizontal(|ui| {
        ui.label(t!("dialog-cond-context-label"));
        ui.label(RichText::new(target_val_str).strong());
    });

    ui.add_space(8.0);

    ui.radio_value(
        &mut state.cond_is_conditional,
        false,
        t!("dialog-table-style-cond-all-values"),
    );

    ui.add_space(6.0);

    ui.radio_value(
        &mut state.cond_is_conditional,
        true,
        t!("dialog-table-style-cond-based-on"),
    );

    ui.add_space(6.0);

    ui.add_enabled_ui(state.cond_is_conditional, |ui| {
        ui.horizontal(|ui| {
            let op_labels = [
                t!("dialog-table-style-cond-op-equal"),
                t!("dialog-table-style-cond-op-not-equal"),
                t!("dialog-table-style-cond-op-less-than"),
                t!("dialog-table-style-cond-op-less-equal"),
                t!("dialog-table-style-cond-op-greater-than"),
                t!("dialog-table-style-cond-op-greater-equal"),
                t!("dialog-table-style-cond-op-between"),
                t!("dialog-table-style-cond-op-not-between"),
                t!("dialog-table-style-cond-op-top-n"),
                t!("dialog-table-style-cond-op-bottom-n"),
                t!("dialog-table-style-cond-op-sysmis"),
            ];

            let selected_label = op_labels
                .get(state.cond_operator_idx)
                .map(|s| s.as_str())
                .unwrap_or("");

            ComboBox::from_id_salt("table_style_cond_op")
                .width(220.0)
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    for (idx, label) in op_labels.iter().enumerate() {
                        ui.selectable_value(&mut state.cond_operator_idx, idx, label);
                    }
                });
        });

        ui.add_space(8.0);

        let is_val1_enabled = matches!(state.cond_operator_idx, 0..=7);
        let is_val2_enabled = matches!(state.cond_operator_idx, 6 | 7);
        let is_count_enabled = matches!(state.cond_operator_idx, 8 | 9);

        // First value row + Absolute value checkbox
        ui.add_enabled_ui(is_val1_enabled, |ui| {
            ui.horizontal(|ui| {
                ui.add_sized([120.0, 20.0], egui::Label::new(t!("dialog-cond-first-val")));
                ui.add(egui::TextEdit::singleline(&mut state.cond_val1_input).desired_width(100.0));
                ui.add_space(8.0);
                ui.checkbox(&mut state.cond_abs_value, t!("dialog-cond-abs-value"));
            });
        });

        ui.add_space(4.0);

        // Second value row
        ui.add_enabled_ui(is_val2_enabled, |ui| {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [120.0, 20.0],
                    egui::Label::new(t!("dialog-cond-second-value")),
                );
                ui.add(egui::TextEdit::singleline(&mut state.cond_val2_input).desired_width(100.0));
            });
        });

        ui.add_space(4.0);

        // Count row
        ui.add_enabled_ui(is_count_enabled, |ui| {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [120.0, 20.0],
                    egui::Label::new(t!("dialog-cond-count-label")),
                );
                ui.add(egui::TextEdit::singleline(&mut state.cond_n_input).desired_width(100.0));
            });
        });
    });

    ui.add_space(12.0);

    let action = SubdialogActionBar::default().show(ui);
    match action {
        SubdialogAction::Continue => {
            let cond = if !state.cond_is_conditional {
                CellCondition::All
            } else {
                match state.cond_operator_idx {
                    0 => CellCondition::Equal(parse_f64_or_zero(&state.cond_val1_input)),
                    1 => CellCondition::NotEqual(parse_f64_or_zero(&state.cond_val1_input)),
                    2 => CellCondition::LessThan(parse_f64_or_zero(&state.cond_val1_input)),
                    3 => CellCondition::LessOrEqual(parse_f64_or_zero(&state.cond_val1_input)),
                    4 => CellCondition::GreaterThan(parse_f64_or_zero(&state.cond_val1_input)),
                    5 => CellCondition::GreaterOrEqual(parse_f64_or_zero(&state.cond_val1_input)),
                    6 => CellCondition::Between(
                        parse_f64_or_zero(&state.cond_val1_input),
                        parse_f64_or_zero(&state.cond_val2_input),
                    ),
                    7 => CellCondition::NotBetween(
                        parse_f64_or_zero(&state.cond_val1_input),
                        parse_f64_or_zero(&state.cond_val2_input),
                    ),
                    8 => {
                        let n = state
                            .cond_n_input
                            .trim()
                            .parse::<usize>()
                            .unwrap_or(10)
                            .max(1);
                        CellCondition::TopN(n)
                    }
                    9 => {
                        let n = state
                            .cond_n_input
                            .trim()
                            .parse::<usize>()
                            .unwrap_or(10)
                            .max(1);
                        CellCondition::BottomN(n)
                    }
                    _ => CellCondition::SysMis,
                }
            };

            if let Some(idx) = state.editing_condition_idx {
                if idx < state.rules.len() {
                    state.rules[idx].condition = cond;
                }
            }
            state.editing_condition_idx = None;
        }
        SubdialogAction::Cancel => {
            state.editing_condition_idx = None;
        }
        _ => {}
    }
}

fn parse_f64_or_zero(s: &str) -> f64 {
    let clean = s.trim().replace(',', ".");
    clean.parse::<f64>().unwrap_or(0.0)
}
