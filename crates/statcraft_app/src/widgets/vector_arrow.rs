use egui::{pos2, Color32, Painter, Rect, Stroke};

pub fn paint_vector_arrow(painter: &Painter, rect: Rect, right: bool, color: Color32) {
    let center = rect.center();
    let dx = 4.0_f32;
    let dy = 5.0_f32;

    if right {
        painter.line_segment(
            [
                pos2(center.x - dx, center.y - dy),
                pos2(center.x + dx, center.y),
            ],
            Stroke::new(2.0_f32, color),
        );
        painter.line_segment(
            [
                pos2(center.x + dx, center.y),
                pos2(center.x - dx, center.y + dy),
            ],
            Stroke::new(2.0_f32, color),
        );
    } else {
        painter.line_segment(
            [
                pos2(center.x + dx, center.y - dy),
                pos2(center.x - dx, center.y),
            ],
            Stroke::new(2.0_f32, color),
        );
        painter.line_segment(
            [
                pos2(center.x - dx, center.y),
                pos2(center.x + dx, center.y + dy),
            ],
            Stroke::new(2.0_f32, color),
        );
    }
}

pub fn paint_vector_chevron(painter: &Painter, rect: Rect, down: bool, color: Color32) {
    let center = rect.center();
    let dx = 3.5_f32;
    let dy = 2.5_f32;
    let stroke = Stroke::new(1.5_f32, color);

    if down {
        painter.line_segment(
            [
                pos2(center.x - dx, center.y - dy),
                pos2(center.x, center.y + dy),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y + dy),
                pos2(center.x + dx, center.y - dy),
            ],
            stroke,
        );
    } else {
        painter.line_segment(
            [
                pos2(center.x - dy, center.y - dx),
                pos2(center.x + dy, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x + dy, center.y),
                pos2(center.x - dy, center.y + dx),
            ],
            stroke,
        );
    }
}
