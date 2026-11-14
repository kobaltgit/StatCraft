use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::PartialCorrResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_partial_correlations(
        &mut self,
        result: PartialCorrResult,
        two_tailed: bool,
        missing: &str,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = statcraft_syntax::commands::PartialCorrCommand::to_sps(
            &result.variables,
            &result.controls,
            two_tailed,
            missing,
        );

        let active_cases = result
            .cells
            .first()
            .and_then(|row| row.first())
            .map(|c| c.n)
            .unwrap_or(0);

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases,
            active_vars: result.variables.len() + result.controls.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-partial-corr-title",
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
                    title_key: "output-partial-corr-table-title",
                    custom_title: None,
                    kind: OutputItemKind::PartialCorrelations(Box::new(result)),
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

pub fn render_spss_partial_corr_tables(ui: &mut Ui, res: &PartialCorrResult) {
    ui.vertical(|ui| {
        ui.strong(t!("output-partial-corr-table-title"));
        ui.add_space(4.0);

        let mut cols: Vec<(String, f32)> = Vec::with_capacity(res.variables.len() + 3);
        cols.push((t!("output-partial-corr-control-vars"), 130.0));
        cols.push(("".to_string(), 100.0));
        cols.push(("".to_string(), 120.0));
        for var in &res.variables {
            cols.push((var.clone(), 90.0));
        }
        adjust_widths(ui, &mut cols);

        let row_height = 20.0;
        let controls_str = if res.controls.is_empty() {
            t!("output-partial-corr-none")
        } else {
            res.controls.join(" & ")
        };

        ScrollArea::horizontal()
            .id_salt("partial_corr_matrix_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols, row_height);

                    for (i, row_var) in res.variables.iter().enumerate() {
                        let bg = if i % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };

                        // Sub-row 1: Correlation
                        let mut r_vals = vec![
                            (
                                if i == 0 {
                                    controls_str.clone()
                                } else {
                                    "".to_string()
                                },
                                Align2::LEFT_CENTER,
                            ),
                            (row_var.clone(), Align2::LEFT_CENTER),
                            (t!("output-partial-corr-correlation"), Align2::LEFT_CENTER),
                        ];
                        for (j, _) in res.variables.iter().enumerate() {
                            let val_str = if i == j {
                                "1.000".to_string()
                            } else if let Some(cell) = res.cells.get(i).and_then(|row| row.get(j)) {
                                cell.r
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "-".to_string())
                            } else {
                                "-".to_string()
                            };
                            r_vals.push((val_str, Align2::RIGHT_CENTER));
                        }
                        render_data_row(ui, &cols, &r_vals, row_height, bg);

                        // Sub-row 2: Significance
                        let mut s_vals = vec![
                            ("".to_string(), Align2::LEFT_CENTER),
                            ("".to_string(), Align2::LEFT_CENTER),
                            (t!("output-partial-corr-sig-2tailed"), Align2::LEFT_CENTER),
                        ];
                        for (j, _) in res.variables.iter().enumerate() {
                            let val_str = if i == j {
                                "".to_string()
                            } else if let Some(cell) = res.cells.get(i).and_then(|row| row.get(j)) {
                                format_sig(cell.p_val)
                            } else {
                                "".to_string()
                            };
                            s_vals.push((val_str, Align2::RIGHT_CENTER));
                        }
                        render_data_row(ui, &cols, &s_vals, row_height, bg);

                        // Sub-row 3: df
                        let mut df_vals = vec![
                            ("".to_string(), Align2::LEFT_CENTER),
                            ("".to_string(), Align2::LEFT_CENTER),
                            (t!("output-partial-corr-df"), Align2::LEFT_CENTER),
                        ];
                        for (j, _) in res.variables.iter().enumerate() {
                            let val_str = if i == j {
                                "0".to_string()
                            } else if let Some(cell) = res.cells.get(i).and_then(|row| row.get(j)) {
                                format!("{}", cell.df)
                            } else {
                                "0".to_string()
                            };
                            df_vals.push((val_str, Align2::RIGHT_CENTER));
                        }
                        render_data_row(ui, &cols, &df_vals, row_height, bg);
                    }
                });
            });

        ui.add_space(16.0);
    });
}

pub fn partial_corr_to_tsv(res: &PartialCorrResult) -> String {
    let mut out = String::new();
    out.push_str("Control Variables\tVariable\tStatistic");
    for v in &res.variables {
        out.push('\t');
        out.push_str(v);
    }
    out.push('\n');

    let controls_str = if res.controls.is_empty() {
        "--none--".to_string()
    } else {
        res.controls.join(" & ")
    };

    for (i, row_var) in res.variables.iter().enumerate() {
        let ctrl = if i == 0 { &controls_str } else { "" };

        // Sub-row 1: Correlation
        out.push_str(&format!("{ctrl}\t{row_var}\tCorrelation"));
        for (j, _) in res.variables.iter().enumerate() {
            out.push('\t');
            if i == j {
                out.push_str("1.000");
            } else if let Some(cell) = res.cells.get(i).and_then(|row| row.get(j)) {
                if let Some(r) = cell.r {
                    out.push_str(&format!("{:.3}", r));
                } else {
                    out.push('-');
                }
            } else {
                out.push('-');
            }
        }
        out.push('\n');

        // Sub-row 2: Sig.
        out.push_str("\t\tSig. (2-tailed)");
        for (j, _) in res.variables.iter().enumerate() {
            out.push('\t');
            if i == j {
                // empty
            } else if let Some(cell) = res.cells.get(i).and_then(|row| row.get(j)) {
                out.push_str(&format_sig(cell.p_val));
            }
        }
        out.push('\n');

        // Sub-row 3: df
        out.push_str("\t\tdf");
        for (j, _) in res.variables.iter().enumerate() {
            out.push('\t');
            if i == j {
                out.push('0');
            } else if let Some(cell) = res.cells.get(i).and_then(|row| row.get(j)) {
                out.push_str(&cell.df.to_string());
            } else {
                out.push('0');
            }
        }
        out.push('\n');
    }

    out
}
