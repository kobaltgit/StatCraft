use crate::theme::Tokens;
use egui::{vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::ExploreResult;
use statcraft_i18n::t;

const ROW_H: f32 = 22.0;

pub(crate) fn fmt_opt(val: Option<f64>) -> String {
    val.map(|v| format!("{:.4}", v))
        .unwrap_or_else(|| ".".to_string())
}

pub(crate) fn render_case_processing_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-case-processing"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let cols = vec![
        (t!("output-col-variable"), 140.0_f32),
        (format!("{} N", t!("output-explore-valid")), 75.0_f32),
        (format!("{} %", t!("output-explore-valid")), 65.0_f32),
        (format!("{} N", t!("output-explore-missing")), 75.0_f32),
        (format!("{} %", t!("output-explore-missing")), 65.0_f32),
        (format!("{} N", t!("output-explore-total")), 75.0_f32),
        (format!("{} %", t!("output-explore-total")), 65.0_f32),
    ];

    ScrollArea::horizontal()
        .id_salt("explore_case_proc_scroll")
        .show(ui, |ui| {
            render_header_row(ui, &cols);

            let mut row_idx = 0;
            for var_res in &res.variables {
                for group in &var_res.groups {
                    let cp = &group.case_processing;
                    let var_title = match &group.factor_level {
                        Some(fv) => format!("{} ({})", var_res.variable, fv),
                        None => var_res.variable.clone(),
                    };

                    let row_vals = vec![
                        var_title,
                        cp.n_valid.to_string(),
                        format!("{:.1}%", cp.pct_valid),
                        cp.n_missing.to_string(),
                        format!("{:.1}%", cp.pct_missing),
                        cp.n_total.to_string(),
                        format!("{:.1}%", cp.pct_total),
                    ];

                    render_data_row(ui, &cols, &row_vals, row_idx % 2 == 1);
                    row_idx += 1;
                }
            }
        });
}

pub(crate) fn render_descriptives_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-descriptives"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let cols = vec![
        (t!("output-col-variable"), 140.0_f32),
        (t!("output-col-statistic"), 170.0_f32),
        (t!("output-explore-statistic"), 90.0_f32),
        (t!("output-explore-std-error"), 90.0_f32),
    ];

    ScrollArea::horizontal()
        .id_salt("explore_desc_scroll")
        .show(ui, |ui| {
            render_header_row(ui, &cols);

            let mut row_idx = 0;
            for var_res in &res.variables {
                for group in &var_res.groups {
                    if let Some(d) = &group.descriptives {
                        let var_label = match &group.factor_level {
                            Some(fv) => format!("{} ({})", var_res.variable, fv),
                            None => var_res.variable.clone(),
                        };

                        let metrics = vec![
                            (
                                t!("output-explore-mean"),
                                fmt_opt(d.mean),
                                fmt_opt(d.mean_se),
                            ),
                            (
                                format!(
                                    "{} ({})",
                                    t!("output-explore-ci-95"),
                                    t!("output-explore-lower-bound")
                                ),
                                fmt_opt(d.mean_ci_lower),
                                String::new(),
                            ),
                            (
                                format!(
                                    "{} ({})",
                                    t!("output-explore-ci-95"),
                                    t!("output-explore-upper-bound")
                                ),
                                fmt_opt(d.mean_ci_upper),
                                String::new(),
                            ),
                            (
                                t!("output-explore-5-trimmed-mean"),
                                fmt_opt(d.trimmed_mean_5pct),
                                String::new(),
                            ),
                            (
                                t!("output-explore-median"),
                                fmt_opt(d.median),
                                String::new(),
                            ),
                            (
                                t!("output-explore-variance"),
                                fmt_opt(d.variance),
                                String::new(),
                            ),
                            (
                                t!("output-explore-std-deviation"),
                                fmt_opt(d.std_dev),
                                String::new(),
                            ),
                            (t!("output-explore-minimum"), fmt_opt(d.min), String::new()),
                            (t!("output-explore-maximum"), fmt_opt(d.max), String::new()),
                            (t!("output-explore-range"), fmt_opt(d.range), String::new()),
                            (t!("output-explore-iqr"), fmt_opt(d.iqr), String::new()),
                            (
                                t!("output-explore-skewness"),
                                fmt_opt(d.skewness),
                                fmt_opt(d.skewness_se),
                            ),
                            (
                                t!("output-explore-kurtosis"),
                                fmt_opt(d.kurtosis),
                                fmt_opt(d.kurtosis_se),
                            ),
                        ];

                        for (i, (stat_name, stat_val, se_val)) in metrics.into_iter().enumerate() {
                            let lead = if i == 0 {
                                var_label.clone()
                            } else {
                                String::new()
                            };
                            let vals = vec![lead, stat_name, stat_val, se_val];
                            render_data_row(ui, &cols, &vals, row_idx % 2 == 1);
                            row_idx += 1;
                        }
                    }
                }
            }
        });
}

