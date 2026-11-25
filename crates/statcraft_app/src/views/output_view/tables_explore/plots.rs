use super::plots_scatter::{
    draw_axes_and_y_ticks, draw_x_range_labels, render_qq_plot, render_spread_level_plot,
};
use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, Ui};
use statcraft_algo::{
    ExploreBoxplotData, ExploreBoxplotDisplay, ExploreHistogramData, ExploreResult,
    StemLeafPlotData,
};
use statcraft_i18n::t;

pub(crate) fn render_explore_plots(ui: &mut Ui, res: &ExploreResult) {
    // 1. Boxplots
    if res.boxplot_mode != ExploreBoxplotDisplay::None {
        render_boxplots_section(ui, res);
    }

    // 2. Stem-and-Leaf & Histogram per group
    for var_res in &res.variables {
        for g in &var_res.groups {
            let Some(ref pdata) = g.plot_data else {
                continue;
            };
            let title = match &g.factor_level {
                Some(fv) => format!("{} ({})", var_res.variable, fv),
                None => var_res.variable.clone(),
            };

            if res.plot_stem_leaf {
                if let Some(ref sl) = pdata.stem_leaf {
                    render_stem_and_leaf(ui, &title, sl);
                    ui.add_space(12.0);
                }
            }
            if res.plot_histogram {
                if let Some(ref hist) = pdata.histogram {
                    render_explore_histogram(ui, &title, hist);
                    ui.add_space(12.0);
                }
            }
            if let Some(ref qq) = pdata.qq_plot {
                render_qq_plot(ui, &title, qq, false);
                ui.add_space(10.0);
                render_qq_plot(ui, &title, qq, true);
                ui.add_space(12.0);
            }
        }

        if let Some(ref sl_res) = var_res.spread_level {
            render_spread_level_plot(ui, sl_res);
            ui.add_space(12.0);
        }
    }
}

fn render_boxplots_section(ui: &mut Ui, res: &ExploreResult) {
    match res.boxplot_mode {
        ExploreBoxplotDisplay::None => {}
        ExploreBoxplotDisplay::FactorLevelsTogether => {
            for var_res in &res.variables {
                let mut entries: Vec<(String, &ExploreBoxplotData)> = Vec::new();
                for g in &var_res.groups {
                    if let Some(bp) = g.plot_data.as_ref().and_then(|p| p.boxplot.as_ref()) {
                        let lbl = g
                            .factor_level
                            .clone()
                            .unwrap_or_else(|| var_res.variable.clone());
                        entries.push((lbl, bp));
                    }
                }
                if !entries.is_empty() {
                    let heading = format!("{}: {}", t!("output-explore-boxplot"), var_res.variable);
                    render_boxplot_chart(ui, &heading, &entries);
                    ui.add_space(12.0);
                }
            }
        }
        ExploreBoxplotDisplay::DependentsTogether => {
            let mut entries: Vec<(String, &ExploreBoxplotData)> = Vec::new();
            for var_res in &res.variables {
                for g in &var_res.groups {
                    if let Some(bp) = g.plot_data.as_ref().and_then(|p| p.boxplot.as_ref()) {
                        let lbl = match &g.factor_level {
                            Some(fv) => format!("{} ({})", var_res.variable, fv),
                            None => var_res.variable.clone(),
                        };
                        entries.push((lbl, bp));
                    }
                }
            }
            if !entries.is_empty() {
                let heading = t!("output-explore-boxplot");
                render_boxplot_chart(ui, &heading, &entries);
                ui.add_space(12.0);
            }
        }
    }
}

