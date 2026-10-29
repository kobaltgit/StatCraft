use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, format_opt, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::{ReliabilityOptions, ReliabilityResult};
use statcraft_i18n::t;

fn format_opt_3dec(val: Option<f64>) -> String {
    val.map(|v| format!("{:.3}", v))
        .unwrap_or_else(|| "-".to_string())
}

impl OutputViewerState {
    pub fn add_reliability(&mut self, result: ReliabilityResult, var_names: Vec<String>) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let options = ReliabilityOptions {
            scale_name: result.scale_name.clone(),
            model: result.model,
            item_statistics: !result.item_stats.is_empty(),
            scale_statistics: result.scale_stats.is_some(),
            scale_if_item_deleted: !result.item_total_stats.is_empty(),
        };

        let sps_log = statcraft_syntax::commands::ReliabilityCommand::to_sps(
            &var_names,
            &result.scale_name,
            result.model,
            &options,
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.case_processing.n_valid,
            active_vars: var_names.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-reliability-title",
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
                    title_key: "output-reliability-table-title",
                    custom_title: None,
                    kind: OutputItemKind::Reliability(Box::new(result)),
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

pub fn render_spss_reliability(ui: &mut Ui, res: &ReliabilityResult) {
    ui.vertical(|ui| {
        // Table 1: Case Processing Summary
        render_case_processing_table(ui, res);
        ui.add_space(14.0);

        // Table 2: Reliability Statistics
        render_reliability_stats_table(ui, res);
        ui.add_space(14.0);

        // Table 3: Item Statistics (if enabled)
        if !res.item_stats.is_empty() {
            render_item_stats_table(ui, res);
            ui.add_space(14.0);
        }

        // Table 4: Item-Total Statistics (if enabled)
        if !res.item_total_stats.is_empty() {
            render_item_total_stats_table(ui, res);
            ui.add_space(14.0);
        }

        // Table 5: Scale Statistics (if enabled)
        if let Some(scale) = &res.scale_stats {
            render_scale_stats_table(ui, scale);
            ui.add_space(16.0);
        }
    });
}

fn render_case_processing_table(ui: &mut Ui, res: &ReliabilityResult) {
    ui.strong(t!("output-reliability-case-processing-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-cases"), 120.0),
        (t!("output-col-n"), 80.0),
        (t!("output-col-percent"), 80.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0;
    ScrollArea::horizontal()
        .id_salt(format!("rel_cp_scroll_{}", res.scale_name))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);

                let rows = [
                    (
                        t!("output-reliability-row-valid"),
                        res.case_processing.n_valid.to_string(),
                        format!("{:.1}", res.case_processing.percent_valid),
                        Tokens::BG_ROW_EVEN,
                    ),
                    (
                        t!("output-reliability-row-excluded"),
                        res.case_processing.n_excluded.to_string(),
                        format!("{:.1}", res.case_processing.percent_excluded),
                        Tokens::BG_ROW_ODD,
                    ),
                    (
                        t!("output-col-total"),
                        res.case_processing.n_total.to_string(),
                        format!("{:.1}", res.case_processing.percent_total),
                        Tokens::BG_ROW_EVEN,
                    ),
                ];

                for (label, n, pct, bg) in rows {
                    let vals: Vec<(String, Align2)> = vec![
                        (label, Align2::LEFT_CENTER),
                        (n, Align2::RIGHT_CENTER),
                        (pct, Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
            });
        });

    ui.add_space(4.0);
    ui.label(format!("a. {}", t!("output-reliability-listwise-note")));
}

fn render_reliability_stats_table(ui: &mut Ui, res: &ReliabilityResult) {
    ui.strong(t!("output-reliability-stats-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-reliability-col-alpha"), 130.0),
        (t!("output-reliability-col-alpha-std"), 210.0),
        (t!("output-reliability-col-n-items"), 90.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0;
    ScrollArea::horizontal()
        .id_salt(format!("rel_stats_scroll_{}", res.scale_name))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                let vals: Vec<(String, Align2)> = vec![
                    (
                        format_opt_3dec(res.reliability_stats.cronbach_alpha),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        format_opt_3dec(res.reliability_stats.cronbach_alpha_standardized),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        res.reliability_stats.n_of_items.to_string(),
                        Align2::CENTER_CENTER,
                    ),
                ];
                render_data_row(ui, &cols, &vals, row_height, Tokens::BG_ROW_EVEN);
            });
        });
}

fn render_item_stats_table(ui: &mut Ui, res: &ReliabilityResult) {
    ui.strong(t!("output-reliability-item-stats-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-item"), 160.0),
        (t!("output-col-mean"), 85.0),
        (t!("output-col-std-dev"), 95.0),
        (t!("output-col-n"), 70.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0;
    ScrollArea::horizontal()
        .id_salt(format!("rel_items_scroll_{}", res.scale_name))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                for (idx, item) in res.item_stats.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::BG_ROW_EVEN
                    } else {
                        Tokens::BG_ROW_ODD
                    };
                    let vals: Vec<(String, Align2)> = vec![
                        (item.item_name.clone(), Align2::LEFT_CENTER),
                        (format!("{:.4}", item.mean), Align2::RIGHT_CENTER),
                        (format!("{:.4}", item.std_dev), Align2::RIGHT_CENTER),
                        (item.n.to_string(), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
            });
        });
}

