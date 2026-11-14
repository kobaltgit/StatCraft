use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::{CodebookResult, SummarizeResult};
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_codebook(&mut self, result: CodebookResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = "CODEBOOK ALL\n  /OPTIONS = MAXCATS(200).".to_string();

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.total_cases,
            active_vars: result.total_variables,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-codebook-title",
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
                    title_key: "output-codebook-table-title",
                    custom_title: None,
                    kind: OutputItemKind::Codebook(Box::new(result)),
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

    pub fn add_summarize(&mut self, result: SummarizeResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = format!(
            "SUMMARIZE\n  /TABLES = {}\n  /FORMAT = VALIDLIST TOTAL.",
            result.variable_names.join(" ")
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.total_cases,
            active_vars: result.variable_names.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-summarize-title",
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
                    title_key: "output-summarize-table-title",
                    custom_title: None,
                    kind: OutputItemKind::Summarize(Box::new(result)),
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

pub fn render_spss_codebook_tables(ui: &mut Ui, res: &CodebookResult) {
    ui.vertical(|ui| {
        for card in &res.cards {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(&card.name);
                    if let Some(lbl) = &card.label {
                        ui.label(format!("— {}", lbl));
                    }
                });
                ui.add_space(4.0);

                ui.label(format!(
                    "{}: {} | {}: {}",
                    t!("output-codebook-position"),
                    card.position,
                    t!("output-codebook-measure"),
                    card.measure
                ));
                ui.label(format!(
                    "{}: {}",
                    t!("output-codebook-type"),
                    card.data_type
                ));

                // Scale statistics
                if let Some(stats) = &card.scale_stats {
                    ui.add_space(6.0);
                    ui.strong(t!("output-codebook-stats-title"));
                    let mut cols: Vec<(String, f32)> = vec![
                        (t!("output-common-n"), 60.0),
                        (t!("output-common-mean"), 70.0),
                        (t!("output-common-stddev"), 70.0),
                        (t!("output-common-median"), 70.0),
                        (t!("output-common-min"), 60.0),
                        (t!("output-common-max"), 60.0),
                    ];
                    adjust_widths(ui, &mut cols);
                    render_header_row(ui, &cols, 20.0);
                    let vals = [
                        (format!("{}", stats.valid_count), Align2::RIGHT_CENTER),
                        (format!("{:.2}", stats.mean), Align2::RIGHT_CENTER),
                        (format!("{:.2}", stats.std_dev), Align2::RIGHT_CENTER),
                        (format!("{:.2}", stats.median), Align2::RIGHT_CENTER),
                        (format!("{:.2}", stats.min), Align2::RIGHT_CENTER),
                        (format!("{:.2}", stats.max), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, 20.0, Tokens::bg_row_even());
                }

                // Categorical frequencies
                if let Some(freq) = &card.frequencies {
                    ui.add_space(6.0);
                    ui.strong(t!("output-codebook-frequencies-title"));
                    let mut cols: Vec<(String, f32)> = vec![
                        (t!("output-common-value"), 80.0),
                        (t!("output-common-label"), 110.0),
                        (t!("output-common-count"), 60.0),
                        (t!("output-common-percent"), 70.0),
                    ];
                    adjust_widths(ui, &mut cols);
                    render_header_row(ui, &cols, 20.0);
                    for (idx, row) in freq.iter().enumerate() {
                        let bg = if idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let vals = [
                            (row.value.clone(), Align2::LEFT_CENTER),
                            (row.label.clone().unwrap_or_default(), Align2::LEFT_CENTER),
                            (format!("{}", row.count), Align2::RIGHT_CENTER),
                            (format!("{:.1}%", row.percent), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols, &vals, 20.0, bg);
                    }
                }
            });
            ui.add_space(8.0);
        }
    });
}

pub fn render_spss_summarize_tables(ui: &mut Ui, res: &SummarizeResult) {
    ui.vertical(|ui| {
        ui.strong(t!("output-summarize-table-title"));
        ui.add_space(4.0);

        let _p = res.variable_names.len();
        let mut cols: Vec<(String, f32)> = Vec::new();
        cols.push((t!("output-summarize-case-num"), 60.0));
        for grp in &res.group_names {
            cols.push((grp.clone(), 90.0));
        }
        for var in &res.variable_names {
            cols.push((var.clone(), 90.0));
        }
        adjust_widths(ui, &mut cols);

        let row_height = 20.0;
        ScrollArea::horizontal()
            .id_salt("summarize_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols, row_height);

                    for block in &res.groups {
                        for (c_idx, case) in block.cases.iter().enumerate() {
                            let bg = if c_idx % 2 == 0 {
                                Tokens::bg_row_even()
                            } else {
                                Tokens::bg_row_odd()
                            };
                            let mut vals: Vec<(String, Align2)> = Vec::new();
                            vals.push((format!("{}", case.case_index), Align2::RIGHT_CENTER));
                            for g_val in &case.group_keys {
                                vals.push((g_val.clone(), Align2::LEFT_CENTER));
                            }
                            for val_opt in &case.values {
                                vals.push((
                                    val_opt.clone().unwrap_or_default(),
                                    Align2::RIGHT_CENTER,
                                ));
                            }
                            render_data_row(ui, &cols, &vals, row_height, bg);
                        }

                        // Group summary row
                        let mut grp_vals: Vec<(String, Align2)> = Vec::new();
                        grp_vals.push((t!("output-common-total"), Align2::LEFT_CENTER));
                        for g_val in &block.group_keys {
                            grp_vals.push((g_val.clone(), Align2::LEFT_CENTER));
                        }
                        for stat in &block.stats {
                            let m_str = stat
                                .mean
                                .map(|m| format!("Mean={:.2}", m))
                                .unwrap_or_else(|| format!("N={}", stat.count));
                            grp_vals.push((m_str, Align2::RIGHT_CENTER));
                        }
                        render_data_row(ui, &cols, &grp_vals, row_height, Tokens::bg_row_odd());
                    }

                    // Overall total
                    let mut tot_vals: Vec<(String, Align2)> = Vec::new();
                    tot_vals.push((t!("output-summarize-lbl-total"), Align2::LEFT_CENTER));
                    for _ in &res.group_names {
                        tot_vals.push(("".to_string(), Align2::LEFT_CENTER));
                    }
                    for stat in &res.total_stats {
                        let m_str = stat
                            .mean
                            .map(|m| format!("Mean={:.2}", m))
                            .unwrap_or_else(|| format!("N={}", stat.count));
                        tot_vals.push((m_str, Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols, &tot_vals, row_height, Tokens::bg_row_odd());
                });
            });
        ui.add_space(16.0);
    });
}

pub fn codebook_to_tsv(res: &CodebookResult) -> String {
    let mut out = String::new();
    out.push_str("Variable\tPosition\tLabel\tType\tMeasure\n");
    for c in &res.cards {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            c.name,
            c.position,
            c.label.as_deref().unwrap_or(""),
            c.data_type,
            c.measure
        ));
    }
    out
}

pub fn summarize_to_tsv(res: &SummarizeResult) -> String {
    let mut out = String::new();
    out.push_str("Case");
    for g in &res.group_names {
        out.push_str(&format!("\t{}", g));
    }
    for v in &res.variable_names {
        out.push_str(&format!("\t{}", v));
    }
    out.push('\n');

    for block in &res.groups {
        for case in &block.cases {
            out.push_str(&format!("{}", case.case_index));
            for g in &case.group_keys {
                out.push_str(&format!("\t{}", g));
            }
            for v in &case.values {
                out.push_str(&format!("\t{}", v.as_deref().unwrap_or("")));
            }
            out.push('\n');
        }
    }
    out
}
