use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, Sense, Stroke, Ui};

pub fn adjust_widths(ui: &mut Ui, cols: &mut [(String, f32)]) {
    for (name, width) in cols {
        let text_w = ui
            .painter()
            .layout_no_wrap(
                name.clone(),
                FontId::proportional(12.0),
                Tokens::TEXT_PRIMARY,
            )
            .rect
            .width();
        *width = (*width).max(text_w + 16.0);
    }
}

pub fn render_header_row(ui: &mut Ui, cols: &[(String, f32)], row_height: f32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (col_name, width) in cols {
            let (rect, _) = ui.allocate_exact_size(vec2(*width, row_height), Sense::hover());
            ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
            ui.painter()
                .rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                col_name,
                FontId::proportional(11.5),
                Tokens::TEXT_PRIMARY,
            );
        }
    });
}

pub fn render_data_row(
    ui: &mut Ui,
    cols: &[(String, f32)],
    vals: &[(String, Align2)],
    row_height: f32,
    bg_color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (c_idx, (text, align)) in vals.iter().enumerate() {
            let width = cols.get(c_idx).map(|c| c.1).unwrap_or(80.0_f32);
            let (rect, _) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
            ui.painter().rect_filled(rect, 0.0_f32, bg_color);
            ui.painter()
                .rect_stroke(rect, 0.0_f32, Stroke::new(0.5_f32, Tokens::BORDER));

            let pos = if *align == Align2::LEFT_CENTER {
                pos2(rect.min.x + 6.0_f32, rect.center().y)
            } else if *align == Align2::CENTER_CENTER {
                rect.center()
            } else {
                pos2(rect.max.x - 6.0_f32, rect.center().y)
            };

            let clip = rect.shrink2(vec2(4.0_f32, 0.0_f32));
            ui.painter().with_clip_rect(clip).text(
                pos,
                *align,
                text.as_str(),
                FontId::proportional(11.5),
                Tokens::TEXT_PRIMARY,
            );
        }
    });
}

pub fn format_opt(val: Option<f64>) -> String {
    val.map(|v| format!("{:.4}", v))
        .unwrap_or_else(|| "-".to_string())
}

pub fn format_opt_2dec(val: Option<f64>) -> String {
    val.map(|v| format!("{:.2}", v))
        .unwrap_or_else(|| "-".to_string())
}

pub fn format_sig(val: Option<f64>) -> String {
    match val {
        Some(v) if v < 0.001 => "< .001".to_string(),
        Some(v) => format!("{:.3}", v),
        None => "-".to_string(),
    }
}
