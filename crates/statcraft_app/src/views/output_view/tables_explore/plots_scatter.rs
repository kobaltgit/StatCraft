use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, Rect, Sense, Stroke, Ui};
use statcraft_algo::{ExploreQqPlotData, SpreadLevelResult};
use statcraft_i18n::t;

pub(crate) fn render_qq_plot(ui: &mut Ui, title: &str, qq: &ExploreQqPlotData, detrended: bool) {
    let key = if detrended {
        "output-explore-detrended-qq"
    } else {
        "output-explore-normal-qq"
    };
    ui.strong(format!("{}: {}", t!(key), title));
    ui.add_space(4.0);

    let (rect, _) = ui.allocate_exact_size(vec2(460.0, 230.0), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 3.0, Tokens::bg_panel());
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, Tokens::border()));

    if qq.points.is_empty() {
        return;
    }

    let plot = Rect::from_min_max(
        pos2(rect.min.x + 56.0, rect.min.y + 18.0),
        pos2(rect.max.x - 20.0, rect.max.y - 34.0),
    );

    let pts: Vec<(f64, f64)> = qq
        .points
        .iter()
        .map(|p| {
            let y = if detrended {
                let z_obs = if qq.std_dev > 1e-12 {
                    (p.observed - qq.mean) / qq.std_dev
                } else {
                    0.0
                };
                z_obs - p.expected_normal
            } else {
                p.expected_normal
            };
            (p.observed, y)
        })
        .collect();

    render_scatter_with_line(
        ui, &painter, plot, &pts, !detrended, qq.mean, qq.std_dev, 0.0,
    );
}

