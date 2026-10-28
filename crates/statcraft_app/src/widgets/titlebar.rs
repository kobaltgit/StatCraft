use crate::theme::Tokens;
use egui::{
    pos2, vec2, Color32, CursorIcon, Id, Pos2, Rect, ResizeDirection, Sense, Stroke, Ui,
    ViewportCommand,
};
use statcraft_i18n::t;

pub const CAPTION_BUTTON_WIDTH: f32 = 46.0;
pub const CAPTION_BAR_HEIGHT: f32 = 34.0;
const RESIZE_EDGE: f32 = 4.0;
const RESIZE_CORNER: f32 = 8.0;
const TITLE_GAP: f32 = 12.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WindowCaption {
    Minimize,
    Maximize,
    Close,
}

pub fn render_resize_zones(ctx: &egui::Context, is_maximized: bool) {
    if is_maximized {
        return;
    }
    let rect = ctx.screen_rect();
    let (e, c) = (RESIZE_EDGE, RESIZE_CORNER);

    let zones: [(Rect, ResizeDirection, CursorIcon); 8] = [
        (
            Rect::from_min_max(rect.left_top(), pos2(rect.right(), rect.top() + e)),
            ResizeDirection::North,
            CursorIcon::ResizeNorth,
        ),
        (
            Rect::from_min_max(pos2(rect.left(), rect.bottom() - e), rect.right_bottom()),
            ResizeDirection::South,
            CursorIcon::ResizeSouth,
        ),
        (
            Rect::from_min_max(rect.left_top(), pos2(rect.left() + e, rect.bottom())),
            ResizeDirection::West,
            CursorIcon::ResizeWest,
        ),
        (
            Rect::from_min_max(pos2(rect.right() - e, rect.top()), rect.right_bottom()),
            ResizeDirection::East,
            CursorIcon::ResizeEast,
        ),
        (
            Rect::from_min_size(rect.left_top(), vec2(c, c)),
            ResizeDirection::NorthWest,
            CursorIcon::ResizeNwSe,
        ),
        (
            Rect::from_min_size(rect.right_top() - vec2(c, 0.0), vec2(c, c)),
            ResizeDirection::NorthEast,
            CursorIcon::ResizeNeSw,
        ),
        (
            Rect::from_min_size(rect.left_bottom() - vec2(0.0, c), vec2(c, c)),
            ResizeDirection::SouthWest,
            CursorIcon::ResizeNeSw,
        ),
        (
            Rect::from_min_size(rect.right_bottom() - vec2(c, c), vec2(c, c)),
            ResizeDirection::SouthEast,
            CursorIcon::ResizeNwSe,
        ),
    ];

    egui::Area::new(Id::new("window_resize_zones"))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .movable(false)
        .sense(Sense::hover())
        .show(ctx, |ui| {
            for (i, (zone_rect, direction, cursor)) in zones.into_iter().enumerate() {
                let resp = ui.interact(zone_rect, Id::new("resize_zone").with(i), Sense::drag());
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(cursor);
                }
                if resp.drag_started_by(egui::PointerButton::Primary) {
                    ui.ctx()
                        .send_viewport_cmd(ViewportCommand::BeginResize(direction));
                }
            }
        });
}

pub fn render_caption_buttons(ui: &mut Ui, bar_rect: Rect, is_maximized: bool) {
    let captions = [
        WindowCaption::Minimize,
        WindowCaption::Maximize,
        WindowCaption::Close,
    ];
    let total_width = CAPTION_BUTTON_WIDTH * captions.len() as f32;
    let start_x = bar_rect.right() - total_width;

    for (i, &caption) in captions.iter().enumerate() {
        let btn_rect = Rect::from_min_size(
            pos2(start_x + i as f32 * CAPTION_BUTTON_WIDTH, bar_rect.top()),
            vec2(CAPTION_BUTTON_WIDTH, bar_rect.height()),
        );

        let resp = ui.interact(btn_rect, Id::new("caption_btn").with(i), Sense::click());

        let is_close = caption == WindowCaption::Close;
        let is_hovered = resp.hovered();
        let is_down = resp.is_pointer_button_down_on();

        let bg_color = match (is_down, is_hovered) {
            (true, _) if is_close => Tokens::CAPTION_CLOSE_PRESSED,
            (_, true) if is_close => Tokens::CAPTION_CLOSE_HOVER,
            (true, _) => Tokens::CAPTION_PRESSED,
            (_, true) => Tokens::CAPTION_HOVER,
            _ => Color32::TRANSPARENT,
        };

        if bg_color != Color32::TRANSPARENT {
            ui.painter().rect_filled(btn_rect, 0.0, bg_color);
        }

        let glyph_color = if is_close && is_hovered {
            Color32::WHITE
        } else {
            Tokens::TEXT_PRIMARY
        };

        paint_windows_glyph(
            ui,
            caption,
            is_maximized,
            btn_rect.center(),
            Stroke::new(1.0_f32, glyph_color),
        );

        let tip = match caption {
            WindowCaption::Minimize => t!("caption-minimize"),
            WindowCaption::Maximize if is_maximized => t!("caption-restore"),
            WindowCaption::Maximize => t!("caption-maximize"),
            WindowCaption::Close => t!("caption-close"),
        };
        let is_clicked = resp.clicked();
        resp.on_hover_text(tip);

        if is_clicked {
            match caption {
                WindowCaption::Minimize => {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                }
                WindowCaption::Maximize => {
                    ui.ctx()
                        .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
                }
                WindowCaption::Close => {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }
    }
}

pub fn paint_windows_glyph(
    ui: &Ui,
    caption: WindowCaption,
    is_maximized: bool,
    at: Pos2,
    stroke: Stroke,
) {
    let p = ui.painter();
    let glyph_rect = Rect::from_center_size(at, vec2(10.0, 10.0));

    match caption {
        WindowCaption::Minimize => {
            p.line_segment(
                [glyph_rect.left_center(), glyph_rect.right_center()],
                stroke,
            );
        }
        WindowCaption::Maximize if is_maximized => {
            let front = Rect::from_min_max(
                glyph_rect.min + vec2(0.0, 2.0),
                glyph_rect.max - vec2(2.0, 0.0),
            );
            p.rect_stroke(front, 0.0, stroke);
            p.line_segment(
                [glyph_rect.min + vec2(2.0, 0.0), glyph_rect.right_top()],
                stroke,
            );
            p.line_segment(
                [glyph_rect.right_top(), glyph_rect.max - vec2(0.0, 2.0)],
                stroke,
            );
        }
        WindowCaption::Maximize => {
            p.rect_stroke(glyph_rect, 0.0, stroke);
        }
        WindowCaption::Close => {
            p.line_segment([glyph_rect.left_top(), glyph_rect.right_bottom()], stroke);
            p.line_segment([glyph_rect.right_top(), glyph_rect.left_bottom()], stroke);
        }
    }
}

pub fn title_x(center_x: f32, left_bound: f32, right_bound: f32, text_width: f32) -> Option<f32> {
    let lo = left_bound + TITLE_GAP;
    let hi = right_bound - TITLE_GAP - text_width;
    if !lo.is_finite() || !hi.is_finite() || !text_width.is_finite() || hi < lo {
        return None;
    }
    if !center_x.is_finite() {
        return Some(lo);
    }
    Some((center_x - text_width / 2.0).clamp(lo, hi))
}
