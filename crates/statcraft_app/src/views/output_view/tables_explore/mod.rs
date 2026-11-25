pub mod plots;
pub mod plots_scatter;
pub mod report_tables;
pub mod tables_advanced;
pub mod tables_summary;
pub mod tsv;

use egui::Ui;
use plots::render_explore_plots;
use statcraft_algo::{ExploreOutputDisplay, ExploreResult};
use tables_advanced::{
    render_bootstrap_table_explore, render_levene_table, render_m_estimators_table,
    render_normality_table, render_percentiles_table,
};
use tables_summary::{
    render_case_processing_table, render_descriptives_table, render_extremes_table,
};

pub use tsv::{explore_to_report_tables, explore_to_tsv};

pub fn render_spss_explore_pivot_tables(ui: &mut Ui, res: &ExploreResult) {
    if res.variables.is_empty() {
        return;
    }

    let show_stats = res.display_mode != ExploreOutputDisplay::Plots;
    let show_plots = res.display_mode != ExploreOutputDisplay::Statistics;

    ui.vertical(|ui| {
        render_case_processing_table(ui, res);
        ui.add_space(14.0);

        if show_stats {
            let has_desc = res
                .variables
                .iter()
                .any(|v| v.groups.iter().any(|g| g.descriptives.is_some()));
            if has_desc {
                render_descriptives_table(ui, res);
                ui.add_space(14.0);
            }

            let has_mest = res
                .variables
                .iter()
                .any(|v| v.groups.iter().any(|g| g.m_estimators.is_some()));
            if has_mest {
                render_m_estimators_table(ui, res);
                ui.add_space(14.0);
            }

            let has_pct = res
                .variables
                .iter()
                .any(|v| v.groups.iter().any(|g| g.percentiles.is_some()));
            if has_pct {
                render_percentiles_table(ui, res);
                ui.add_space(14.0);
            }

            let has_ext = res
                .variables
                .iter()
                .any(|v| v.groups.iter().any(|g| g.extremes.is_some()));
            if has_ext {
                render_extremes_table(ui, res);
                ui.add_space(14.0);
            }

            if res.bootstrap.is_some() {
                render_bootstrap_table_explore(ui, res);
                ui.add_space(14.0);
            }
        }

        if show_plots {
            let has_norm = res
                .variables
                .iter()
                .any(|v| v.groups.iter().any(|g| g.normality.is_some()));
            if has_norm {
                render_normality_table(ui, res);
                ui.add_space(14.0);
            }

            let has_levene = res.variables.iter().any(|v| v.levene_test.is_some());
            if has_levene {
                render_levene_table(ui, res);
                ui.add_space(14.0);
            }

            render_explore_plots(ui, res);
        }
    });
}
