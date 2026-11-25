use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{
    adjust_widths, bootstrap_column_headers, format_bootstrap_cells, format_opt, format_sig,
    render_bootstrap_specifications_table, render_bootstrap_table, render_data_row,
    render_header_row,
};
pub use super::tables_means_tsv::means_to_tsv;
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::{MeansOptions, MeansProcedureResult, MeansTableResult};
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_means(
        &mut self,
        result: MeansProcedureResult,
        deps: Vec<String>,
        factors: Vec<String>,
        options: &MeansOptions,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = statcraft_syntax::commands::MeansCommand::to_sps(&deps, &factors, options);

        let active_cases = result
            .tables
            .first()
            .map(|t| t.summary.total_n)
            .unwrap_or(0);

        let notes = OutputNotes {
            dataset_name: self.active_dataset_name.clone(),
            timestamp: "Session".to_string(),
            active_cases,
            active_vars: deps.len() + factors.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-means-title",
            custom_title: None,
            dataset_name: Some(self.active_dataset_name.clone()),
            is_expanded: true,
            items: vec![
                OutputItem {
                    id: log_id,
                    title_key: "output-log",
                    custom_title: None,
                    kind: OutputItemKind::Log(sps_log),
                },
                OutputItem {
                    id: notes_id,
                    title_key: "output-notes",
                    custom_title: None,
                    kind: OutputItemKind::Notes(notes),
                },
                OutputItem {
                    id: item_id,
                    title_key: "output-means-report-title",
                    custom_title: None,
                    kind: OutputItemKind::Means(Box::new(result)),
                },
            ],
        };

        self.procedures.push(proc);
        let new_proc_idx = self.procedures.len() - 1;
        self.selected_target = Some(SelectedTarget::Item {
            proc_idx: new_proc_idx,
            item_idx: 2,
        });
        self.scroll_to_target = self.selected_target;
    }
}

pub fn render_spss_means(ui: &mut Ui, res: &MeansProcedureResult) {
    if res.tables.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        if let Some(opts) = res
            .tables
            .iter()
            .find_map(|t| t.bootstrap.as_ref().map(|b| &b.options))
        {
            render_bootstrap_specifications_table(ui, opts);
            ui.add_space(14.0_f32);
        }

        for (idx, table) in res.tables.iter().enumerate() {
            render_means_case_summary(ui, table, idx);
            ui.add_space(14.0_f32);

            render_means_report_table(ui, table, idx);
            ui.add_space(14.0_f32);

            if let Some((headers, rows)) = build_means_bootstrap_rows(table) {
                let title = format!(
                    "{} ({} - {})",
                    t!("output-means-report-title"),
                    table.dep_var,
                    t!("output-bootstrap-group-header")
                );
                render_bootstrap_table(
                    ui,
                    &title,
                    &format!("means_boot_{}", idx),
                    &headers,
                    &rows,
                    2,
                );
                ui.add_space(14.0_f32);
            }

            if let Some(ref anova) = table.anova {
                render_means_anova_table(ui, table, anova, idx);
                ui.add_space(14.0_f32);
            }

            if let Some(ref assoc) = table.association {
                render_means_measures_of_association(ui, table, assoc, idx);
                ui.add_space(16.0_f32);
            }
        }
    });
}

fn render_means_case_summary(ui: &mut Ui, table: &MeansTableResult, idx: usize) {
    ui.strong(t!("output-means-case-summary-title"));
    ui.add_space(4.0_f32);

    let mut cols: Vec<(String, f32)> = vec![
        (format!("{} * {}", table.dep_var, table.factor_var), 180.0),
        (t!("output-col-included-n"), 85.0),
        (t!("output-col-included-pct"), 85.0),
        (t!("output-col-excluded-n"), 85.0),
        (t!("output-col-excluded-pct"), 85.0),
        (t!("output-col-total-n"), 85.0),
        (t!("output-col-total-pct"), 85.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_h = 22.0_f32;
    ScrollArea::horizontal()
        .id_salt(format!("means_case_summary_scroll_{}", idx))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_h);

                let vals = [
                    (
                        format!("{} * {}", table.dep_var, table.factor_var),
                        Align2::LEFT_CENTER,
                    ),
                    (table.summary.included_n.to_string(), Align2::RIGHT_CENTER),
                    (
                        format!("{:.1}%", table.summary.included_percent),
                        Align2::RIGHT_CENTER,
                    ),
                    (table.summary.excluded_n.to_string(), Align2::RIGHT_CENTER),
                    (
                        format!("{:.1}%", table.summary.excluded_percent),
                        Align2::RIGHT_CENTER,
                    ),
                    (table.summary.total_n.to_string(), Align2::RIGHT_CENTER),
                    (
                        format!("{:.1}%", table.summary.total_percent),
                        Align2::RIGHT_CENTER,
                    ),
                ];
                render_data_row(ui, &cols, &vals, row_h, Tokens::bg_row_even());
            });
        });
}

