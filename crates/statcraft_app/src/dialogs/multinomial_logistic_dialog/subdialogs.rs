use super::super::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use super::MultinomialLogisticDialogState;
use statcraft_i18n::t;

pub fn render_ref_category_subdialog(
    ctx: &egui::Context,
    state: &mut MultinomialLogisticDialogState,
) {
    let mut open = true;
    DialogShell::subdialog(t!("dialog-multinomial-ref-title")).show(ctx, &mut open, |ui| {
        ui.vertical(|ui| {
            GroupBox::new(t!("dialog-multinomial-ref-title")).show(ui, |ui| {
                ui.radio_value(
                    &mut state.reference_category,
                    "last".to_string(),
                    t!("dialog-multinomial-ref-last"),
                );
                ui.radio_value(
                    &mut state.reference_category,
                    "first".to_string(),
                    t!("dialog-multinomial-ref-first"),
                );
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut state.reference_category,
                        "custom".to_string(),
                        t!("dialog-multinomial-ref-custom"),
                    );
                    ui.add_enabled(
                        state.reference_category == "custom",
                        egui::TextEdit::singleline(&mut state.custom_ref_val).desired_width(60.0),
                    );
                });
            });

            ui.add_space(8.0);
            let bar = SubdialogActionBar::new(true).show(ui);
            if bar == SubdialogAction::Continue || bar == SubdialogAction::Cancel {
                state.ref_dialog_open = false;
            }
        });
    });

    if !open {
        state.ref_dialog_open = false;
    }
}

pub fn render_stats_subdialog(ctx: &egui::Context, state: &mut MultinomialLogisticDialogState) {
    let mut open = true;
    DialogShell::subdialog(t!("dialog-multinomial-stats-title")).show(ctx, &mut open, |ui| {
        ui.vertical(|ui| {
            GroupBox::new(t!("dialog-multinomial-stats-title")).show(ui, |ui| {
                ui.checkbox(
                    &mut state.print_classification,
                    t!("dialog-multinomial-opt-classification"),
                );
                ui.checkbox(
                    &mut state.print_goodness_of_fit,
                    t!("dialog-multinomial-opt-goodness-of-fit"),
                );
                ui.checkbox(&mut state.print_lrt, t!("dialog-multinomial-opt-lrt"));
                ui.checkbox(
                    &mut state.include_constant,
                    t!("dialog-multinomial-opt-constant"),
                );
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-multinomial-confidence-level"));
                    ui.text_edit_singleline(&mut state.ci_str);
                });
            });

            ui.add_space(8.0);
            let bar = SubdialogActionBar::new(true).show(ui);
            if bar == SubdialogAction::Continue || bar == SubdialogAction::Cancel {
                state.stats_dialog_open = false;
            }
        });
    });

    if !open {
        state.stats_dialog_open = false;
    }
}
