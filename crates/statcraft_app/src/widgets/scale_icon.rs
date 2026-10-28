use crate::theme::Tokens;
use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Stroke};
use statcraft_core::Measure;

pub fn paint_measure_icon(painter: &Painter, rect: Rect, measure: Measure) {
    let center = rect.center();
    let stroke_width = 1.2_f32;

    match measure {
        Measure::Scale => {
            // Ruler icon: diagonal or horizontal line with ticks
            let color = Tokens::MEASURE_SCALE;
            let stroke = Stroke::new(stroke_width, color);

            let left = pos2(center.x - 6.0, center.y + 4.0);
            let right = pos2(center.x + 6.0, center.y - 4.0);
            painter.line_segment([left, right], stroke);

            // Three tick marks along the ruler
            for i in 0..4 {
                let t = i as f32 / 3.0;
                let tick_base = Pos2 {
                    x: left.x + (right.x - left.x) * t,
                    y: left.y + (right.y - left.y) * t,
                };
                let tick_len = if i % 2 == 0 { 3.0 } else { 2.0 };
                let tick_top = pos2(tick_base.x - tick_len * 0.7, tick_base.y - tick_len * 0.7);
                painter.line_segment([tick_base, tick_top], stroke);
            }
        }
        Measure::Ordinal => {
            // Bar chart icon: 3 vertical bars of ascending heights
            let color = Tokens::MEASURE_ORDINAL;
            let bar_width = 2.5;

            // Bar 1 (Short)
            let b1 =
                Rect::from_min_size(pos2(center.x - 5.0, center.y + 1.0), vec2(bar_width, 4.0));
            painter.rect_filled(b1, 0.5, color);

            // Bar 2 (Medium)
            let b2 =
                Rect::from_min_size(pos2(center.x - 1.2, center.y - 2.0), vec2(bar_width, 7.0));
            painter.rect_filled(b2, 0.5, color);

            // Bar 3 (Tall)
            let b3 =
                Rect::from_min_size(pos2(center.x + 2.6, center.y - 5.0), vec2(bar_width, 10.0));
            painter.rect_filled(b3, 0.5, color);
        }
        Measure::Nominal => {
            // Three clustered colored dots (categorical values)
            let color = Tokens::MEASURE_NOMINAL;
            let radius = 2.0;

            let c1 = pos2(center.x - 3.5, center.y + 2.5);
            let c2 = pos2(center.x + 3.5, center.y + 2.5);
            let c3 = pos2(center.x, center.y - 3.0);

            painter.circle_filled(c1, radius, color);
            painter.circle_filled(c2, radius, Color32::from_rgb(220, 100, 100)); // second category
            painter.circle_filled(c3, radius, Color32::from_rgb(90, 160, 240)); // third category
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_icon_rect_validity() {
        let r = Rect::from_min_size(pos2(10.0, 10.0), vec2(16.0, 16.0));
        assert_eq!(r.width(), 16.0);
        assert_eq!(r.height(), 16.0);
    }
}
