pub mod freq_chart_bar_pie;
pub mod freq_chart_histogram;
pub mod freq_table;
pub mod stats_summary_table;

pub use freq_chart_bar_pie::*;
pub use freq_chart_histogram::*;
pub use freq_table::render_single_frequency_table;
pub use stats_summary_table::render_stats_summary_table;

use egui::Ui;
use statcraft_algo::{FrequencyChartResult, FrequencyResult};

pub fn render_spss_frequencies_tables(ui: &mut Ui, results: &[FrequencyResult]) {
    if results.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        render_stats_summary_table(ui, results);
        ui.add_space(16.0_f32);

        for res in results {
            if !res.categories.is_empty() {
                render_single_frequency_table(ui, res);
                ui.add_space(16.0_f32);
            }
        }
    });
}

pub fn render_frequency_chart(ui: &mut Ui, chart: &FrequencyChartResult) {
    match chart {
        FrequencyChartResult::Bar { .. } | FrequencyChartResult::Pie { .. } => {
            render_bar_or_pie_chart(ui, chart);
        }
        FrequencyChartResult::Histogram { .. } => {
            render_histogram_chart(ui, chart);
        }
    }
}

pub fn render_frequency_charts(ui: &mut Ui, charts: &[FrequencyChartResult]) {
    if charts.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        for chart in charts {
            render_frequency_chart(ui, chart);
            ui.add_space(16.0_f32);
        }
    });
}

pub fn frequencies_to_tsv(results: &[FrequencyResult]) -> String {
    let mut out = String::new();
    if results.is_empty() {
        return out;
    }

    let stats_tsv = stats_summary_table::stats_summary_to_tsv(results);
    out.push_str(&stats_tsv);
    out.push('\n');

    for res in results {
        if !res.categories.is_empty() {
            let table_tsv = freq_table::frequency_table_to_tsv(res);
            out.push_str(&table_tsv);
            out.push('\n');
        }
    }

    out
}

pub fn frequency_charts_to_tsv(charts: &[FrequencyChartResult]) -> String {
    let mut out = String::new();
    for chart in charts {
        match chart {
            FrequencyChartResult::Bar { .. } | FrequencyChartResult::Pie { .. } => {
                out.push_str(&bar_or_pie_chart_to_tsv(chart));
                out.push('\n');
            }
            FrequencyChartResult::Histogram { .. } => {
                out.push_str(&histogram_chart_to_tsv(chart));
                out.push('\n');
            }
        }
    }
    out
}