fn render_item_total_stats_table(ui: &mut Ui, res: &ReliabilityResult) {
    ui.strong(t!("output-reliability-item-total-stats-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-item"), 140.0),
        (t!("output-reliability-col-mean-if-del"), 170.0),
        (t!("output-reliability-col-var-if-del"), 180.0),
        (t!("output-reliability-col-corr-r"), 190.0),
        (t!("output-reliability-col-alpha-if-del"), 180.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0;
    ScrollArea::horizontal()
        .id_salt(format!("rel_total_scroll_{}", res.scale_name))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                for (idx, it) in res.item_total_stats.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::BG_ROW_EVEN
                    } else {
                        Tokens::BG_ROW_ODD
                    };
                    let vals: Vec<(String, Align2)> = vec![
                        (it.item_name.clone(), Align2::LEFT_CENTER),
                        (
                            format!("{:.4}", it.scale_mean_if_deleted),
                            Align2::RIGHT_CENTER,
                        ),
                        (
                            format!("{:.4}", it.scale_variance_if_deleted),
                            Align2::RIGHT_CENTER,
                        ),
                        (
                            format_opt(it.corrected_item_total_correlation),
                            Align2::RIGHT_CENTER,
                        ),
                        (
                            format_opt_3dec(it.cronbach_alpha_if_deleted),
                            Align2::RIGHT_CENTER,
                        ),
                    ];
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
            });
        });
}

fn render_scale_stats_table(ui: &mut Ui, s: &statcraft_algo::ScaleStatistics) {
    ui.strong(t!("output-reliability-scale-stats-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-mean"), 90.0),
        (t!("output-col-variance"), 95.0),
        (t!("output-col-std-dev"), 95.0),
        (t!("output-reliability-col-n-items"), 90.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0;
    ScrollArea::horizontal()
        .id_salt("rel_scale_stats_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                let vals: Vec<(String, Align2)> = vec![
                    (format!("{:.4}", s.mean), Align2::RIGHT_CENTER),
                    (format!("{:.4}", s.variance), Align2::RIGHT_CENTER),
                    (format!("{:.4}", s.std_dev), Align2::RIGHT_CENTER),
                    (s.n_of_items.to_string(), Align2::CENTER_CENTER),
                ];
                render_data_row(ui, &cols, &vals, row_height, Tokens::BG_ROW_EVEN);
            });
        });
}

pub fn reliability_to_tsv(res: &ReliabilityResult) -> String {
    export_spss_reliability_tsv(res)
}

pub fn export_spss_reliability_tsv(res: &ReliabilityResult) -> String {
    let mut out = String::new();

    // 1. Case Processing Summary
    out.push_str(&format!(
        "{}\n",
        t!("output-reliability-case-processing-title")
    ));
    out.push_str(&format!(
        "{}\t{}\t{}\n",
        t!("output-col-cases"),
        t!("output-col-n"),
        t!("output-col-percent")
    ));
    out.push_str(&format!(
        "{}\t{}\t{:.1}\n",
        t!("output-reliability-row-valid"),
        res.case_processing.n_valid,
        res.case_processing.percent_valid
    ));
    out.push_str(&format!(
        "{}\t{}\t{:.1}\n",
        t!("output-reliability-row-excluded"),
        res.case_processing.n_excluded,
        res.case_processing.percent_excluded
    ));
    out.push_str(&format!(
        "{}\t{}\t{:.1}\n\n",
        t!("output-col-total"),
        res.case_processing.n_total,
        res.case_processing.percent_total
    ));

    // 2. Reliability Statistics
    out.push_str(&format!("{}\n", t!("output-reliability-stats-title")));
    out.push_str(&format!(
        "{}\t{}\t{}\n",
        t!("output-reliability-col-alpha"),
        t!("output-reliability-col-alpha-std"),
        t!("output-reliability-col-n-items")
    ));
    out.push_str(&format!(
        "{}\t{}\t{}\n\n",
        format_opt_3dec(res.reliability_stats.cronbach_alpha),
        format_opt_3dec(res.reliability_stats.cronbach_alpha_standardized),
        res.reliability_stats.n_of_items
    ));

    // 3. Item Statistics
    if !res.item_stats.is_empty() {
        out.push_str(&format!("{}\n", t!("output-reliability-item-stats-title")));
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            t!("output-col-item"),
            t!("output-col-mean"),
            t!("output-col-std-dev"),
            t!("output-col-n")
        ));
        for item in &res.item_stats {
            out.push_str(&format!(
                "{}\t{:.4}\t{:.4}\t{}\n",
                item.item_name, item.mean, item.std_dev, item.n
            ));
        }
        out.push('\n');
    }

    // 4. Item-Total Statistics
    if !res.item_total_stats.is_empty() {
        out.push_str(&format!(
            "{}\n",
            t!("output-reliability-item-total-stats-title")
        ));
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            t!("output-col-item"),
            t!("output-reliability-col-mean-if-del"),
            t!("output-reliability-col-var-if-del"),
            t!("output-reliability-col-corr-r"),
            t!("output-reliability-col-alpha-if-del")
        ));
        for it in &res.item_total_stats {
            out.push_str(&format!(
                "{}\t{:.4}\t{:.4}\t{}\t{}\n",
                it.item_name,
                it.scale_mean_if_deleted,
                it.scale_variance_if_deleted,
                format_opt(it.corrected_item_total_correlation),
                format_opt_3dec(it.cronbach_alpha_if_deleted)
            ));
        }
        out.push('\n');
    }

    // 5. Scale Statistics
    if let Some(s) = &res.scale_stats {
        out.push_str(&format!("{}\n", t!("output-reliability-scale-stats-title")));
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            t!("output-col-mean"),
            t!("output-col-variance"),
            t!("output-col-std-dev"),
            t!("output-reliability-col-n-items")
        ));
        out.push_str(&format!(
            "{:.4}\t{:.4}\t{:.4}\t{}\n",
            s.mean, s.variance, s.std_dev, s.n_of_items
        ));
    }

    out
}
