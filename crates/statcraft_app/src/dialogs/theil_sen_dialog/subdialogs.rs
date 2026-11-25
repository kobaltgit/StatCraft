use super::state::TheilSenDialogState;
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use statcraft_i18n::t;

pub fn render_options_subdialog(ctx: &egui::Context, state: &mut TheilSenDialogState) {
    let mut sub_open = true;
    DialogShell::subdialog(t!("dialog-theil-sen-options-title")).show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-theil-sen-options-group")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t!("dialog-theil-sen-ci-level"));
                ui.add(egui::TextEdit::singleline(&mut state.ci_level_str).desired_width(50.0));
                ui.label("%");
            });
            ui.add_space(6.0);
            ui.checkbox(&mut state.mann_kendall, t!("dialog-theil-sen-mann-kendall"));
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

pub fn render_plots_subdialog(ctx: &egui::Context, state: &mut TheilSenDialogState) {
    let mut sub_open = true;
    DialogShell::subdialog(t!("dialog-theil-sen-plots-title")).show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-theil-sen-plots-group")).show(ui, |ui| {
            ui.checkbox(
                &mut state.include_scatter_plot,
                t!("dialog-theil-sen-plot-scatter"),
            );
            ui.checkbox(
                &mut state.include_residual_plot,
                t!("dialog-theil-sen-plot-residual"),
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

pub fn render_save_subdialog(ctx: &egui::Context, state: &mut TheilSenDialogState) {
    let mut sub_open = true;
    DialogShell::subdialog(t!("dialog-theil-sen-save-title")).show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-theil-sen-save-group")).show(ui, |ui| {
            ui.checkbox(
                &mut state.save_predicted,
                t!("dialog-theil-sen-save-predicted"),
            );
            ui.checkbox(
                &mut state.save_residuals,
                t!("dialog-theil-sen-save-residuals"),
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
