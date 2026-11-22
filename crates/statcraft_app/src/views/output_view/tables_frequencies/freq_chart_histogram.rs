use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, Ui};
use statcraft_algo::frequencies::charts::{FrequencyChartResult, HistogramSummary};
use statcraft_i18n::t;

/// Renders a vector-first Histogram with optional Normal Curve in the Output Viewer.
pub fn render_histogram(
    ui: &mut Ui,
    var_name: &str,
    var_label: Option<&str>,
    summary: &HistogramSummary,
) {
    let title = if let Some(lbl) = var_label {
        if !lbl.trim().is_empty() {
            format!("{} [{}]", lbl.trim(), var_name)
        } else {
            var_name.to_string()
        }
    } else {
        var_name.to_string()
    };

    ui.strong(format!("{}: {}", t!("output-chart-histogram"), title));
    ui.add_space(6.0_f32);

    let chart_width = 540.0_f32;
    let chart_height = 290.0_f32;
    let (rect, _) = ui.allocate_exact_size(vec2(chart_width, chart_height), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);

    // Background & Border
    painter.rect_filled(rect, 3.0_f32, Tokens::bg_panel());
    painter.rect_stroke(rect, 3.0_f32, Stroke::new(1.0_f32, Tokens::border()));

    if summary.bins.is_empty() {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            t!("chart-no-data"),
            FontId::proportional(12.0),
            Tokens::text_muted(),
        );
        return;
    }

    let left_m = 65.0_f32;
    let right_m = 30.0_f32;
    let top_m = 35.0_f32;
    let bottom_m = 45.0_f32;

    let plot_rect = Rect::from_min_max(
        pos2(rect.min.x + left_m, rect.min.y + top_m),
        pos2(rect.max.x - right_m, rect.max.y - bottom_m),
    );

    let x_min = summary.bins.first().map(|b| b.left).unwrap_or(0.0);
    let x_max = summary.bins.last().map(|b| b.right).unwrap_or(1.0);
    let x_span = (x_max - x_min).max(1e-9);

    // Find maximum count for Y axis scaling
    let mut raw_max_count = 0.0_f64;
    for bin in &summary.bins {
        if (bin.count as f64) > raw_max_count {
            raw_max_count = bin.count as f64;
        }
    }
    // Also consider maximum height of normal curve if enabled
    if let Some(ref curve) = summary.normal_curve_points {
        for &(_, y) in curve {
            if y > raw_max_count {
                raw_max_count = y;
            }
        }
    }
    if raw_max_count <= 0.0 {
        raw_max_count = 10.0;
    }
    let nice_y_max = calculate_nice_axis_max(raw_max_count);

    // Y Axis label & ticks
    painter.text(
        pos2(plot_rect.min.x - 8.0_f32, rect.min.y + 12.0_f32),
        Align2::RIGHT_TOP,
        t!("chart-frequency"),
        FontId::proportional(11.0),
        Tokens::text_muted(),
    );

    for i in 0..=4 {
        let frac = i as f32 / 4.0_f32;
        let y_pos = plot_rect.max.y - frac * plot_rect.height();
        let val = nice_y_max * (frac as f64);

        painter.line_segment(
            [pos2(plot_rect.min.x, y_pos), pos2(plot_rect.max.x, y_pos)],
            Stroke::new(1.0_f32, Tokens::bg_row_even()),
        );

        let tick_str = if nice_y_max >= 10.0 {
            format!("{:.0}", val)
        } else {
            format!("{:.1}", val)
        };

        painter.text(
            pos2(plot_rect.min.x - 6.0_f32, y_pos),
            Align2::RIGHT_CENTER,
            tick_str,
            FontId::proportional(10.0),
            Tokens::text_muted(),
        );
    }

    // X Axis baseline
    painter.line_segment(
        [
            pos2(plot_rect.min.x, plot_rect.max.y),
            pos2(plot_rect.max.x, plot_rect.max.y),
        ],
        Stroke::new(1.2_f32, Tokens::border()),
    );

    // Render contiguous Histogram Bars
    let mouse_pos = ui.input(|i| i.pointer.hover_pos());

    for bin in &summary.bins {
        let b_left_x = plot_rect.min.x + (((bin.left - x_min) / x_span) as f32) * plot_rect.width();
        let b_right_x =
            plot_rect.min.x + (((bin.right - x_min) / x_span) as f32) * plot_rect.width();
        let h_ratio = ((bin.count as f64 / nice_y_max) as f32).clamp(0.0_f32, 1.0_f32);
        let bar_h = h_ratio * plot_rect.height();

        let bar_rect = Rect::from_min_max(
            pos2(b_left_x, plot_rect.max.y - bar_h),
            pos2(b_right_x, plot_rect.max.y),
        );

        let is_hovered = mouse_pos.map(|p| bar_rect.contains(p)).unwrap_or(false);
        let fill_color = if is_hovered {
            Color32::from_rgb(0x65, 0x9B, 0xD7)
        } else {
            Color32::from_rgb(0x4F, 0x81, 0xBD)
        };

        painter.rect_filled(bar_rect, 0.0_f32, fill_color);
        painter.rect_stroke(
            bar_rect,
            0.0_f32,
            Stroke::new(1.0_f32, Color32::from_rgb(0x3B, 0x69, 0x98)),
        );

        // Tick mark and midpoint label on X axis
        let mid_x =
            plot_rect.min.x + (((bin.midpoint - x_min) / x_span) as f32) * plot_rect.width();
        painter.line_segment(
            [
                pos2(mid_x, plot_rect.max.y),
                pos2(mid_x, plot_rect.max.y + 4.0_f32),
            ],
            Stroke::new(1.0_f32, Tokens::border()),
        );

        // Draw tick label (format based on bin width magnitude)
        let lbl_text = if summary.bin_width >= 10.0 {
            format!("{:.0}", bin.midpoint)
        } else if summary.bin_width >= 1.0 {
            format!("{:.1}", bin.midpoint)
        } else {
            format!("{:.2}", bin.midpoint)
        };

        painter.text(
            pos2(mid_x, plot_rect.max.y + 6.0_f32),
            Align2::CENTER_TOP,
            lbl_text,
            FontId::proportional(10.0),
            Tokens::text_primary(),
        );
    }

    // Normal Curve Overlay (if present)
    if let Some(ref curve_pts) = summary.normal_curve_points {
        if curve_pts.len() >= 2 {
            let curve_stroke = Stroke::new(2.0_f32, Color32::from_rgb(0xED, 0x7D, 0x31)); // SPSS Orange
            for window in curve_pts.windows(2) {
                let (x1, y1) = window[0];
                let (x2, y2) = window[1];

                let sx1 = plot_rect.min.x + (((x1 - x_min) / x_span) as f32) * plot_rect.width();
                let sy1 = (plot_rect.max.y - ((y1 / nice_y_max) as f32) * plot_rect.height())
                    .clamp(plot_rect.min.y, plot_rect.max.y);

                let sx2 = plot_rect.min.x + (((x2 - x_min) / x_span) as f32) * plot_rect.width();
                let sy2 = (plot_rect.max.y - ((y2 / nice_y_max) as f32) * plot_rect.height())
                    .clamp(plot_rect.min.y, plot_rect.max.y);

                painter.line_segment([pos2(sx1, sy1), pos2(sx2, sy2)], curve_stroke);
            }
        }
    }

    // Authentic SPSS Statistics Badge in upper-right corner
    let badge_w = 135.0_f32;
    let badge_h = 56.0_f32;
    let badge_rect = Rect::from_min_size(
        pos2(
            plot_rect.max.x - badge_w - 4.0_f32,
            plot_rect.min.y + 6.0_f32,
        ),
        vec2(badge_w, badge_h),
    );

    painter.rect_filled(badge_rect, 2.0_f32, Tokens::bg_panel());
    painter.rect_stroke(badge_rect, 2.0_f32, Stroke::new(1.0_f32, Tokens::border()));

    painter.text(
        pos2(badge_rect.min.x + 8.0_f32, badge_rect.min.y + 6.0_f32),
        Align2::LEFT_TOP,
        format!("{} = {:.2}", t!("chart-mean"), summary.mean),
        FontId::proportional(11.0),
        Tokens::text_primary(),
    );
    painter.text(
        pos2(badge_rect.min.x + 8.0_f32, badge_rect.min.y + 22.0_f32),
        Align2::LEFT_TOP,
        format!("{} = {:.3}", t!("chart-std-dev"), summary.std_dev),
        FontId::proportional(11.0),
        Tokens::text_primary(),
    );
    painter.text(
        pos2(badge_rect.min.x + 8.0_f32, badge_rect.min.y + 38.0_f32),
        Align2::LEFT_TOP,
        format!("{} = {}", t!("chart-n"), summary.n_valid),
        FontId::proportional(11.0),
        Tokens::text_primary(),
    );
}