fn render_means_report_table(ui: &mut Ui, table: &MeansTableResult, idx: usize) {
    ui.strong(format!(
        "{} ({})",
        t!("output-means-report-title"),
        table.dep_var
    ));
    ui.add_space(4.0_f32);

    let show_variance =
        table.cells.iter().any(|c| c.variance.is_some()) || table.total.variance.is_some();
    let show_min_max = table
        .cells
        .iter()
        .any(|c| c.min.is_some() || c.max.is_some());

    let mut cols: Vec<(String, f32)> = vec![
        (table.factor_var.clone(), 120.0),
        (t!("output-col-mean"), 85.0),
        (t!("output-col-n"), 60.0),
        (t!("output-col-std"), 85.0),
        (t!("output-col-median"), 85.0),
    ];
    if show_variance {
        cols.push((t!("output-col-variance"), 85.0));
    }
    if show_min_max {
        cols.push((t!("output-col-min"), 75.0));
        cols.push((t!("output-col-max"), 75.0));
    }
    adjust_widths(ui, &mut cols);

    let row_h = 22.0_f32;
    ScrollArea::horizontal()
        .id_salt(format!("means_report_scroll_{}", idx))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_h);

                for (r_idx, cell) in table.cells.iter().enumerate() {
                    let bg = if r_idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let mut vals = vec![
                        (cell.group_label.clone(), Align2::LEFT_CENTER),
                        (format_opt(cell.mean), Align2::RIGHT_CENTER),
                        (cell.count.to_string(), Align2::RIGHT_CENTER),
                        (format_opt(cell.std_dev), Align2::RIGHT_CENTER),
                        (format_opt(cell.median), Align2::RIGHT_CENTER),
                    ];
                    if show_variance {
                        vals.push((format_opt(cell.variance), Align2::RIGHT_CENTER));
                    }
                    if show_min_max {
                        vals.push((format_opt(cell.min), Align2::RIGHT_CENTER));
                        vals.push((format_opt(cell.max), Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols, &vals, row_h, bg);
                }

                // Total row
                let mut tot_vals = vec![
                    (t!("output-col-total"), Align2::LEFT_CENTER),
                    (format_opt(table.total.mean), Align2::RIGHT_CENTER),
                    (table.total.count.to_string(), Align2::RIGHT_CENTER),
                    (format_opt(table.total.std_dev), Align2::RIGHT_CENTER),
                    (format_opt(table.total.median), Align2::RIGHT_CENTER),
                ];
                if show_variance {
                    tot_vals.push((format_opt(table.total.variance), Align2::RIGHT_CENTER));
                }
                if show_min_max {
                    tot_vals.push((format_opt(table.total.min), Align2::RIGHT_CENTER));
                    tot_vals.push((format_opt(table.total.max), Align2::RIGHT_CENTER));
                }
                render_data_row(ui, &cols, &tot_vals, row_h, Tokens::bg_header());
            });
        });
}

