use egui::{pos2, vec2, Color32, Pos2, Rect, Sense, Stroke, Ui};
use statcraft_core::WorkspaceManager;
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentTabAction {
    None,
    Select(usize),
    Close(usize),
    NewBlank,
}

pub struct DocumentTabStrip;

impl DocumentTabStrip {
    pub const HEIGHT: f32 = 28.0;

    /// Renders the horizontal vector document tab strip above the data grid.
    pub fn render(
        ui: &mut Ui,
        workspace: &WorkspaceManager,
        scroll_offset: &mut f32,
    ) -> DocumentTabAction {
        let mut action = DocumentTabAction::None;
        let palette = crate::theme::palette();

        let (strip_rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::hover());

        // Strip background
        ui.painter().rect_filled(strip_rect, 0.0, palette.bg_panel);
        // Bottom divider
        ui.painter().line_segment(
            [
                pos2(strip_rect.min.x, strip_rect.max.y),
                pos2(strip_rect.max.x, strip_rect.max.y),
            ],
            Stroke::new(1.0_f32, palette.border),
        );

        let sessions = workspace.sessions();
        if sessions.is_empty() {
            return action;
        }

        let nav_btn_width = 20.0;
        let plus_btn_width = 26.0;
        let right_reserved = plus_btn_width + nav_btn_width * 2.0 + 8.0;

        let content_max_x = strip_rect.max.x - right_reserved;
        let tabs_area_rect = Rect::from_min_max(
            pos2(strip_rect.min.x + 4.0, strip_rect.min.y),
            pos2(content_max_x, strip_rect.max.y),
        );

        // Compute total width needed for all tabs
        let tab_height = Self::HEIGHT - 2.0;
        let mut tab_widths: Vec<f32> = Vec::with_capacity(sessions.len());
        let font_id = egui::FontId::proportional(12.0);

        for session in sessions {
            let label = if session.is_dirty {
                format!("{} *", session.file_name())
            } else {
                session.file_name().to_string()
            };
            let galley = ui
                .painter()
                .layout_no_wrap(label, font_id.clone(), palette.text_primary);
            // icon (14) + gap (6) + text + close btn (14) + padding (16)
            let w = (galley.size().x + 50.0).clamp(90.0, 220.0);
            tab_widths.push(w);
        }

        let total_tabs_width: f32 =
            tab_widths.iter().sum::<f32>() + (tab_widths.len() as f32 * 2.0);
        let visible_width = tabs_area_rect.width();
        let max_scroll = (total_tabs_width - visible_width).max(0.0);

        // Clamp scroll offset
        *scroll_offset = scroll_offset.clamp(0.0, max_scroll);

        // Clip tab painting to the tabs area
        let painter = ui.painter_at(tabs_area_rect);
        let mut cur_x = tabs_area_rect.min.x - *scroll_offset;

        for (i, session) in sessions.iter().enumerate() {
            let w = tab_widths[i];
            let tab_rect =
                Rect::from_min_size(pos2(cur_x, strip_rect.min.y + 1.0), vec2(w, tab_height));
            cur_x += w + 2.0;

            if tab_rect.max.x < tabs_area_rect.min.x || tab_rect.min.x > tabs_area_rect.max.x {
                continue;
            }

            let is_active = Some(session.id) == workspace.active_session_id();
            let tab_id = ui.make_persistent_id(format!("doc_tab_{}", session.id));
            let tab_resp = ui.interact(tab_rect, tab_id, Sense::click());

            let ds_color = session
                .tab_color
                .as_deref()
                .and_then(crate::theme::hex_to_color32)
                .unwrap_or_else(|| crate::theme::dataset_color(session.file_name()));

            // Tab background & active underline
            let bg_color = if is_active {
                crate::theme::color_with_alpha(ds_color, 25)
            } else if tab_resp.hovered() {
                palette.caption_hover
            } else {
                palette.bg_panel
            };

            painter.rect_filled(
                tab_rect,
                egui::Rounding {
                    nw: 3.0,
                    ne: 3.0,
                    sw: 0.0,
                    se: 0.0,
                },
                bg_color,
            );
            painter.rect_stroke(
                tab_rect,
                egui::Rounding {
                    nw: 3.0,
                    ne: 3.0,
                    sw: 0.0,
                    se: 0.0,
                },
                Stroke::new(1.0_f32, palette.border),
            );

            if is_active {
                painter.line_segment(
                    [
                        pos2(tab_rect.min.x + 1.0, tab_rect.max.y),
                        pos2(tab_rect.max.x - 1.0, tab_rect.max.y),
                    ],
                    Stroke::new(2.0_f32, ds_color),
                );
            }

            // Vector table grid icon
            let icon_center = pos2(tab_rect.min.x + 12.0, tab_rect.center().y);
            render_vector_grid_icon(
                &painter,
                icon_center,
                if is_active {
                    ds_color
                } else {
                    crate::theme::color_with_alpha(ds_color, 160)
                },
            );

            // Tab Title & Dirty asterisk
            let text_color = if is_active {
                palette.text_primary
            } else {
                palette.text_muted
            };
            let text_x = tab_rect.min.x + 24.0;
            let display_name = session.file_name();

            let right_reserved = if session.is_dirty { 32.0 } else { 20.0 };
            let max_text_w = (w - 24.0 - right_reserved).max(10.0);

            let mut job = egui::text::LayoutJob::simple_singleline(
                display_name.to_string(),
                font_id.clone(),
                text_color,
            );
            job.wrap = egui::text::TextWrapping::truncate_at_width(max_text_w);
            let galley = painter.layout_job(job);

            let text_pos = pos2(text_x, tab_rect.center().y - galley.size().y / 2.0);
            let text_w = galley.size().x;
            painter.galley(text_pos, galley, text_color);

            if session.is_dirty {
                let star_color = palette.measure_ordinal;
                let star_x = text_x + text_w + 3.0;
                let star_pos = pos2(star_x, tab_rect.center().y);
                painter.text(
                    star_pos,
                    egui::Align2::LEFT_CENTER,
                    "*",
                    egui::FontId::proportional(14.0),
                    star_color,
                );
            }

            // Close button (14x14)
            let close_rect = Rect::from_center_size(
                pos2(tab_rect.max.x - 12.0, tab_rect.center().y),
                vec2(14.0, 14.0),
            );
            let close_id = ui.make_persistent_id(format!("close_doc_tab_{}", session.id));
            let close_resp = ui.interact(close_rect, close_id, Sense::click());

            if close_resp.hovered() {
                painter.rect_filled(close_rect, 2.0, palette.caption_close_hover);
            }
            let close_icon_color = if close_resp.hovered() {
                Color32::WHITE
            } else {
                palette.text_muted
            };
            render_vector_cross(&painter, close_rect.center(), 3.5, close_icon_color);

            if close_resp.clicked() {
                action = DocumentTabAction::Close(session.id);
            } else if tab_resp.clicked() {
                action = DocumentTabAction::Select(session.id);
            }
        }

        // Handle mouse wheel scrolling over tabs
        let hover_tabs = ui.rect_contains_pointer(tabs_area_rect);
        if hover_tabs {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.x + i.raw_scroll_delta.y);
            if scroll_delta.abs() > 0.0 {
                *scroll_offset = (*scroll_offset - scroll_delta).clamp(0.0, max_scroll);
            }
        }

