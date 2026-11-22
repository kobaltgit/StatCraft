use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::FrequencyResult;
use statcraft_i18n::t;

enum StatMetric {
    NValid,
    NMissing,
    Mean,
    SeMean,
    Median,
    Mode,
    StdDev,
    Variance,
    Skewness,
    SeSkewness,
    Kurtosis,
    SeKurtosis,
    Range,
    Min,
    Max,
    Sum,
    PercentilesHeader,
    Percentile(f64),
}

struct RowDefinition {
    label: String,
    is_header_only: bool,
    values: Vec<String>,
}

pub fn render_stats_summary_table(ui: &mut Ui, results: &[FrequencyResult]) {
    if results.is_empty() {
        return;
    }

    ui.strong(t!("output-stats-title"));
    ui.add_space(6.0_f32);

    let rows = build_summary_rows(results);
    let mut cols: Vec<(String, f32)> = Vec::with_capacity(results.len() + 1);
    cols.push((String::new(), 160.0_f32));
    for res in results {
        cols.push((res.display_label_or_name().to_string(), 90.0_f32));
    }

    // Dynamic width adjustment (i18n Shield)
    let label_font = FontId::proportional(12.0);
    for row in &rows {
        let text_w = ui
            .painter()
            .layout_no_wrap(
                row.label.clone(),
                label_font.clone(),
                Tokens::text_primary(),
            )
            .rect
            .width();
        cols[0].1 = cols[0].1.max(text_w + 16.0);
    }
    for (i, res) in results.iter().enumerate() {
        let col_idx = i + 1;
        let header_w = ui
            .painter()
            .layout_no_wrap(
                res.display_label_or_name().to_string(),
                label_font.clone(),
                Tokens::text_primary(),
            )
            .rect
            .width();
        cols[col_idx].1 = cols[col_idx].1.max(header_w + 16.0);
        for row in &rows {
            if let Some(val_str) = row.values.get(i) {
                let cell_w = ui
                    .painter()
                    .layout_no_wrap(
                        val_str.clone(),
                        FontId::proportional(11.5),
                        Tokens::text_primary(),
                    )
                    .rect
                    .width();
                cols[col_idx].1 = cols[col_idx].1.max(cell_w + 16.0);
            }
        }
    }

    let row_height = 22.0_f32;

    ScrollArea::horizontal()
        .id_salt("freq_stats_summary_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Table Header Row
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    for (c_idx, (col_name, width)) in cols.iter().enumerate() {
                        let (rect, resp) =
                            ui.allocate_exact_size(vec2(*width, row_height), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0_f32, Tokens::bg_header());
                        ui.painter().rect_stroke(
                            rect,
                            0.0_f32,
                            Stroke::new(1.0_f32, Tokens::border()),
                        );
                        if c_idx > 0 {
                            let res = &results[c_idx - 1];
                            if res.variable_label.is_some() {
                                resp.on_hover_text(format!("{} [{}]", col_name, res.variable));
                            }
                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                col_name,
                                FontId::proportional(12.0),
                                Tokens::text_primary(),
                            );
                        }
                    }
                });

                // Table Data Rows
                for (r_idx, row) in rows.iter().enumerate() {
                    let bg_color = if r_idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                        // Column 0: Metric label
                        let width0 = cols[0].1;
                        let (rect0, _) =
                            ui.allocate_exact_size(vec2(width0, row_height), Sense::hover());
                        ui.painter().rect_filled(rect0, 0.0_f32, bg_color);
                        ui.painter().rect_stroke(
                            rect0,
                            0.0_f32,
                            Stroke::new(0.5_f32, Tokens::border()),
                        );
                        let pos0 = pos2(rect0.min.x + 8.0_f32, rect0.center().y);
                        ui.painter().text(
                            pos0,
                            Align2::LEFT_CENTER,
                            &row.label,
                            FontId::proportional(11.5),
                            Tokens::text_primary(),
                        );

                        // Columns 1..N: Variables values
                        for (v_idx, val_str) in row.values.iter().enumerate() {
                            let width = cols[v_idx + 1].1;
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
                            ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                            ui.painter().rect_stroke(
                                rect,
                                0.0_f32,
                                Stroke::new(0.5_f32, Tokens::border()),
                            );

                            if !row.is_header_only {
                                let pos = pos2(rect.max.x - 8.0_f32, rect.center().y);
                                ui.painter().text(
                                    pos,
                                    Align2::RIGHT_CENTER,
                                    val_str.as_str(),
                                    FontId::proportional(11.5),
                                    Tokens::text_primary(),
                                );
                            }
                        }
                    });
                }
            });
        });

    // Footnote if any variable had multiple modes
    let any_multiple_modes = results.iter().any(|r| {
        r.statistics
            .as_ref()
            .map(|s| s.has_multiple_modes)
            .unwrap_or(false)
    });
    if any_multiple_modes {
        ui.add_space(4.0_f32);
        ui.label(
            egui::RichText::new(t!("output-freq-mode-footnote"))
                .font(FontId::proportional(11.0))
                .color(Tokens::text_muted()),
        );
    }
}

