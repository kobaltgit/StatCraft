use super::state::DescriptivesDialogState;
use crate::dialogs::common::{
    DialogShell, DialogSize, GroupBox, SubdialogAction, SubdialogActionBar,
};
use egui::vec2;
use statcraft_algo::DescriptivesSortOrder;
use statcraft_i18n::t;

/// Renders the `Descriptives: Options` subdialog with canonical SPSS layout.
pub fn render_descriptives_options_subdialog(
    ctx: &egui::Context,
    state: &mut DescriptivesDialogState,
) {
    let mut sub_open = true;
    DialogShell::new(
        t!("dialog-desc-options-title"),
        DialogSize::Custom(vec2(440.0, 390.0)),
    )
    .show(ctx, &mut sub_open, |ui| {
        // Top row: Mean & Sum
        ui.columns(2, |cols| {
            cols[0].checkbox(&mut state.options.mean, t!("stat-mean"));
            cols[1].checkbox(&mut state.options.sum, t!("stat-sum"));
        });
        ui.add_space(6.0);

        // GroupBox: Dispersion
        GroupBox::new(t!("dialog-desc-group-dispersion")).show(ui, |ui| {
            ui.columns(2, |cols| {
                cols[0].checkbox(&mut state.options.std_dev, t!("stat-std-dev"));
                cols[0].checkbox(&mut state.options.variance, t!("stat-variance"));
                cols[0].checkbox(&mut state.options.range, t!("stat-range"));

                cols[1].checkbox(&mut state.options.min, t!("stat-min"));
                cols[1].checkbox(&mut state.options.max, t!("stat-max"));
                cols[1].checkbox(&mut state.options.se_mean, t!("stat-se-mean"));
            });
        });
        ui.add_space(6.0);

        // GroupBox: Distribution
        GroupBox::new(t!("dialog-desc-group-distribution")).show(ui, |ui| {
            ui.columns(2, |cols| {
                cols[0].checkbox(&mut state.options.kurtosis, t!("stat-kurtosis"));
                cols[1].checkbox(&mut state.options.skewness, t!("stat-skewness"));
            });
        });
        ui.add_space(6.0);

        // GroupBox: Display Order
        GroupBox::new(t!("dialog-desc-group-display-order")).show(ui, |ui| {
            ui.radio_value(
                &mut state.options.sort,
                DescriptivesSortOrder::VariableList,
                t!("dialog-desc-order-variable-list"),
            );
            ui.radio_value(
                &mut state.options.sort,
                DescriptivesSortOrder::Alphabetic,
                t!("dialog-desc-order-alphabetic"),
            );
            ui.radio_value(
                &mut state.options.sort,
                DescriptivesSortOrder::AscendingMeans,
                t!("dialog-desc-order-ascending-means"),
            );
            ui.radio_value(
                &mut state.options.sort,
                DescriptivesSortOrder::DescendingMeans,
                t!("dialog-desc-order-descending-means"),
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