fn render_means_anova_table(
    ui: &mut Ui,
    table: &MeansTableResult,
    anova: &statcraft_algo::MeansAnovaTable,
    idx: usize,
) {
    ui.strong(format!(
        "{} ({} * {})",
        t!("output-anova-title"),
        table.dep_var,
        table.factor_var
    ));
    ui.add_space(4.0_f32);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-variable"), 160.0),
        (t!("output-col-sum-of-squares"), 110.0),
        (t!("output-col-df"), 55.0),
        (t!("output-col-mean-square"), 100.0),
        (t!("output-col-f"), 85.0),
        (t!("output-col-sig"), 75.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_h = 22.0_f32;
    ScrollArea::horizontal()
        .id_salt(format!("means_anova_scroll_{}", idx))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_h);

                let rows = [
                    (
                        t!("output-col-between-groups"),
                        anova.between_groups.sum_of_squares,
                        anova.between_groups.df,
                        anova.between_groups.mean_square,
                        anova.between_groups.f_stat,
                        anova.between_groups.p_value,
                        Tokens::bg_row_even(),
                    ),
                    (
                        t!("output-col-within-groups"),
                        anova.within_groups.sum_of_squares,
                        anova.within_groups.df,
                        anova.within_groups.mean_square,
                        anova.within_groups.f_stat,
                        anova.within_groups.p_value,
                        Tokens::bg_row_odd(),
                    ),
                    (
                        t!("output-col-total"),
                        anova.total.sum_of_squares,
                        anova.total.df,
                        anova.total.mean_square,
                        anova.total.f_stat,
                        anova.total.p_value,
                        Tokens::bg_header(),
                    ),
                ];

                for (source, ss, df, ms, f, sig, bg) in rows {
                    let vals = [
                        (source, Align2::LEFT_CENTER),
                        (format!("{:.3}", ss), Align2::RIGHT_CENTER),
                        (df.to_string(), Align2::RIGHT_CENTER),
                        (format_opt(ms), Align2::RIGHT_CENTER),
                        (format_opt(f), Align2::RIGHT_CENTER),
                        (format_sig(sig), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, row_h, bg);
                }
            });
        });
}

fn render_means_measures_of_association(
    ui: &mut Ui,
    table: &MeansTableResult,
    assoc: &statcraft_algo::MeansAssociationMeasures,
    idx: usize,
) {
    ui.strong(format!(
        "{} ({} * {})",
        t!("output-means-association-title"),
        table.dep_var,
        table.factor_var
    ));
    ui.add_space(4.0_f32);

    let mut cols: Vec<(String, f32)> = vec![
        (format!("{} * {}", table.dep_var, table.factor_var), 180.0),
        ("Eta".to_string(), 85.0),
        ("Eta Squared".to_string(), 95.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_h = 22.0_f32;
    ScrollArea::horizontal()
        .id_salt(format!("means_assoc_scroll_{}", idx))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_h);

                let vals = [
                    (
                        format!("{} * {}", table.dep_var, table.factor_var),
                        Align2::LEFT_CENTER,
                    ),
                    (format!("{:.3}", assoc.eta), Align2::RIGHT_CENTER),
                    (format!("{:.3}", assoc.eta_squared), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &vals, row_h, Tokens::bg_row_even());
            });
        });
}

pub(super) fn build_means_bootstrap_rows(
    table: &MeansTableResult,
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let boot = table.bootstrap.as_ref()?;
    let mut rows = Vec::new();
    let groups_iter = boot.groups.iter().chain(std::iter::once(&boot.total));

    for grp in groups_iter {
        let candidates = [
            (t!("output-col-mean"), &grp.mean),
            (t!("output-col-std"), &grp.std_dev),
            (t!("output-stats-row-median"), &grp.median),
        ];
        for (stat_name, sum_opt) in candidates {
            if let Some(s) = sum_opt {
                let cells = format_bootstrap_cells(s, 4);
                rows.push(vec![
                    grp.group_label.clone(),
                    stat_name,
                    format!("{:.4}", s.estimate),
                    cells[0].clone(),
                    cells[1].clone(),
                    cells[2].clone(),
                    cells[3].clone(),
                ]);
            }
        }
    }

    if rows.is_empty() {
        return None;
    }

    let boot_cols = bootstrap_column_headers(boot.options.ci_level);
    let headers = vec![
        table.factor_var.clone(),
        t!("output-bootstrap-col-statistic"),
        t!("output-km-col-estimate"),
        boot_cols[0].clone(),
        boot_cols[1].clone(),
        boot_cols[2].clone(),
        boot_cols[3].clone(),
    ];
    Some((headers, rows))
}