pub fn stats_summary_to_tsv(results: &[FrequencyResult]) -> String {
    if results.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&t!("output-stats-title"));
    out.push('\n');

    // Header row
    out.push('\t');
    let var_names: Vec<&str> = results.iter().map(|r| r.display_label_or_name()).collect();
    out.push_str(&var_names.join("\t"));
    out.push('\n');

    let rows = build_summary_rows(results);
    for row in rows {
        out.push_str(&row.label);
        for val in &row.values {
            out.push('\t');
            if !row.is_header_only {
                out.push_str(val);
            }
        }
        out.push('\n');
    }

    let any_multiple_modes = results.iter().any(|r| {
        r.statistics
            .as_ref()
            .map(|s| s.has_multiple_modes)
            .unwrap_or(false)
    });
    if any_multiple_modes {
        out.push_str(&t!("output-freq-mode-footnote"));
        out.push('\n');
    }

    out
}

fn build_summary_rows(results: &[FrequencyResult]) -> Vec<RowDefinition> {
    let mut metrics = vec![StatMetric::NValid, StatMetric::NMissing];

    let has_mean = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.mean).is_some());
    let has_se_mean = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.se_mean).is_some());
    let has_median = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.median).is_some());
    let has_mode = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.mode).is_some());
    let has_std_dev = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.std_dev).is_some());
    let has_variance = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.variance).is_some());
    let has_skewness = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.skewness).is_some());
    let has_se_skew = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.se_skewness).is_some());
    let has_kurtosis = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.kurtosis).is_some());
    let has_se_kurt = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.se_kurtosis).is_some());
    let has_range = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.range).is_some());
    let has_min = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.min).is_some());
    let has_max = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.max).is_some());
    let has_sum = results
        .iter()
        .any(|r| r.statistics.as_ref().and_then(|s| s.sum).is_some());

    if has_mean {
        metrics.push(StatMetric::Mean);
    }
    if has_se_mean {
        metrics.push(StatMetric::SeMean);
    }
    if has_median {
        metrics.push(StatMetric::Median);
    }
    if has_mode {
        metrics.push(StatMetric::Mode);
    }
    if has_std_dev {
        metrics.push(StatMetric::StdDev);
    }
    if has_variance {
        metrics.push(StatMetric::Variance);
    }
    if has_skewness {
        metrics.push(StatMetric::Skewness);
    }
    if has_se_skew {
        metrics.push(StatMetric::SeSkewness);
    }
    if has_kurtosis {
        metrics.push(StatMetric::Kurtosis);
    }
    if has_se_kurt {
        metrics.push(StatMetric::SeKurtosis);
    }
    if has_range {
        metrics.push(StatMetric::Range);
    }
    if has_min {
        metrics.push(StatMetric::Min);
    }
    if has_max {
        metrics.push(StatMetric::Max);
    }
    if has_sum {
        metrics.push(StatMetric::Sum);
    }

    // Collect percentiles
    let mut p_vals: Vec<f64> = Vec::new();
    for res in results {
        if let Some(stats) = &res.statistics {
            for pv in &stats.percentiles {
                if !p_vals.iter().any(|&p| (p - pv.percentile).abs() < 1e-4) {
                    p_vals.push(pv.percentile);
                }
            }
        }
        if let Some((q1, q2, q3)) = res.quartiles {
            let _ = (q1, q2, q3);
            for &p in &[25.0, 50.0, 75.0] {
                if !p_vals.iter().any(|&x| (x - p).abs() < 1e-4) {
                    p_vals.push(p);
                }
            }
        }
    }
    p_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if !p_vals.is_empty() {
        metrics.push(StatMetric::PercentilesHeader);
        for p in p_vals {
            metrics.push(StatMetric::Percentile(p));
        }
    }

    let mut rows = Vec::with_capacity(metrics.len());
    for metric in metrics {
        let (label, is_header_only) = match &metric {
            StatMetric::NValid => (t!("output-stats-row-valid"), false),
            StatMetric::NMissing => (t!("output-stats-row-missing"), false),
            StatMetric::Mean => (t!("output-stats-row-mean"), false),
            StatMetric::SeMean => (t!("output-stats-row-se-mean"), false),
            StatMetric::Median => (t!("output-stats-row-median"), false),
            StatMetric::Mode => (t!("output-stats-row-mode"), false),
            StatMetric::StdDev => (t!("output-stats-row-std-dev"), false),
            StatMetric::Variance => (t!("output-stats-row-variance"), false),
            StatMetric::Skewness => (t!("output-stats-row-skewness"), false),
            StatMetric::SeSkewness => (t!("output-stats-row-se-skewness"), false),
            StatMetric::Kurtosis => (t!("output-stats-row-kurtosis"), false),
            StatMetric::SeKurtosis => (t!("output-stats-row-se-kurtosis"), false),
            StatMetric::Range => (t!("output-stats-row-range"), false),
            StatMetric::Min => (t!("output-stats-row-min"), false),
            StatMetric::Max => (t!("output-stats-row-max"), false),
            StatMetric::Sum => (t!("output-stats-row-sum"), false),
            StatMetric::PercentilesHeader => (t!("output-stats-row-percentiles"), true),
            StatMetric::Percentile(p) => {
                let p_str = if p.fract() == 0.0 {
                    format!("  {:.0}", p)
                } else {
                    format!("  {:.2}", p)
                };
                (p_str, false)
            }
        };

        let mut values = Vec::with_capacity(results.len());
        for res in results {
            let val = match &metric {
                StatMetric::NValid => format!("{}", res.n_valid),
                StatMetric::NMissing => format!("{}", res.n_missing),
                StatMetric::Mean => fmt_opt(res.statistics.as_ref().and_then(|s| s.mean)),
                StatMetric::SeMean => fmt_opt(res.statistics.as_ref().and_then(|s| s.se_mean)),
                StatMetric::Median => fmt_opt(res.statistics.as_ref().and_then(|s| s.median)),
                StatMetric::Mode => {
                    if let Some(stats) = &res.statistics {
                        if let Some(m) = stats.mode {
                            if stats.has_multiple_modes {
                                format!("{:.4} a", m)
                            } else {
                                format!("{:.4}", m)
                            }
                        } else {
                            ".".to_string()
                        }
                    } else {
                        ".".to_string()
                    }
                }
                StatMetric::StdDev => fmt_opt(res.statistics.as_ref().and_then(|s| s.std_dev)),
                StatMetric::Variance => fmt_opt(res.statistics.as_ref().and_then(|s| s.variance)),
                StatMetric::Skewness => fmt_opt(res.statistics.as_ref().and_then(|s| s.skewness)),
                StatMetric::SeSkewness => {
                    fmt_opt(res.statistics.as_ref().and_then(|s| s.se_skewness))
                }
                StatMetric::Kurtosis => fmt_opt(res.statistics.as_ref().and_then(|s| s.kurtosis)),
                StatMetric::SeKurtosis => {
                    fmt_opt(res.statistics.as_ref().and_then(|s| s.se_kurtosis))
                }
                StatMetric::Range => fmt_opt(res.statistics.as_ref().and_then(|s| s.range)),
                StatMetric::Min => fmt_opt(res.statistics.as_ref().and_then(|s| s.min)),
                StatMetric::Max => fmt_opt(res.statistics.as_ref().and_then(|s| s.max)),
                StatMetric::Sum => {
                    if let Some(s) = res.statistics.as_ref().and_then(|st| st.sum) {
                        if s.fract() == 0.0 {
                            format!("{:.0}", s)
                        } else {
                            format!("{:.4}", s)
                        }
                    } else {
                        ".".to_string()
                    }
                }
                StatMetric::PercentilesHeader => String::new(),
                StatMetric::Percentile(p) => {
                    let mut found = None;
                    if let Some(stats) = &res.statistics {
                        if let Some(pv) = stats
                            .percentiles
                            .iter()
                            .find(|pv| (pv.percentile - *p).abs() < 1e-4)
                        {
                            found = Some(pv.value);
                        }
                    }
                    if found.is_none() {
                        if let Some((q1, q2, q3)) = res.quartiles {
                            if (*p - 25.0).abs() < 1e-4 {
                                found = Some(q1);
                            } else if (*p - 50.0).abs() < 1e-4 {
                                found = Some(q2);
                            } else if (*p - 75.0).abs() < 1e-4 {
                                found = Some(q3);
                            }
                        }
                    }
                    fmt_opt(found)
                }
            };
            values.push(val);
        }

        rows.push(RowDefinition {
            label,
            is_header_only,
            values,
        });
    }

    rows
}

fn fmt_opt(opt: Option<f64>) -> String {
    opt.map(|v| format!("{:.4}", v))
        .unwrap_or_else(|| ".".to_string())
}
