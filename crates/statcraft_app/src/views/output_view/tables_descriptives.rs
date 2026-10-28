use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::DescriptivesResult;
use statcraft_i18n::t;

pub fn render_spss_descriptives_pivot_table(ui: &mut Ui, results: &[DescriptivesResult]) {
    if results.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        // Determine optional columns
        let has_var = results.iter().any(|r| r.variance.is_some());
        let has_skew = results.iter().any(|r| r.skewness.is_some());
        let has_kurt = results.iter().any(|r| r.kurtosis.is_some());

        let mut cols: Vec<(String, f32)> = vec![
            (t!("output-col-variable"), 110.0_f32),
            (t!("output-col-n"), 50.0_f32),
            (t!("output-col-min"), 70.0_f32),
            (t!("output-col-max"), 70.0_f32),
            (t!("output-col-mean"), 80.0_f32),
            (t!("output-col-std"), 85.0_f32),
        ];

        if has_var {
            cols.push((t!("output-col-var"), 80.0_f32));
        }
        if has_skew {
            cols.push((t!("output-col-skew"), 80.0_f32));
        }
        if has_kurt {
            cols.push((t!("output-col-kurt"), 80.0_f32));
        }

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
            .id_salt("descriptives_table_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // 1. Table Header Row
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

                    // 2. Data Rows
                    for (row_idx, res) in results.iter().enumerate() {
                        let bg = if row_idx % 2 == 0 {
                            Tokens::BG_ROW_EVEN
                        } else {
                            Tokens::BG_ROW_ODD
                        };

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                            let mut row_vals: Vec<String> = vec![
                                res.variable.clone(),
                                res.n_valid.to_string(),
                                res.min
                                    .map(|v| format!("{:.4}", v))
                                    .unwrap_or_else(|| ".".to_string()),
                                res.max
                                    .map(|v| format!("{:.4}", v))
                                    .unwrap_or_else(|| ".".to_string()),
                                res.mean
                                    .map(|v| format!("{:.4}", v))
                                    .unwrap_or_else(|| ".".to_string()),
                                res.std_dev
                                    .map(|v| format!("{:.4}", v))
                                    .unwrap_or_else(|| ".".to_string()),
                            ];

                            if has_var {
                                row_vals.push(
                                    res.variance
                                        .map(|v| format!("{:.4}", v))
                                        .unwrap_or_else(|| ".".to_string()),
                                );
                            }
                            if has_skew {
                                row_vals.push(
                                    res.skewness
                                        .map(|v| format!("{:.4}", v))
                                        .unwrap_or_else(|| ".".to_string()),
                                );
                            }
                            if has_kurt {
                                row_vals.push(
                                    res.kurtosis
                                        .map(|v| format!("{:.4}", v))
                                        .unwrap_or_else(|| ".".to_string()),
                                );
                            }

                            for (c_idx, val) in row_vals.iter().enumerate() {
                                let width = cols[c_idx].1;
                                let (rect, _) =
                                    ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
                                ui.painter().rect_filled(rect, 0.0_f32, bg);
                                ui.painter().rect_stroke(
                                    rect,
                                    0.0_f32,
                                    Stroke::new(1.0_f32, Tokens::BORDER),
                                );

                                let (align, pos) = if c_idx == 0 {
                                    (Align2::LEFT_CENTER, pos2(rect.min.x + 6.0, rect.center().y))
                                } else {
                                    (
                                        Align2::RIGHT_CENTER,
                                        pos2(rect.max.x - 6.0, rect.center().y),
                                    )
                                };

                                ui.painter().text(
                                    pos,
                                    align,
                                    val,
                                    FontId::proportional(11.5),
                                    Tokens::TEXT_PRIMARY,
                                );
                            }
                        });
                    }
                });
            });
    });
}

pub fn descriptives_to_tsv(results: &[DescriptivesResult]) -> String {
    let mut out = String::new();
    let has_var = results.iter().any(|r| r.variance.is_some());
    let has_skew = results.iter().any(|r| r.skewness.is_some());
    let has_kurt = results.iter().any(|r| r.kurtosis.is_some());

    let mut headers = vec![
        t!("output-col-variable"),
        t!("output-col-n"),
        t!("output-col-min"),
        t!("output-col-max"),
        t!("output-col-mean"),
        t!("output-col-std"),
    ];

    if has_var {
        headers.push(t!("output-col-var"));
    }
    if has_skew {
        headers.push(t!("output-col-skew"));
    }
    if has_kurt {
        headers.push(t!("output-col-kurt"));
    }

    out.push_str(&headers.join("\t"));
    out.push('\n');

    for res in results {
        let mut row = vec![
            res.variable.clone(),
            res.n_valid.to_string(),
            res.min.map(|v| format!("{:.4}", v)).unwrap_or_default(),
            res.max.map(|v| format!("{:.4}", v)).unwrap_or_default(),
            res.mean.map(|v| format!("{:.4}", v)).unwrap_or_default(),
            res.std_dev.map(|v| format!("{:.4}", v)).unwrap_or_default(),
        ];

        if has_var {
            row.push(
                res.variance
                    .map(|v| format!("{:.4}", v))
                    .unwrap_or_default(),
            );
        }
        if has_skew {
            row.push(
                res.skewness
                    .map(|v| format!("{:.4}", v))
                    .unwrap_or_default(),
            );
        }
        if has_kurt {
            row.push(
                res.kurtosis
                    .map(|v| format!("{:.4}", v))
                    .unwrap_or_default(),
            );
        }

        out.push_str(&row.join("\t"));
        out.push('\n');
    }

    let listwise_n = results.iter().map(|r| r.n_valid).min().unwrap_or(0);
    out.push_str(&format!("Valid N (listwise)\t{}\n", listwise_n));

    out
}