        // Right-side controls: [ ‹ ] [ › ] [ ➕ ]
        let mut ctrl_x = content_max_x + 4.0;

        // Scroll Left Button
        let left_rect = Rect::from_min_size(
            pos2(ctrl_x, strip_rect.min.y + 3.0),
            vec2(nav_btn_width, tab_height - 2.0),
        );
        let left_resp = ui.interact(
            left_rect,
            ui.make_persistent_id("tab_scroll_left"),
            Sense::click(),
        );
        render_nav_chevron(
            ui.painter(),
            left_rect,
            true,
            left_resp.hovered(),
            palette.text_muted,
            palette.caption_hover,
        );
        if left_resp.clicked() {
            *scroll_offset = (*scroll_offset - 120.0).clamp(0.0, max_scroll);
        }
        ctrl_x += nav_btn_width + 2.0;

        // Scroll Right Button
        let right_rect = Rect::from_min_size(
            pos2(ctrl_x, strip_rect.min.y + 3.0),
            vec2(nav_btn_width, tab_height - 2.0),
        );
        let right_resp = ui.interact(
            right_rect,
            ui.make_persistent_id("tab_scroll_right"),
            Sense::click(),
        );
        render_nav_chevron(
            ui.painter(),
            right_rect,
            false,
            right_resp.hovered(),
            palette.text_muted,
            palette.caption_hover,
        );
        if right_resp.clicked() {
            *scroll_offset = (*scroll_offset + 120.0).clamp(0.0, max_scroll);
        }
        ctrl_x += nav_btn_width + 4.0;

