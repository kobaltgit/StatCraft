use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{
    adjust_widths, format_opt, format_sig, render_data_row, render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, FontId, Ui};
use statcraft_algo::{CrosstabOptions, CrosstabResult};
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_crosstabs(&mut self, results: Vec<CrosstabResult>, options: CrosstabOptions) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let rows: Vec<String> = results.iter().map(|r| r.row_var.clone()).collect();
        let cols: Vec<String> = results.iter().map(|r| r.col_var.clone()).collect();
        let sps_log = statcraft_syntax::commands::CrosstabsCommand::to_sps(&rows, &cols, &options);

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: results
                .first()
                .map(|r| r.n_valid + r.n_missing)
                .unwrap_or(0),
            active_vars: results.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-crosstabs-title",
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
                    title_key: "output-crosstabs-title",
                    custom_title: None,
                    kind: OutputItemKind::Crosstabs(results),
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

pub fn render_crosstabs_item(ui: &mut Ui, results: &[CrosstabResult], selected: bool) {
    let row_height = 20.0_f32;

    for (res_idx, res) in results.iter().enumerate() {
        if res_idx > 0 {
            ui.add_space(16.0);
        }

        // 1. Case Processing Summary
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(t!("output-crosstabs-summary-title"))
                    .font(FontId::proportional(12.5))
                    .strong()
                    .color(if selected {
                        Tokens::ACCENT
                    } else {
                        Tokens::TEXT_PRIMARY
                    }),
            );
        });
        ui.add_space(2.0);

        let mut summary_cols = vec![
            (format!("{} * {}", res.row_var, res.col_var), 180.0_f32),
            (format!("{} N", t!("output-crosstabs-valid")), 80.0_f32),
            (format!("{} %", t!("output-crosstabs-valid")), 80.0_f32),
            (format!("{} N", t!("output-crosstabs-missing")), 80.0_f32),
            (format!("{} %", t!("output-crosstabs-missing")), 80.0_f32),
            (format!("{} N", t!("output-crosstabs-total")), 80.0_f32),
            (format!("{} %", t!("output-crosstabs-total")), 80.0_f32),
        ];
        adjust_widths(ui, &mut summary_cols);
        render_header_row(ui, &summary_cols, row_height);

        let n_tot = res.n_valid + res.n_missing;
        let v_pct = if n_tot > 0 {
            (res.n_valid as f64 / n_tot as f64) * 100.0
        } else {
            0.0
        };
        let m_pct = if n_tot > 0 {
            (res.n_missing as f64 / n_tot as f64) * 100.0
        } else {
            0.0
        };

        let summary_vals = vec![
            (
                format!("{} * {}", res.row_var, res.col_var),
                Align2::LEFT_CENTER,
            ),
            (res.n_valid.to_string(), Align2::RIGHT_CENTER),
            (format!("{:.1}%", v_pct), Align2::RIGHT_CENTER),
            (res.n_missing.to_string(), Align2::RIGHT_CENTER),
            (format!("{:.1}%", m_pct), Align2::RIGHT_CENTER),
            (n_tot.to_string(), Align2::RIGHT_CENTER),
            ("100.0%".to_string(), Align2::RIGHT_CENTER),
        ];
        render_data_row(
            ui,
            &summary_cols,
            &summary_vals,
            row_height,
            Tokens::BG_ROW_EVEN,
        );

        ui.add_space(12.0);

        // 2. Crosstabulation Table
        let title = t!(
            "output-crosstabs-table-title",
            row = res.row_var.as_str(),
            col = res.col_var.as_str()
        );
        ui.label(
            egui::RichText::new(title)
                .font(FontId::proportional(12.5))
                .strong()
                .color(if selected {
                    Tokens::ACCENT
                } else {
                    Tokens::TEXT_PRIMARY
                }),
        );
        ui.add_space(2.0);

        let mut crosstab_cols = vec![
            (res.row_var.clone(), 120.0_f32),
            ("".to_string(), 140.0_f32),
        ];
        for col_cat in &res.col_categories {
            let col_hdr = col_cat
                .label
                .as_ref()
                .cloned()
                .unwrap_or_else(|| col_cat.value.clone());
            crosstab_cols.push((col_hdr, 90.0_f32));
        }
        crosstab_cols.push((t!("output-crosstabs-total"), 90.0_f32));
        adjust_widths(ui, &mut crosstab_cols);
        render_header_row(ui, &crosstab_cols, row_height);

        let mut row_flip = false;
        for row in &res.rows {
            let row_name = row
                .category
                .label
                .as_ref()
                .cloned()
                .unwrap_or_else(|| row.category.value.clone());

            // Layer 1: Count
            let bg = if row_flip {
                Tokens::BG_ROW_ODD
            } else {
                Tokens::BG_ROW_EVEN
            };
            let mut c_vals = vec![
                (row_name.clone(), Align2::LEFT_CENTER),
                (t!("output-crosstabs-count"), Align2::LEFT_CENTER),
            ];
            for cell in &row.cells {
                c_vals.push((cell.observed.to_string(), Align2::RIGHT_CENTER));
            }
            c_vals.push((row.row_total.to_string(), Align2::RIGHT_CENTER));
            render_data_row(ui, &crosstab_cols, &c_vals, row_height, bg);

            // Layer 2: Expected Count
            if res.options.expected {
                let mut e_vals = vec![
                    ("".to_string(), Align2::LEFT_CENTER),
                    (t!("output-crosstabs-expected-count"), Align2::LEFT_CENTER),
                ];
                for cell in &row.cells {
                    e_vals.push((format_opt(cell.expected), Align2::RIGHT_CENTER));
                }
                e_vals.push((format!("{:.1}", row.row_total as f64), Align2::RIGHT_CENTER));
                render_data_row(ui, &crosstab_cols, &e_vals, row_height, bg);
            }

            // Layer 3: % within Row
            if res.options.row_pct {
                let mut r_vals = vec![
                    ("".to_string(), Align2::LEFT_CENTER),
                    (
                        t!(
                            "output-crosstabs-pct-within-row",
                            var = res.row_var.as_str()
                        ),
                        Align2::LEFT_CENTER,
                    ),
                ];
                for cell in &row.cells {
                    r_vals.push((
                        format!("{:.1}%", cell.row_pct.unwrap_or(0.0)),
                        Align2::RIGHT_CENTER,
                    ));
                }
                r_vals.push(("100.0%".to_string(), Align2::RIGHT_CENTER));
                render_data_row(ui, &crosstab_cols, &r_vals, row_height, bg);
            }

            // Layer 4: % within Col
            if res.options.col_pct {
                let mut col_vals = vec![
                    ("".to_string(), Align2::LEFT_CENTER),
                    (
                        t!(
                            "output-crosstabs-pct-within-col",
                            var = res.col_var.as_str()
                        ),
                        Align2::LEFT_CENTER,
                    ),
                ];
                for cell in &row.cells {
                    col_vals.push((
                        format!("{:.1}%", cell.col_pct.unwrap_or(0.0)),
                        Align2::RIGHT_CENTER,
                    ));
                }
                let tot_pct = if res.grand_total > 0 {
                    (row.row_total as f64 / res.grand_total as f64) * 100.0
                } else {
                    0.0
                };
                col_vals.push((format!("{:.1}%", tot_pct), Align2::RIGHT_CENTER));
                render_data_row(ui, &crosstab_cols, &col_vals, row_height, bg);
            }

            // Layer 5: % of Total
            if res.options.total_pct {
                let mut tot_vals = vec![
                    ("".to_string(), Align2::LEFT_CENTER),
                    (t!("output-crosstabs-pct-of-total"), Align2::LEFT_CENTER),
                ];
                for cell in &row.cells {
                    tot_vals.push((
                        format!("{:.1}%", cell.total_pct.unwrap_or(0.0)),
                        Align2::RIGHT_CENTER,
                    ));
                }
                let tot_pct = if res.grand_total > 0 {
                    (row.row_total as f64 / res.grand_total as f64) * 100.0
                } else {
                    0.0
                };
                tot_vals.push((format!("{:.1}%", tot_pct), Align2::RIGHT_CENTER));
                render_data_row(ui, &crosstab_cols, &tot_vals, row_height, bg);
            }

            // Layer 6: Residual
            if res.options.residuals {
                let mut res_vals = vec![
                    ("".to_string(), Align2::LEFT_CENTER),
                    (t!("output-crosstabs-residual"), Align2::LEFT_CENTER),
                ];
                for cell in &row.cells {
                    res_vals.push((format_opt(cell.residual), Align2::RIGHT_CENTER));
                }
                res_vals.push(("0.0".to_string(), Align2::RIGHT_CENTER));
                render_data_row(ui, &crosstab_cols, &res_vals, row_height, bg);
            }

            row_flip = !row_flip;
        }

        // Summary Total Row
        let bg = if row_flip {
            Tokens::BG_ROW_ODD
        } else {
            Tokens::BG_ROW_EVEN
        };
        let mut tot_vals = vec![
            (t!("output-crosstabs-total"), Align2::LEFT_CENTER),
            (t!("output-crosstabs-count"), Align2::LEFT_CENTER),
        ];
        for ct in &res.col_totals {
            tot_vals.push((ct.to_string(), Align2::RIGHT_CENTER));
        }
        tot_vals.push((res.grand_total.to_string(), Align2::RIGHT_CENTER));
        render_data_row(ui, &crosstab_cols, &tot_vals, row_height, bg);

        ui.add_space(12.0);

        // 3. Chi-Square Tests Table
        if !res.chi_square_tests.is_empty() {
            ui.label(
                egui::RichText::new(t!("output-crosstabs-chisq-title"))
                    .font(FontId::proportional(12.5))
                    .strong()
                    .color(if selected {
                        Tokens::ACCENT
                    } else {
                        Tokens::TEXT_PRIMARY
                    }),
            );
            ui.add_space(2.0);

            let mut chisq_cols = vec![
                (t!("output-crosstabs-chisq-title"), 180.0_f32),
                (t!("output-crosstabs-value"), 90.0_f32),
                (t!("output-crosstabs-df"), 60.0_f32),
                (t!("output-crosstabs-asymp-sig"), 150.0_f32),
                (t!("output-crosstabs-exact-sig"), 150.0_f32),
            ];
            adjust_widths(ui, &mut chisq_cols);
            render_header_row(ui, &chisq_cols, row_height);

            for (i, t_row) in res.chi_square_tests.iter().enumerate() {
                let bg_c = if i % 2 == 0 {
                    Tokens::BG_ROW_EVEN
                } else {
                    Tokens::BG_ROW_ODD
                };
                let val_str = if t_row.name == "Fisher's Exact Test" {
                    "".to_string()
                } else if t_row.name == "N of Valid Cases" {
                    format!("{:.0}", t_row.value)
                } else {
                    format!("{:.3}", t_row.value)
                };
                let df_str = t_row
                    .df
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "".to_string());
                let asymp_str = format_sig(t_row.asymp_sig);
                let exact_str = format_sig(t_row.exact_sig);

                let r_vals = vec![
                    (t_row.name.clone(), Align2::LEFT_CENTER),
                    (val_str, Align2::RIGHT_CENTER),
                    (df_str, Align2::RIGHT_CENTER),
                    (asymp_str, Align2::RIGHT_CENTER),
                    (exact_str, Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &chisq_cols, &r_vals, row_height, bg_c);
            }

            ui.add_space(12.0);
        }

        // 4. Symmetric Measures Table
        if !res.association_measures.is_empty() {
            ui.label(
                egui::RichText::new(t!("output-crosstabs-symmetric-title"))
                    .font(FontId::proportional(12.5))
                    .strong()
                    .color(if selected {
                        Tokens::ACCENT
                    } else {
                        Tokens::TEXT_PRIMARY
                    }),
            );
            ui.add_space(2.0);

            let mut sym_cols = vec![
                (t!("output-crosstabs-nominal-by-nominal"), 180.0_f32),
                (t!("output-crosstabs-value"), 90.0_f32),
                (t!("output-crosstabs-approx-sig"), 150.0_f32),
            ];
            adjust_widths(ui, &mut sym_cols);
            render_header_row(ui, &sym_cols, row_height);

            for (i, m) in res.association_measures.iter().enumerate() {
                let bg_c = if i % 2 == 0 {
                    Tokens::BG_ROW_EVEN
                } else {
                    Tokens::BG_ROW_ODD
                };
                let m_vals = vec![
                    (m.name.clone(), Align2::LEFT_CENTER),
                    (format!("{:.3}", m.value), Align2::RIGHT_CENTER),
                    (format_sig(m.approx_sig), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &sym_cols, &m_vals, row_height, bg_c);
            }
        }
    }
}

