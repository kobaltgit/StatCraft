use super::state::FrequenciesDialogState;
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use egui::Context;
use statcraft_algo::{CategoryOrder, MultipleVarsMode};
use statcraft_i18n::t;

pub fn render_format_subdialog(ctx: &Context, state: &mut FrequenciesDialogState) {
    if !state.show_format {
        return;
    }

    let mut is_open = true;
    DialogShell::subdialog(t!("dialog-freq-format-title")).show(ctx, &mut is_open, |ui| {
        // 1. Two-column layout for Order by and Multiple variables
        ui.columns(2, |columns| {
            // Left column: Order by
            GroupBox::new(t!("dialog-freq-format-order-group")).show(&mut columns[0], |ui| {
                ui.radio_value(
                    &mut state.format.order,
                    CategoryOrder::AValue,
                    t!("dialog-freq-format-order-asc-values"),
                );
                ui.radio_value(
                    &mut state.format.order,
                    CategoryOrder::DValue,
                    t!("dialog-freq-format-order-desc-values"),
                );
                ui.radio_value(
                    &mut state.format.order,
                    CategoryOrder::AFreq,
                    t!("dialog-freq-format-order-asc-counts"),
                );
                ui.radio_value(
                    &mut state.format.order,
                    CategoryOrder::DFreq,
                    t!("dialog-freq-format-order-desc-counts"),
                );
            });

            // Right column: Multiple variables
            GroupBox::new(t!("dialog-freq-format-multivars-group")).show(&mut columns[1], |ui| {
                ui.radio_value(
                    &mut state.format.multiple_vars_mode,
                    MultipleVarsMode::Compare,
                    t!("dialog-freq-format-multivars-compare"),
                );
                ui.radio_value(
                    &mut state.format.multiple_vars_mode,
                    MultipleVarsMode::Variable,
                    t!("dialog-freq-format-multivars-variable"),
                );
            });
        });

        ui.add_space(8.0);

        // 2. Suppress tables with many categories
        ui.checkbox(
            &mut state.format.suppress_tables,
            t!("dialog-freq-format-suppress-checkbox"),
        );
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.add_enabled_ui(state.format.suppress_tables, |ui| {
                ui.label(t!("dialog-freq-format-max-categories"));
                ui.add(
                    egui::TextEdit::singleline(&mut state.format.max_categories_input)
                        .desired_width(50.0),
                );
            });
        });

        // 3. Subdialog Action Bar [ Continue ] [ Cancel ] [ Help ]
        let bar_action = SubdialogActionBar::new(true).show(ui);
        match bar_action {
            SubdialogAction::Continue => {
                if state.format.suppress_tables {
                    let parsed = match state.format.max_categories_input.trim().parse::<usize>() {
                        Ok(n) if n >= 1 => n,
                        _ => 10,
                    };
                    state.format.max_categories_input = parsed.to_string();
                }
                state.format_backup = None;
                state.show_format = false;
            }
            SubdialogAction::Cancel => {
                if let Some(backup) = state.format_backup.take() {
                    state.format = backup;
                }
                state.show_format = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !is_open {
        if let Some(backup) = state.format_backup.take() {
            state.format = backup;
        }
        state.show_format = false;
    }
}
