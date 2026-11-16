use super::geo::Geo;
use super::selection::GridSelection;
use crate::theme::Tokens;
use crate::widgets::paint_measure_icon;
use egui::{pos2, vec2, Align2, FontId, Painter, Rect, Stroke};
use statcraft_core::Measure;

/// Rendering utilities for 4-quadrant frozen grid components.
pub struct QuadrantPaint;

impl QuadrantPaint {
    /// Renders the fixed top-left corner (Quadrant 1).
    pub fn paint_corner(painter: &Painter, geo: &Geo, label: &str) {
        let p = painter.with_clip_rect(geo.quad_top_left);
        let rect = geo.quad_top_left;

        p.rect_filled(rect, 0.0_f32, Tokens::bg_header());
        p.rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::border()));

        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(12.0),
            Tokens::text_primary(),
        );
    }

    /// Renders the top column headers (Quadrant 2).
    pub fn paint_col_headers<F>(
        painter: &Painter,
        geo: &Geo,
        hovered_col: Option<usize>,
        mut col_info: F,
    ) where
        F: FnMut(usize) -> (String, Measure),
    {
        let p = painter.with_clip_rect(geo.quad_top_right);
        p.rect_filled(geo.quad_top_right, 0.0_f32, Tokens::bg_header());

        for c in geo.visible_cols() {
            let rect = match geo.col_header_rect(c) {
                Some(r) => r,
                None => continue,
            };

            let (name, measure) = col_info(c);
            let is_hovered = hovered_col == Some(c);

            let bg_color = if is_hovered {
                Tokens::bg_panel()
            } else {
                Tokens::bg_header()
            };

            p.rect_filled(rect, 0.0_f32, bg_color);
            p.rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::border()));

            // Measure icon (16x16)
            let icon_rect = Rect::from_min_size(
                pos2(rect.min.x + 4.0_f32, rect.min.y + 4.0_f32),
                vec2(16.0_f32, 16.0_f32),
            );
            paint_measure_icon(&p, icon_rect, measure);

            // Column title with clip
            let text_clip = Rect::from_min_max(
                pos2(rect.min.x + 22.0_f32, rect.min.y),
                pos2(rect.max.x - 4.0_f32, rect.max.y),
            );
            p.with_clip_rect(text_clip).text(
                pos2(rect.min.x + 24.0_f32, rect.center().y),
                Align2::LEFT_CENTER,
                name,
                FontId::proportional(12.0),
                Tokens::text_primary(),
            );
        }
    }

    /// Renders the left row headers (Quadrant 3).
    pub fn paint_row_headers<F>(
        painter: &Painter,
        geo: &Geo,
        _hovered_row: Option<usize>,
        mut row_info: F,
    ) where
        F: FnMut(usize) -> (String, bool),
    {
        let p = painter.with_clip_rect(geo.quad_bottom_left);
        p.rect_filled(geo.quad_bottom_left, 0.0_f32, Tokens::bg_header());

        for r in geo.visible_rows() {
            let rect = match geo.row_header_rect(r) {
                Some(rec) => rec,
                None => continue,
            };

            let (label, is_active) = row_info(r);

            p.rect_filled(rect, 0.0_f32, Tokens::bg_header());
            p.rect_stroke(rect, 0.0_f32, Stroke::new(0.5_f32, Tokens::border()));

            p.text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
                Tokens::text_muted(),
            );

            // Diagonal strikethrough for filtered / inactive rows
            if !is_active {
                let stroke = Stroke::new(1.0_f32, Tokens::text_muted());
                let slash_start = pos2(rect.center().x - 12.0, rect.center().y + 7.0);
                let slash_end = pos2(rect.center().x + 12.0, rect.center().y - 7.0);
                p.line_segment([slash_start, slash_end], stroke);
            }
        }
    }

    /// Renders the center data cells (Quadrant 4).
    pub fn paint_data_cells<F>(
        painter: &Painter,
        geo: &Geo,
        selection: Option<&GridSelection>,
        editing_cell: Option<(usize, usize)>,
        mut cell_info: F,
    ) where
        F: FnMut(usize, usize) -> String,
    {
        let p = painter.with_clip_rect(geo.quad_center);
        p.rect_filled(geo.quad_center, 0.0_f32, Tokens::bg_app());

        let vis_cols = geo.visible_cols();

        for r in geo.visible_rows() {
            let is_even = r % 2 == 0;

            for c in vis_cols.clone() {
                let rect = match geo.cell_rect(r, c) {
                    Some(rec) => rec,
                    None => continue,
                };

                let is_selected = selection.is_some_and(|s| s.contains(r, c));
                let is_active_cursor = selection.is_some_and(|s| s.active_cell == (r, c));
                let is_multi_cell = selection.is_some_and(|s| !s.range.is_single_cell());
                let is_editing = editing_cell == Some((r, c));

                let bg_color = if is_selected {
                    if is_active_cursor && is_multi_cell {
                        Tokens::bg_panel()
                    } else {
                        Tokens::bg_selected()
                    }
                } else if is_even {
                    Tokens::bg_row_even()
                } else {
                    Tokens::bg_row_odd()
                };

                p.rect_filled(rect, 0.0_f32, bg_color);
                p.rect_stroke(rect, 0.0_f32, Stroke::new(0.5_f32, Tokens::border()));

                // When actively editing inline, the TextEdit widget renders in-place
                if is_editing {
                    continue;
                }

                let text = cell_info(r, c);
                if text.is_empty() {
                    continue;
                }

                let inner_rect = rect.shrink2(vec2(4.0_f32, 1.0_f32));
                let is_numeric =
                    text.starts_with(|ch: char| ch.is_ascii_digit() || ch == '-' || ch == '.');

                let (align, text_pos) = if is_numeric {
                    (
                        Align2::RIGHT_CENTER,
                        pos2(inner_rect.max.x - 2.0_f32, inner_rect.center().y),
                    )
                } else {
                    (
                        Align2::LEFT_CENTER,
                        pos2(inner_rect.min.x + 2.0_f32, inner_rect.center().y),
                    )
                };

                p.with_clip_rect(inner_rect).text(
                    text_pos,
                    align,
                    text,
                    FontId::proportional(12.0),
                    Tokens::text_primary(),
                );
            }
        }
    }

    /// Renders a vertical guideline across the entire grid height during column resizing.
    pub fn paint_resize_guide(painter: &Painter, geo: &Geo, screen_x: f32) {
        let p = painter.with_clip_rect(geo.viewport_rect);
        let top = geo.quad_top_right.min.y;
        let bottom = geo.quad_center.max.y;
        let stroke = Stroke::new(1.5_f32, Tokens::accent());

        p.line_segment([pos2(screen_x, top), pos2(screen_x, bottom)], stroke);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::line_index::LineIndex;
    use egui::vec2;

    #[test]
    fn test_quadrant_paint_does_not_panic() {
        let vp = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0));
        let rows = LineIndex::uniform(20, 20.0);
        let cols = LineIndex::uniform(10, 80.0);
        let geo = Geo::new(vp, 50.0, 25.0, rows, cols, vec2(0.0, 0.0));

        // Create dummy egui painter
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let painter = ui.painter();

                QuadrantPaint::paint_corner(painter, &geo, "Row");

                QuadrantPaint::paint_col_headers(painter, &geo, Some(1), |c| {
                    (format!("COL_{}", c), Measure::Scale)
                });

                QuadrantPaint::paint_row_headers(painter, &geo, None, |r| {
                    (format!("{}", r + 1), r % 2 == 0)
                });

                let sel = GridSelection::single(0, 0);
                QuadrantPaint::paint_data_cells(painter, &geo, Some(&sel), None, |r, c| {
                    format!("{}.{}", r, c)
                });

                QuadrantPaint::paint_resize_guide(painter, &geo, 130.0);
            });
        });
    }
}
