use super::tables_common::{
    bootstrap_column_headers, bootstrap_specs_to_tsv, bootstrap_table_to_tsv,
    format_bootstrap_cells, render_bootstrap_specifications_table, render_bootstrap_table,
};
use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::{BootstrapSummary, DescriptivesResult};
use statcraft_i18n::t;

pub fn render_spss_descriptives_pivot_table(ui: &mut Ui, results: &[DescriptivesResult]) {
    if results.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        if let Some(opts) = results
            .iter()
            .find_map(|r| r.bootstrap.as_ref().map(|b| &b.options))
        {
            render_bootstrap_specifications_table(ui, opts);
            ui.add_space(14.0_f32);
        }
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
                    Tokens::text_primary(),
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
                            ui.painter().rect_filled(rect, 0.0_f32, Tokens::bg_header());
                            ui.painter().rect_stroke(
                                rect,
                                0.0_f32,
                                Stroke::new(1.0_f32, Tokens::border()),
                            );
                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                col_name,
                                FontId::proportional(12.0),
                                Tokens::text_primary(),
                            );
                        }
                    });

                    // 2. Data Rows
                    for (row_idx, res) in results.iter().enumerate() {
                        let bg = if row_idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
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
                                    Stroke::new(1.0_f32, Tokens::border()),
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
                                    Tokens::text_primary(),
                                );
                            }
                        });
                    }
                });
            });

        if let Some((headers, rows)) = build_descriptives_bootstrap_rows(results) {
            ui.add_space(14.0_f32);
            let title = format!(
                "{} ({})",
                t!("output-descriptives-title"),
                t!("output-bootstrap-group-header")
            );
            render_bootstrap_table(ui, &title, "desc_boot_table", &headers, &rows, 2);
        }
    });
}

fn build_descriptives_bootstrap_rows(
    results: &[DescriptivesResult],
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let mut rows = Vec::new();
    let mut ci_level = 95.0;

    for res in results {
        let Some(ref boot) = res.bootstrap else {
            continue;
        };
        ci_level = boot.options.ci_level;
        let candidates: [(String, &Option<BootstrapSummary>); 5] = [
            (t!("output-col-mean"), &boot.mean),
            (t!("output-col-std"), &boot.std_dev),
            (t!("output-col-var"), &boot.variance),
            (t!("output-col-skew"), &boot.skewness),
            (t!("output-col-kurt"), &boot.kurtosis),
        ];
        for (stat_name, sum_opt) in candidates {
            if let Some(s) = sum_opt {
                let cells = format_bootstrap_cells(s, 4);
                rows.push(vec![
                    res.variable.clone(),
                    stat_name,
                    format!("{:.4}", s.estimate),
                    cells[0].clone(),
                    cells[1].clone(),
                    cells[2].clone(),
                    cells[3].clone(),
                ]);
            }
        }
    }

    if rows.is_empty() {
        return None;
    }

    let boot_cols = bootstrap_column_headers(ci_level);
    let headers = vec![
        t!("output-col-variable"),
        t!("output-bootstrap-col-statistic"),
        t!("output-km-col-estimate"),
        boot_cols[0].clone(),
        boot_cols[1].clone(),
        boot_cols[2].clone(),
        boot_cols[3].clone(),
    ];
    Some((headers, rows))
}

pub fn descriptives_to_tsv(results: &[DescriptivesResult]) -> String {
    let mut out = String::new();
    if let Some(opts) = results
        .iter()
        .find_map(|r| r.bootstrap.as_ref().map(|b| &b.options))
    {
        out.push_str(&bootstrap_specs_to_tsv(opts));
        out.push('\n');
    }
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

    if let Some((headers, rows)) = build_descriptives_bootstrap_rows(results) {
        let title = format!(
            "{} ({})",
            t!("output-descriptives-title"),
            t!("output-bootstrap-group-header")
        );
        out.push_str(&bootstrap_table_to_tsv(&title, &headers, &rows));
        out.push('\n');
    }

    out
}
