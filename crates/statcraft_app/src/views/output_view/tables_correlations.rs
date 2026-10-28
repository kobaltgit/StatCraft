use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{
    adjust_widths, format_opt, format_sig, render_data_row, render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::{
    BivariateCorrelationsResult, CorrelationCell, CorrelationMatrixResult, CorrelationType,
    SignificanceFlag, SignificanceTest, VarDescriptives,
};
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_correlations(&mut self, result: BivariateCorrelationsResult, vars: &[String]) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log =
            statcraft_syntax::commands::CorrelationsCommand::to_sps(vars, &result.options);

        let active_cases = result
            .matrices
            .first()
            .and_then(|m| m.cells.first())
            .and_then(|row| row.first())
            .map(|c| c.n)
            .unwrap_or(0);

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases,
            active_vars: vars.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-correlations-title",
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
                    title_key: "output-correlations-table-title",
                    custom_title: None,
                    kind: OutputItemKind::Correlations(result),
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

pub fn render_spss_correlations_tables(ui: &mut Ui, res: &BivariateCorrelationsResult) {
    ui.vertical(|ui| {
        // 1. Descriptives Table (if present)
        if let Some(ref desc_list) = res.descriptives {
            render_descriptives_table(ui, desc_list);
            ui.add_space(14.0_f32);
        }

        // 2. Correlation Matrices
        for (m_idx, matrix) in res.matrices.iter().enumerate() {
            render_single_matrix(ui, m_idx, matrix, &res.options);
            ui.add_space(16.0_f32);
        }
    });
}

fn render_descriptives_table(ui: &mut Ui, desc_list: &[VarDescriptives]) {
    ui.strong(t!("output-correlations-descriptives-title"));
    ui.add_space(4.0_f32);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-variable"), 120.0_f32),
        (t!("output-col-mean"), 90.0_f32),
        (t!("output-col-std"), 90.0_f32),
        (t!("output-col-n"), 70.0_f32),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0_f32;

    ScrollArea::horizontal()
        .id_salt("corr_descriptives_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);

                for (idx, item) in desc_list.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::BG_ROW_EVEN
                    } else {
                        Tokens::BG_ROW_ODD
                    };
                    let vals = [
                        (item.var_name.clone(), Align2::LEFT_CENTER),
                        (format_opt(item.mean), Align2::RIGHT_CENTER),
                        (format_opt(item.std_dev), Align2::RIGHT_CENTER),
                        (format!("{}", item.n), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
            });
        });
}

