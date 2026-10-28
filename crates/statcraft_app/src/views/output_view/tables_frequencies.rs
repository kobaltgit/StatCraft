use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::FrequencyResult;
use statcraft_i18n::t;

pub fn render_spss_frequencies_tables(ui: &mut Ui, results: &[FrequencyResult]) {
    if results.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        for res in results {
            render_single_frequency_table(ui, res);
            if let Some((q1, q2, q3)) = res.quartiles {
                render_quartiles_table(ui, &res.variable, q1, q2, q3);
            }
            ui.add_space(16.0_f32);
        }
    });
}

fn render_single_frequency_table(ui: &mut Ui, res: &FrequencyResult) {
    ui.strong(t!("output-frequencies-title", name = res.variable.as_str()));
    ui.add_space(6.0_f32);

    let n_total = res.n_valid + res.n_missing;
    let valid_percent_total = if n_total > 0 {
        (res.n_valid as f64 / n_total as f64) * 100.0
    } else {
        0.0
    };
    let missing_percent = if n_total > 0 {
        (res.n_missing as f64 / n_total as f64) * 100.0
    } else {
        0.0
    };

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-value"), 120.0_f32),
        (t!("output-col-frequency"), 80.0_f32),
        (t!("output-col-percent"), 80.0_f32),
        (t!("output-col-valid-percent"), 95.0_f32),
        (t!("output-col-cum-percent"), 110.0_f32),
    ];

    // Dynamic width adjustment (i18n Shield)
    for (col_name, width) in &mut cols {
        let text_w = ui
            .painter()
            .layout_no_wrap(
                col_name.clone(),
                FontId::proportional(12.0),
                Tokens::TEXT_PRIMARY,
            )
            .rect
            .width();
        *width = (*width).max(text_w + 16.0);
    }

    let row_height = 22.0_f32;

    ScrollArea::horizontal()
        .id_salt(format!("freq_scroll_{}", res.variable))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    for (col_name, width) in &cols {
                        let (rect, _) =
                            ui.allocate_exact_size(vec2(*width, row_height), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
                        ui.painter().rect_stroke(
                            rect,
                            0.0_f32,
                            Stroke::new(1.0_f32, Tokens::BORDER),
                        );
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            col_name,
                            FontId::proportional(12.0),
                            Tokens::TEXT_PRIMARY,
                        );
                    }
                });

                // Categories
                for (r_idx, cat) in res.categories.iter().enumerate() {
                    let bg_color = if r_idx % 2 == 0 {
                        Tokens::BG_ROW_EVEN
                    } else {
                        Tokens::BG_ROW_ODD
                    };

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                        let values: [(String, Align2); 5] = [
                            (cat.display_label().to_string(), Align2::LEFT_CENTER),
                            (format!("{}", cat.count), Align2::RIGHT_CENTER),
                            (format!("{:.1}", cat.percent), Align2::RIGHT_CENTER),
                            (
                                cat.valid_percent
                                    .map(|v| format!("{:.1}", v))
                                    .unwrap_or_else(|| "-".to_string()),
                                Align2::RIGHT_CENTER,
                            ),
                            (
                                cat.cumulative_percent
                                    .map(|v| format!("{:.1}", v))
                                    .unwrap_or_else(|| "-".to_string()),
                                Align2::RIGHT_CENTER,
                            ),
                        ];

                        for (c_idx, (text, align)) in values.iter().enumerate() {
                            let width = cols[c_idx].1;
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
                            ui.painter().rect_filled(rect, 0.0_f32, bg_color);
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
                }

                // Total Valid
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    let row_vals = [
                        (t!("output-row-total-valid"), Align2::LEFT_CENTER),
                        (format!("{}", res.n_valid), Align2::RIGHT_CENTER),
                        (format!("{:.1}", valid_percent_total), Align2::RIGHT_CENTER),
                        ("100.0".to_string(), Align2::RIGHT_CENTER),
                        (String::new(), Align2::RIGHT_CENTER),
                    ];

                    for (c_idx, (text, align)) in row_vals.iter().enumerate() {
                        let width = cols[c_idx].1;
                        let (rect, _) =
                            ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_ROW_EVEN);
                        ui.painter().rect_stroke(
                            rect,
                            0.0_f32,
                            Stroke::new(1.0_f32, Tokens::BORDER),
                        );

                        let pos = if *align == Align2::LEFT_CENTER {
                            pos2(rect.min.x + 6.0_f32, rect.center().y)
                        } else {
                            pos2(rect.max.x - 6.0_f32, rect.center().y)
                        };

                        ui.painter().text(
                            pos,
                            *align,
                            text,
                            FontId::proportional(11.5),
                            Tokens::TEXT_PRIMARY,
                        );
                    }
                });

                // Missing Count (if any)
                if res.n_missing > 0 {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                        let row_vals = [
                            (t!("output-row-system-missing"), Align2::LEFT_CENTER),
                            (format!("{}", res.n_missing), Align2::RIGHT_CENTER),
                            (format!("{:.1}", missing_percent), Align2::RIGHT_CENTER),
                            (String::new(), Align2::RIGHT_CENTER),
                            (String::new(), Align2::RIGHT_CENTER),
                        ];

                        for (c_idx, (text, align)) in row_vals.iter().enumerate() {
                            let width = cols[c_idx].1;
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
                            ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_ROW_ODD);
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

                            ui.painter().text(
                                pos,
                                *align,
                                text,
                                FontId::proportional(11.5),
                                Tokens::TEXT_PRIMARY,
                            );
                        }
                    });
                }

                // Grand Total
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    let row_vals = [
                        (t!("output-row-total"), Align2::LEFT_CENTER),
                        (format!("{}", n_total), Align2::RIGHT_CENTER),
                        ("100.0".to_string(), Align2::RIGHT_CENTER),
                        (String::new(), Align2::RIGHT_CENTER),
                        (String::new(), Align2::RIGHT_CENTER),
                    ];

                    for (c_idx, (text, align)) in row_vals.iter().enumerate() {
                        let width = cols[c_idx].1;
                        let (rect, _) =
                            ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
                        ui.painter().rect_stroke(
                            rect,
                            0.0_f32,
                            Stroke::new(1.0_f32, Tokens::BORDER),
                        );

                        let pos = if *align == Align2::LEFT_CENTER {
                            pos2(rect.min.x + 6.0_f32, rect.center().y)
                        } else {
                            pos2(rect.max.x - 6.0_f32, rect.center().y)
                        };

                        ui.painter().text(
                            pos,
                            *align,
                            text,
                            FontId::proportional(11.5),
                            Tokens::TEXT_PRIMARY,
                        );
                    }
                });
            });
        });
}