pub(crate) fn render_extremes_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-extremes"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let has_labels = res.variables.iter().any(|v| {
        v.groups.iter().any(|g| {
            g.extremes.as_ref().is_some_and(|e| {
                e.highest.iter().any(|i| i.case_label.is_some())
                    || e.lowest.iter().any(|i| i.case_label.is_some())
            })
        })
    });

    let mut cols = vec![
        (t!("output-col-variable"), 140.0_f32),
        (t!("output-explore-order"), 85.0_f32),
        (t!("output-explore-case-number"), 100.0_f32),
    ];
    if has_labels {
        cols.push((t!("output-explore-case-label"), 110.0_f32));
    }
    cols.push((t!("output-explore-value"), 90.0_f32));

    ScrollArea::horizontal()
        .id_salt("explore_extremes_scroll")
        .show(ui, |ui| {
            render_header_row(ui, &cols);

            let mut row_idx = 0;
            for var_res in &res.variables {
                for group in &var_res.groups {
                    if let Some(ext) = &group.extremes {
                        let var_title = match &group.factor_level {
                            Some(fv) => format!("{} ({})", var_res.variable, fv),
                            None => var_res.variable.clone(),
                        };

                        for (i, item) in ext.highest.iter().enumerate() {
                            let lead = if i == 0 {
                                var_title.clone()
                            } else {
                                String::new()
                            };
                            let order = format!("{} {}", t!("output-explore-highest"), item.order);
                            let mut vals = vec![lead, order, item.case_number.to_string()];
                            if has_labels {
                                vals.push(item.case_label.clone().unwrap_or_default());
                            }
                            vals.push(fmt_opt(Some(item.value)));
                            render_data_row(ui, &cols, &vals, row_idx % 2 == 1);
                            row_idx += 1;
                        }

                        for item in ext.lowest.iter() {
                            let order = format!("{} {}", t!("output-explore-lowest"), item.order);
                            let mut vals = vec![String::new(), order, item.case_number.to_string()];
                            if has_labels {
                                vals.push(item.case_label.clone().unwrap_or_default());
                            }
                            vals.push(fmt_opt(Some(item.value)));
                            render_data_row(ui, &cols, &vals, row_idx % 2 == 1);
                            row_idx += 1;
                        }
                    }
                }
            }
        });
}

pub(crate) fn render_header_row(ui: &mut Ui, cols: &[(String, f32)]) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (name, width) in cols {
            let (rect, _) = ui.allocate_exact_size(vec2(*width, ROW_H), Sense::hover());
            ui.painter().rect_filled(rect, 0.0, Tokens::bg_header());
            ui.painter()
                .rect_stroke(rect, 0.0, Stroke::new(1.0_f32, Tokens::border()));
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                name,
                FontId::proportional(12.0),
                Tokens::text_primary(),
            );
        }
    });
}

pub(crate) fn render_data_row(ui: &mut Ui, cols: &[(String, f32)], vals: &[String], is_odd: bool) {
    let bg = if is_odd {
        Tokens::bg_row_odd()
    } else {
        Tokens::bg_row_even()
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (i, (_, width)) in cols.iter().enumerate() {
            let text = vals.get(i).map(|s| s.as_str()).unwrap_or("");
            let (rect, _) = ui.allocate_exact_size(vec2(*width, ROW_H), Sense::hover());
            ui.painter().rect_filled(rect, 0.0, bg);
            ui.painter()
                .rect_stroke(rect, 0.0, Stroke::new(1.0_f32, Tokens::border()));
            let align = if i == 0 || i == 1 {
                Align2::LEFT_CENTER
            } else {
                Align2::RIGHT_CENTER
            };
            let pos = if i == 0 || i == 1 {
                rect.left_center() + vec2(6.0, 0.0)
            } else {
                rect.right_center() - vec2(6.0, 0.0)
            };
            let clip = rect.shrink2(vec2(4.0, 0.0));
            ui.painter().with_clip_rect(clip).text(
                pos,
                align,
                text,
                FontId::proportional(12.0),
                Tokens::text_primary(),
            );
        }
    });
}
