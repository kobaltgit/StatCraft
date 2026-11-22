use super::state::FrequenciesDialogState;
use crate::dialogs::common::{
    DialogShell, DialogSize, GroupBox, SubdialogAction, SubdialogActionBar,
};
use crate::theme::Tokens;
use egui::{vec2, Frame, Margin, ScrollArea, Stroke, Ui};
use statcraft_i18n::t;

pub fn render_statistics_subdialog(ctx: &egui::Context, state: &mut FrequenciesDialogState) {
    if !state.show_statistics {
        return;
    }

    let mut sub_open = true;
    DialogShell::new(
        t!("dialog-freq-statistics-title"),
        DialogSize::Custom(vec2(380.0, 460.0)),
    )
    .show(ctx, &mut sub_open, |ui| {
        ui.horizontal(|ui| {
            // Left Column (~174 px)
            ui.allocate_ui_with_layout(
                vec2(174.0, 360.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_percentiles_group(ui, state);
                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut state.stats.group_midpoints,
                        t!("dialog-freq-group-midpoints"),
                    );
                    ui.add_space(4.0);
                    render_dispersion_group(ui, state);
                },
            );

            ui.add_space(8.0);

            // Right Column (~174 px)
            ui.allocate_ui_with_layout(
                vec2(174.0, 360.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_central_tendency_group(ui, state);
                    ui.add_space(6.0);
                    render_distribution_group(ui, state);
                },
            );
        });

        let sub_action = SubdialogActionBar::new(true).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                state.show_statistics = false;
                state.stats_backup = None;
            }
            SubdialogAction::Cancel => {
                if let Some(backup) = state.stats_backup.take() {
                    state.stats = backup;
                }
                state.show_statistics = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !sub_open {
        if let Some(backup) = state.stats_backup.take() {
            state.stats = backup;
        }
        state.show_statistics = false;
    }
}

fn render_percentiles_group(ui: &mut Ui, state: &mut FrequenciesDialogState) {
    GroupBox::new(t!("dialog-freq-group-percentiles")).show(ui, |ui| {
        ui.checkbox(&mut state.stats.quartiles, t!("stat-quartiles"));

        ui.horizontal(|ui| {
            ui.checkbox(
                &mut state.stats.cutpoints_enabled,
                t!("dialog-freq-cutpoints-prefix"),
            );
            ui.add(egui::TextEdit::singleline(&mut state.stats.cutpoints_n).desired_width(26.0));
            ui.label(t!("dialog-freq-cutpoints-suffix"));
        });

        ui.horizontal(|ui| {
            ui.label(t!("dialog-freq-percentiles"));
            ui.add(
                egui::TextEdit::singleline(&mut state.stats.percentile_input).desired_width(36.0),
            );
        });

        let parsed_val = state
            .stats
            .percentile_input
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|&v| v > 0.0 && v < 100.0);

        let can_add = parsed_val.is_some();
        let can_change = state.stats.selected_percentile_idx.is_some() && can_add;
        let can_remove = state.stats.selected_percentile_idx.is_some();

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    can_add,
                    egui::Button::new(t!("dialog-btn-add")).min_size(vec2(36.0, 18.0)),
                )
                .clicked()
            {
                if let Some(val) = parsed_val {
                    if !state
                        .stats
                        .custom_percentiles
                        .iter()
                        .any(|&x| (x - val).abs() < 1e-6)
                    {
                        state.stats.custom_percentiles.push(val);
                        state
                            .stats
                            .custom_percentiles
                            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    }
                    state.stats.percentile_input.clear();
                    state.stats.selected_percentile_idx = None;
                }
            }

            if ui
                .add_enabled(
                    can_change,
                    egui::Button::new(t!("dialog-btn-change")).min_size(vec2(44.0, 18.0)),
                )
                .clicked()
            {
                if let (Some(idx), Some(val)) = (state.stats.selected_percentile_idx, parsed_val) {
                    if idx < state.stats.custom_percentiles.len() {
                        state.stats.custom_percentiles[idx] = val;
                        state
                            .stats
                            .custom_percentiles
                            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        state.stats.percentile_input.clear();
                        state.stats.selected_percentile_idx = None;
                    }
                }
            }

            if ui
                .add_enabled(
                    can_remove,
                    egui::Button::new(t!("dialog-btn-remove")).min_size(vec2(40.0, 18.0)),
                )
                .clicked()
            {
                if let Some(idx) = state.stats.selected_percentile_idx {
                    if idx < state.stats.custom_percentiles.len() {
                        state.stats.custom_percentiles.remove(idx);
                        state.stats.selected_percentile_idx = None;
                        state.stats.percentile_input.clear();
                    }
                }
            }
        });

        // Compact list of custom percentiles
        Frame::group(ui.style())
            .fill(Tokens::bg_panel())
            .stroke(Stroke::new(1.0_f32, Tokens::border()))
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .max_height(48.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for (idx, pct) in state.stats.custom_percentiles.clone().iter().enumerate()
                        {
                            let is_selected = state.stats.selected_percentile_idx == Some(idx);
                            let text = format!("{:.1}", pct);
                            let resp = ui.selectable_label(is_selected, text);
                            if resp.clicked() {
                                state.stats.selected_percentile_idx = Some(idx);
                                state.stats.percentile_input = format!("{:.1}", pct);
                            }
                        }
                    });
            });
    });
}

fn render_dispersion_group(ui: &mut Ui, state: &mut FrequenciesDialogState) {
    GroupBox::new(t!("dialog-freq-group-dispersion")).show(ui, |ui| {
        ui.checkbox(&mut state.stats.std_dev, t!("stat-std-dev"));
        ui.checkbox(&mut state.stats.variance, t!("stat-variance"));
        ui.checkbox(&mut state.stats.range, t!("stat-range"));
        ui.checkbox(&mut state.stats.min, t!("stat-min"));
        ui.checkbox(&mut state.stats.max, t!("stat-max"));
        ui.checkbox(&mut state.stats.se_mean, t!("stat-se-mean"));
    });
}

fn render_central_tendency_group(ui: &mut Ui, state: &mut FrequenciesDialogState) {
    GroupBox::new(t!("dialog-freq-group-central")).show(ui, |ui| {
        ui.checkbox(&mut state.stats.mean, t!("stat-mean"));
        ui.checkbox(&mut state.stats.median, t!("dialog-freq-stat-median"));
        ui.checkbox(&mut state.stats.mode, t!("stat-mode"));
        ui.checkbox(&mut state.stats.sum, t!("stat-sum"));
    });
}

fn render_distribution_group(ui: &mut Ui, state: &mut FrequenciesDialogState) {
    GroupBox::new(t!("dialog-freq-group-distribution")).show(ui, |ui| {
        ui.checkbox(&mut state.stats.skewness, t!("stat-skewness"));
        ui.checkbox(&mut state.stats.kurtosis, t!("stat-kurtosis"));
    });
}
