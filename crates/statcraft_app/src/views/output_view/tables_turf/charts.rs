use super::TurfOutputItem;
use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Ui};
use statcraft_i18n::t;

const CORAL_COLOR: Color32 = Color32::from_rgb(0xC0, 0x50, 0x4D);

pub fn render_reach_and_frequency_chart(ui: &mut Ui, item: &TurfOutputItem) {
    if !item.options.reach_and_frequency_chart {
        return;
    }

    let cycle0 = match item.result.cycles.first() {
        Some(c) => c,
        None => return,
    };

    if cycle0.rows_by_size.is_empty() {
        return;
    }

    ui.add_space(8.0);
    ui.strong(t!("output-turf-chart-title"));
    ui.add_space(4.0);

    // Extract best reach % and frequency % for each combination size
    let mut data_points: Vec<(usize, f64, f64)> = Vec::new();
    for (size, rows) in &cycle0.rows_by_size {
        if let Some(best_row) = rows.first() {
            data_points.push((*size, best_row.reach_pct, best_row.frequency_pct));
        }
    }

    if data_points.is_empty() {
        return;
    }

    let total_size = vec2(560.0, 300.0);
    let (resp, painter) = ui.allocate_painter(total_size, Sense::hover());
    let outer = resp.rect;

    // Background and border
    painter.rect_filled(outer, 4.0_f32, Tokens::bg_panel());
    painter.rect_stroke(outer, 4.0_f32, Stroke::new(1.0_f32, Tokens::border()));

    let pad_l = 60.0;
    let pad_r = 24.0;
    let pad_t = 36.0;
    let pad_b = 40.0;
    let plot_rect = Rect::from_min_max(
        pos2(outer.min.x + pad_l, outer.min.y + pad_t),
        pos2(outer.max.x - pad_r, outer.max.y - pad_b),
    );

    let k_min = data_points.first().map(|d| d.0).unwrap_or(1);
    let k_max = data_points.last().map(|d| d.0).unwrap_or(k_min);
    let k_span = if k_max > k_min {
        (k_max - k_min) as f32
    } else {
        1.0_f32
    };

    let y_min = 0.0_f32;
    let y_span = 100.0_f32;

    let to_screen = |k: usize, pct: f64| -> Pos2 {
        let x_norm = if k_max > k_min {
            (k - k_min) as f32 / k_span
        } else {
            0.5_f32
        };
        let y_norm = (pct as f32 - y_min) / y_span;
        pos2(
            plot_rect.min.x + x_norm * plot_rect.width(),
            plot_rect.max.y - y_norm.clamp(0.0, 1.0) * plot_rect.height(),
        )
    };

    // Horizontal grid lines and Y-axis tick labels (0%, 25%, 50%, 75%, 100%)
    let font_tick = FontId::proportional(10.5);
    for tick_pct in [0.0_f32, 25.0, 50.0, 75.0, 100.0] {
        let y_px = plot_rect.max.y - (tick_pct / y_span) * plot_rect.height();
        painter.line_segment(
            [pos2(plot_rect.min.x, y_px), pos2(plot_rect.max.x, y_px)],
            Stroke::new(0.5_f32, Tokens::border()),
        );
        painter.text(
            pos2(plot_rect.min.x - 6.0, y_px),
            Align2::RIGHT_CENTER,
            format!("{:.0}%", tick_pct),
            font_tick.clone(),
            Tokens::text_muted(),
        );
    }

    // Axes lines
    painter.line_segment(
        [
            pos2(plot_rect.min.x, plot_rect.min.y),
            pos2(plot_rect.min.x, plot_rect.max.y),
        ],
        Stroke::new(1.0_f32, Tokens::border()),
    );
    painter.line_segment(
        [
            pos2(plot_rect.min.x, plot_rect.max.y),
            pos2(plot_rect.max.x, plot_rect.max.y),
        ],
        Stroke::new(1.0_f32, Tokens::border()),
    );

    // X-axis ticks and labels
    for (k, _, _) in &data_points {
        let p = to_screen(*k, 0.0);
        painter.line_segment(
            [pos2(p.x, plot_rect.max.y), pos2(p.x, plot_rect.max.y + 4.0)],
            Stroke::new(1.0_f32, Tokens::border()),
        );
        painter.text(
            pos2(p.x, plot_rect.max.y + 12.0),
            Align2::CENTER_CENTER,
            k.to_string(),
            font_tick.clone(),
            Tokens::text_primary(),
        );
    }

    // Series 1: Reach (%) curve (Accent Blue)
    let reach_pts: Vec<Pos2> = data_points
        .iter()
        .map(|&(k, r_pct, _)| to_screen(k, r_pct))
        .collect();

    for w in reach_pts.windows(2) {
        painter.line_segment([w[0], w[1]], Stroke::new(2.5_f32, Tokens::accent()));
    }
    for &pt in &reach_pts {
        painter.circle_filled(pt, 4.0_f32, Tokens::accent());
        painter.circle_stroke(pt, 4.0_f32, Stroke::new(1.0_f32, Tokens::bg_panel()));
    }

    // Series 2: Frequency (%) curve (Coral)
    let freq_pts: Vec<Pos2> = data_points
        .iter()
        .map(|&(k, _, f_pct)| to_screen(k, f_pct))
        .collect();

    for w in freq_pts.windows(2) {
        painter.line_segment([w[0], w[1]], Stroke::new(1.8_f32, CORAL_COLOR));
    }
    for &pt in &freq_pts {
        let sq_rect = Rect::from_center_size(pt, vec2(6.0, 6.0));
        painter.rect_filled(sq_rect, 1.0_f32, CORAL_COLOR);
        painter.rect_stroke(sq_rect, 1.0_f32, Stroke::new(1.0_f32, Tokens::bg_panel()));
    }

    // Axis titles
    let font_title = FontId::proportional(11.0);
    painter.text(
        pos2(plot_rect.center().x, outer.max.y - 10.0),
        Align2::CENTER_CENTER,
        t!("output-turf-chart-size-axis"),
        font_title.clone(),
        Tokens::text_muted(),
    );
    painter.text(
        pos2(outer.min.x + 10.0, outer.min.y + 14.0),
        Align2::LEFT_CENTER,
        t!("output-turf-chart-pct-axis"),
        font_title.clone(),
        Tokens::text_muted(),
    );

    // Legend at top right
    let legend_y = outer.min.y + 14.0;
    let legend_r = outer.max.x - 24.0;

    // Series 2 legend (Frequency)
    let freq_lbl = t!("output-turf-chart-freq-legend");
    let freq_lbl_w = painter
        .layout_no_wrap(freq_lbl.clone(), font_tick.clone(), Tokens::text_primary())
        .rect
        .width();
    let freq_x = legend_r - freq_lbl_w;
    painter.text(
        pos2(freq_x, legend_y),
        Align2::LEFT_CENTER,
        freq_lbl,
        font_tick.clone(),
        Tokens::text_primary(),
    );
    let freq_icon_pos = pos2(freq_x - 12.0, legend_y);
    painter.line_segment(
        [
            pos2(freq_icon_pos.x - 6.0, legend_y),
            pos2(freq_icon_pos.x + 6.0, legend_y),
        ],
        Stroke::new(1.8_f32, CORAL_COLOR),
    );
    painter.rect_filled(
        Rect::from_center_size(freq_icon_pos, vec2(6.0, 6.0)),
        1.0_f32,
        CORAL_COLOR,
    );

    // Series 1 legend (Reach)
    let reach_lbl = t!("output-turf-chart-reach-legend");
    let reach_lbl_w = painter
        .layout_no_wrap(reach_lbl.clone(), font_tick.clone(), Tokens::text_primary())
        .rect
        .width();
    let reach_x = freq_x - 30.0 - reach_lbl_w;
    painter.text(
        pos2(reach_x, legend_y),
        Align2::LEFT_CENTER,
        reach_lbl,
        font_tick,
        Tokens::text_primary(),
    );
    let reach_icon_pos = pos2(reach_x - 12.0, legend_y);
    painter.line_segment(
        [
            pos2(reach_icon_pos.x - 6.0, legend_y),
            pos2(reach_icon_pos.x + 6.0, legend_y),
        ],
        Stroke::new(2.5_f32, Tokens::accent()),
    );
    painter.circle_filled(reach_icon_pos, 4.0_f32, Tokens::accent());
}
