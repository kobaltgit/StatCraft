use super::state::CrosstabsDialogState;
use crate::dialogs::common::{
    DialogShell, DialogSize, GroupBox, SubdialogAction, SubdialogActionBar,
};
use egui::vec2;
use statcraft_algo::{ExactMode, RowOrder};
use statcraft_i18n::t;

pub fn render_exact_subdialog(ctx: &egui::Context, state: &mut CrosstabsDialogState) {
    let mut sub_open = true;
    DialogShell::new(
        t!("dialog-crosstabs-exact-title"),
        DialogSize::Custom(vec2(420.0, 355.0)),
    )
    .show(ctx, &mut sub_open, |ui| {
        let mut mode_idx = match state.options.exact_mode {
            ExactMode::AsymptoticOnly => 0,
            ExactMode::MonteCarlo { .. } => 1,
            ExactMode::Exact { .. } => 2,
        };

        if ui
            .radio_value(&mut mode_idx, 0, t!("dialog-crosstabs-exact-asymp"))
            .clicked()
        {
            state.options.exact_mode = ExactMode::AsymptoticOnly;
        }
        ui.add_space(4.0);

        if ui
            .radio_value(&mut mode_idx, 1, t!("dialog-crosstabs-exact-mc"))
            .clicked()
        {
            let conf = state
                .mc_confidence_input
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| (50.0..100.0).contains(v))
                .unwrap_or(99.0)
                / 100.0;
            let samples = state
                .mc_samples_input
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|&n| n > 0)
                .unwrap_or(10000);
            state.options.exact_mode = ExactMode::MonteCarlo {
                confidence: conf,
                samples,
            };
        }

        let is_mc = matches!(state.options.exact_mode, ExactMode::MonteCarlo { .. });
        ui.indent("crosstabs_mc_indent", |ui| {
            ui.add_enabled_ui(is_mc, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-crosstabs-exact-conf-level"));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.mc_confidence_input)
                            .desired_width(48.0),
                    );
                    ui.label("%");
                });
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-crosstabs-exact-samples"));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.mc_samples_input).desired_width(68.0),
                    );
                });
            });
        });
        ui.add_space(4.0);

        if ui
            .radio_value(&mut mode_idx, 2, t!("dialog-crosstabs-exact-exact"))
            .clicked()
        {
            let limit = if state.exact_time_enabled {
                Some(
                    state
                        .exact_time_input
                        .trim()
                        .parse::<u32>()
                        .ok()
                        .filter(|&m| m > 0)
                        .unwrap_or(5),
                )
            } else {
                None
            };
            state.options.exact_mode = ExactMode::Exact {
                time_limit_min: limit,
            };
        }

        let is_exact = matches!(state.options.exact_mode, ExactMode::Exact { .. });
        ui.indent("crosstabs_exact_indent", |ui| {
            ui.add_enabled_ui(is_exact, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut state.exact_time_enabled,
                        t!("dialog-crosstabs-exact-time-limit"),
                    );
                    ui.add_enabled(
                        state.exact_time_enabled,
                        egui::TextEdit::singleline(&mut state.exact_time_input).desired_width(40.0),
                    );
                    ui.label(t!("dialog-crosstabs-exact-minutes"));
                });
            });
        });

        ui.add_space(8.0);
        ui.add(
            egui::Label::new(
                egui::RichText::new(t!("dialog-crosstabs-exact-note-mc"))
                    .size(12.0)
                    .color(crate::theme::Tokens::text_primary()),
            )
            .wrap(),
        );
        ui.add_space(3.0);
        ui.add(
            egui::Label::new(
                egui::RichText::new(t!("dialog-crosstabs-exact-note-time"))
                    .size(12.0)
                    .color(crate::theme::Tokens::text_primary()),
            )
            .wrap(),
        );

        ui.add_space(8.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                state.sync_exact_from_buffers();
                state.exact_backup = None;
                state.show_exact = false;
            }
            SubdialogAction::Cancel => {
                restore_exact_backup(state);
                state.show_exact = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !sub_open {
        restore_exact_backup(state);
        state.show_exact = false;
    }
}

fn restore_exact_backup(state: &mut CrosstabsDialogState) {
    if let Some((mode, conf, samples, time_en, time_in)) = state.exact_backup.take() {
        state.options.exact_mode = mode;
        state.mc_confidence_input = conf;
        state.mc_samples_input = samples;
        state.exact_time_enabled = time_en;
        state.exact_time_input = time_in;
    }
}

pub fn render_format_subdialog(ctx: &egui::Context, state: &mut CrosstabsDialogState) {
    let mut sub_open = true;
    DialogShell::new(
        t!("dialog-crosstabs-format-title"),
        DialogSize::Custom(vec2(300.0, 200.0)),
    )
    .show(ctx, &mut sub_open, |ui| {
        GroupBox::new(t!("dialog-crosstabs-format-row-order")).show(ui, |ui| {
            ui.radio_value(
                &mut state.options.row_order,
                RowOrder::Ascending,
                t!("dialog-crosstabs-format-asc"),
            );
            ui.radio_value(
                &mut state.options.row_order,
                RowOrder::Descending,
                t!("dialog-crosstabs-format-desc"),
            );
        });

        ui.add_space(12.0);
        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                state.format_backup = None;
                state.show_format = false;
            }
            SubdialogAction::Cancel => {
                if let Some(prev) = state.format_backup.take() {
                    state.options = prev;
                }
                state.show_format = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !sub_open {
        if let Some(prev) = state.format_backup.take() {
            state.options = prev;
        }
        state.show_format = false;
    }
}
