use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{
    adjust_widths, format_opt, format_sig, render_data_row, render_header_row,
};
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
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases,
            active_vars: deps.len() + factors.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-means-title",
            custom_title: None,
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
        for (idx, table) in res.tables.iter().enumerate() {
            render_means_case_summary(ui, table, idx);
            ui.add_space(14.0_f32);

            render_means_report_table(ui, table, idx);
            ui.add_space(14.0_f32);

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
                render_data_row(ui, &cols, &vals, row_h, Tokens::BG_ROW_EVEN);
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
                        Tokens::BG_ROW_EVEN
                    } else {
                        Tokens::BG_ROW_ODD
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
                render_data_row(ui, &cols, &tot_vals, row_h, Tokens::BG_HEADER);
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
                        Tokens::BG_ROW_EVEN,
                    ),
                    (
                        t!("output-col-within-groups"),
                        anova.within_groups.sum_of_squares,
                        anova.within_groups.df,
                        anova.within_groups.mean_square,
                        anova.within_groups.f_stat,
                        anova.within_groups.p_value,
                        Tokens::BG_ROW_ODD,
                    ),
                    (
                        t!("output-col-total"),
                        anova.total.sum_of_squares,
                        anova.total.df,
                        anova.total.mean_square,
                        anova.total.f_stat,
                        anova.total.p_value,
                        Tokens::BG_HEADER,
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
                render_data_row(ui, &cols, &vals, row_h, Tokens::BG_ROW_EVEN);
            });
        });
}

pub fn means_to_tsv(res: &MeansProcedureResult) -> String {
    let mut lines = Vec::new();

    for table in &res.tables {
        // Summary
        lines.push(format!(
            "Case Processing Summary ({} * {})",
            table.dep_var, table.factor_var
        ));
        lines.push(
            "Pair\tIncluded N\tIncluded %\tExcluded N\tExcluded %\tTotal N\tTotal %".to_string(),
        );
        lines.push(format!(
            "{} * {}\t{}\t{:.1}%\t{}\t{:.1}%\t{}\t{:.1}%",
            table.dep_var,
            table.factor_var,
            table.summary.included_n,
            table.summary.included_percent,
            table.summary.excluded_n,
            table.summary.excluded_percent,
            table.summary.total_n,
            table.summary.total_percent
        ));
        lines.push(String::new());

        // Report
        lines.push(format!("Report ({})", table.dep_var));
        lines.push(format!(
            "{}\tMean\tN\tStd. Deviation\tMedian",
            table.factor_var
        ));
        for cell in &table.cells {
            lines.push(format!(
                "{}\t{}\t{}\t{}\t{}",
                cell.group_label,
                format_opt(cell.mean),
                cell.count,
                format_opt(cell.std_dev),
                format_opt(cell.median)
            ));
        }
        lines.push(format!(
            "Total\t{}\t{}\t{}\t{}",
            format_opt(table.total.mean),
            table.total.count,
            format_opt(table.total.std_dev),
            format_opt(table.total.median)
        ));
        lines.push(String::new());

        // ANOVA
        if let Some(ref anova) = table.anova {
            lines.push(format!(
                "ANOVA Table ({} * {})",
                table.dep_var, table.factor_var
            ));
            lines.push("Source\tSum of Squares\tdf\tMean Square\tF\tSig.".to_string());
            lines.push(format!(
                "Between Groups\t{:.3}\t{}\t{}\t{}\t{}",
                anova.between_groups.sum_of_squares,
                anova.between_groups.df,
                format_opt(anova.between_groups.mean_square),
                format_opt(anova.between_groups.f_stat),
                format_sig(anova.between_groups.p_value)
            ));
            lines.push(format!(
                "Within Groups\t{:.3}\t{}\t{}\t\t",
                anova.within_groups.sum_of_squares,
                anova.within_groups.df,
                format_opt(anova.within_groups.mean_square)
            ));
            lines.push(format!(
                "Total\t{:.3}\t{}\t\t\t",
                anova.total.sum_of_squares, anova.total.df
            ));
            lines.push(String::new());
        }

        // Association
        if let Some(ref assoc) = table.association {
            lines.push(format!(
                "Measures of Association ({} * {})",
                table.dep_var, table.factor_var
            ));
            lines.push("Pair\tEta\tEta Squared".to_string());
            lines.push(format!(
                "{} * {}\t{:.3}\t{:.3}",
                table.dep_var, table.factor_var, assoc.eta, assoc.eta_squared
            ));
            lines.push(String::new());
        }
    }

    lines.join("\n")
}
