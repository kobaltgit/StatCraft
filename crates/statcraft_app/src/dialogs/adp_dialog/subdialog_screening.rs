use super::AdpDialogState;
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use egui::TextEdit;
use statcraft_i18n::t;

pub fn render_screening_subdialog(ctx: &egui::Context, state: &mut AdpDialogState) {
    if !state.show_screening {
        return;
    }

    let mut is_open = true;
    DialogShell::subdialog(t!("dialog-adp-screening-title")).show(ctx, &mut is_open, |ui| {
        GroupBox::new(t!("dialog-adp-group-screening")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t!("dialog-adp-max-missing-pct"));
                ui.add(TextEdit::singleline(&mut state.max_missing_pct_text).desired_width(70.0));
                ui.label("%");
            });
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(t!("dialog-adp-max-category-pct"));
                ui.add(TextEdit::singleline(&mut state.max_category_pct_text).desired_width(70.0));
                ui.label("%");
            });
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(t!("dialog-adp-max-unique-pct"));
                ui.add(TextEdit::singleline(&mut state.max_unique_pct_text).desired_width(70.0));
                ui.label("%");
            });
        });

        ui.add_space(8.0);

        GroupBox::new(t!("dialog-adp-group-outliers")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut state.outlier_method_idx,
                    0,
                    t!("dialog-adp-outliers-none"),
                );
                ui.radio_value(
                    &mut state.outlier_method_idx,
                    1,
                    t!("dialog-adp-outliers-zscore"),
                );
                ui.radio_value(
                    &mut state.outlier_method_idx,
                    2,
                    t!("dialog-adp-outliers-iqr"),
                );
            });

            if state.outlier_method_idx > 0 {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let label = if state.outlier_method_idx == 1 {
                        t!("dialog-adp-outliers-threshold")
                    } else {
                        t!("dialog-adp-outliers-multiplier")
                    };
                    ui.label(label);
                    ui.add(
                        TextEdit::singleline(&mut state.outlier_threshold_text).desired_width(70.0),
                    );
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-adp-outliers-action"));
                    ui.radio_value(
                        &mut state.outlier_action_idx,
                        0,
                        t!("dialog-adp-outliers-winsorize"),
                    );
                    ui.radio_value(
                        &mut state.outlier_action_idx,
                        1,
                        t!("dialog-adp-outliers-sysmis"),
                    );
                });
            }
        });

        ui.add_space(8.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue | SubdialogAction::Cancel => {
                state.show_screening = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !is_open {
        state.show_screening = false;
    }
}
