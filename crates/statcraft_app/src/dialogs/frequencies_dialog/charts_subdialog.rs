use super::state::{FrequenciesChartType, FrequenciesChartValue, FrequenciesDialogState};
use crate::dialogs::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use egui::Context;
use statcraft_i18n::t;

pub fn render_charts_subdialog(ctx: &Context, state: &mut FrequenciesDialogState) {
    if !state.show_charts {
        return;
    }

    let mut is_open = true;
    DialogShell::subdialog(t!("dialog-freq-charts-title")).show(ctx, &mut is_open, |ui| {
        // 1. GroupBox "Chart Type"
        GroupBox::new(t!("dialog-freq-charts-type-group")).show(ui, |ui| {
            ui.radio_value(
                &mut state.charts.chart_type,
                FrequenciesChartType::None,
                t!("dialog-freq-charts-type-none"),
            );
            ui.radio_value(
                &mut state.charts.chart_type,
                FrequenciesChartType::Bar,
                t!("dialog-freq-charts-type-bar"),
            );
            ui.radio_value(
                &mut state.charts.chart_type,
                FrequenciesChartType::Pie,
                t!("dialog-freq-charts-type-pie"),
            );
            ui.radio_value(
                &mut state.charts.chart_type,
                FrequenciesChartType::Histogram,
                t!("dialog-freq-charts-type-histogram"),
            );

            // Indented normal curve checkbox: only enabled for Histogram
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let can_normal = state.charts.chart_type == FrequenciesChartType::Histogram;
                ui.add_enabled(
                    can_normal,
                    egui::Checkbox::new(
                        &mut state.charts.normal_curve,
                        t!("dialog-freq-charts-histogram-normal"),
                    ),
                );
            });
        });

        ui.add_space(8.0);

        // 2. GroupBox "Chart Values": only enabled for Bar and Pie
        let values_enabled = matches!(
            state.charts.chart_type,
            FrequenciesChartType::Bar | FrequenciesChartType::Pie
        );
        GroupBox::new(t!("dialog-freq-charts-values-group")).show(ui, |ui| {
            ui.add_enabled_ui(values_enabled, |ui| {
                ui.radio_value(
                    &mut state.charts.values,
                    FrequenciesChartValue::Frequencies,
                    t!("dialog-freq-charts-values-freq"),
                );
                ui.radio_value(
                    &mut state.charts.values,
                    FrequenciesChartValue::Percentages,
                    t!("dialog-freq-charts-values-percent"),
                );
            });
        });

        // 3. Subdialog Action Bar [ Continue ] [ Cancel ] [ Help ]
        let bar_action = SubdialogActionBar::new(true).show(ui);
        match bar_action {
            SubdialogAction::Continue => {
                state.charts_backup = None;
                state.show_charts = false;
            }
            SubdialogAction::Cancel => {
                if let Some(backup) = state.charts_backup.take() {
                    state.charts = backup;
                }
                state.show_charts = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !is_open {
        if let Some(backup) = state.charts_backup.take() {
            state.charts = backup;
        }
        state.show_charts = false;
    }
}
