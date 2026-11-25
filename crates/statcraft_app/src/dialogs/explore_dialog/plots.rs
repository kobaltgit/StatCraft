use super::state::{
    ExploreBoxplotMode, ExploreDialogState, ExploreSpreadLevelMode, ExploreSpreadPower,
};
use crate::dialogs::common::{
    DialogShell, DialogSize, GroupBox, SubdialogAction, SubdialogActionBar,
};
use egui::vec2;
use statcraft_i18n::t;

pub fn render_plots_subdialog(ctx: &egui::Context, state: &mut ExploreDialogState) {
    let mut sub_open = true;
    DialogShell::new(
        t!("dialog-explore-plots-title"),
        DialogSize::Custom(vec2(380.0, 390.0)),
    )
    .show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-explore-boxplots")).show(ui, |ui| {
            ui.radio_value(
                &mut state.boxplot_mode,
                ExploreBoxplotMode::FactorLevelsTogether,
                t!("dialog-explore-boxplots-factor-together"),
            );
            ui.radio_value(
                &mut state.boxplot_mode,
                ExploreBoxplotMode::DependentsTogether,
                t!("dialog-explore-boxplots-dependents-together"),
            );
            ui.radio_value(
                &mut state.boxplot_mode,
                ExploreBoxplotMode::None,
                t!("dialog-explore-boxplots-none"),
            );
        });

        GroupBox::new(t!("dialog-explore-descriptive-plots")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut state.plot_stem_leaf,
                    t!("dialog-explore-stem-and-leaf"),
                );
                ui.add_space(16.0);
                ui.checkbox(&mut state.plot_histogram, t!("dialog-explore-histogram"));
            });
        });

        ui.add_space(2.0);
        ui.checkbox(
            &mut state.compute_normality,
            t!("dialog-explore-normality-tests"),
        );

        GroupBox::new(t!("dialog-explore-spread-level")).show(ui, |ui| {
            ui.radio_value(
                &mut state.spread_level_mode,
                ExploreSpreadLevelMode::None,
                t!("dialog-explore-spread-none"),
            );
            ui.radio_value(
                &mut state.spread_level_mode,
                ExploreSpreadLevelMode::PowerEstimation,
                t!("dialog-explore-spread-power-est"),
            );
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut state.spread_level_mode,
                    ExploreSpreadLevelMode::Transformed,
                    t!("dialog-explore-spread-transformed"),
                );
                ui.add_enabled_ui(
                    state.spread_level_mode == ExploreSpreadLevelMode::Transformed,
                    |ui| {
                        ui.label(t!("dialog-explore-spread-power-label"));
                        egui::ComboBox::from_id_salt("explore_spread_power_combo")
                            .width(135.0)
                            .selected_text(t!(state.spread_level_power.label_key()))
                            .show_ui(ui, |ui| {
                                for power in ExploreSpreadPower::ALL {
                                    ui.selectable_value(
                                        &mut state.spread_level_power,
                                        power,
                                        t!(power.label_key()),
                                    );
                                }
                            });
                    },
                );
            });
            ui.radio_value(
                &mut state.spread_level_mode,
                ExploreSpreadLevelMode::Untransformed,
                t!("dialog-explore-spread-untransformed"),
            );
        });

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
