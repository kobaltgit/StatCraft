use super::line_index::LineIndex;
use egui::{pos2, vec2, Pos2, Rect, Vec2};
use std::ops::Range;

/// Spatial layout and geometry calculations for a 4-quadrant grid with frozen headers.
///
/// Layout:
/// ```text
/// ┌─────────────────┬──────────────────────────────────────────┐
/// │ quad_top_left   │ quad_top_right (Column Headers)          │
/// ├─────────────────┼──────────────────────────────────────────┤
/// │ quad_bottom_left│ quad_center (Data Grid)                  │
/// │ (Row Headers)   │                                          │
/// └─────────────────┴──────────────────────────────────────────┘
/// ```
#[derive(Debug, Clone)]
pub struct Geo {
    /// Overall boundary allocated for the entire grid widget.
    pub viewport_rect: Rect,

    /// Width of the left row-headers pane (in screen pixels).
    pub header_width: f32,

    /// Height of the top column-headers pane (in screen pixels).
    pub header_height: f32,

    /// 1D line index for rows.
    pub rows: LineIndex,

    /// 1D line index for columns.
    pub cols: LineIndex,

    /// Current scroll position within the data grid: `(scroll_x, scroll_y)`.
    pub scroll: Vec2,

    /// Quadrant 1: Top-Left corner (fixed, non-scrolling).
    pub quad_top_left: Rect,

    /// Quadrant 2: Top header pane (fixed vertically, scrolls horizontally).
    pub quad_top_right: Rect,

    /// Quadrant 3: Left header pane (fixed horizontally, scrolls vertically).
    pub quad_bottom_left: Rect,

    /// Quadrant 4: Center data pane (scrolls horizontally and vertically).
    pub quad_center: Rect,
}

impl Geo {
    /// Computes the grid layout geometry for given viewport, headers size, lines, and scroll.
    pub fn new(
        viewport_rect: Rect,
        header_width: f32,
        header_height: f32,
        rows: LineIndex,
        cols: LineIndex,
        requested_scroll: Vec2,
    ) -> Self {
        let valid_hw = if header_width.is_finite() && header_width >= 0.0 {
            header_width.min(viewport_rect.width().max(0.0))
        } else {
            0.0
        };

        let valid_hh = if header_height.is_finite() && header_height >= 0.0 {
            header_height.min(viewport_rect.height().max(0.0))
        } else {
            0.0
        };

        let split_x = viewport_rect.min.x + valid_hw;
        let split_y = viewport_rect.min.y + valid_hh;

        let quad_top_left = Rect::from_min_max(viewport_rect.min, pos2(split_x, split_y));

        let quad_top_right = Rect::from_min_max(
            pos2(split_x, viewport_rect.min.y),
            pos2(viewport_rect.max.x, split_y),
        );

        let quad_bottom_left = Rect::from_min_max(
            pos2(viewport_rect.min.x, split_y),
            pos2(split_x, viewport_rect.max.y),
        );

        let quad_center = Rect::from_min_max(pos2(split_x, split_y), viewport_rect.max);

        // Clamp scroll within [0, max_scroll]
        let max_scroll_x = (cols.total_size() - quad_center.width()).max(0.0);
        let max_scroll_y = (rows.total_size() - quad_center.height()).max(0.0);

        let clamped_scroll_x = requested_scroll.x.max(0.0).min(max_scroll_x).max(0.0);
        let clamped_scroll_y = requested_scroll.y.max(0.0).min(max_scroll_y).max(0.0);

        let scroll = vec2(clamped_scroll_x, clamped_scroll_y);

        Self {
            viewport_rect,
            header_width: valid_hw,
            header_height: valid_hh,
            rows,
            cols,
            scroll,
            quad_top_left,
            quad_top_right,
            quad_bottom_left,
            quad_center,
        }
    }

    /// Maximum valid scroll offset for the data viewport.
    #[inline]
    pub fn max_scroll(&self) -> Vec2 {
        let max_x = (self.cols.total_size() - self.quad_center.width()).max(0.0);
        let max_y = (self.rows.total_size() - self.quad_center.height()).max(0.0);
        vec2(max_x, max_y)
    }

    /// Half-open range of column indices visible within `quad_center` and `quad_top_right`.
    #[inline]
    pub fn visible_cols(&self) -> Range<usize> {
        let right_bound = self.scroll.x + self.quad_center.width();
        self.cols.visible_range(self.scroll.x, right_bound)
    }

    /// Half-open range of row indices visible within `quad_center` and `quad_bottom_left`.
    #[inline]
    pub fn visible_rows(&self) -> Range<usize> {
        let bottom_bound = self.scroll.y + self.quad_center.height();
        self.rows.visible_range(self.scroll.y, bottom_bound)
    }

    /// Screen rectangle for the data cell at `(row, col)`.
    /// Note: the resulting rect may lie partially or entirely outside `quad_center` if not visible.
    pub fn cell_rect(&self, row: usize, col: usize) -> Option<Rect> {
        let col_start = self.cols.get_offset(col)?;
        let col_width = self.cols.get_size(col)?;
        let row_start = self.rows.get_offset(row)?;
        let row_height = self.rows.get_size(row)?;

        let screen_x = self.quad_center.min.x + (col_start - self.scroll.x);
        let screen_y = self.quad_center.min.y + (row_start - self.scroll.y);

        Some(Rect::from_min_size(
            pos2(screen_x, screen_y),
            vec2(col_width, row_height),
        ))
    }