fn render_quartiles_table(ui: &mut Ui, var_name: &str, q1: f64, q2: f64, q3: f64) {
    ui.add_space(4.0_f32);
    ui.strong(t!("output-quartiles-title", name = var_name));
    ui.add_space(4.0_f32);

    let row_height = 20.0_f32;
    let col_w = 110.0_f32;

    let quartiles = [("25 (Q1)", q1), ("50 (Median)", q2), ("75 (Q3)", q3)];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        for (label, val) in &quartiles {
            let (r1, _) = ui.allocate_exact_size(vec2(col_w, row_height), Sense::hover());
            ui.painter().rect_filled(r1, 0.0_f32, Tokens::BG_HEADER);
            ui.painter()
                .rect_stroke(r1, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));
            ui.painter().text(
                r1.center(),
                Align2::CENTER_CENTER,
                *label,
                FontId::proportional(11.0),
                Tokens::TEXT_PRIMARY,
            );

            let (r2, _) = ui.allocate_exact_size(vec2(col_w, row_height), Sense::hover());
            ui.painter().rect_filled(r2, 0.0_f32, Tokens::BG_ROW_EVEN);
            ui.painter()
                .rect_stroke(r2, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));
            ui.painter().text(
                r2.center(),
                Align2::CENTER_CENTER,
                format!("{:.4}", val),
                FontId::proportional(11.0),
                Tokens::TEXT_PRIMARY,
            );
        }
    });
}

pub fn frequencies_to_tsv(results: &[FrequencyResult]) -> String {
    let mut out = String::new();

    for res in results {
        out.push_str(&format!("{}\n", res.variable));
        let headers = [
            t!("output-col-value"),
            t!("output-col-frequency"),
            t!("output-col-percent"),
            t!("output-col-valid-percent"),
            t!("output-col-cum-percent"),
        ];
        out.push_str(&headers.join("\t"));
        out.push('\n');

        for cat in &res.categories {
            let row = [
                cat.display_label().to_string(),
                format!("{}", cat.count),
                format!("{:.1}", cat.percent),
                cat.valid_percent
                    .map(|v| format!("{:.1}", v))
                    .unwrap_or_default(),
                cat.cumulative_percent
                    .map(|v| format!("{:.1}", v))
                    .unwrap_or_default(),
            ];
            out.push_str(&row.join("\t"));
            out.push('\n');
        }

        let n_total = res.n_valid + res.n_missing;
        let valid_percent_total = if n_total > 0 {
            (res.n_valid as f64 / n_total as f64) * 100.0
        } else {
            0.0
        };
        let missing_percent = if n_total > 0 {
            (res.n_missing as f64 / n_total as f64) * 100.0
        } else {
            0.0
        };

        out.push_str(&format!(
            "{}\t{}\t{:.1}\t100.0\n",
            t!("output-row-total-valid"),
            res.n_valid,
            valid_percent_total
        ));

        if res.n_missing > 0 {
            out.push_str(&format!(
                "{}\t{}\t{:.1}\n",
                t!("output-row-system-missing"),
                res.n_missing,
                missing_percent
            ));
        }

        out.push_str(&format!(
            "{}\t{}\t100.0\n\n",
            t!("output-row-total"),
            n_total
        ));
    }

    out
}