fn render_single_matrix(
    ui: &mut Ui,
    m_idx: usize,
    mat: &CorrelationMatrixResult,
    opts: &statcraft_algo::BivariateCorrelationsOptions,
) {
    let title = match mat.corr_type {
        CorrelationType::Pearson => t!("output-correlations-table-title"),
        CorrelationType::KendallTauB | CorrelationType::Spearman => {
            t!("output-correlations-nonpar-title")
        }
    };
    ui.strong(title);
    ui.add_space(4.0_f32);

    let mut cols: Vec<(String, f32)> = Vec::with_capacity(mat.variables.len() + 2);
    cols.push(("".to_string(), 100.0_f32));
    cols.push(("".to_string(), 170.0_f32));
    for var in &mat.variables {
        cols.push((var.clone(), 95.0_f32));
    }
    adjust_widths(ui, &mut cols);

    let row_height = 20.0_f32;

    ScrollArea::horizontal()
        .id_salt(format!("corr_matrix_scroll_{}", m_idx))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);

                for (i, row_var) in mat.variables.iter().enumerate() {
                    let is_even_block = i % 2 == 0;
                    let bg = if is_even_block {
                        Tokens::BG_ROW_EVEN
                    } else {
                        Tokens::BG_ROW_ODD
                    };

                    // Sub-row 1: Correlation coefficient
                    let coeff_name = match mat.corr_type {
                        CorrelationType::Pearson => t!("output-correlations-pearson-r"),
                        CorrelationType::KendallTauB => t!("output-correlations-kendall-tau"),
                        CorrelationType::Spearman => t!("output-correlations-spearman-rho"),
                    };
                    let mut r_vals = vec![
                        (row_var.clone(), Align2::LEFT_CENTER),
                        (coeff_name, Align2::LEFT_CENTER),
                    ];
                    for j in 0..mat.variables.len() {
                        let cell = &mat.cells[i][j];
                        let val_str = format_cell_r(cell);
                        r_vals.push((val_str, Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols, &r_vals, row_height, bg);

                    // Sub-row 2: Significance
                    let sig_label = match opts.significance {
                        SignificanceTest::TwoTailed => t!("output-correlations-sig-2tailed"),
                        SignificanceTest::OneTailed => t!("output-correlations-sig-1tailed"),
                    };
                    let mut s_vals = vec![
                        ("".to_string(), Align2::LEFT_CENTER),
                        (sig_label, Align2::LEFT_CENTER),
                    ];
                    for j in 0..mat.variables.len() {
                        let cell = &mat.cells[i][j];
                        let val_str = if i == j {
                            "".to_string()
                        } else {
                            format_sig(cell.sig)
                        };
                        s_vals.push((val_str, Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols, &s_vals, row_height, bg);

                    // Optional Sub-rows: Cross-product deviations & Covariance (Pearson only)
                    if mat.corr_type == CorrelationType::Pearson && opts.cross_prod_cov {
                        let mut cp_vals = vec![
                            ("".to_string(), Align2::LEFT_CENTER),
                            (t!("output-correlations-ss-cross"), Align2::LEFT_CENTER),
                        ];
                        for j in 0..mat.variables.len() {
                            let cell = &mat.cells[i][j];
                            let val_str = cell
                                .ss_cross_prod
                                .map(|v| format!("{:.3}", v))
                                .unwrap_or_else(|| "-".to_string());
                            cp_vals.push((val_str, Align2::RIGHT_CENTER));
                        }
                        render_data_row(ui, &cols, &cp_vals, row_height, bg);

                        let mut cov_vals = vec![
                            ("".to_string(), Align2::LEFT_CENTER),
                            (t!("output-correlations-covariance"), Align2::LEFT_CENTER),
                        ];
                        for j in 0..mat.variables.len() {
                            let cell = &mat.cells[i][j];
                            let val_str = cell
                                .covariance
                                .map(|v| format!("{:.3}", v))
                                .unwrap_or_else(|| "-".to_string());
                            cov_vals.push((val_str, Align2::RIGHT_CENTER));
                        }
                        render_data_row(ui, &cols, &cov_vals, row_height, bg);
                    }

                    // Sub-row: N
                    let mut n_vals = vec![
                        ("".to_string(), Align2::LEFT_CENTER),
                        (t!("output-correlations-n"), Align2::LEFT_CENTER),
                    ];
                    for j in 0..mat.variables.len() {
                        let cell = &mat.cells[i][j];
                        n_vals.push((format!("{}", cell.n), Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols, &n_vals, row_height, bg);
                }
            });
        });

    // Footnotes for significance
    if opts.flag_significant {
        ui.add_space(3.0_f32);
        let note_05 = match opts.significance {
            SignificanceTest::TwoTailed => t!("output-correlations-footnote-sig05-2t"),
            SignificanceTest::OneTailed => t!("output-correlations-footnote-sig05-1t"),
        };
        let note_01 = match opts.significance {
            SignificanceTest::TwoTailed => t!("output-correlations-footnote-sig01-2t"),
            SignificanceTest::OneTailed => t!("output-correlations-footnote-sig01-1t"),
        };
        ui.label(note_05);
        ui.label(note_01);
    }
}

fn format_cell_r(cell: &CorrelationCell) -> String {
    match cell.r {
        Some(v) => {
            let base = format!("{:.3}", v);
            match cell.flag {
                SignificanceFlag::DoubleStar => format!("{}**", base),
                SignificanceFlag::SingleStar => format!("{}*", base),
                SignificanceFlag::None => base,
            }
        }
        None => "-".to_string(),
    }
}

pub fn correlations_to_tsv(res: &BivariateCorrelationsResult) -> String {
    let mut out = String::new();

    // 1. Descriptives
    if let Some(ref desc_list) = res.descriptives {
        out.push_str("Descriptive Statistics\n");
        out.push_str("Variable\tMean\tStd. Deviation\tN\n");
        for d in desc_list {
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                d.var_name,
                format_opt(d.mean),
                format_opt(d.std_dev),
                d.n
            ));
        }
        out.push('\n');
    }

    // 2. Correlation Matrices
    for mat in &res.matrices {
        let name = match mat.corr_type {
            CorrelationType::Pearson => "Correlations (Pearson)",
            CorrelationType::KendallTauB => "Correlations (Kendall's tau-b)",
            CorrelationType::Spearman => "Correlations (Spearman's rho)",
        };
        out.push_str(name);
        out.push('\n');

        out.push('\t');
        for v in &mat.variables {
            out.push('\t');
            out.push_str(v);
        }
        out.push('\n');

        for (i, v_row) in mat.variables.iter().enumerate() {
            // Correlation
            out.push_str(&format!("{}\tCorrelation", v_row));
            for j in 0..mat.variables.len() {
                out.push('\t');
                out.push_str(&format_cell_r(&mat.cells[i][j]));
            }
            out.push('\n');

            // Sig
            out.push_str("\tSig.");
            for j in 0..mat.variables.len() {
                out.push('\t');
                let sig_str = if i == j {
                    "".to_string()
                } else {
                    format_sig(mat.cells[i][j].sig)
                };
                out.push_str(&sig_str);
            }
            out.push('\n');

            // N
            out.push_str("\tN");
            for j in 0..mat.variables.len() {
                out.push('\t');
                out.push_str(&format!("{}", mat.cells[i][j].n));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    out
}
