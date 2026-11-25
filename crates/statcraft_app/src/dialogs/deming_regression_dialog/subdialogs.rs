use super::state::{DemingMode, DemingRegressionDialogState};
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use statcraft_i18n::t;

pub fn render_options_subdialog(ctx: &egui::Context, state: &mut DemingRegressionDialogState) {
    let mut sub_open = true;
    DialogShell::subdialog(t!("dialog-deming-options-title")).show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-deming-options-group")).show(ui, |ui| {
            ui.radio_value(
                &mut state.mode,
                DemingMode::Orthogonal,
                t!("dialog-deming-mode-orthogonal"),
            );
            ui.radio_value(
                &mut state.mode,
                DemingMode::CustomRatio,
                t!("dialog-deming-mode-custom"),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.label(t!("dialog-deming-lambda-label"));
                let enabled = state.mode == DemingMode::CustomRatio;
                ui.add_enabled(
                    enabled,
                    egui::TextEdit::singleline(&mut state.lambda_str).desired_width(60.0),
                );
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(t!("dialog-deming-ci-level"));
                ui.add(egui::TextEdit::singleline(&mut state.ci_level_str).desired_width(50.0));
                ui.label("%");
            });
        });

        ui.add_space(12.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue | SubdialogAction::Cancel => {
                state.show_options = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });
    if !sub_open {
        state.show_options = false;
    }
}

pub fn render_plots_subdialog(ctx: &egui::Context, state: &mut DemingRegressionDialogState) {
    let mut sub_open = true;
    DialogShell::subdialog(t!("dialog-deming-plots-title")).show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-deming-plots-group")).show(ui, |ui| {
            ui.checkbox(
                &mut state.include_scatter_plot,
                t!("dialog-deming-plot-scatter"),
            );
            ui.checkbox(
                &mut state.include_identity_line,
                t!("dialog-deming-plot-identity"),
            );
            ui.checkbox(
                &mut state.include_residual_plot,
                t!("dialog-deming-plot-residual"),
            );
        });

        ui.add_space(12.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue | SubdialogAction::Cancel => {
                state.show_plots = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });
    if !sub_open {
        state.show_plots = false;
    }
}

pub fn render_save_subdialog(ctx: &egui::Context, state: &mut DemingRegressionDialogState) {
    let mut sub_open = true;
    DialogShell::subdialog(t!("dialog-deming-save-title")).show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-deming-save-group")).show(ui, |ui| {
            ui.checkbox(
                &mut state.save_predicted,
                t!("dialog-deming-save-predicted"),
            );
            ui.checkbox(
                &mut state.save_residuals,
                t!("dialog-deming-save-residuals"),
            );
        });

        ui.add_space(12.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue | SubdialogAction::Cancel => {
                state.show_save = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });
    if !sub_open {
        state.show_save = false;
    }
}
