use super::geo::Geo;
use crate::theme::Tokens;
use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Stroke};

/// A normalized rectangular range of grid cells `(min_row..=max_row, min_col..=max_col)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRange {
    pub min_row: usize,
    pub max_row: usize,
    pub min_col: usize,
    pub max_col: usize,
}

impl CellRange {
    /// Creates a normalized range from any two corner coordinates.
    pub fn from_corners(r0: usize, c0: usize, r1: usize, c1: usize) -> Self {
        Self {
            min_row: r0.min(r1),
            max_row: r0.max(r1),
            min_col: c0.min(c1),
            max_col: c0.max(c1),
        }
    }

    /// Creates a 1x1 range representing a single cell.
    pub fn single(row: usize, col: usize) -> Self {
        Self {
            min_row: row,
            max_row: row,
            min_col: col,
            max_col: col,
        }
    }

    /// Range covering an entire column across all rows.
    pub fn full_col(col: usize, total_rows: usize) -> Self {
        Self {
            min_row: 0,
            max_row: total_rows.saturating_sub(1),
            min_col: col,
            max_col: col,
        }
    }

    /// Range covering an entire row across all columns.
    pub fn full_row(row: usize, total_cols: usize) -> Self {
        Self {
            min_row: row,
            max_row: row,
            min_col: 0,
            max_col: total_cols.saturating_sub(1),
        }
    }

    /// Range covering the entire table.
    pub fn all(total_rows: usize, total_cols: usize) -> Self {
        Self {
            min_row: 0,
            max_row: total_rows.saturating_sub(1),
            min_col: 0,
            max_col: total_cols.saturating_sub(1),
        }
    }

    /// Whether this range contains cell `(row, col)`.
    #[inline]
    pub fn contains_cell(&self, row: usize, col: usize) -> bool {
        row >= self.min_row && row <= self.max_row && col >= self.min_col && col <= self.max_col
    }

    /// Number of rows spanned by this range.
    #[inline]
    pub fn row_count(&self) -> usize {
        self.max_row - self.min_row + 1
    }

    /// Number of columns spanned by this range.
    #[inline]
    pub fn col_count(&self) -> usize {
        self.max_col - self.min_col + 1
    }

    /// Total number of cells within this range.
    #[inline]
    pub fn cell_count(&self) -> usize {
        self.row_count() * self.col_count()
    }

    /// Whether this range is exactly one cell.
    #[inline]
    pub fn is_single_cell(&self) -> bool {
        self.min_row == self.max_row && self.min_col == self.max_col
    }
}

/// Represents the active selection state in the 2D grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSelection {
    /// The active cursor cell (keyboard focus target).
    pub active_cell: (usize, usize),
    /// The anchor cell from where the selection expansion originated.
    pub anchor_cell: (usize, usize),
    /// The bounding cell range.
    pub range: CellRange,
    /// Whether entire columns are selected.
    pub is_full_cols: bool,
    /// Whether entire rows are selected.
    pub is_full_rows: bool,
}

impl Default for GridSelection {
    fn default() -> Self {
        Self::single(0, 0)
    }
}

impl GridSelection {
    /// Creates a selection on a single cell.
    pub fn single(row: usize, col: usize) -> Self {
        Self {
            active_cell: (row, col),
            anchor_cell: (row, col),
            range: CellRange::single(row, col),
            is_full_cols: false,
            is_full_rows: false,
        }
    }

    /// Creates a range selection between `anchor` and `active`.
    pub fn from_anchor_and_active(anchor: (usize, usize), active: (usize, usize)) -> Self {
        Self {
            active_cell: active,
            anchor_cell: anchor,
            range: CellRange::from_corners(anchor.0, anchor.1, active.0, active.1),
            is_full_cols: false,
            is_full_rows: false,
        }
    }

    /// Selects an entire column.
    pub fn select_col(col: usize, total_rows: usize) -> Self {
        Self {
            active_cell: (0, col),
            anchor_cell: (0, col),
            range: CellRange::full_col(col, total_rows),
            is_full_cols: true,
            is_full_rows: false,
        }
    }

    /// Selects an entire row.
    pub fn select_row(row: usize, total_cols: usize) -> Self {
        Self {
            active_cell: (row, 0),
            anchor_cell: (row, 0),
            range: CellRange::full_row(row, total_cols),
            is_full_cols: false,
            is_full_rows: true,
        }
    }

    /// Selects the entire dataset.
    pub fn select_all(total_rows: usize, total_cols: usize) -> Self {
        Self {
            active_cell: (0, 0),
            anchor_cell: (0, 0),
            range: CellRange::all(total_rows, total_cols),
            is_full_cols: true,
            is_full_rows: true,
        }
    }

    /// Extends the current selection to a new active cell while keeping the anchor.
    pub fn extend_to(&mut self, active_row: usize, active_col: usize) {
        self.active_cell = (active_row, active_col);
        self.range = CellRange::from_corners(
            self.anchor_cell.0,
            self.anchor_cell.1,
            active_row,
            active_col,
        );
        self.is_full_cols = false;
        self.is_full_rows = false;
    }

    /// Whether cell `(row, col)` is inside this selection.
    #[inline]
    pub fn contains(&self, row: usize, col: usize) -> bool {
        self.range.contains_cell(row, col)
    }

