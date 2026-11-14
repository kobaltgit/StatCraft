use super::super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::glm::RepeatedGlmResult;
use statcraft_i18n::t;

pub fn render_spss_repeated_glm_tables(ui: &mut Ui, res: &RepeatedGlmResult) {
    ui.vertical(|ui| {
        // 1. Mauchly's Sphericity Table
        ui.strong(t!("output-glm-mauchly-title"));
        ui.add_space(2.0);

        let mut sph_cols = vec![
            (t!("output-glm-col-effect"), 120.0_f32),
            (t!("output-glm-col-mauchly-w"), 90.0_f32),
            (t!("output-glm-col-chi-square"), 100.0_f32),
            (t!("output-glm-col-df"), 50.0_f32),
            (t!("output-glm-col-sig"), 70.0_f32),
            (t!("output-glm-col-greenhouse"), 90.0_f32),
            (t!("output-glm-col-huynh"), 90.0_f32),
            (t!("output-glm-col-lower-bound"), 90.0_f32),
        ];
        adjust_widths(ui, &mut sph_cols);

        ScrollArea::horizontal()
            .id_salt("repeated_sphericity_table")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &sph_cols, 24.0);

                    let sph = &res.sphericity;
                    let vals = vec![
                        (sph.within_subject_effect.clone(), Align2::LEFT_CENTER),
                        (format!("{:.3}", sph.mauchly_w), Align2::RIGHT_CENTER),
                        (
                            format!("{:.3}", sph.approx_chi_square),
                            Align2::RIGHT_CENTER,
                        ),
                        (sph.df.to_string(), Align2::RIGHT_CENTER),
                        (format_sig(Some(sph.p_value)), Align2::RIGHT_CENTER),
                        (
                            format!("{:.3}", sph.greenhouse_geisser),
                            Align2::RIGHT_CENTER,
                        ),
                        (format!("{:.3}", sph.huynh_feldt), Align2::RIGHT_CENTER),
                        (format!("{:.3}", sph.lower_bound), Align2::RIGHT_CENTER),
                    ];

                    render_data_row(ui, &sph_cols, &vals, 20.0, Tokens::bg_row_even());
                });
            });

        ui.add_space(8.0);

        // 2. Tests of Within-Subjects Effects Table
        ui.strong(t!("output-glm-within-subjects-title"));
        ui.add_space(2.0);

        let mut w_cols = vec![
            (t!("output-glm-col-source"), 120.0_f32),
            (t!("output-glm-col-measure"), 130.0_f32),
            (t!("output-glm-col-type3-ss"), 100.0_f32),
            (t!("output-glm-col-df"), 60.0_f32),
            (t!("output-glm-col-ms"), 100.0_f32),
            (t!("output-glm-col-f"), 80.0_f32),
            (t!("output-glm-col-sig"), 70.0_f32),
        ];
        adjust_widths(ui, &mut w_cols);

        let all_rows = res
            .within_subjects_tests
            .iter()
            .chain(std::iter::once(&res.error_row));

        ScrollArea::horizontal()
            .id_salt("repeated_within_table")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &w_cols, 24.0);

                    for (row_idx, row) in all_rows.enumerate() {
                        let sub_types = [
                            (
                                "Sphericity Assumed",
                                row.sphericity_assumed_ss,
                                row.sphericity_assumed_df,
                                row.sphericity_assumed_ms,
                                row.sphericity_assumed_f,
                                row.sphericity_assumed_p,
                            ),
                            (
                                "Greenhouse-Geisser",
                                row.greenhouse_geisser_ss,
                                row.greenhouse_geisser_df,
                                row.greenhouse_geisser_ms,
                                row.greenhouse_geisser_f,
                                row.greenhouse_geisser_p,
                            ),
                            (
                                "Huynh-Feldt",
                                row.huynh_feldt_ss,
                                row.huynh_feldt_df,
                                row.huynh_feldt_ms,
                                row.huynh_feldt_f,
                                row.huynh_feldt_p,
                            ),
                            (
                                "Lower-bound",
                                row.lower_bound_ss,
                                row.lower_bound_df,
                                row.lower_bound_ms,
                                row.lower_bound_f,
                                row.lower_bound_p,
                            ),
                        ];

                        for (s_idx, (st_name, ss, df, ms, f_val, p_val)) in
                            sub_types.into_iter().enumerate()
                        {
                            let bg = if (row_idx * 4 + s_idx) % 2 == 1 {
                                Tokens::bg_row_odd()
                            } else {
                                Tokens::bg_row_even()
                            };

                            let src_col = if s_idx == 0 {
                                row.source.clone()
                            } else {
                                "".to_string()
                            };

                            let vals = vec![
                                (src_col, Align2::LEFT_CENTER),
                                (st_name.to_string(), Align2::LEFT_CENTER),
                                (format!("{:.3}", ss), Align2::RIGHT_CENTER),
                                (format!("{:.3}", df), Align2::RIGHT_CENTER),
                                (format!("{:.3}", ms), Align2::RIGHT_CENTER),
                                (
                                    f_val
                                        .map(|v| format!("{:.3}", v))
                                        .unwrap_or_else(|| ".".to_string()),
                                    Align2::RIGHT_CENTER,
                                ),
                                (format_sig(p_val), Align2::RIGHT_CENTER),
                            ];

                            render_data_row(ui, &w_cols, &vals, 20.0, bg);
                        }
                    }
                });
            });
    });
}

pub fn repeated_glm_to_tsv(res: &RepeatedGlmResult) -> String {
    let mut out =
        String::from("Source\tMeasure\tType III Sum of Squares\tdf\tMean Square\tF\tSig.\n");
    for row in res
        .within_subjects_tests
        .iter()
        .chain(std::iter::once(&res.error_row))
    {
        out.push_str(&format!(
            "{}\tSphericity Assumed\t{:.3}\t{:.3}\t{:.3}\t{}\t{}\n",
            row.source,
            row.sphericity_assumed_ss,
            row.sphericity_assumed_df,
            row.sphericity_assumed_ms,
            row.sphericity_assumed_f
                .map(|v| format!("{:.3}", v))
                .unwrap_or_default(),
            format_sig(row.sphericity_assumed_p)
        ));
    }
    out
}
