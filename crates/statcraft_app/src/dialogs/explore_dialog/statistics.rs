use super::state::ExploreDialogState;
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use statcraft_i18n::t;

pub fn render_statistics_subdialog(ctx: &egui::Context, state: &mut ExploreDialogState) {
    let mut sub_open = true;
    DialogShell::subdialog(t!("dialog-explore-statistics-title")).show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-explore-statistics")).show(ui, |ui| {
            ui.checkbox(
                &mut state.compute_descriptives,
                t!("dialog-explore-descriptives"),
            );
            ui.add_enabled_ui(state.compute_descriptives, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.label(t!("dialog-explore-confidence-interval"));
                    ui.add(egui::TextEdit::singleline(&mut state.ci_level_str).desired_width(38.0));
                    ui.label("%");
                });
            });
            ui.add_space(4.0);
            ui.checkbox(
                &mut state.compute_m_estimators,
                t!("dialog-explore-m-estimators"),
            );
            ui.checkbox(&mut state.compute_extremes, t!("dialog-explore-extremes"));
            ui.checkbox(
                &mut state.compute_percentiles,
                t!("dialog-explore-percentiles"),
            );
        });

        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue | SubdialogAction::Cancel => {
                state.show_statistics = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });
    if !sub_open {
        state.show_statistics = false;
    }
}