pub(crate) fn render_spread_level_plot(ui: &mut Ui, sl: &SpreadLevelResult) {
    ui.strong(format!(
        "{}: {}",
        t!("output-explore-spread-level"),
        sl.variable
    ));
    ui.add_space(4.0);

    let pts: Vec<(f64, f64)> = sl
        .groups
        .iter()
        .filter_map(|g| match (g.ln_median, g.ln_iqr) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        })
        .collect();
    if pts.is_empty() {
        return;
    }

    let (rect, _) = ui.allocate_exact_size(vec2(460.0, 220.0), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 3.0, Tokens::bg_panel());
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, Tokens::border()));

    let plot = Rect::from_min_max(
        pos2(rect.min.x + 56.0, rect.min.y + 18.0),
        pos2(rect.max.x - 20.0, rect.max.y - 34.0),
    );
    let slope = sl.slope.unwrap_or(0.0);
    let mean_x = pts.iter().map(|p| p.0).sum::<f64>() / pts.len() as f64;
    let mean_y = pts.iter().map(|p| p.1).sum::<f64>() / pts.len() as f64;
    let intercept = mean_y - slope * mean_x;

    render_scatter_with_line(ui, &painter, plot, &pts, false, 0.0, 0.0, 0.0);

    if sl.slope.is_some() {
        let x_min = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let x_max = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let y_min = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let y_max = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let x_span = (x_max - x_min).max(1e-6);
        let y_span = (y_max - y_min).max(1e-6);
        let lx0 = x_min - x_span * 0.1;
        let lx1 = x_max + x_span * 0.1;
        let ly0 = slope * lx0 + intercept;
        let ly1 = slope * lx1 + intercept;
        let y_lo = y_min - y_span * 0.1;
        let y_hi = y_max + y_span * 0.1;
        let map = |x: f64, y: f64| {
            pos2(
                plot.min.x + ((x - lx0) / (lx1 - lx0)).clamp(0.0, 1.0) as f32 * plot.width(),
                plot.max.y - ((y - y_lo) / (y_hi - y_lo)).clamp(0.0, 1.0) as f32 * plot.height(),
            )
        };
        painter.line_segment(
            [map(lx0, ly0), map(lx1, ly1)],
            Stroke::new(1.5_f32, Tokens::text_muted()),
        );
    }

    if let (Some(slope), Some(power)) = (sl.slope, sl.power) {
        ui.label(
            egui::RichText::new(format!(
                "{}: {:.4}   {}: {:.4}",
                t!("output-explore-slope"),
                slope,
                t!("output-explore-power"),
                power
            ))
            .font(FontId::proportional(11.5))
            .color(Tokens::text_muted()),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_scatter_with_line(
    _ui: &mut Ui,
    painter: &egui::Painter,
    plot: Rect,
    pts: &[(f64, f64)],
    draw_qq_ref: bool,
    mean: f64,
    std_dev: f64,
    zero_y: f64,
) {
    let x_min = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let x_max = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let y_min = pts
        .iter()
        .map(|p| p.1)
        .fold(f64::INFINITY, f64::min)
        .min(zero_y);
    let y_max = pts
        .iter()
        .map(|p| p.1)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(zero_y);
    let x_span = (x_max - x_min).max(1e-6);
    let y_span = (y_max - y_min).max(1e-6);
    let x_lo = x_min - x_span * 0.1;
    let x_hi = x_max + x_span * 0.1;
    let y_lo = y_min - y_span * 0.1;
    let y_hi = y_max + y_span * 0.1;

    draw_axes_and_y_ticks(painter, plot, y_lo, y_hi, 4);
    draw_x_range_labels(painter, plot, x_lo, x_hi);

    let map = |x: f64, y: f64| {
        pos2(
            plot.min.x + ((x - x_lo) / (x_hi - x_lo)).clamp(0.0, 1.0) as f32 * plot.width(),
            plot.max.y - ((y - y_lo) / (y_hi - y_lo)).clamp(0.0, 1.0) as f32 * plot.height(),
        )
    };

    if draw_qq_ref && std_dev > 1e-12 {
        let y0 = (x_lo - mean) / std_dev;
        let y1 = (x_hi - mean) / std_dev;
        painter.line_segment(
            [map(x_lo, y0), map(x_hi, y1)],
            Stroke::new(1.2_f32, Tokens::text_muted()),
        );
    } else if !draw_qq_ref {
        painter.line_segment(
            [map(x_lo, zero_y), map(x_hi, zero_y)],
            Stroke::new(1.0_f32, Tokens::text_muted()),
        );
    }

    let accent = Tokens::accent();
    for &(x, y) in pts {
        painter.circle_filled(map(x, y), 3.0, accent);
    }
}

pub(crate) fn draw_axes_and_y_ticks(
    painter: &egui::Painter,
    plot: Rect,
    lo: f64,
    hi: f64,
    steps: usize,
) {
    let axis_stroke = Stroke::new(1.0_f32, Tokens::text_muted());
    painter.line_segment(
        [pos2(plot.min.x, plot.max.y), pos2(plot.max.x, plot.max.y)],
        axis_stroke,
    );
    painter.line_segment(
        [pos2(plot.min.x, plot.max.y), pos2(plot.min.x, plot.min.y)],
        axis_stroke,
    );
    for i in 0..=steps {
        let frac = i as f32 / steps as f32;
        let y = plot.max.y - frac * plot.height();
        let val = lo + (hi - lo) * frac as f64;
        painter.line_segment(
            [pos2(plot.min.x - 4.0, y), pos2(plot.min.x, y)],
            axis_stroke,
        );
        painter.text(
            pos2(plot.min.x - 6.0, y),
            Align2::RIGHT_CENTER,
            format!("{:.1}", val),
            FontId::proportional(10.0),
            Tokens::text_muted(),
        );
    }
}

pub(crate) fn draw_x_range_labels(painter: &egui::Painter, plot: Rect, x_min: f64, x_max: f64) {
    painter.text(
        pos2(plot.min.x, plot.max.y + 8.0),
        Align2::LEFT_TOP,
        format!("{:.1}", x_min),
        FontId::proportional(10.0),
        Tokens::text_muted(),
    );
    painter.text(
        pos2(plot.max.x, plot.max.y + 8.0),
        Align2::RIGHT_TOP,
        format!("{:.1}", x_max),
        FontId::proportional(10.0),
        Tokens::text_muted(),
    );
}
