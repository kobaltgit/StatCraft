use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::{AnovaGroupDescriptives, OneWayAnovaResult};
use statcraft_i18n::t;

pub fn render_spss_oneway_anova_tables(ui: &mut Ui, results: &[OneWayAnovaResult], ci: f64) {
    if results.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        for res in results {
            // 1. Descriptives Table
            if !res.descriptives.is_empty() {
                render_anova_descriptives_table(ui, res, ci);
                ui.add_space(14.0_f32);
            }

            // 2. Homogeneity of Variances Table
            if let Some(ref hom) = res.homogeneity {
                render_anova_homogeneity_table(ui, &res.dep_var, hom);
                ui.add_space(14.0_f32);
            }

            // 3. ANOVA Table
            render_anova_summary_table(ui, res);
            ui.add_space(16.0_f32);
        }
    });
}

fn render_anova_descriptives_table(ui: &mut Ui, res: &OneWayAnovaResult, _ci: f64) {
    ui.strong(t!("output-anova-descriptives-title"));
    ui.add_space(4.0_f32);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-variable"), 110.0_f32),
        (t!("output-col-n"), 55.0_f32),
        (t!("output-col-mean"), 80.0_f32),
        (t!("output-col-std"), 80.0_f32),
        (t!("output-col-std-error"), 80.0_f32),
        (t!("output-col-ci-lower"), 80.0_f32),
        (t!("output-col-ci-upper"), 80.0_f32),
        (t!("output-col-min"), 70.0_f32),
        (t!("output-col-max"), 70.0_f32),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0_f32;

    ScrollArea::horizontal()
        .id_salt(format!("anova_desc_scroll_{}", res.dep_var))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header(ui, &cols, row_height);

                for (r_idx, grp) in res.descriptives.iter().enumerate() {
                    let bg = if r_idx % 2 == 0 {
                        Tokens::BG_ROW_EVEN
                    } else {
                        Tokens::BG_ROW_ODD
                    };
                    render_desc_group_row(ui, &cols, grp, row_height, bg);
                }

                if let Some(ref total) = res.total_descriptives {
                    let mut grp_copy = total.clone();
                    grp_copy.group_label = t!("output-col-total");
                    render_desc_group_row(ui, &cols, &grp_copy, row_height, Tokens::BG_HEADER);
                }
            });
        });
}

fn render_desc_group_row(
    ui: &mut Ui,
    cols: &[(String, f32)],
    grp: &AnovaGroupDescriptives,
    row_height: f32,
    bg_color: egui::Color32,
) {
    let vals = [
        (grp.group_label.clone(), Align2::LEFT_CENTER),
        (format!("{}", grp.n), Align2::RIGHT_CENTER),
        (format!("{:.4}", grp.mean), Align2::RIGHT_CENTER),
        (format!("{:.4}", grp.std_dev), Align2::RIGHT_CENTER),
        (format!("{:.4}", grp.std_err), Align2::RIGHT_CENTER),
        (format!("{:.4}", grp.ci_lower), Align2::RIGHT_CENTER),
        (format!("{:.4}", grp.ci_upper), Align2::RIGHT_CENTER),
        (format!("{:.2}", grp.min), Align2::RIGHT_CENTER),
        (format!("{:.2}", grp.max), Align2::RIGHT_CENTER),
    ];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (c_idx, (text, align)) in vals.iter().enumerate() {
            let width = cols.get(c_idx).map(|c| c.1).unwrap_or(75.0_f32);
            let (rect, _) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
            ui.painter().rect_filled(rect, 0.0_f32, bg_color);
            ui.painter()
                .rect_stroke(rect, 0.0_f32, Stroke::new(0.5_f32, Tokens::BORDER));

            let pos = if *align == Align2::LEFT_CENTER {
                pos2(rect.min.x + 6.0_f32, rect.center().y)
            } else {
                pos2(rect.max.x - 6.0_f32, rect.center().y)
            };

            let clip = rect.shrink2(vec2(4.0_f32, 0.0_f32));
            ui.painter().with_clip_rect(clip).text(
                pos,
                *align,
                text.as_str(),
                FontId::proportional(11.5),
                Tokens::TEXT_PRIMARY,
            );
        }
    });
}