/// Helper for Dispatching Histogram chart rendering.
pub fn render_histogram_chart(ui: &mut Ui, chart: &FrequencyChartResult) {
    if let FrequencyChartResult::Histogram {
        var_name,
        var_label,
        summary,
    } = chart
    {
        render_histogram(ui, var_name, var_label.as_deref(), summary);
    }
}

/// Formats Histogram chart as TSV.
pub fn histogram_chart_to_tsv(chart: &FrequencyChartResult) -> String {
    let mut out = String::new();
    if let FrequencyChartResult::Histogram {
        var_name, summary, ..
    } = chart
    {
        out.push_str(&format!("Histogram: {}\n", var_name));
        out.push_str(&format!(
            "Statistics\tMean={:.2}\tStdDev={:.3}\tN={}\n",
            summary.mean, summary.std_dev, summary.n_valid
        ));
        out.push_str("Bin Left\tBin Right\tMidpoint\tCount\tPercent\n");
        for b in &summary.bins {
            out.push_str(&format!(
                "{:.3}\t{:.3}\t{:.3}\t{}\t{:.2}%\n",
                b.left, b.right, b.midpoint, b.count, b.percent
            ));
        }
    }
    out
}

fn calculate_nice_axis_max(raw_max: f64) -> f64 {
    if raw_max <= 0.0 {
        return 1.0;
    }
    let exponent = raw_max.log10().floor();
    let frac = raw_max / 10.0_f64.powf(exponent);
    let nice_frac = if frac <= 1.0 {
        1.0
    } else if frac <= 2.0 {
        2.0
    } else if frac <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice_frac * 10.0_f64.powf(exponent)
}
