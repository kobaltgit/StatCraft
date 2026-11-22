pub mod freq_table;
pub mod stats_summary_table;

use egui::Ui;
pub use freq_table::render_single_frequency_table;
use statcraft_algo::FrequencyResult;
pub use stats_summary_table::render_stats_summary_table;

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