    /// Screen rectangle for column header `col` in `quad_top_right`.
    pub fn col_header_rect(&self, col: usize) -> Option<Rect> {
        let col_start = self.cols.get_offset(col)?;
        let col_width = self.cols.get_size(col)?;

        let screen_x = self.quad_center.min.x + (col_start - self.scroll.x);
        Some(Rect::from_min_size(
            pos2(screen_x, self.quad_top_right.min.y),
            vec2(col_width, self.header_height),
        ))
    }

    /// Screen rectangle for row header `row` in `quad_bottom_left`.
    pub fn row_header_rect(&self, row: usize) -> Option<Rect> {
        let row_start = self.rows.get_offset(row)?;
        let row_height = self.rows.get_size(row)?;

        let screen_y = self.quad_center.min.y + (row_start - self.scroll.y);
        Some(Rect::from_min_size(
            pos2(self.quad_bottom_left.min.x, screen_y),
            vec2(self.header_width, row_height),
        ))
    }

    /// Finds cell coordinates `(row, col)` if `pos` lies within `quad_center`.
    pub fn cell_at_pos(&self, pos: Pos2) -> Option<(usize, usize)> {
        if !self.quad_center.contains(pos) {
            return None;
        }

        let data_x = pos.x - self.quad_center.min.x + self.scroll.x;
        let data_y = pos.y - self.quad_center.min.y + self.scroll.y;

        let col = self.cols.index_at(data_x)?;
        let row = self.rows.index_at(data_y)?;

        Some((row, col))
    }

    /// Finds cell coordinates `(row, col)` clamped to grid bounds during active drag.
    pub fn cell_at_pos_clamped(&self, pos: Pos2) -> Option<(usize, usize)> {
        if self.rows.count() == 0 || self.cols.count() == 0 {
            return None;
        }

        let data_x = (pos.x - self.quad_center.min.x + self.scroll.x)
            .clamp(0.0, (self.cols.total_size() - 1.0).max(0.0));
        let data_y = (pos.y - self.quad_center.min.y + self.scroll.y)
            .clamp(0.0, (self.rows.total_size() - 1.0).max(0.0));

        let col = self
            .cols
            .index_at(data_x)
            .unwrap_or(self.cols.count().saturating_sub(1));
        let row = self
            .rows
            .index_at(data_y)
            .unwrap_or(self.rows.count().saturating_sub(1));

        Some((row, col))
    }

    /// Finds column index if `pos` lies within `quad_top_right`.
    pub fn col_header_at_pos(&self, pos: Pos2) -> Option<usize> {
        if !self.quad_top_right.contains(pos) {
            return None;
        }

        let data_x = pos.x - self.quad_center.min.x + self.scroll.x;
        self.cols.index_at(data_x)
    }

    /// Finds row index if `pos` lies within `quad_bottom_left`.
    pub fn row_header_at_pos(&self, pos: Pos2) -> Option<usize> {
        if !self.quad_bottom_left.contains(pos) {
            return None;
        }

        let data_y = pos.y - self.quad_center.min.y + self.scroll.y;
        self.rows.index_at(data_y)
    }

    /// Hit-tests whether `pos` is near a vertical column boundary for resizing.
    /// Works when hovering headers or entire grid area, returning `(col_idx, screen_x)`.
    pub fn col_boundary_near_pos(&self, pos: Pos2, tolerance: f32) -> Option<(usize, f32)> {
        if pos.x < self.quad_center.min.x - tolerance || pos.x > self.quad_center.max.x + tolerance
        {
            return None;
        }
        if pos.y < self.viewport_rect.min.y || pos.y > self.viewport_rect.max.y {
            return None;
        }

        let data_x = pos.x - self.quad_center.min.x + self.scroll.x;
        let (col_idx, data_boundary) = self.cols.find_boundary_near(data_x, tolerance)?;
        let screen_x = self.quad_center.min.x + (data_boundary - self.scroll.x);

        Some((col_idx, screen_x))
    }

