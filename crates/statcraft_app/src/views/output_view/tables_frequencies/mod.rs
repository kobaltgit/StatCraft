pub mod bootstrap_tables;
pub mod freq_chart_bar_pie;
pub mod freq_chart_histogram;
pub mod freq_table;
pub mod stats_summary_table;
pub mod style_report;

pub use bootstrap_tables::{
    bootstrap_freq_to_tsv, bootstrap_specs_to_tsv, bootstrap_stats_to_tsv,
    render_bootstrap_freq_table, render_bootstrap_specifications_table,
    render_bootstrap_stats_table,
};
pub use freq_chart_bar_pie::*;
pub use freq_chart_histogram::*;
pub use freq_table::render_single_frequency_table;
pub use stats_summary_table::render_stats_summary_table;
pub use style_report::*;

use egui::Ui;
use statcraft_algo::{FrequencyChartResult, FrequencyResult, MultipleVarsMode};

pub fn render_spss_frequencies_tables(ui: &mut Ui, results: &[FrequencyResult]) {
    if results.is_empty() {
        return;
    }

    let mode = results
        .first()
        .map(|r| r.multiple_vars_mode)
        .unwrap_or(MultipleVarsMode::Compare);

    let boot_opts = results
        .iter()
        .find_map(|r| r.bootstrap.as_ref().map(|b| &b.options));

    match mode {
        MultipleVarsMode::Compare => {
            ui.vertical(|ui| {
                if let Some(opts) = boot_opts {
                    render_bootstrap_specifications_table(ui, opts);
                    ui.add_space(16.0_f32);
                }

                render_stats_summary_table(ui, results);
                ui.add_space(16.0_f32);

                if boot_opts.is_some() {
                    render_bootstrap_stats_table(ui, results);
                    ui.add_space(16.0_f32);
                }

                for res in results {
                    if !res.categories.is_empty() || res.table_suppressed {
                        render_single_frequency_table(ui, res);
                        ui.add_space(16.0_f32);
                        render_bootstrap_freq_table(ui, res);
                        if res.bootstrap.is_some() {
                            ui.add_space(16.0_f32);
                        }
                    }
                }
            });
        }
        MultipleVarsMode::Variable => {
            ui.vertical(|ui| {
                if let Some(opts) = boot_opts {
                    render_bootstrap_specifications_table(ui, opts);
                    ui.add_space(16.0_f32);
                }

                for res in results {
                    render_stats_summary_table(ui, std::slice::from_ref(res));
                    ui.add_space(16.0_f32);

                    if res.bootstrap.is_some() {
                        render_bootstrap_stats_table(ui, std::slice::from_ref(res));
                        ui.add_space(16.0_f32);
                    }

                    if !res.categories.is_empty() || res.table_suppressed {
                        render_single_frequency_table(ui, res);
                        ui.add_space(16.0_f32);
                        render_bootstrap_freq_table(ui, res);
                        if res.bootstrap.is_some() {
                            ui.add_space(16.0_f32);
                        }
                    }

                    if let Some(ref chart) = res.chart {
                        render_frequency_chart(ui, chart);
                        ui.add_space(16.0_f32);
                    }
                }
            });
        }
    }
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

    let mode = results
        .first()
        .map(|r| r.multiple_vars_mode)
        .unwrap_or(MultipleVarsMode::Compare);

    let boot_opts = results
        .iter()
        .find_map(|r| r.bootstrap.as_ref().map(|b| &b.options));
    if let Some(opts) = boot_opts {
        out.push_str(&bootstrap_specs_to_tsv(opts));
        out.push('\n');
    }

    match mode {
        MultipleVarsMode::Compare => {
            let stats_tsv = stats_summary_table::stats_summary_to_tsv(results);
            out.push_str(&stats_tsv);
            out.push('\n');

            let boot_stats_tsv = bootstrap_stats_to_tsv(results);
            if !boot_stats_tsv.is_empty() {
                out.push_str(&boot_stats_tsv);
                out.push('\n');
            }

            for res in results {
                if !res.categories.is_empty() || res.table_suppressed {
                    let table_tsv = freq_table::frequency_table_to_tsv(res);
                    out.push_str(&table_tsv);
                    out.push('\n');
                    let boot_freq_tsv = bootstrap_freq_to_tsv(res);
                    if !boot_freq_tsv.is_empty() {
                        out.push_str(&boot_freq_tsv);
                        out.push('\n');
                    }
                }
            }
        }
        MultipleVarsMode::Variable => {
            for res in results {
                let stats_tsv =
                    stats_summary_table::stats_summary_to_tsv(std::slice::from_ref(res));
                out.push_str(&stats_tsv);
                out.push('\n');

                let boot_stats_tsv = bootstrap_stats_to_tsv(std::slice::from_ref(res));
                if !boot_stats_tsv.is_empty() {
                    out.push_str(&boot_stats_tsv);
                    out.push('\n');
                }

                if !res.categories.is_empty() || res.table_suppressed {
                    let table_tsv = freq_table::frequency_table_to_tsv(res);
                    out.push_str(&table_tsv);
                    out.push('\n');
                    let boot_freq_tsv = bootstrap_freq_to_tsv(res);
                    if !boot_freq_tsv.is_empty() {
                        out.push_str(&boot_freq_tsv);
                        out.push('\n');
                    }
                }

                if let Some(ref chart) = res.chart {
                    let chart_tsv = match chart {
                        FrequencyChartResult::Bar { .. } | FrequencyChartResult::Pie { .. } => {
                            bar_or_pie_chart_to_tsv(chart)
                        }
                        FrequencyChartResult::Histogram { .. } => histogram_chart_to_tsv(chart),
                    };
                    out.push_str(&chart_tsv);
                    out.push('\n');
                }
            }
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
