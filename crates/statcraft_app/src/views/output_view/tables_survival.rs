use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::{KMResult, LifeTableResult};
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_kaplan_meier(&mut self, result: KMResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = format!(
            "KM {}\n  /STATUS = {}(1)\n  /TEST = LOGRANK.",
            result.time_variable, result.status_variable
        );

        let notes = OutputNotes {
            dataset_name: self.active_dataset_name.clone(),
            timestamp: "Session".to_string(),
            active_cases: result.overall_summary.total_n,
            active_vars: 2,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-km-title",
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
                    title_key: "output-km-table-title",
                    custom_title: None,
                    kind: OutputItemKind::KaplanMeier(Box::new(result)),
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

    pub fn add_life_tables(&mut self, result: LifeTableResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = format!(
            "SURVIVAL\n  /TABLE = {}\n  /STATUS = {}(1).",
            result.time_variable, result.status_variable
        );

        let notes = OutputNotes {
            dataset_name: self.active_dataset_name.clone(),
            timestamp: "Session".to_string(),
            active_cases: result.total_cases,
            active_vars: 2,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-lifetables-title",
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
                    title_key: "output-lifetables-table-title",
                    custom_title: None,
                    kind: OutputItemKind::LifeTables(Box::new(result)),
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

pub fn render_spss_survival_tables(ui: &mut Ui, res: &KMResult) {
    ui.vertical(|ui| {
        // 1. Case Processing Summary
        ui.strong(t!("output-km-case-processing-summary"));
        ui.add_space(4.0);
        render_km_case_summary(ui, res);

        ui.add_space(16.0);

        // 2. Means and Medians for Survival Time
        ui.strong(t!("output-km-means-medians-title"));
        ui.add_space(4.0);
        render_km_means_medians(ui, res);

        ui.add_space(16.0);

        // 3. Log-Rank Test (if available)
        if let Some(lr) = &res.log_rank_test {
            ui.strong(t!("output-km-logrank-title"));
            ui.add_space(4.0);
            render_km_logrank(ui, lr);
            ui.add_space(16.0);
        }

        // 4. Survival Table by groups
        for grp in &res.groups {
            ui.strong(format!(
                "{} ({})",
                t!("output-km-table-title"),
                grp.group_label
            ));
            ui.add_space(4.0);
            render_km_event_table(ui, grp);
            ui.add_space(12.0);
        }
    });
}

fn render_km_case_summary(ui: &mut Ui, res: &KMResult) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-common-group"), 100.0),
        (t!("output-common-total-n"), 80.0),
        (t!("output-km-events-n"), 80.0),
        (t!("output-km-censored-n"), 80.0),
        (t!("output-km-censored-pct"), 80.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("km_summary_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 20.0);
                for (idx, grp) in res.groups.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let cs = &grp.case_summary;
                    let vals = [
                        (cs.group_label.clone(), Align2::LEFT_CENTER),
                        (format!("{}", cs.total_n), Align2::RIGHT_CENTER),
                        (format!("{}", cs.events_n), Align2::RIGHT_CENTER),
                        (format!("{}", cs.censored_n), Align2::RIGHT_CENTER),
                        (format!("{:.1}%", cs.censored_percent), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, 20.0, bg);
                }
            });
        });
}

fn render_km_means_medians(ui: &mut Ui, res: &KMResult) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-common-group"), 90.0),
        (t!("output-common-mean"), 70.0),
        (t!("output-common-se"), 70.0),
        (t!("output-common-ci-lower"), 75.0),
        (t!("output-common-ci-upper"), 75.0),
        (t!("output-common-median"), 70.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("km_means_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 20.0);
                for (idx, grp) in res.groups.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let te = &grp.time_estimates;
                    let med_str = te
                        .median
                        .map(|m| format!("{:.2}", m))
                        .unwrap_or_else(|| ".".to_string());
                    let vals = [
                        (te.group_label.clone(), Align2::LEFT_CENTER),
                        (format!("{:.2}", te.mean), Align2::RIGHT_CENTER),
                        (format!("{:.2}", te.mean_se), Align2::RIGHT_CENTER),
                        (format!("{:.2}", te.mean_ci_lower), Align2::RIGHT_CENTER),
                        (format!("{:.2}", te.mean_ci_upper), Align2::RIGHT_CENTER),
                        (med_str, Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, 20.0, bg);
                }
            });
        });
}

