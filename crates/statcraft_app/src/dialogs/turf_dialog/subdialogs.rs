use super::state::TurfDialogState;
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use statcraft_i18n::t;

pub fn render_weights_subdialog(ctx: &egui::Context, state: &mut TurfDialogState) {
    if !state.weights_open {
        return;
    }

    let mut open = true;
    DialogShell::subdialog(t!("dialog-turf-weights-title")).show(ctx, &mut open, |ui| {
        ui.label(t!("dialog-turf-weights-desc"));
        ui.add_space(6.0);

        GroupBox::new(t!("dialog-turf-weights-group-checking")).show(ui, |ui| {
            ui.radio_value(
                &mut state.weights_strict,
                true,
                t!("dialog-turf-weights-strict-all"),
            );
            ui.radio_value(
                &mut state.weights_strict,
                false,
                t!("dialog-turf-weights-strict-default1"),
            );
        });

        ui.add_space(8.0);
        ui.label(t!("dialog-turf-weights-spec-label"));
        ui.add(
            egui::TextEdit::multiline(&mut state.weights_spec_text)
                .desired_rows(4)
                .desired_width(ui.available_width()),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(t!("dialog-turf-weights-spec-hint"))
                .weak()
                .size(11.0),
        );

        ui.add_space(10.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                state.close_weights(true);
            }
            SubdialogAction::Cancel => {
                state.close_weights(false);
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !open {
        state.close_weights(false);
    }
}

pub fn render_advanced_subdialog(ctx: &egui::Context, state: &mut TurfDialogState) {
    if !state.advanced_open {
        return;
    }

    let mut open = true;
    DialogShell::subdialog(t!("dialog-turf-advanced-title")).show(ctx, &mut open, |ui| {
        ui.label(t!("dialog-turf-advanced-desc"));
        ui.add_space(6.0);

        ui.checkbox(
            &mut state.use_advanced_selection,
            t!("dialog-turf-advanced-use-checkbox"),
        );
        ui.add_space(6.0);

        ui.add_enabled_ui(state.use_advanced_selection, |ui| {
            GroupBox::new(t!("dialog-turf-advanced-group-params")).show(ui, |ui| {
                ui.radio_value(
                    &mut state.advanced_auto,
                    true,
                    t!("dialog-turf-advanced-auto"),
                );
                ui.radio_value(
                    &mut state.advanced_auto,
                    false,
                    t!("dialog-turf-advanced-specify"),
                );

                ui.add_space(6.0);
                let specify_enabled = !state.advanced_auto;

                ui.horizontal(|ui| {
                    ui.add_enabled_ui(specify_enabled, |ui| {
                        ui.label(t!("dialog-turf-advanced-size-label"));
                        ui.add(
                            egui::TextEdit::singleline(&mut state.advanced_size_text)
                                .desired_width(60.0),
                        );
                    });
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(specify_enabled, |ui| {
                        ui.label(t!("dialog-turf-advanced-winners-label"));
                        ui.add(
                            egui::TextEdit::singleline(&mut state.advanced_winners_text)
                                .desired_width(60.0),
                        );
                    });
                });
            });
        });

        ui.add_space(10.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                state.close_advanced(true);
            }
            SubdialogAction::Cancel => {
                state.close_advanced(false);
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !open {
        state.close_advanced(false);
    }
}
