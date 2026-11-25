use super::state::CrosstabsDialogState;
use crate::dialogs::common::{
    DialogShell, DialogSize, GroupBox, SubdialogAction, SubdialogActionBar,
};
use egui::vec2;
use statcraft_i18n::t;

pub fn render_stats_subdialog(ctx: &egui::Context, state: &mut CrosstabsDialogState) {
    let mut sub_open = true;
    DialogShell::new(
        t!("dialog-crosstabs-stats-title"),
        DialogSize::Custom(vec2(450.0, 380.0)),
    )
    .show(ctx, &mut sub_open, |ui| {
        ui.horizontal(|ui| {
            ui.checkbox(&mut state.options.chisq, t!("dialog-crosstabs-stat-chisq"));
            ui.add_space(24.0);
            ui.checkbox(
                &mut state.options.correlations,
                t!("dialog-crosstabs-stat-correlations"),
            );
        });

        ui.add_space(6.0);

        ui.columns(2, |cols| {
            // Left Column: Nominal + Nominal by Interval
            GroupBox::new(t!("dialog-crosstabs-stats-nominal")).show(&mut cols[0], |ui| {
                ui.checkbox(
                    &mut state.options.contingency_coef,
                    t!("dialog-crosstabs-stat-contingency"),
                );
                ui.checkbox(
                    &mut state.options.phi_cramer,
                    t!("dialog-crosstabs-stat-phi-cramer"),
                );
                ui.checkbox(
                    &mut state.options.lambda,
                    t!("dialog-crosstabs-stat-lambda"),
                );
                ui.checkbox(
                    &mut state.options.uncertainty_coef,
                    t!("dialog-crosstabs-stat-uncertainty"),
                );
            });

            cols[0].add_space(6.0);

            GroupBox::new(t!("dialog-crosstabs-stats-nom-interval")).show(&mut cols[0], |ui| {
                ui.checkbox(&mut state.options.eta, t!("dialog-crosstabs-stat-eta"));
            });

            // Right Column: Ordinal + Kappa, Risk, McNemar
            GroupBox::new(t!("dialog-crosstabs-stats-ordinal")).show(&mut cols[1], |ui| {
                ui.checkbox(&mut state.options.gamma, t!("dialog-crosstabs-stat-gamma"));
                ui.checkbox(
                    &mut state.options.somers_d,
                    t!("dialog-crosstabs-stat-somers-d"),
                );
                ui.checkbox(
                    &mut state.options.kendall_tau_b,
                    t!("dialog-crosstabs-stat-tau-b"),
                );
                ui.checkbox(
                    &mut state.options.kendall_tau_c,
                    t!("dialog-crosstabs-stat-tau-c"),
                );
            });

            cols[1].add_space(6.0);
            cols[1].checkbox(&mut state.options.kappa, t!("dialog-crosstabs-stat-kappa"));
            cols[1].checkbox(&mut state.options.risk, t!("dialog-crosstabs-stat-risk"));
            cols[1].checkbox(
                &mut state.options.mcnemar,
                t!("dialog-crosstabs-stat-mcnemar"),
            );
        });

        ui.add_space(6.0);

        ui.checkbox(
            &mut state.options.cochran_mantel_haenszel,
            t!("dialog-crosstabs-stat-cmh"),
        );
        ui.indent("crosstabs_cmh_indent", |ui| {
            ui.add_enabled_ui(state.options.cochran_mantel_haenszel, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-crosstabs-stat-cmh-or"));
                    ui.add(egui::TextEdit::singleline(&mut state.cmh_or_input).desired_width(56.0));
                });
            });
        });

        ui.add_space(8.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                state.sync_stats_from_buffers();
                state.stats_backup = None;
                state.show_stats = false;
            }
            SubdialogAction::Cancel => {
                if let Some((prev_opts, prev_or)) = state.stats_backup.take() {
                    state.options = prev_opts;
                    state.cmh_or_input = prev_or;
                }
                state.show_stats = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !sub_open {
        if let Some((prev_opts, prev_or)) = state.stats_backup.take() {
            state.options = prev_opts;
            state.cmh_or_input = prev_or;
        }
        state.show_stats = false;
    }
}