    /// Computes the screen bounding rectangle for this selection within `quad_center`.
    pub fn bounding_rect(&self, geo: &Geo) -> Option<Rect> {
        let r0 = self.range.min_row;
        let r1 = self.range.max_row;
        let c0 = self.range.min_col;
        let c1 = self.range.max_col;

        let col_start = geo.cols.get_offset(c0)?;
        let col_end = geo.cols.get_offset(c1)? + geo.cols.get_size(c1)?;
        let row_start = geo.rows.get_offset(r0)?;
        let row_end = geo.rows.get_offset(r1)? + geo.rows.get_size(r1)?;

        let screen_x0 = geo.quad_center.min.x + (col_start - geo.scroll.x);
        let screen_x1 = geo.quad_center.min.x + (col_end - geo.scroll.x);
        let screen_y0 = geo.quad_center.min.y + (row_start - geo.scroll.y);
        let screen_y1 = geo.quad_center.min.y + (row_end - geo.scroll.y);

        Some(Rect::from_min_max(
            pos2(screen_x0, screen_y0),
            pos2(screen_x1, screen_y1),
        ))
    }

    /// Screen rectangle of the Fill Handle (bottom-right corner marker, 6x6 px).
    pub fn fill_handle_rect(&self, geo: &Geo) -> Option<Rect> {
        let bounds = self.bounding_rect(geo)?;
        let size = 6.0_f32;
        Some(Rect::from_center_size(bounds.max, vec2(size, size)))
    }

    /// Hit-tests whether `pos` is on the Fill Handle marker.
    pub fn hit_test_fill_handle(&self, geo: &Geo, pos: Pos2, tolerance: f32) -> bool {
        if let Some(rect) = self.fill_handle_rect(geo) {
            let hit_rect = rect.expand(tolerance);
            hit_rect.contains(pos)
        } else {
            false
        }
    }
}

/// Renders the range selection marquee, fill, and fill handle.
pub fn paint_selection(painter: &Painter, geo: &Geo, selection: &GridSelection) {
    let bounds = match selection.bounding_rect(geo) {
        Some(b) => b,
        None => return,
    };

    // Clip rendering strictly to quad_center so it never bleeds into headers
    let p = painter.with_clip_rect(geo.quad_center);

    // Multi-cell background fill
    if !selection.range.is_single_cell() {
        p.rect_filled(bounds, 0.0_f32, Tokens::bg_selected());
    }

    // Outer accent stroke (1.5 px)
    p.rect_stroke(bounds, 0.0_f32, Stroke::new(1.5_f32, Tokens::accent()));

    // Fill handle marker at bottom-right corner (6x6 px)
    if let Some(handle_rect) = selection.fill_handle_rect(geo) {
        p.rect_filled(handle_rect, 0.0_f32, Tokens::accent());
        p.rect_stroke(handle_rect, 0.0_f32, Stroke::new(1.0_f32, Color32::WHITE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::line_index::LineIndex;

    #[test]
    fn test_cell_range_normalization() {
        let r1 = CellRange::from_corners(5, 10, 2, 4);
        assert_eq!(r1.min_row, 2);
        assert_eq!(r1.max_row, 5);
        assert_eq!(r1.min_col, 4);
        assert_eq!(r1.max_col, 10);
        assert_eq!(r1.row_count(), 4);
        assert_eq!(r1.col_count(), 7);
        assert_eq!(r1.cell_count(), 28);
        assert!(!r1.is_single_cell());

        assert!(r1.contains_cell(2, 4));
        assert!(r1.contains_cell(5, 10));
        assert!(r1.contains_cell(3, 7));
        assert!(!r1.contains_cell(1, 4));
        assert!(!r1.contains_cell(2, 11));
    }

    #[test]
    fn test_grid_selection_extension() {
        let mut sel = GridSelection::single(3, 3);
        assert!(sel.range.is_single_cell());

        sel.extend_to(5, 7);
        assert_eq!(sel.anchor_cell, (3, 3));
        assert_eq!(sel.active_cell, (5, 7));
        assert_eq!(sel.range, CellRange::from_corners(3, 3, 5, 7));
        assert_eq!(sel.range.cell_count(), 15);
    }

    #[test]
    fn test_selection_screen_bounds_and_fill_handle() {
        let vp = Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 400.0));
        let rows = LineIndex::uniform(20, 20.0);
        let cols = LineIndex::uniform(10, 100.0);
        let geo = Geo::new(vp, 50.0, 20.0, rows, cols, vec2(0.0, 0.0));

        let sel = GridSelection::from_anchor_and_active((1, 1), (2, 3));
        // Rows: 1..=2 (row 1 starts at 20.0, row 2 ends at 60.0, screen_y = 40.0..80.0)
        // Cols: 1..=3 (col 1 starts at 100.0, col 3 ends at 400.0, screen_x = 150.0..450.0)
        let bounds = sel.bounding_rect(&geo).expect("bounds must exist");
        assert_eq!(bounds.min, pos2(150.0, 40.0));
        assert_eq!(bounds.max, pos2(450.0, 80.0));

        // Fill handle should be centered at (450.0, 80.0)
        let handle = sel.fill_handle_rect(&geo).expect("handle must exist");
        assert_eq!(handle.center(), pos2(450.0, 80.0));

        assert!(sel.hit_test_fill_handle(&geo, pos2(450.0, 80.0), 3.0));
        assert!(!sel.hit_test_fill_handle(&geo, pos2(400.0, 50.0), 3.0));
    }
}
