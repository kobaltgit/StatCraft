use std::collections::HashMap;
use std::ops::Range;

/// Indexing structure for a 1D sequence of line segments (rows or columns)
/// supporting efficient $O(1)$ (for uniform lines) or $O(\log N)$ (for variable lines)
/// coordinate-to-index lookup, visible range calculation, and boundary hit-testing.
#[derive(Debug, Clone, PartialEq)]
pub enum LineIndex {
    /// All lines share the exact same height or width.
    /// Uses zero heap memory for offsets and provides $O(1)$ lookups.
    Uniform { count: usize, size: f32 },

    /// Lines have variable heights or widths.
    /// `offsets` has length `count + 1`, where `offsets[0] = 0.0`
    /// and `offsets[i + 1] = offsets[i] + size_of_line_i`.
    Variable { offsets: Vec<f32> },
}

impl LineIndex {
    /// Creates a uniform line index where every item has `size`.
    pub fn uniform(count: usize, size: f32) -> Self {
        let size = if size.is_finite() && size >= 0.0 {
            size
        } else {
            0.0
        };
        Self::Uniform { count, size }
    }

    /// Creates a variable line index from an array of individual line sizes.
    pub fn variable(sizes: &[f32]) -> Self {
        let mut offsets = Vec::with_capacity(sizes.len() + 1);
        offsets.push(0.0_f32);
        let mut accum = 0.0_f32;
        for &s in sizes {
            let valid_s = if s.is_finite() && s >= 0.0 { s } else { 0.0 };
            accum += valid_s;
            offsets.push(accum);
        }
        Self::Variable { offsets }
    }

    /// Creates a line index with a default uniform size, applying any custom overrides.
    /// If `overrides` is empty, returns `Uniform` to avoid heap allocations.
    pub fn from_default_and_overrides(
        count: usize,
        default_size: f32,
        overrides: &HashMap<usize, f32>,
    ) -> Self {
        if overrides.is_empty() {
            return Self::uniform(count, default_size);
        }

        let default_val = if default_size.is_finite() && default_size >= 0.0 {
            default_size
        } else {
            0.0
        };

        let mut offsets = Vec::with_capacity(count + 1);
        offsets.push(0.0_f32);
        let mut accum = 0.0_f32;
        for i in 0..count {
            let s = overrides
                .get(&i)
                .copied()
                .filter(|v| v.is_finite() && *v >= 0.0)
                .unwrap_or(default_val);
            accum += s;
            offsets.push(accum);
        }

        Self::Variable { offsets }
    }

    /// Number of lines (rows or columns) managed by this index.
    #[inline]
    pub fn count(&self) -> usize {
        match self {
            Self::Uniform { count, .. } => *count,
            Self::Variable { offsets } => {
                if offsets.is_empty() {
                    0
                } else {
                    offsets.len() - 1
                }
            }
        }
    }

    /// Whether there are zero lines.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Total extent (width or height) of all lines combined.
    #[inline]
    pub fn total_size(&self) -> f32 {
        match self {
            Self::Uniform { count, size } => (*count as f32) * (*size),
            Self::Variable { offsets } => offsets.last().copied().unwrap_or(0.0),
        }
    }

    /// Returns the start coordinate of line `index` (0-indexed).
    /// Returns `None` if `index >= count`.
    pub fn get_offset(&self, index: usize) -> Option<f32> {
        if index >= self.count() {
            return None;
        }
        match self {
            Self::Uniform { size, .. } => Some((index as f32) * (*size)),
            Self::Variable { offsets } => offsets.get(index).copied(),
        }
    }

    /// Returns the extent (width or height) of line `index`.
    /// Returns `None` if `index >= count`.
    pub fn get_size(&self, index: usize) -> Option<f32> {
        if index >= self.count() {
            return None;
        }
        match self {
            Self::Uniform { size, .. } => Some(*size),
            Self::Variable { offsets } => {
                let start = offsets.get(index).copied()?;
                let end = offsets.get(index + 1).copied()?;
                Some((end - start).max(0.0))
            }
        }
    }