pub fn crosstabs_to_tsv(res: &CrosstabResult) -> String {
    let mut out = String::new();

    // 1. Crosstabulation Matrix
    out.push_str(&format!(
        "{} * {} Crosstabulation\n",
        res.row_var, res.col_var
    ));
    out.push_str(&res.row_var);
    out.push('\t');
    out.push_str("Layer\t");
    for col_cat in &res.col_categories {
        let col_hdr = col_cat
            .label
            .as_ref()
            .cloned()
            .unwrap_or_else(|| col_cat.value.clone());
        out.push_str(&col_hdr);
        out.push('\t');
    }
    out.push_str("Total\n");

    for row in &res.rows {
        let row_name = row
            .category
            .label
            .as_ref()
            .cloned()
            .unwrap_or_else(|| row.category.value.clone());
        // Count
        out.push_str(&format!("{}\tCount\t", row_name));
        for cell in &row.cells {
            out.push_str(&format!("{}\t", cell.observed));
        }
        out.push_str(&format!("{}\n", row.row_total));

        // Expected
        if res.options.expected {
            out.push_str("\tExpected Count\t");
            for cell in &row.cells {
                out.push_str(&format!("{}\t", format_opt(cell.expected)));
            }
            out.push_str(&format!("{:.1}\n", row.row_total as f64));
        }
    }

    // Total row
    out.push_str("Total\tCount\t");
    for c in &res.col_totals {
        out.push_str(&format!("{}\t", c));
    }
    out.push_str(&format!("{}\n\n", res.grand_total));

    // 2. Chi-Square tests
    if !res.chi_square_tests.is_empty() {
        out.push_str("Chi-Square Tests\n");
        out.push_str("Test\tValue\tdf\tAsymptotic Sig (2-sided)\tExact Sig (2-sided)\n");
        for t in &res.chi_square_tests {
            let df_str = t.df.map(|d| d.to_string()).unwrap_or_default();
            out.push_str(&format!(
                "{}\t{:.3}\t{}\t{}\t{}\n",
                t.name,
                t.value,
                df_str,
                format_sig(t.asymp_sig),
                format_sig(t.exact_sig)
            ));
        }
        out.push('\n');
    }

    // 3. Symmetric Measures
    if !res.association_measures.is_empty() {
        out.push_str("Symmetric Measures\n");
        out.push_str("Measure\tValue\tApproximate Sig\n");
        for m in &res.association_measures {
            out.push_str(&format!(
                "{}\t{:.3}\t{}\n",
                m.name,
                m.value,
                format_sig(m.approx_sig)
            ));
        }
    }

    out
}