fn render_boxplot_chart(ui: &mut Ui, title: &str, entries: &[(String, &ExploreBoxplotData)]) {
    ui.strong(title);
    ui.add_space(4.0);

    let width = 520.0_f32
        .max((entries.len() as f32) * 85.0 + 90.0)
        .min(760.0);
    let height = 260.0_f32;
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);

    painter.rect_filled(rect, 3.0, Tokens::bg_panel());
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, Tokens::border()));

    let plot = Rect::from_min_max(
        pos2(rect.min.x + 60.0, rect.min.y + 20.0),
        pos2(rect.max.x - 20.0, rect.max.y - 36.0),
    );

    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for (_, bp) in entries {
        y_min = y_min.min(bp.whisker_low).min(bp.q1);
        y_max = y_max.max(bp.whisker_high).max(bp.q3);
        for o in &bp.outliers {
            y_min = y_min.min(o.value);
            y_max = y_max.max(o.value);
        }
    }
    if !y_min.is_finite() || !y_max.is_finite() {
        return;
    }
    let span = (y_max - y_min).max(1e-6);
    let pad = span * 0.08;
    let lo = y_min - pad;
    let hi = y_max + pad;
    let map_y = |v: f64| -> f32 {
        let t_val = ((v - lo) / (hi - lo)).clamp(0.0, 1.0) as f32;
        plot.max.y - t_val * plot.height()
    };

    draw_axes_and_y_ticks(&painter, plot, lo, hi, 4);

    let slot_w = plot.width() / entries.len() as f32;
    let box_w = (slot_w * 0.45).clamp(16.0, 48.0);
    let accent = Tokens::accent();
    let fill = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 65);
    let stroke = Stroke::new(1.5_f32, accent);

    for (idx, (lbl, bp)) in entries.iter().enumerate() {
        let cx = plot.min.x + slot_w * (idx as f32 + 0.5);
        let y_q1 = map_y(bp.q1);
        let y_med = map_y(bp.median);
        let y_q3 = map_y(bp.q3);
        let y_wl = map_y(bp.whisker_low);
        let y_wh = map_y(bp.whisker_high);

        // Whiskers
        painter.line_segment([pos2(cx, y_wh), pos2(cx, y_q3)], stroke);
        painter.line_segment([pos2(cx, y_q1), pos2(cx, y_wl)], stroke);
        let cap_w = box_w * 0.45;
        painter.line_segment([pos2(cx - cap_w, y_wh), pos2(cx + cap_w, y_wh)], stroke);
        painter.line_segment([pos2(cx - cap_w, y_wl), pos2(cx + cap_w, y_wl)], stroke);

        // Box
        let box_rect =
            Rect::from_min_max(pos2(cx - box_w * 0.5, y_q3), pos2(cx + box_w * 0.5, y_q1));
        painter.rect_filled(box_rect, 0.0, fill);
        painter.rect_stroke(box_rect, 0.0, stroke);
        painter.line_segment(
            [pos2(box_rect.min.x, y_med), pos2(box_rect.max.x, y_med)],
            Stroke::new(2.2_f32, Tokens::text_primary()),
        );

        // Outliers (circle) & Extremes (vector cross)
        for o in &bp.outliers {
            let oy = map_y(o.value);
            if o.is_extreme {
                let d = 3.5_f32;
                painter.line_segment([pos2(cx - d, oy - d), pos2(cx + d, oy + d)], stroke);
                painter.line_segment([pos2(cx - d, oy + d), pos2(cx + d, oy - d)], stroke);
            } else {
                painter.circle_stroke(pos2(cx, oy), 3.5, stroke);
            }
        }

        painter.text(
            pos2(cx, plot.max.y + 8.0),
            Align2::CENTER_TOP,
            lbl,
            FontId::proportional(10.5),
            Tokens::text_primary(),
        );
    }
}

fn render_stem_and_leaf(ui: &mut Ui, title: &str, sl: &StemLeafPlotData) {
    ui.strong(format!("{}: {}", t!("output-explore-stem-leaf"), title));
    ui.add_space(4.0);

    let mono = FontId::monospace(11.5);
    egui::Frame::none()
        .fill(Tokens::bg_panel())
        .stroke(Stroke::new(1.0_f32, Tokens::border()))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{:>6} | {} & {}",
                    t!("output-explore-stem-freq"),
                    t!("output-explore-stem"),
                    t!("output-explore-leaf")
                ))
                .font(mono.clone())
                .color(Tokens::text_muted()),
            );
            for row in &sl.rows {
                ui.label(
                    egui::RichText::new(format!(
                        "{:>6} | {:>4} . {}",
                        row.frequency, row.stem, row.leaves
                    ))
                    .font(mono.clone())
                    .color(Tokens::text_primary()),
                );
            }
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!(
                    "{}: {:.4}   {}: {:.4}",
                    t!("output-explore-stem-width"),
                    sl.stem_width,
                    t!("output-explore-leaf-unit"),
                    sl.leaf_unit
                ))
                .font(FontId::proportional(11.0))
                .color(Tokens::text_muted()),
            );
        });
}

fn render_explore_histogram(ui: &mut Ui, title: &str, hist: &ExploreHistogramData) {
    ui.strong(format!("{}: {}", t!("output-chart-histogram"), title));
    ui.add_space(4.0);

    let (rect, _) = ui.allocate_exact_size(vec2(520.0, 240.0), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 3.0, Tokens::bg_panel());
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, Tokens::border()));

    if hist.bins.is_empty() {
        return;
    }

    let plot = Rect::from_min_max(
        pos2(rect.min.x + 56.0, rect.min.y + 18.0),
        pos2(rect.max.x - 20.0, rect.max.y - 34.0),
    );
    let max_cnt = hist.bins.iter().map(|b| b.count).max().unwrap_or(1).max(1) as f64;
    draw_axes_and_y_ticks(&painter, plot, 0.0, max_cnt, 4);

    let x_min = hist.bins.first().map(|b| b.left).unwrap_or(0.0);
    let x_max = hist.bins.last().map(|b| b.right).unwrap_or(1.0);
    let x_span = (x_max - x_min).max(1e-9);

    let accent = Tokens::accent();
    let fill = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 130);
    for b in &hist.bins {
        let x0 = plot.min.x + ((b.left - x_min) / x_span) as f32 * plot.width();
        let x1 = plot.min.x + ((b.right - x_min) / x_span) as f32 * plot.width();
        let h = (b.count as f64 / max_cnt) as f32 * plot.height();
        let bar = Rect::from_min_max(pos2(x0, plot.max.y - h), pos2(x1, plot.max.y));
        painter.rect_filled(bar, 0.0, fill);
        painter.rect_stroke(bar, 0.0, Stroke::new(1.0_f32, accent));
    }

    draw_x_range_labels(&painter, plot, x_min, x_max);
}
