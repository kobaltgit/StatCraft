use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, FontId, ScrollArea, Ui};
use statcraft_algo::ProgrammabilityResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_programmability(&mut self, result: ProgrammabilityResult, sps_log: Option<String>) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps = sps_log.unwrap_or_else(|| {
            "BEGIN PROGRAM.\n# StatCraft Programmability Script\nEND PROGRAM.\nEXECUTE.".to_string()
        });

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.rows_affected,
            active_vars: result.created_vars.len() + result.modified_vars.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-programmability-title",
            custom_title: None,
            is_expanded: true,
            items: vec![
                OutputItem {
                    id: log_id,
                    title_key: "output-log",
                    custom_title: None,
                    kind: OutputItemKind::Log(sps),
                },
                OutputItem {
                    id: notes_id,
                    title_key: "output-notes",
                    custom_title: None,
                    kind: OutputItemKind::Notes(notes),
                },
                OutputItem {
                    id: item_id,
                    title_key: "output-programmability-summary-title",
                    custom_title: None,
                    kind: OutputItemKind::Programmability(Box::new(result)),
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

pub fn render_spss_programmability_tables(ui: &mut Ui, res: &ProgrammabilityResult) {
    ScrollArea::horizontal()
        .id_salt("programmability_tables_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Table 1: Summary of Execution
                render_summary_table(ui, res);
                ui.add_space(16.0);

                // Table 2: Statement Details Log
                if !res.logs.is_empty() {
                    render_logs_table(ui, res);
                }
            });
        });
}

fn render_summary_table(ui: &mut Ui, res: &ProgrammabilityResult) {
    ui.label(
        egui::RichText::new(t!("output-programmability-summary-title"))
            .font(FontId::proportional(13.0))
            .strong()
            .color(Tokens::text_primary()),
    );
    ui.add_space(4.0);

    let row_height = 20.0_f32;
    let mut cols = vec![
        (t!("output-programmability-property"), 190.0_f32),
        (t!("output-programmability-value"), 240.0_f32),
    ];
    adjust_widths(ui, &mut cols);
    render_header_row(ui, &cols, 22.0);

    let created_str = if res.created_vars.is_empty() {
        t!("output-programmability-none")
    } else {
        res.created_vars.join(", ")
    };

    let modified_str = if res.modified_vars.is_empty() {
        t!("output-programmability-none")
    } else {
        res.modified_vars.join(", ")
    };

    let rows: Vec<(&str, String)> = vec![
        (
            "output-programmability-prop-statements",
            res.statements_count.to_string(),
        ),
        ("output-programmability-prop-created", created_str),
        ("output-programmability-prop-modified", modified_str),
        (
            "output-programmability-prop-rows",
            res.rows_affected.to_string(),
        ),
        (
            "output-programmability-prop-status",
            t!("output-programmability-status-ok"),
        ),
    ];

    for (idx, (key, val)) in rows.into_iter().enumerate() {
        let cells = vec![(t!(key), Align2::LEFT_CENTER), (val, Align2::LEFT_CENTER)];
        let bg = if idx % 2 == 0 {
            Tokens::bg_row_even()
        } else {
            Tokens::bg_row_odd()
        };
        render_data_row(ui, &cols, &cells, row_height, bg);
    }
}

fn render_logs_table(ui: &mut Ui, res: &ProgrammabilityResult) {
    ui.label(
        egui::RichText::new(t!("output-programmability-logs-title"))
            .font(FontId::proportional(13.0))
            .strong()
            .color(Tokens::text_primary()),
    );
    ui.add_space(4.0);

    let row_height = 20.0_f32;
    let mut cols = vec![
        (t!("output-programmability-log-num"), 50.0_f32),
        (t!("output-programmability-log-detail"), 380.0_f32),
    ];
    adjust_widths(ui, &mut cols);
    render_header_row(ui, &cols, 22.0);

    for (idx, log_entry) in res.logs.iter().enumerate() {
        let cells = vec![
            ((idx + 1).to_string(), Align2::RIGHT_CENTER),
            (log_entry.clone(), Align2::LEFT_CENTER),
        ];
        let bg = if idx % 2 == 0 {
            Tokens::bg_row_even()
        } else {
            Tokens::bg_row_odd()
        };
        render_data_row(ui, &cols, &cells, row_height, bg);
    }
}

pub fn programmability_to_tsv(res: &ProgrammabilityResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-programmability-property"),
        t!("output-programmability-value")
    ));
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-programmability-prop-statements"),
        res.statements_count
    ));
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-programmability-prop-created"),
        res.created_vars.join(", ")
    ));
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-programmability-prop-modified"),
        res.modified_vars.join(", ")
    ));
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-programmability-prop-rows"),
        res.rows_affected
    ));
    out
}
