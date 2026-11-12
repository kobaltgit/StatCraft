use crate::theme::Tokens;
use egui::{vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::ExploreResult;
use statcraft_i18n::t;

const ROW_H: f32 = 22.0;

pub(crate) fn fmt_opt(val: Option<f64>) -> String {
    val.map(|v| format!("{:.4}", v))
        .unwrap_or_else(|| ".".to_string())
}

pub fn render_spss_explore_pivot_tables(ui: &mut Ui, res: &ExploreResult) {
    if res.variables.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        render_case_processing_table(ui, res);
        ui.add_space(14.0);

        let has_desc = res
            .variables
            .iter()
            .any(|v| v.groups.iter().any(|g| g.descriptives.is_some()));
        if has_desc {
            render_descriptives_table(ui, res);
            ui.add_space(14.0);
        }

        let has_norm = res
            .variables
            .iter()
            .any(|v| v.groups.iter().any(|g| g.normality.is_some()));
        if has_norm {
            render_normality_table(ui, res);
            ui.add_space(14.0);
        }

        let has_ext = res
            .variables
            .iter()
            .any(|v| v.groups.iter().any(|g| g.extremes.is_some()));
        if has_ext {
            render_extremes_table(ui, res);
        }
    });
}

fn render_case_processing_table(ui: &mut Ui, res: &ExploreResult) {
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

fn render_descriptives_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-descriptives"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let cols = vec![
        (t!("output-col-variable"), 140.0_f32),
        (t!("output-col-statistic"), 160.0_f32),
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
                                "".to_string(),
                            ),
                            (
                                format!(
                                    "{} ({})",
                                    t!("output-explore-ci-95"),
                                    t!("output-explore-upper-bound")
                                ),
                                fmt_opt(d.mean_ci_upper),
                                "".to_string(),
                            ),
                            (
                                t!("output-explore-5-trimmed-mean"),
                                fmt_opt(d.trimmed_mean_5pct),
                                "".to_string(),
                            ),
                            (
                                t!("output-explore-median"),
                                fmt_opt(d.median),
                                "".to_string(),
                            ),
                            (
                                t!("output-explore-variance"),
                                fmt_opt(d.variance),
                                "".to_string(),
                            ),
                            (
                                t!("output-explore-std-deviation"),
                                fmt_opt(d.std_dev),
                                "".to_string(),
                            ),
                            (t!("output-explore-minimum"), fmt_opt(d.min), "".to_string()),
                            (t!("output-explore-maximum"), fmt_opt(d.max), "".to_string()),
                            (t!("output-explore-range"), fmt_opt(d.range), "".to_string()),
                            (t!("output-explore-iqr"), fmt_opt(d.iqr), "".to_string()),
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
                                "".to_string()
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

fn render_normality_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-normality"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let cols = vec![
        (t!("output-col-variable"), 140.0_f32),
        (
            format!(
                "{} {}",
                t!("output-explore-kolmogorov-smirnov"),
                t!("output-explore-statistic")
            ),
            90.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-kolmogorov-smirnov"),
                t!("output-explore-df")
            ),
            55.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-kolmogorov-smirnov"),
                t!("output-explore-sig")
            ),
            65.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-shapiro-wilk"),
                t!("output-explore-statistic")
            ),
            90.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-shapiro-wilk"),
                t!("output-explore-df")
            ),
            55.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-shapiro-wilk"),
                t!("output-explore-sig")
            ),
            65.0_f32,
        ),
    ];

    ScrollArea::horizontal()
        .id_salt("explore_normality_scroll")
        .show(ui, |ui| {
            render_header_row(ui, &cols);

            let mut row_idx = 0;
            for var_res in &res.variables {
                for group in &var_res.groups {
                    if let Some(norm) = &group.normality {
                        let var_title = match &group.factor_level {
                            Some(fv) => format!("{} ({})", var_res.variable, fv),
                            None => var_res.variable.clone(),
                        };

                        let (ks_stat, ks_df, ks_sig) = match &norm.kolmogorov_smirnov {
                            Some(ks) => {
                                let sig_str = if ks.sig >= 0.20 {
                                    ".200*".to_string()
                                } else {
                                    format!("{:.3}", ks.sig)
                                };
                                (format!("{:.3}", ks.statistic), ks.df.to_string(), sig_str)
                            }
                            None => (".".to_string(), ".".to_string(), ".".to_string()),
                        };

                        let (sw_stat, sw_df, sw_sig) = match &norm.shapiro_wilk {
                            Some(sw) => (
                                format!("{:.3}", sw.statistic),
                                sw.df.to_string(),
                                format!("{:.3}", sw.sig),
                            ),
                            None => (".".to_string(), ".".to_string(), ".".to_string()),
                        };

                        let vals = vec![var_title, ks_stat, ks_df, ks_sig, sw_stat, sw_df, sw_sig];
                        render_data_row(ui, &cols, &vals, row_idx % 2 == 1);
                        row_idx += 1;
                    }
                }
            }

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(t!("output-explore-lilliefors-footnote"))
                    .font(FontId::proportional(11.0))
                    .color(Tokens::text_muted()),
            );
            ui.label(
                egui::RichText::new(t!("output-explore-lilliefors-correction"))
                    .font(FontId::proportional(11.0))
                    .color(Tokens::text_muted()),
            );
        });
}

fn render_extremes_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-extremes"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let cols = vec![
        (t!("output-col-variable"), 140.0_f32),
        (t!("output-explore-order"), 80.0_f32),
        (t!("output-explore-case-number"), 100.0_f32),
        (t!("output-explore-value"), 90.0_f32),
    ];

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
                                "".to_string()
                            };
                            let order = format!("{} {}", t!("output-explore-highest"), item.order);
                            let vals = vec![
                                lead,
                                order,
                                item.case_number.to_string(),
                                fmt_opt(Some(item.value)),
                            ];
                            render_data_row(ui, &cols, &vals, row_idx % 2 == 1);
                            row_idx += 1;
                        }

                        for item in ext.lowest.iter() {
                            let order = format!("{} {}", t!("output-explore-lowest"), item.order);
                            let vals = vec![
                                "".to_string(),
                                order,
                                item.case_number.to_string(),
                                fmt_opt(Some(item.value)),
                            ];
                            render_data_row(ui, &cols, &vals, row_idx % 2 == 1);
                            row_idx += 1;
                        }
                    }
                }
            }
        });
}

fn render_header_row(ui: &mut Ui, cols: &[(String, f32)]) {
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

fn render_data_row(ui: &mut Ui, cols: &[(String, f32)], vals: &[String], is_odd: bool) {
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
            ui.painter().text(
                pos,
                align,
                text,
                FontId::proportional(12.0),
                Tokens::text_primary(),
            );
        }
    });
}
