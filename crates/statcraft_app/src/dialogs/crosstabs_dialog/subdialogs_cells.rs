use super::state::CrosstabsDialogState;
use crate::dialogs::common::{
    DialogShell, DialogSize, GroupBox, SubdialogAction, SubdialogActionBar,
};
use egui::vec2;
use statcraft_algo::NonIntegerWeights;
use statcraft_i18n::t;

pub fn render_cells_subdialog(ctx: &egui::Context, state: &mut CrosstabsDialogState) {
    let mut sub_open = true;
    DialogShell::new(
        t!("dialog-crosstabs-cells-title"),
        DialogSize::Custom(vec2(500.0, 420.0)),
    )
    .show(ctx, &mut sub_open, |ui| {
        ui.columns(2, |cols| {
            // Left Column: Counts, Percentages, Residuals, APA
            GroupBox::new(t!("dialog-crosstabs-group-counts")).show(&mut cols[0], |ui| {
                ui.checkbox(
                    &mut state.options.observed,
                    t!("dialog-crosstabs-count-observed"),
                );
                ui.checkbox(
                    &mut state.options.expected,
                    t!("dialog-crosstabs-count-expected"),
                );
                ui.checkbox(
                    &mut state.options.hide_small_counts,
                    t!("dialog-crosstabs-count-hide-small"),
                );
                ui.indent("crosstabs_hide_small_indent", |ui| {
                    ui.add_enabled_ui(state.options.hide_small_counts, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(t!("dialog-crosstabs-count-less-than"));
                            ui.add(
                                egui::TextEdit::singleline(&mut state.hide_small_input)
                                    .desired_width(48.0),
                            );
                        });
                    });
                });
            });

            cols[0].add_space(6.0);

            GroupBox::new(t!("dialog-crosstabs-group-percentages")).show(&mut cols[0], |ui| {
                ui.checkbox(&mut state.options.row_pct, t!("dialog-crosstabs-pct-row"));
                ui.checkbox(
                    &mut state.options.col_pct,
                    t!("dialog-crosstabs-pct-column"),
                );
                ui.checkbox(
                    &mut state.options.total_pct,
                    t!("dialog-crosstabs-pct-total"),
                );
            });

            cols[0].add_space(6.0);

            GroupBox::new(t!("dialog-crosstabs-group-residuals")).show(&mut cols[0], |ui| {
                ui.checkbox(
                    &mut state.options.resid_unstandardized,
                    t!("dialog-crosstabs-resid-unstd"),
                );
                ui.checkbox(
                    &mut state.options.resid_standardized,
                    t!("dialog-crosstabs-resid-std"),
                );
                ui.checkbox(
                    &mut state.options.resid_adjusted,
                    t!("dialog-crosstabs-resid-adj"),
                );
            });

            cols[0].add_space(6.0);
            cols[0].checkbox(
                &mut state.options.apa_style,
                t!("dialog-crosstabs-apa-style"),
            );

            // Right Column: z-test + Noninteger Weights
            GroupBox::new(t!("dialog-crosstabs-group-ztest")).show(&mut cols[1], |ui| {
                ui.checkbox(
                    &mut state.options.compare_col_proportions,
                    t!("dialog-crosstabs-ztest-compare"),
                );
                ui.indent("crosstabs_ztest_indent", |ui| {
                    ui.add_enabled(
                        state.options.compare_col_proportions,
                        egui::Checkbox::new(
                            &mut state.options.bonferroni_adjust,
                            t!("dialog-crosstabs-ztest-bonferroni"),
                        ),
                    );
                });
            });

            cols[1].add_space(6.0);

            GroupBox::new(t!("dialog-crosstabs-group-weights")).show(&mut cols[1], |ui| {
                ui.radio_value(
                    &mut state.options.noninteger_weights,
                    NonIntegerWeights::RoundCell,
                    t!("dialog-crosstabs-weights-round-cell"),
                );
                ui.radio_value(
                    &mut state.options.noninteger_weights,
                    NonIntegerWeights::TruncateCell,
                    t!("dialog-crosstabs-weights-trunc-cell"),
                );
                ui.radio_value(
                    &mut state.options.noninteger_weights,
                    NonIntegerWeights::RoundCase,
                    t!("dialog-crosstabs-weights-round-case"),
                );
                ui.radio_value(
                    &mut state.options.noninteger_weights,
                    NonIntegerWeights::TruncateCase,
                    t!("dialog-crosstabs-weights-trunc-case"),
                );
                ui.radio_value(
                    &mut state.options.noninteger_weights,
                    NonIntegerWeights::NoAdjust,
                    t!("dialog-crosstabs-weights-no-adjust"),
                );
            });
        });

        ui.add_space(8.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                state.sync_cells_from_buffers();
                state.options.residuals = state.options.resid_unstandardized
                    || state.options.resid_standardized
                    || state.options.resid_adjusted;
                state.cells_backup = None;
                state.show_cells = false;
            }
            SubdialogAction::Cancel => {
                if let Some((prev_opts, prev_hide)) = state.cells_backup.take() {
                    state.options = prev_opts;
                    state.hide_small_input = prev_hide;
                }
                state.show_cells = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !sub_open {
        if let Some((prev_opts, prev_hide)) = state.cells_backup.take() {
            state.options = prev_opts;
            state.hide_small_input = prev_hide;
        }
        state.show_cells = false;
    }
}
