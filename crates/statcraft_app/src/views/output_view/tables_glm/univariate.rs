use super::super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::glm::UnivariateGlmResult;
use statcraft_i18n::t;

pub fn render_spss_univariate_glm_tables(ui: &mut Ui, res: &UnivariateGlmResult) {
    ui.vertical(|ui| {
        let mut cols = vec![
            (t!("output-glm-col-source"), 160.0_f32),
            (t!("output-glm-col-type3-ss"), 110.0_f32),
            (t!("output-glm-col-df"), 50.0_f32),
            (t!("output-glm-col-ms"), 100.0_f32),
            (t!("output-glm-col-f"), 80.0_f32),
            (t!("output-glm-col-sig"), 70.0_f32),
            (t!("output-glm-col-partial-eta"), 90.0_f32),
        ];

        adjust_widths(ui, &mut cols);

        ScrollArea::horizontal()
            .id_salt("univariate_glm_table")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols, 24.0);

                    for (idx, row) in res.tests.iter().enumerate() {
                        let bg = if idx % 2 == 1 {
                            Tokens::bg_row_odd()
                        } else {
                            Tokens::bg_row_even()
                        };

                        let vals = vec![
                            (row.source.clone(), Align2::LEFT_CENTER),
                            (format!("{:.3}", row.type3_ss), Align2::RIGHT_CENTER),
                            (row.df.to_string(), Align2::RIGHT_CENTER),
                            (
                                row.mean_square
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| ".".to_string()),
                                Align2::RIGHT_CENTER,
                            ),
                            (
                                row.f_stat
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| ".".to_string()),
                                Align2::RIGHT_CENTER,
                            ),
                            (format_sig(row.p_value), Align2::RIGHT_CENTER),
                            (
                                row.partial_eta_squared
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| ".".to_string()),
                                Align2::RIGHT_CENTER,
                            ),
                        ];

                        render_data_row(ui, &cols, &vals, 20.0, bg);
                    }
                });
            });

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "a. R Squared = {:.3} (Adjusted R Squared = {:.3})",
                res.r_squared, res.adjusted_r_squared
            ))
            .small()
            .color(Tokens::text_muted()),
        );
    });
}

pub fn univariate_glm_to_tsv(res: &UnivariateGlmResult) -> String {
    let mut out = String::from(
        "Source\tType III Sum of Squares\tdf\tMean Square\tF\tSig.\tPartial Eta Squared\n",
    );
    for row in &res.tests {
        let ms_str = row
            .mean_square
            .map(|v| format!("{:.3}", v))
            .unwrap_or_default();
        let f_str = row.f_stat.map(|v| format!("{:.3}", v)).unwrap_or_default();
        let p_str = format_sig(row.p_value);
        let eta_str = row
            .partial_eta_squared
            .map(|v| format!("{:.3}", v))
            .unwrap_or_default();
        out.push_str(&format!(
            "{}\t{:.3}\t{}\t{}\t{}\t{}\t{}\n",
            row.source, row.type3_ss, row.df, ms_str, f_str, p_str, eta_str
        ));
    }
    out
}