    /// Computes an updated scroll offset that brings cell `(row, col)` into full view.
    pub fn scroll_to_cell(&self, row: usize, col: usize) -> Vec2 {
        let mut new_scroll = self.scroll;

        if let (Some(col_start), Some(col_width)) =
            (self.cols.get_offset(col), self.cols.get_size(col))
        {
            let col_end = col_start + col_width;
            let view_w = self.quad_center.width();

            if col_start < new_scroll.x {
                new_scroll.x = col_start;
            } else if col_end > new_scroll.x + view_w {
                new_scroll.x = (col_end - view_w).max(0.0);
            }
        }

        if let (Some(row_start), Some(row_height)) =
            (self.rows.get_offset(row), self.rows.get_size(row))
        {
            let row_end = row_start + row_height;
            let view_h = self.quad_center.height();

            if row_start < new_scroll.y {
                new_scroll.y = row_start;
            } else if row_end > new_scroll.y + view_h {
                new_scroll.y = (row_end - view_h).max(0.0);
            }
        }

        let max = self.max_scroll();
        vec2(
            new_scroll.x.clamp(0.0, max.x),
            new_scroll.y.clamp(0.0, max.y),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_quadrants_layout() {
        let vp = Rect::from_min_size(pos2(10.0, 20.0), vec2(400.0, 300.0));
        let rows = LineIndex::uniform(100, 20.0); // Total height = 2000.0
        let cols = LineIndex::uniform(10, 80.0); // Total width = 800.0

        let geo = Geo::new(vp, 50.0, 25.0, rows, cols, vec2(0.0, 0.0));

        // Quad Top-Left: [10, 20] .. [60, 45]
        assert_eq!(geo.quad_top_left.min, pos2(10.0, 20.0));
        assert_eq!(geo.quad_top_left.max, pos2(60.0, 45.0));

        // Quad Top-Right: [60, 20] .. [410, 45] (width = 350.0)
        assert_eq!(geo.quad_top_right.min, pos2(60.0, 20.0));
        assert_eq!(geo.quad_top_right.max, pos2(410.0, 45.0));

        // Quad Bottom-Left: [10, 45] .. [60, 320] (height = 275.0)
        assert_eq!(geo.quad_bottom_left.min, pos2(10.0, 45.0));
        assert_eq!(geo.quad_bottom_left.max, pos2(60.0, 320.0));

        // Quad Center: [60, 45] .. [410, 320] (width = 350.0, height = 275.0)
        assert_eq!(geo.quad_center.min, pos2(60.0, 45.0));
        assert_eq!(geo.quad_center.max, pos2(410.0, 320.0));
    }

    #[test]
    fn test_geo_cell_coords_and_hit_test() {
        let vp = Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 400.0));
        let rows = LineIndex::uniform(20, 20.0);
        let cols = LineIndex::uniform(10, 100.0);

        let geo = Geo::new(vp, 50.0, 20.0, rows, cols, vec2(50.0, 10.0));

        // quad_center starts at (50.0, 20.0)
        // Cell (0, 0): data_x = 0..100, data_y = 0..20
        // Screen x = 50.0 + (0.0 - 50.0) = 0.0, Screen y = 20.0 + (0.0 - 10.0) = 10.0
        let c00 = geo.cell_rect(0, 0);
        assert_eq!(
            c00,
            Some(Rect::from_min_size(pos2(0.0, 10.0), vec2(100.0, 20.0)))
        );

        // Cell (1, 1): data_x = 100..200, data_y = 20..40
        // Screen x = 50.0 + (100.0 - 50.0) = 100.0, Screen y = 20.0 + (20.0 - 10.0) = 30.0
        let c11 = geo.cell_rect(1, 1);
        assert_eq!(
            c11,
            Some(Rect::from_min_size(pos2(100.0, 30.0), vec2(100.0, 20.0)))
        );

        // Hit testing inside quad_center
        // Pos (120.0, 35.0) -> inside Cell (1, 1)
        assert_eq!(geo.cell_at_pos(pos2(120.0, 35.0)), Some((1, 1)));

        // Hit testing outside quad_center
        assert_eq!(geo.cell_at_pos(pos2(20.0, 10.0)), None);
    }

    #[test]
    fn test_geo_boundary_hit_test() {
        let vp = Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 400.0));
        let rows = LineIndex::uniform(20, 20.0);
        let cols = LineIndex::uniform(10, 100.0);

        // quad_center.min.x = 50.0. With scroll_x = 0:
        // Col 0 right boundary: data_x = 100.0 -> screen_x = 150.0
        let geo = Geo::new(vp, 50.0, 20.0, rows, cols, vec2(0.0, 0.0));

        assert_eq!(
            geo.col_boundary_near_pos(pos2(151.0, 10.0), 3.0),
            Some((0, 150.0))
        );
        assert_eq!(geo.col_boundary_near_pos(pos2(160.0, 10.0), 3.0), None);
    }

    #[test]
    fn test_geo_scroll_to_cell() {
        let vp = Rect::from_min_size(pos2(0.0, 0.0), vec2(350.0, 250.0));
        let rows = LineIndex::uniform(100, 20.0); // 2000 px
        let cols = LineIndex::uniform(20, 100.0); // 2000 px

        // quad_center size: width = 300.0, height = 225.0
        let geo = Geo::new(vp, 50.0, 25.0, rows, cols, vec2(0.0, 0.0));

        // Scroll to cell (10, 5)
        // row 10 start = 200.0, height = 20.0, end = 220.0
        // col 5 start = 500.0, width = 100.0, end = 600.0
        // Visible window width = 300.0 -> scroll_x should be 600.0 - 300.0 = 300.0
        // Visible window height = 225.0 -> 220.0 <= 225.0, so scroll_y remains 0.0
        let target_scroll = geo.scroll_to_cell(10, 5);
        assert_eq!(target_scroll.x, 300.0);
        assert_eq!(target_scroll.y, 0.0);
    }
}