fn render_anova_homogeneity_table(
    ui: &mut Ui,
    dep_var: &str,
    hom: &statcraft_algo::AnovaHomogeneityTest,
) {
    ui.strong(t!("output-anova-homogeneity-title"));
    ui.add_space(4.0_f32);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-variable"), 120.0_f32),
        (t!("output-col-levene-statistic"), 120.0_f32),
        (t!("output-col-df1"), 60.0_f32),
        (t!("output-col-df2"), 60.0_f32),
        (t!("output-col-sig"), 75.0_f32),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0_f32;

    ScrollArea::horizontal()
        .id_salt(format!("anova_hom_scroll_{}", dep_var))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header(ui, &cols, row_height);

                let vals = [
                    (dep_var.to_string(), Align2::LEFT_CENTER),
                    (format!("{:.4}", hom.levene_stat), Align2::RIGHT_CENTER),
                    (format!("{}", hom.df1), Align2::RIGHT_CENTER),
                    (format!("{}", hom.df2), Align2::RIGHT_CENTER),
                    (format_sig(Some(hom.p_value)), Align2::RIGHT_CENTER),
                ];

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    for (c_idx, (text, align)) in vals.iter().enumerate() {
                        let width = cols.get(c_idx).map(|c| c.1).unwrap_or(80.0_f32);
                        let (rect, _) =
                            ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_ROW_EVEN);
                        ui.painter().rect_stroke(
                            rect,
                            0.0_f32,
                            Stroke::new(0.5_f32, Tokens::BORDER),
                        );

                        let pos = if *align == Align2::LEFT_CENTER {
                            pos2(rect.min.x + 6.0_f32, rect.center().y)
                        } else {
                            pos2(rect.max.x - 6.0_f32, rect.center().y)
                        };

                        let clip = rect.shrink2(vec2(4.0_f32, 0.0_f32));
                        ui.painter().with_clip_rect(clip).text(
                            pos,
                            *align,
                            text.as_str(),
                            FontId::proportional(11.5),
                            Tokens::TEXT_PRIMARY,
                        );
                    }
                });
            });
        });
}

fn render_anova_summary_table(ui: &mut Ui, res: &OneWayAnovaResult) {
    ui.strong(t!("output-anova-title"));
    ui.add_space(4.0_f32);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-variable"), 130.0_f32),
        (t!("output-col-sum-of-squares"), 120.0_f32),
        (t!("output-col-df"), 55.0_f32),
        (t!("output-col-mean-square"), 105.0_f32),
        (t!("output-col-f"), 70.0_f32),
        (t!("output-col-sig"), 75.0_f32),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0_f32;

    ScrollArea::horizontal()
        .id_salt(format!("anova_sum_scroll_{}", res.dep_var))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header(ui, &cols, row_height);

                // 1. Between Groups
                let between = &res.between_groups;
                let vals_b = [
                    (t!("output-col-between-groups"), Align2::LEFT_CENTER),
                    (
                        format!("{:.4}", between.sum_of_squares),
                        Align2::RIGHT_CENTER,
                    ),
                    (format!("{}", between.df), Align2::RIGHT_CENTER),
                    (format_opt(between.mean_square), Align2::RIGHT_CENTER),
                    (format_opt(between.f_stat), Align2::RIGHT_CENTER),
                    (format_sig(between.p_value), Align2::RIGHT_CENTER),
                ];
                render_summary_row(ui, &cols, &vals_b, row_height, Tokens::BG_ROW_EVEN);

                // 2. Within Groups
                let within = &res.within_groups;
                let vals_w = [
                    (t!("output-col-within-groups"), Align2::LEFT_CENTER),
                    (
                        format!("{:.4}", within.sum_of_squares),
                        Align2::RIGHT_CENTER,
                    ),
                    (format!("{}", within.df), Align2::RIGHT_CENTER),
                    (format_opt(within.mean_square), Align2::RIGHT_CENTER),
                    ("-".to_string(), Align2::CENTER_CENTER),
                    ("-".to_string(), Align2::CENTER_CENTER),
                ];
                render_summary_row(ui, &cols, &vals_w, row_height, Tokens::BG_ROW_ODD);

                // 3. Total
                let total = &res.total;
                let vals_t = [
                    (t!("output-col-total"), Align2::LEFT_CENTER),
                    (format!("{:.4}", total.sum_of_squares), Align2::RIGHT_CENTER),
                    (format!("{}", total.df), Align2::RIGHT_CENTER),
                    ("-".to_string(), Align2::CENTER_CENTER),
                    ("-".to_string(), Align2::CENTER_CENTER),
                    ("-".to_string(), Align2::CENTER_CENTER),
                ];
                render_summary_row(ui, &cols, &vals_t, row_height, Tokens::BG_HEADER);
            });
        });
}