    /// Returns `(start_coord, end_coord)` for line `index`.
    /// Returns `None` if `index >= count`.
    pub fn cell_bounds(&self, index: usize) -> Option<(f32, f32)> {
        let start = self.get_offset(index)?;
        let size = self.get_size(index)?;
        Some((start, start + size))
    }

    /// Finds the index of the line containing `coord`.
    /// Returns `None` if `coord < 0.0`, `coord >= total_size()`, or `count == 0`.
    pub fn index_at(&self, coord: f32) -> Option<usize> {
        if !coord.is_finite() || coord < 0.0 || self.is_empty() {
            return None;
        }
        if coord >= self.total_size() {
            return None;
        }

        match self {
            Self::Uniform { count, size } => {
                if *size <= 0.0 {
                    return None;
                }
                let idx = (coord / *size).floor() as usize;
                if idx < *count {
                    Some(idx)
                } else {
                    None
                }
            }
            Self::Variable { offsets } => {
                if offsets.len() <= 1 {
                    return None;
                }
                let pos = offsets.partition_point(|&x| x <= coord);
                if pos > 0 && pos < offsets.len() {
                    Some(pos - 1)
                } else if pos == offsets.len() {
                    Some(offsets.len() - 2)
                } else {
                    None
                }
            }
        }
    }

    /// Finds the index of the line containing `coord`, clamping to `0..count-1`.
    /// Returns `None` only if `count == 0`.
    pub fn clamp_index_at(&self, coord: f32) -> Option<usize> {
        let cnt = self.count();
        if cnt == 0 {
            return None;
        }
        if !coord.is_finite() || coord <= 0.0 {
            return Some(0);
        }
        if coord >= self.total_size() {
            return Some(cnt - 1);
        }
        self.index_at(coord)
    }

    /// Computes the subslice `Range<usize>` of lines that intersect the viewport `[min_coord, max_coord]`.
    /// The returned range is half-open `[start..end)`.
    /// Returns `0..0` if empty or outside bounds.
    pub fn visible_range(&self, min_coord: f32, max_coord: f32) -> Range<usize> {
        let cnt = self.count();
        if cnt == 0 || min_coord > max_coord || max_coord <= 0.0 {
            return 0..0;
        }
        let total = self.total_size();
        if min_coord >= total {
            return 0..0;
        }

        let clamped_min = min_coord.max(0.0);
        let clamped_max = max_coord.min(total);

        let start = match self {
            Self::Uniform { size, .. } => {
                if *size <= 0.0 {
                    0
                } else {
                    ((clamped_min / *size).floor() as usize).min(cnt)
                }
            }
            Self::Variable { offsets } => {
                let pos = offsets.partition_point(|&x| x <= clamped_min);
                pos.saturating_sub(1).min(cnt)
            }
        };

        let end = match self {
            Self::Uniform { size, .. } => {
                if *size <= 0.0 {
                    cnt
                } else {
                    let last_idx = ((clamped_max - 0.0001_f32).max(0.0) / *size).floor() as usize;
                    (last_idx + 1).min(cnt)
                }
            }
            Self::Variable { offsets } => {
                let pos = offsets.partition_point(|&x| x < clamped_max);
                pos.min(cnt)
            }
        };

        if start <= end {
            start..end
        } else {
            0..0
        }
    }

