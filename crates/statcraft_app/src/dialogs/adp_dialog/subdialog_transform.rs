use super::AdpDialogState;
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use statcraft_i18n::t;

pub fn render_transform_subdialog(ctx: &egui::Context, state: &mut AdpDialogState) {
    if !state.show_transform {
        return;
    }

    let mut is_open = true;
    DialogShell::subdialog(t!("dialog-adp-transform-title")).show(ctx, &mut is_open, |ui| {
        GroupBox::new(t!("dialog-adp-group-rescale")).show(ui, |ui| {
            ui.radio_value(
                &mut state.rescale_method_idx,
                0,
                t!("dialog-adp-rescale-none"),
            );
            ui.radio_value(
                &mut state.rescale_method_idx,
                1,
                t!("dialog-adp-rescale-zscore"),
            );
            ui.radio_value(
                &mut state.rescale_method_idx,
                2,
                t!("dialog-adp-rescale-minmax"),
            );
        });

        ui.add_space(8.0);

        GroupBox::new(t!("dialog-adp-group-categories")).show(ui, |ui| {
            ui.checkbox(&mut state.merge_sparse, t!("dialog-adp-merge-sparse-label"));
        });

        ui.add_space(8.0);

        GroupBox::new(t!("dialog-adp-group-advanced")).show(ui, |ui| {
            ui.checkbox(
                &mut state.bin_continuous,
                t!("dialog-adp-bin-continuous-label"),
            );
            ui.checkbox(
                &mut state.create_interactions,
                t!("dialog-adp-interactions-label"),
            );
        });

        ui.add_space(8.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue | SubdialogAction::Cancel => {
                state.show_transform = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !is_open {
        state.show_transform = false;
    }
}
