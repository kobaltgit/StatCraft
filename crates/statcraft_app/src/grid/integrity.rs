use egui::{pos2, Color32, Painter, Pos2, Rect};
use statcraft_core::RawValue;

/// Checks if a cell contains a valid number stored as a string.
pub fn is_number_stored_as_text(val: &RawValue) -> bool {
    let s = match val {
        RawValue::Text(t) => t.trim(),
        _ => return false,
    };

    if s.is_empty() {
        return false;
    }

    // Replace comma with dot if locale uses comma for decimals
    let normalized = s.replace(',', ".");

    if let Ok(num) = normalized.parse::<f64>() {
        // Exclude NaN and infinite values
        !num.is_nan() && num.is_finite()
    } else {
        false
    }
}

/// Converts a text cell containing a number into `RawValue::Number`.
pub fn convert_text_to_number(val: &RawValue) -> Option<RawValue> {
    let s = match val {
        RawValue::Text(t) => t.trim(),
        _ => return None,
    };

    let normalized = s.replace(',', ".");
    if let Ok(num) = normalized.parse::<f64>() {
        if !num.is_nan() && num.is_finite() {
            Some(RawValue::Number(num))
        } else {
            None
        }
    } else {
        None
    }
}

/// Renders Excel-style small green triangle badge in the top-left corner of the cell.
pub fn paint_integrity_badge(painter: &Painter, cell_rect: Rect) {
    let size = 6.0_f32;
    let badge_color = Color32::from_rgb(0, 160, 60);

    let p0 = cell_rect.min;
    let p1 = pos2(cell_rect.min.x + size, cell_rect.min.y);
    let p2 = pos2(cell_rect.min.x, cell_rect.min.y + size);

    painter.add(egui::Shape::convex_polygon(
        vec![p0, p1, p2],
        badge_color,
        egui::Stroke::NONE,
    ));
}

/// Hit-tests whether pointer position is on the top-left integrity badge.
pub fn hit_test_integrity_badge(cell_rect: Rect, pos: Pos2) -> bool {
    let badge_rect = Rect::from_min_size(cell_rect.min, egui::vec2(8.0, 8.0));
    badge_rect.contains(pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_stored_as_text_detection() {
        assert!(is_number_stored_as_text(&RawValue::Text(" 123.45 ".into())));
        assert!(is_number_stored_as_text(&RawValue::Text("-99.9 ".into())));
        assert!(is_number_stored_as_text(&RawValue::Text("12,34".into())));
        assert!(is_number_stored_as_text(&RawValue::Text("0".into())));

        // Real text should not be flagged
        assert!(!is_number_stored_as_text(&RawValue::Text("Hello".into())));
        assert!(!is_number_stored_as_text(&RawValue::Text("".into())));
        assert!(!is_number_stored_as_text(&RawValue::Text("12a".into())));
        assert!(!is_number_stored_as_text(&RawValue::Number(12.0)));
        assert!(!is_number_stored_as_text(&RawValue::Null));
    }

    #[test]
    fn test_convert_text_to_number() {
        let text_val = RawValue::Text(" 42.5 ".into());
        let converted = convert_text_to_number(&text_val).expect("must convert");
        assert_eq!(converted, RawValue::Number(42.5));

        let comma_val = RawValue::Text("10,75".into());
        let converted2 = convert_text_to_number(&comma_val).expect("must convert");
        assert_eq!(converted2, RawValue::Number(10.75));

        let invalid = RawValue::Text("invalid".into());
        assert_eq!(convert_text_to_number(&invalid), None);
    }

    #[test]
    fn test_integrity_badge_hit_test() {
        let rect = Rect::from_min_size(pos2(100.0, 50.0), egui::vec2(80.0, 24.0));
        assert!(hit_test_integrity_badge(rect, pos2(101.0, 51.0)));
        assert!(hit_test_integrity_badge(rect, pos2(107.0, 57.0)));
        assert!(!hit_test_integrity_badge(rect, pos2(120.0, 60.0)));
    }
}