        // Plus Button [ ➕ ]
        let plus_rect = Rect::from_min_size(
            pos2(ctrl_x, strip_rect.min.y + 3.0),
            vec2(plus_btn_width, tab_height - 2.0),
        );
        let plus_resp = ui.interact(
            plus_rect,
            ui.make_persistent_id("tab_new_dataset"),
            Sense::click(),
        );
        let plus_bg = if plus_resp.hovered() {
            palette.caption_hover
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(plus_rect, 3.0, plus_bg);
        ui.painter()
            .rect_stroke(plus_rect, 3.0, Stroke::new(1.0_f32, palette.border));
        render_vector_plus(
            ui.painter(),
            plus_rect.center(),
            4.5,
            if plus_resp.hovered() {
                palette.accent
            } else {
                palette.text_primary
            },
        );
        let plus_clicked = plus_resp.clicked();
        plus_resp.on_hover_text(t!("tab-new-dataset-tooltip"));

        if plus_clicked {
            action = DocumentTabAction::NewBlank;
        }

        action
    }
}

fn render_vector_grid_icon(painter: &egui::Painter, center: Pos2, color: Color32) {
    let half_w = 6.0;
    let half_h = 5.0;
    let rect = Rect::from_min_max(
        pos2(center.x - half_w, center.y - half_h),
        pos2(center.x + half_w, center.y + half_h),
    );
    let stroke = Stroke::new(1.0_f32, color);
    painter.rect_stroke(rect, 1.0, stroke);
    // Vertical split
    painter.line_segment(
        [pos2(center.x, rect.min.y), pos2(center.x, rect.max.y)],
        stroke,
    );
    // Horizontal split
    painter.line_segment(
        [pos2(rect.min.x, center.y), pos2(rect.max.x, center.y)],
        stroke,
    );
}

fn render_vector_cross(painter: &egui::Painter, center: Pos2, r: f32, color: Color32) {
    let stroke = Stroke::new(1.2_f32, color);
    painter.line_segment(
        [
            pos2(center.x - r, center.y - r),
            pos2(center.x + r, center.y + r),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(center.x + r, center.y - r),
            pos2(center.x - r, center.y + r),
        ],
        stroke,
    );
}

fn render_vector_plus(painter: &egui::Painter, center: Pos2, r: f32, color: Color32) {
    let stroke = Stroke::new(1.4_f32, color);
    painter.line_segment(
        [pos2(center.x - r, center.y), pos2(center.x + r, center.y)],
        stroke,
    );
    painter.line_segment(
        [pos2(center.x, center.y - r), pos2(center.x, center.y + r)],
        stroke,
    );
}

fn render_nav_chevron(
    painter: &egui::Painter,
    rect: Rect,
    is_left: bool,
    hovered: bool,
    color: Color32,
    hover_bg: Color32,
) {
    if hovered {
        painter.rect_filled(rect, 2.0, hover_bg);
    }
    let center = rect.center();
    let r = 3.5;
    let stroke = Stroke::new(1.3_f32, if hovered { Color32::WHITE } else { color });
    if is_left {
        painter.line_segment(
            [
                pos2(center.x + r * 0.5, center.y - r),
                pos2(center.x - r * 0.5, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x - r * 0.5, center.y),
                pos2(center.x + r * 0.5, center.y + r),
            ],
            stroke,
        );
    } else {
        painter.line_segment(
            [
                pos2(center.x - r * 0.5, center.y - r),
                pos2(center.x + r * 0.5, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x + r * 0.5, center.y),
                pos2(center.x - r * 0.5, center.y + r),
            ],
            stroke,
        );
    }
}