    /// Hit-tests whether `coord` is within `tolerance` of any line boundary.
    /// A boundary is between line `i` and `i + 1` (located at `offset[i + 1]`).
    /// Returns `Some((line_index, boundary_coord))` where `line_index` is the line to the left/top of the boundary.
    pub fn find_boundary_near(&self, coord: f32, tolerance: f32) -> Option<(usize, f32)> {
        let cnt = self.count();
        if cnt == 0 || !coord.is_finite() || tolerance < 0.0 {
            return None;
        }

        // Clamp search candidate to [0, cnt - 1]
        let candidate_idx = self.clamp_index_at(coord)?;

        let mut best: Option<(usize, f32, f32)> = None; // (index, boundary_coord, dist)

        // Check boundary on the left of candidate (between candidate_idx - 1 and candidate_idx)
        if candidate_idx > 0 {
            if let Some(left_b) = self.get_offset(candidate_idx) {
                let dist = (coord - left_b).abs();
                if dist <= tolerance {
                    best = Some((candidate_idx - 1, left_b, dist));
                }
            }
        }

        // Check boundary on the right of candidate (between candidate_idx and candidate_idx + 1)
        if let Some((start, end)) = self.cell_bounds(candidate_idx) {
            let right_b = end;
            let _ = start;
            let dist = (coord - right_b).abs();
            if dist <= tolerance {
                match best {
                    Some((_, _, prev_dist)) if dist < prev_dist => {
                        best = Some((candidate_idx, right_b, dist));
                    }
                    None => {
                        best = Some((candidate_idx, right_b, dist));
                    }
                    _ => {}
                }
            }
        }

        best.map(|(idx, boundary, _)| (idx, boundary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_line_index_basic() {
        let idx = LineIndex::uniform(10, 25.0);
        assert_eq!(idx.count(), 10);
        assert_eq!(idx.total_size(), 250.0);
        assert!(!idx.is_empty());

        assert_eq!(idx.get_offset(0), Some(0.0));
        assert_eq!(idx.get_offset(4), Some(100.0));
        assert_eq!(idx.get_offset(10), None);

        assert_eq!(idx.get_size(0), Some(25.0));
        assert_eq!(idx.cell_bounds(2), Some((50.0, 75.0)));

        assert_eq!(idx.index_at(0.0), Some(0));
        assert_eq!(idx.index_at(24.9), Some(0));
        assert_eq!(idx.index_at(25.0), Some(1));
        assert_eq!(idx.index_at(249.9), Some(9));
        assert_eq!(idx.index_at(250.0), None);
        assert_eq!(idx.index_at(-1.0), None);

        assert_eq!(idx.clamp_index_at(-10.0), Some(0));
        assert_eq!(idx.clamp_index_at(300.0), Some(9));
    }

    #[test]
    fn test_uniform_visible_range() {
        let idx = LineIndex::uniform(100, 20.0);
        // Elements: 0: [0..20], 1: [20..40], 2: [40..60], etc.
        let range = idx.visible_range(15.0, 45.0);
        // Intersects element 0 (15..20), element 1 (20..40), element 2 (40..45)
        assert_eq!(range, 0..3);

        let outside_left = idx.visible_range(-100.0, -10.0);
        assert_eq!(outside_left, 0..0);

        let outside_right = idx.visible_range(2500.0, 3000.0);
        assert_eq!(outside_right, 0..0);

        let inverted = idx.visible_range(50.0, 20.0);
        assert_eq!(inverted, 0..0);
    }

    #[test]
    fn test_variable_line_index_basic() {
        let sizes = vec![50.0, 100.0, 30.0, 80.0];
        let idx = LineIndex::variable(&sizes);

        assert_eq!(idx.count(), 4);
        assert_eq!(idx.total_size(), 260.0);

        // Offsets: [0.0, 50.0, 150.0, 180.0, 260.0]
        assert_eq!(idx.get_offset(0), Some(0.0));
        assert_eq!(idx.get_offset(1), Some(50.0));
        assert_eq!(idx.get_offset(2), Some(150.0));
        assert_eq!(idx.get_offset(3), Some(180.0));
        assert_eq!(idx.get_offset(4), None);

        assert_eq!(idx.get_size(1), Some(100.0));
        assert_eq!(idx.get_size(2), Some(30.0));

        assert_eq!(idx.index_at(0.0), Some(0));
        assert_eq!(idx.index_at(49.9), Some(0));
        assert_eq!(idx.index_at(50.0), Some(1));
        assert_eq!(idx.index_at(149.9), Some(1));
        assert_eq!(idx.index_at(150.0), Some(2));
        assert_eq!(idx.index_at(179.9), Some(2));
        assert_eq!(idx.index_at(180.0), Some(3));
        assert_eq!(idx.index_at(259.9), Some(3));
        assert_eq!(idx.index_at(260.0), None);
    }

    #[test]
    fn test_variable_visible_range() {
        let sizes = vec![50.0, 100.0, 30.0, 80.0];
        let idx = LineIndex::variable(&sizes);

        // Offsets: 0: [0..50], 1: [50..150], 2: [150..180], 3: [180..260]
        let range = idx.visible_range(60.0, 160.0);
        // Intersects item 1 (50..150) and item 2 (150..180)
        assert_eq!(range, 1..3);

        let all = idx.visible_range(0.0, 260.0);
        assert_eq!(all, 0..4);
    }

    #[test]
    fn test_from_default_and_overrides() {
        let mut map = HashMap::new();
        map.insert(1, 140.0);
        map.insert(3, 200.0);

        let idx = LineIndex::from_default_and_overrides(4, 100.0, &map);
        assert_eq!(idx.count(), 4);
        assert_eq!(idx.get_size(0), Some(100.0));
        assert_eq!(idx.get_size(1), Some(140.0));
        assert_eq!(idx.get_size(2), Some(100.0));
        assert_eq!(idx.get_size(3), Some(200.0));
        assert_eq!(idx.total_size(), 540.0);

        // Empty overrides returns Uniform
        let empty_map = HashMap::new();
        let uniform_idx = LineIndex::from_default_and_overrides(5, 80.0, &empty_map);
        assert!(matches!(uniform_idx, LineIndex::Uniform { .. }));
        assert_eq!(uniform_idx.total_size(), 400.0);
    }

    #[test]
    fn test_boundary_hit_test() {
        let sizes = vec![100.0, 120.0, 80.0];
        let idx = LineIndex::variable(&sizes);
        // Boundaries at:
        // Col 0 right edge: 100.0
        // Col 1 right edge: 220.0
        // Col 2 right edge: 300.0

        // Near col 0 boundary (100.0) with tolerance 4.0
        assert_eq!(idx.find_boundary_near(98.0, 4.0), Some((0, 100.0)));
        assert_eq!(idx.find_boundary_near(100.0, 4.0), Some((0, 100.0)));
        assert_eq!(idx.find_boundary_near(103.0, 4.0), Some((0, 100.0)));
        assert_eq!(idx.find_boundary_near(95.0, 4.0), None);

        // Near col 1 boundary (220.0)
        assert_eq!(idx.find_boundary_near(221.0, 3.0), Some((1, 220.0)));

        // Near last col 2 boundary (300.0)
        assert_eq!(idx.find_boundary_near(299.0, 3.0), Some((2, 300.0)));
    }

    #[test]
    fn test_empty_line_index() {
        let empty_uniform = LineIndex::uniform(0, 20.0);
        assert_eq!(empty_uniform.count(), 0);
        assert!(empty_uniform.is_empty());
        assert_eq!(empty_uniform.total_size(), 0.0);
        assert_eq!(empty_uniform.get_offset(0), None);
        assert_eq!(empty_uniform.index_at(0.0), None);
        assert_eq!(empty_uniform.clamp_index_at(10.0), None);
        assert_eq!(empty_uniform.visible_range(0.0, 100.0), 0..0);
        assert_eq!(empty_uniform.find_boundary_near(0.0, 5.0), None);

        let empty_variable = LineIndex::variable(&[]);
        assert_eq!(empty_variable.count(), 0);
        assert!(empty_variable.is_empty());
        assert_eq!(empty_variable.total_size(), 0.0);
    }
}
