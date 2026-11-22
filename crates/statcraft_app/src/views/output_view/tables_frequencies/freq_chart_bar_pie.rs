use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, Ui};
use statcraft_algo::frequencies::charts::{BarCategoryData, FrequencyChartResult};
use statcraft_i18n::t;
use std::f32::consts::PI;

const PIE_PALETTE: [Color32; 10] = [
    Color32::from_rgb(0x4F, 0x81, 0xBD), // SPSS Blue
    Color32::from_rgb(0xC0, 0x50, 0x4D), // Coral
    Color32::from_rgb(0x9B, 0xBB, 0x59), // Olive/Green
    Color32::from_rgb(0x80, 0x64, 0xA2), // Purple
    Color32::from_rgb(0x4B, 0xAC, 0xC6), // Cyan
    Color32::from_rgb(0xF7, 0x96, 0x46), // Amber
    Color32::from_rgb(0x71, 0x84, 0x99), // Slate
    Color32::from_rgb(0x8F, 0xAA, 0x4C), // Light Green
    Color32::from_rgb(0xD9, 0x7A, 0x53), // Terracotta
    Color32::from_rgb(0x60, 0x7D, 0x8B), // Steel
];

/// Renders a vector-first Bar Chart in the Output Viewer.
pub fn render_bar_chart(
    ui: &mut Ui,
    var_name: &str,
    var_label: Option<&str>,
    is_percentages: bool,
    categories: &[BarCategoryData],
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

    ui.strong(format!("{}: {}", t!("output-chart-bar"), title));
    ui.add_space(6.0_f32);

    let chart_width = 540.0_f32;
    let chart_height = 280.0_f32;
    let (rect, _) = ui.allocate_exact_size(vec2(chart_width, chart_height), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);

    // Background & Border
    painter.rect_filled(rect, 3.0_f32, Tokens::bg_panel());
    painter.rect_stroke(rect, 3.0_f32, Stroke::new(1.0_f32, Tokens::border()));

    if categories.is_empty() {
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

    // Max value calculation for Y axis
    let mut raw_max = 0.0_f64;
    for cat in categories {
        let v = if is_percentages {
            cat.percent
        } else {
            cat.count
        };
        if v > raw_max {
            raw_max = v;
        }
    }
    if raw_max <= 0.0 {
        raw_max = if is_percentages { 100.0 } else { 10.0 };
    }
    let nice_y_max = calculate_nice_axis_max(raw_max);

    // Y Axis ticks and gridlines (5 ticks: 0%, 25%, 50%, 75%, 100% of max)
    let y_label = if is_percentages {
        t!("chart-percent")
    } else {
        t!("chart-frequency")
    };
    painter.text(
        pos2(plot_rect.min.x - 8.0_f32, rect.min.y + 12.0_f32),
        Align2::RIGHT_TOP,
        y_label,
        FontId::proportional(11.0),
        Tokens::text_muted(),
    );

    for i in 0..=4 {
        let frac = i as f32 / 4.0_f32;
        let y_pos = plot_rect.max.y - frac * plot_rect.height();
        let val = nice_y_max * (frac as f64);

        // Gridline
        painter.line_segment(
            [pos2(plot_rect.min.x, y_pos), pos2(plot_rect.max.x, y_pos)],
            Stroke::new(1.0_f32, Tokens::bg_row_even()),
        );

        let tick_str = if is_percentages {
            format!("{:.0}%", val)
        } else if nice_y_max >= 10.0 {
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

    // Draw Bars
    let n_cats = categories.len();
    let slot_w = plot_rect.width() / n_cats as f32;
    let bar_w = (slot_w * 0.65_f32).clamp(6.0_f32, 45.0_f32);
    let mouse_pos = ui.input(|i| i.pointer.hover_pos());

    for (i, cat) in categories.iter().enumerate() {
        let center_x = plot_rect.min.x + (i as f32 + 0.5_f32) * slot_w;
        let v = if is_percentages {
            cat.percent
        } else {
            cat.count
        };
        let h_ratio = ((v / nice_y_max) as f32).clamp(0.0_f32, 1.0_f32);
        let bar_h = h_ratio * plot_rect.height();

        let bar_rect = Rect::from_min_max(
            pos2(center_x - bar_w / 2.0_f32, plot_rect.max.y - bar_h),
            pos2(center_x + bar_w / 2.0_f32, plot_rect.max.y),
        );

        let is_hovered = mouse_pos.map(|p| bar_rect.contains(p)).unwrap_or(false);
        let bar_color = if is_hovered {
            Color32::from_rgb(0x65, 0x9B, 0xD7)
        } else {
            Color32::from_rgb(0x4F, 0x81, 0xBD)
        };

        painter.rect_filled(bar_rect, 0.0_f32, bar_color);
        painter.rect_stroke(
            bar_rect,
            0.0_f32,
            Stroke::new(1.0_f32, Color32::from_rgb(0x3B, 0x69, 0x98)),
        );

        // Value text above bar
        let val_text = if is_percentages {
            format!("{:.1}%", cat.percent)
        } else {
            format!("{:.0}", cat.count)
        };
        painter.text(
            pos2(center_x, plot_rect.max.y - bar_h - 3.0_f32),
            Align2::CENTER_BOTTOM,
            val_text,
            FontId::proportional(10.0),
            Tokens::text_muted(),
        );

        // Category label below X axis
        let display_lbl = if !cat.label.trim().is_empty() {
            &cat.label
        } else {
            &cat.value_repr
        };
        let truncated = truncate_label(display_lbl, 14);
        painter.text(
            pos2(center_x, plot_rect.max.y + 6.0_f32),
            Align2::CENTER_TOP,
            truncated,
            FontId::proportional(11.0),
            Tokens::text_primary(),
        );
    }
}

/// Renders a vector-first Pie Chart in the Output Viewer.
pub fn render_pie_chart(
    ui: &mut Ui,
    var_name: &str,
    var_label: Option<&str>,
    is_percentages: bool,
    slices: &[BarCategoryData],
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

    ui.strong(format!("{}: {}", t!("output-chart-pie"), title));
    ui.add_space(6.0_f32);

    let chart_width = 540.0_f32;
    let chart_height = 280.0_f32;
    let (rect, _) = ui.allocate_exact_size(vec2(chart_width, chart_height), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);

    painter.rect_filled(rect, 3.0_f32, Tokens::bg_panel());
    painter.rect_stroke(rect, 3.0_f32, Stroke::new(1.0_f32, Tokens::border()));

    if slices.is_empty() {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            t!("chart-no-data"),
            FontId::proportional(12.0),
            Tokens::text_muted(),
        );
        return;
    }

    let pie_center = pos2(rect.min.x + 140.0_f32, rect.min.y + 145.0_f32);
    let pie_radius = 92.0_f32;

    let total: f64 = slices.iter().map(|s| s.count).sum();
    let safe_total = if total > 0.0 { total } else { 1.0 };

    let mut start_angle = -PI / 2.0_f32; // 12 o'clock start

    for (i, slice) in slices.iter().enumerate() {
        let frac = (slice.count / safe_total) as f32;
        let sweep = frac * 2.0_f32 * PI;
        let end_angle = start_angle + sweep;
        let color = PIE_PALETTE[i % PIE_PALETTE.len()];

        // Draw sector polygon with arc interpolation
        let steps = ((sweep / (PI / 18.0_f32)).ceil() as usize).max(4);
        let mut points = Vec::with_capacity(steps + 2);
        points.push(pie_center);

        for step in 0..=steps {
            let a = start_angle + (step as f32 / steps as f32) * sweep;
            points.push(pos2(
                pie_center.x + pie_radius * a.cos(),
                pie_center.y + pie_radius * a.sin(),
            ));
        }

        painter.add(egui::Shape::convex_polygon(
            points,
            color,
            Stroke::new(1.0_f32, Tokens::bg_panel()),
        ));

        start_angle = end_angle;
    }

    // Legend on right side
    let legend_x = rect.min.x + 270.0_f32;
    let legend_start_y = rect.min.y + 35.0_f32;
    let item_h = 21.0_f32;

    for (i, slice) in slices.iter().take(10).enumerate() {
        let item_y = legend_start_y + (i as f32) * item_h;
        let swatch_rect = Rect::from_min_size(pos2(legend_x, item_y + 3.0_f32), vec2(11.0, 11.0));
        let color = PIE_PALETTE[i % PIE_PALETTE.len()];

        painter.rect_filled(swatch_rect, 1.0_f32, color);
        painter.rect_stroke(swatch_rect, 1.0_f32, Stroke::new(1.0_f32, Tokens::border()));

        let display_lbl = if !slice.label.trim().is_empty() {
            &slice.label
        } else {
            &slice.value_repr
        };
        let text = if is_percentages {
            format!(
                "{} ({:.1}%)",
                truncate_label(display_lbl, 20),
                slice.percent
            )
        } else {
            format!("{} (N={:.0})", truncate_label(display_lbl, 20), slice.count)
        };

        painter.text(
            pos2(legend_x + 18.0_f32, item_y + 2.0_f32),
            Align2::LEFT_TOP,
            text,
            FontId::proportional(11.0),
            Tokens::text_primary(),
        );
    }

    if slices.len() > 10 {
        let extra_y = legend_start_y + 10.0_f32 * item_h;
        painter.text(
            pos2(legend_x, extra_y),
            Align2::LEFT_TOP,
            format!("... +{} more", slices.len() - 10),
            FontId::proportional(11.0),
            Tokens::text_muted(),
        );
    }
}

/// Helper for Dispatching Bar / Pie chart rendering.
pub fn render_bar_or_pie_chart(ui: &mut Ui, chart: &FrequencyChartResult) {
    match chart {
        FrequencyChartResult::Bar {
            var_name,
            var_label,
            is_percentages,
            categories,
        } => {
            render_bar_chart(
                ui,
                var_name,
                var_label.as_deref(),
                *is_percentages,
                categories,
            );
        }
        FrequencyChartResult::Pie {
            var_name,
            var_label,
            is_percentages,
            slices,
        } => {
            render_pie_chart(ui, var_name, var_label.as_deref(), *is_percentages, slices);
        }
        _ => {}
    }
}

/// Formats Bar / Pie chart as TSV.
pub fn bar_or_pie_chart_to_tsv(chart: &FrequencyChartResult) -> String {
    let mut out = String::new();
    match chart {
        FrequencyChartResult::Bar {
            var_name,
            categories,
            is_percentages,
            ..
        } => {
            out.push_str(&format!("Bar Chart: {}\n", var_name));
            out.push_str("Category\tCount\tPercent\n");
            for c in categories {
                let lbl = if !c.label.trim().is_empty() {
                    &c.label
                } else {
                    &c.value_repr
                };
                out.push_str(&format!("{}\t{:.0}\t{:.2}%\n", lbl, c.count, c.percent));
            }
            let _ = is_percentages;
        }
        FrequencyChartResult::Pie {
            var_name,
            slices,
            is_percentages,
            ..
        } => {
            out.push_str(&format!("Pie Chart: {}\n", var_name));
            out.push_str("Slice\tCount\tPercent\n");
            for s in slices {
                let lbl = if !s.label.trim().is_empty() {
                    &s.label
                } else {
                    &s.value_repr
                };
                out.push_str(&format!("{}\t{:.0}\t{:.2}%\n", lbl, s.count, s.percent));
            }
            let _ = is_percentages;
        }
        _ => {}
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

fn truncate_label(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_chars.saturating_sub(2)).collect();
        truncated.push_str("..");
        truncated
    }
}
