use super::AdpDialogState;
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use egui::TextEdit;
use statcraft_i18n::t;

pub fn render_partition_subdialog(ctx: &egui::Context, state: &mut AdpDialogState) {
    if !state.show_partition {
        return;
    }

    let mut is_open = true;
    DialogShell::subdialog(t!("dialog-adp-partition-title")).show(ctx, &mut is_open, |ui| {
        GroupBox::new(t!("dialog-adp-partition-title")).show(ui, |ui| {
            ui.checkbox(
                &mut state.partition_enabled,
                t!("dialog-adp-partition-enable-label"),
            );
            ui.add_space(6.0);

            if state.partition_enabled {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-adp-partition-train-label"));
                    ui.add(TextEdit::singleline(&mut state.train_pct_text).desired_width(50.0));
                    ui.label("%");
                });
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(t!("dialog-adp-partition-test-label"));
                    ui.add(TextEdit::singleline(&mut state.test_pct_text).desired_width(50.0));
                    ui.label("%");
                });
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(t!("dialog-adp-partition-validate-label"));
                    ui.add(TextEdit::singleline(&mut state.validate_pct_text).desired_width(50.0));
                    ui.label("%");
                });
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(t!("dialog-adp-partition-seed-label"));
                    ui.add(TextEdit::singleline(&mut state.seed_text).desired_width(80.0));
                });
            }
        });

        ui.add_space(8.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue | SubdialogAction::Cancel => {
                state.show_partition = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !is_open {
        state.show_partition = false;
    }
}