fn adjust_widths(ui: &mut Ui, cols: &mut [(String, f32)]) {
    for (name, width) in cols {
        let text_w = ui
            .painter()
            .layout_no_wrap(
                name.clone(),
                FontId::proportional(12.0),
                Tokens::TEXT_PRIMARY,
            )
            .rect
            .width();
        *width = (*width).max(text_w + 16.0);
    }
}

fn render_header(ui: &mut Ui, cols: &[(String, f32)], row_height: f32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (col_name, width) in cols {
            let (rect, _) = ui.allocate_exact_size(vec2(*width, row_height), Sense::hover());
            ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
            ui.painter()
                .rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                col_name,
                FontId::proportional(11.5),
                Tokens::TEXT_PRIMARY,
            );
        }
    });
}

fn render_summary_row(
    ui: &mut Ui,
    cols: &[(String, f32)],
    vals: &[(String, Align2)],
    row_height: f32,
    bg_color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (c_idx, (text, align)) in vals.iter().enumerate() {
            let width = cols.get(c_idx).map(|c| c.1).unwrap_or(80.0_f32);
            let (rect, _) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
            ui.painter().rect_filled(rect, 0.0_f32, bg_color);
            ui.painter()
                .rect_stroke(rect, 0.0_f32, Stroke::new(0.5_f32, Tokens::BORDER));

            let pos = if *align == Align2::LEFT_CENTER {
                pos2(rect.min.x + 6.0_f32, rect.center().y)
            } else if *align == Align2::CENTER_CENTER {
                rect.center()
            } else {
                pos2(rect.max.x - 6.0_f32, rect.center().y)
            };

            let clip = rect.shrink2(vec2(4.0_f32, 0.0_f32));
            ui.painter().with_clip_rect(clip).text(
                pos,
                *align,
                text.as_str(),
                FontId::proportional(11.5),
                Tokens::TEXT_PRIMARY,
            );
        }
    });
}

fn format_opt(val: Option<f64>) -> String {
    val.map(|v| format!("{:.4}", v))
        .unwrap_or_else(|| "-".to_string())
}

fn format_sig(val: Option<f64>) -> String {
    match val {
        Some(v) if v < 0.001 => "< .001".to_string(),
        Some(v) => format!("{:.3}", v),
        None => "-".to_string(),
    }
}

pub fn anova_to_tsv(results: &[OneWayAnovaResult]) -> String {
    let mut out =
        String::from("Dependent Variable\tSource\tSum of Squares\tdf\tMean Square\tF\tSig.\n");
    for r in results {
        let b = &r.between_groups;
        out.push_str(&format!(
            "{}\tBetween Groups\t{:.4}\t{}\t{}\t{}\t{}\n",
            r.dep_var,
            b.sum_of_squares,
            b.df,
            format_opt(b.mean_square),
            format_opt(b.f_stat),
            format_sig(b.p_value)
        ));

        let w = &r.within_groups;
        out.push_str(&format!(
            "{}\tWithin Groups\t{:.4}\t{}\t{}\t-\t-\n",
            r.dep_var,
            w.sum_of_squares,
            w.df,
            format_opt(w.mean_square)
        ));

        let t = &r.total;
        out.push_str(&format!(
            "{}\tTotal\t{:.4}\t{}\t-\t-\t-\n\n",
            r.dep_var, t.sum_of_squares, t.df
        ));
    }
    out
}
