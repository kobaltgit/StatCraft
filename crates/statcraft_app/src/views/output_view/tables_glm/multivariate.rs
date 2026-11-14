use super::super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::glm::{MultivariateGlmResult, MultivariateStatName};
use statcraft_i18n::t;

pub fn render_spss_multivariate_glm_tables(ui: &mut Ui, res: &MultivariateGlmResult) {
    ui.vertical(|ui| {
        let mut cols = vec![
            (t!("output-glm-col-effect"), 140.0_f32),
            (t!("output-glm-col-test-stat"), 130.0_f32),
            (t!("output-glm-col-value"), 80.0_f32),
            (t!("output-glm-col-f"), 80.0_f32),
            (t!("output-glm-col-hypo-df"), 80.0_f32),
            (t!("output-glm-col-error-df"), 80.0_f32),
            (t!("output-glm-col-sig"), 70.0_f32),
            (t!("output-glm-col-partial-eta"), 90.0_f32),
        ];

        adjust_widths(ui, &mut cols);

        ScrollArea::horizontal()
            .id_salt("multivariate_glm_table")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols, 24.0);

                    for (idx, row) in res.effects.iter().enumerate() {
                        let bg = if idx % 2 == 1 {
                            Tokens::bg_row_odd()
                        } else {
                            Tokens::bg_row_even()
                        };

                        let stat_name = match row.stat_name {
                            MultivariateStatName::PillaiTrace => "Pillai's Trace",
                            MultivariateStatName::WilksLambda => "Wilks' Lambda",
                            MultivariateStatName::HotellingTrace => "Hotelling's Trace",
                            MultivariateStatName::RoyLargestRoot => "Roy's Largest Root",
                        };

                        let vals = vec![
                            (row.effect.clone(), Align2::LEFT_CENTER),
                            (stat_name.to_string(), Align2::LEFT_CENTER),
                            (format!("{:.3}", row.value), Align2::RIGHT_CENTER),
                            (format!("{:.3}", row.f_stat), Align2::RIGHT_CENTER),
                            (format!("{:.3}", row.hypothesis_df), Align2::RIGHT_CENTER),
                            (format!("{:.3}", row.error_df), Align2::RIGHT_CENTER),
                            (format_sig(Some(row.p_value)), Align2::RIGHT_CENTER),
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
    });
}

pub fn multivariate_glm_to_tsv(res: &MultivariateGlmResult) -> String {
    let mut out = String::from(
        "Effect\tStatistic\tValue\tF\tHypothesis df\tError df\tSig.\tPartial Eta Squared\n",
    );
    for row in &res.effects {
        let stat_name = match row.stat_name {
            MultivariateStatName::PillaiTrace => "Pillai's Trace",
            MultivariateStatName::WilksLambda => "Wilks' Lambda",
            MultivariateStatName::HotellingTrace => "Hotelling's Trace",
            MultivariateStatName::RoyLargestRoot => "Roy's Largest Root",
        };
        let eta_str = row
            .partial_eta_squared
            .map(|v| format!("{:.3}", v))
            .unwrap_or_default();
        out.push_str(&format!(
            "{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{}\t{}\n",
            row.effect,
            stat_name,
            row.value,
            row.f_stat,
            row.hypothesis_df,
            row.error_df,
            format_sig(Some(row.p_value)),
            eta_str
        ));
    }
    out
}