fn render_km_logrank(ui: &mut Ui, lr: &statcraft_algo::KMLogRankTest) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-km-test-name"), 140.0),
        (t!("output-common-chi-square"), 80.0),
        (t!("output-common-df"), 50.0),
        (t!("output-common-sig"), 70.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("km_logrank_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 20.0);
                let vals = [
                    ("Log Rank (Mantel-Cox)".to_string(), Align2::LEFT_CENTER),
                    (format!("{:.3}", lr.chi_square), Align2::RIGHT_CENTER),
                    (format!("{}", lr.df), Align2::RIGHT_CENTER),
                    (format_sig(Some(lr.p_value)), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &vals, 20.0, Tokens::bg_row_even());
            });
        });
}

fn render_km_event_table(ui: &mut Ui, grp: &statcraft_algo::KMGroupResult) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-common-time"), 60.0),
        (t!("output-common-status"), 70.0),
        (t!("output-km-cumulative-survival"), 90.0),
        (t!("output-common-se"), 70.0),
        (t!("output-km-cum-events"), 70.0),
        (t!("output-km-remaining"), 70.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt(format!("km_table_{}", grp.group_label))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 20.0);
                for (idx, row) in grp.table_rows.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let vals = [
                        (format!("{:.1}", row.time), Align2::RIGHT_CENTER),
                        (row.status.clone(), Align2::LEFT_CENTER),
                        (
                            format!("{:.3}", row.cumulative_survival),
                            Align2::RIGHT_CENTER,
                        ),
                        (format!("{:.3}", row.std_error), Align2::RIGHT_CENTER),
                        (format!("{}", row.cumulative_events), Align2::RIGHT_CENTER),
                        (format!("{}", row.remaining_cases), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, 20.0, bg);
                }
            });
        });
}

pub fn render_spss_lifetables(ui: &mut Ui, res: &LifeTableResult) {
    ui.vertical(|ui| {
        ui.strong(t!("output-lifetables-table-title"));
        ui.add_space(4.0);

        let mut cols: Vec<(String, f32)> = vec![
            (t!("output-lifetables-interval"), 90.0),
            (t!("output-lifetables-entering"), 70.0),
            (t!("output-lifetables-withdrawn"), 70.0),
            (t!("output-lifetables-exposed"), 70.0),
            (t!("output-lifetables-events"), 60.0),
            (t!("output-lifetables-q"), 65.0),
            (t!("output-lifetables-cum-p"), 75.0),
            (t!("output-lifetables-hazard"), 75.0),
        ];
        adjust_widths(ui, &mut cols);

        ScrollArea::horizontal()
            .id_salt("lifetables_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols, 20.0);
                    for (idx, row) in res.rows.iter().enumerate() {
                        let bg = if idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let int_str =
                            format!("{:.0} - {:.0}", row.interval_start, row.interval_end);
                        let vals = [
                            (int_str, Align2::CENTER_CENTER),
                            (format!("{}", row.cases_entering), Align2::RIGHT_CENTER),
                            (format!("{}", row.cases_withdrawn), Align2::RIGHT_CENTER),
                            (format!("{:.1}", row.cases_exposed), Align2::RIGHT_CENTER),
                            (format!("{}", row.terminal_events), Align2::RIGHT_CENTER),
                            (
                                format!("{:.3}", row.proportion_terminating),
                                Align2::RIGHT_CENTER,
                            ),
                            (
                                format!("{:.3}", row.cumulative_surviving),
                                Align2::RIGHT_CENTER,
                            ),
                            (format!("{:.4}", row.hazard_rate), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols, &vals, 20.0, bg);
                    }
                });
            });
        ui.add_space(16.0);
    });
}

pub fn survival_to_tsv(res: &KMResult) -> String {
    let mut out = String::new();
    out.push_str("Time\tStatus\tCumulative Survival\tStd Error\tRemaining Cases\n");
    for grp in &res.groups {
        for row in &grp.table_rows {
            out.push_str(&format!(
                "{:.2}\t{}\t{:.4}\t{:.4}\t{}\n",
                row.time, row.status, row.cumulative_survival, row.std_error, row.remaining_cases
            ));
        }
    }
    out
}

pub fn lifetables_to_tsv(res: &LifeTableResult) -> String {
    let mut out = String::new();
    out.push_str("Interval\tEntering\tWithdrawn\tExposed\tEvents\tq\tCum P\tHazard\n");
    for r in &res.rows {
        out.push_str(&format!(
            "{:.1}-{:.1}\t{}\t{}\t{:.1}\t{}\t{:.4}\t{:.4}\t{:.4}\n",
            r.interval_start,
            r.interval_end,
            r.cases_entering,
            r.cases_withdrawn,
            r.cases_exposed,
            r.terminal_events,
            r.proportion_terminating,
            r.cumulative_surviving,
            r.hazard_rate
        ));
    }
    out
}
