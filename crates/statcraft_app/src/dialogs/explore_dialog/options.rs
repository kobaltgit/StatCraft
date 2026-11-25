use super::state::{ExploreDialogState, ExploreMissingMode};
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use statcraft_i18n::t;

pub fn render_options_subdialog(ctx: &egui::Context, state: &mut ExploreDialogState) {
    let mut sub_open = true;
    DialogShell::subdialog(t!("dialog-explore-options-title")).show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-explore-missing-values")).show(ui, |ui| {
            ui.radio_value(
                &mut state.missing_mode,
                ExploreMissingMode::Listwise,
                t!("dialog-explore-exclude-listwise"),
            );
            ui.radio_value(
                &mut state.missing_mode,
                ExploreMissingMode::Pairwise,
                t!("dialog-explore-exclude-pairwise"),
            );
            ui.radio_value(
                &mut state.missing_mode,
                ExploreMissingMode::Report,
                t!("dialog-explore-report-values"),
            );
        });

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
