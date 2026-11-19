use crate::dataset::DataSet;
use crate::variable::CoreError;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum WorkspaceError {
    #[error("Session id {0} not found")]
    NotFound(usize),

    #[error("Dataset with name '{0}' not found in workspace")]
    NameNotFound(String),

    #[error("Invalid dataset name: {0}")]
    InvalidName(String),

    #[error("Core error: {0}")]
    Core(#[from] CoreError),
}

/// Individual document session holding an isolated dataset, file association,
/// view state (scroll/selection), and modification status.
#[derive(Debug, Clone)]
pub struct DocumentSession {
    pub id: usize,
    pub name: String,
    pub file_path: Option<PathBuf>,
    pub dataset: DataSet,
    pub is_dirty: bool,
    pub scroll_offset: (f32, f32),
    pub selected_cell: Option<(usize, usize)>,
    pub delta_undo_checkpoint_len: usize,
    /// Persistent hex color code of the tab (e.g. "#2E86AB")
    pub tab_color: Option<String>,
}

impl DocumentSession {
    pub fn new(
        id: usize,
        name: impl Into<String>,
        dataset: DataSet,
        file_path: Option<PathBuf>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            file_path,
            dataset,
            is_dirty: false,
            scroll_offset: (0.0, 0.0),
            selected_cell: None,
            delta_undo_checkpoint_len: 0,
            tab_color: None,
        }
    }

    pub fn set_tab_color(&mut self, color: impl Into<String>) {
        self.tab_color = Some(color.into());
    }

    pub fn tab_color(&self) -> Option<&str> {
        self.tab_color.as_deref()
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
        self.delta_undo_checkpoint_len = self.dataset.delta.undo_stack.len();
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
        self.delta_undo_checkpoint_len = self.dataset.delta.undo_stack.len();
    }

    pub fn can_undo(&self) -> bool {
        self.dataset.delta.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.dataset.delta.can_redo()
    }

    pub fn undo(&mut self) -> Result<(), CoreError> {
        let res = self.dataset.delta.undo(&mut self.dataset.table);
        if res.is_ok() {
            self.is_dirty = self.dataset.delta.undo_stack.len() != self.delta_undo_checkpoint_len;
        }
        res
    }

    pub fn redo(&mut self) -> Result<(), CoreError> {
        let res = self.dataset.delta.redo(&mut self.dataset.table);
        if res.is_ok() {
            self.is_dirty = self.dataset.delta.undo_stack.len() != self.delta_undo_checkpoint_len;
        }
        res
    }

    pub fn file_name(&self) -> &str {
        if let Some(ref path) = self.file_path {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                return name;
            }
        }
        &self.name
    }
}

/// Workspace manager governing multiple document sessions, tab activation,
/// dataset switching, and global persistence states.
#[derive(Debug, Clone)]
pub struct WorkspaceManager {
    sessions: Vec<DocumentSession>,
    active_session_id: Option<usize>,
    next_id: usize,
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new_blank()
    }
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            active_session_id: None,
            next_id: 1,
        }
    }

    /// Initializes a workspace with a single clean DataSet1 (40 rows, 0 cols).
    pub fn new_blank() -> Self {
        let mut ws = Self::new();
        let blank_ds = DataSet::blank(40, 0).unwrap_or_else(|_| DataSet::empty());
        ws.add_dataset(blank_ds, "DataSet1", None);
        ws
    }

    pub fn next_default_name(&self) -> String {
        let mut idx = 1;
        loop {
            let candidate = format!("DataSet{}", idx);
            if !self
                .sessions
                .iter()
                .any(|s| s.name.eq_ignore_ascii_case(&candidate))
            {
                return candidate;
            }
            idx += 1;
        }
    }

    pub fn add_dataset(
        &mut self,
        dataset: DataSet,
        name: impl Into<String>,
        path: Option<PathBuf>,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let name_str = name.into();
        let final_name = if name_str.trim().is_empty() {
            self.next_default_name()
        } else {
            name_str
        };

        let mut session = DocumentSession::new(id, final_name, dataset, path);
        let existing = self.existing_tab_colors();
        session.tab_color = Some(generate_contrast_hex(&existing, &session.name));
        self.sessions.push(session);
        self.active_session_id = Some(id);
        id
    }

    pub fn activate_dataset(&mut self, id: usize) -> Result<(), WorkspaceError> {
        if self.sessions.iter().any(|s| s.id == id) {
            self.active_session_id = Some(id);
            Ok(())
        } else {
            Err(WorkspaceError::NotFound(id))
        }
    }

    pub fn activate_by_name(&mut self, name: &str) -> Result<usize, WorkspaceError> {
        let target = self
            .sessions
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name) || s.file_name().eq_ignore_ascii_case(name))
            .map(|s| s.id);

        if let Some(id) = target {
            self.active_session_id = Some(id);
            Ok(id)
        } else {
            Err(WorkspaceError::NameNotFound(name.to_string()))
        }
    }

    pub fn activate_next(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let current_idx = self
            .active_session_id
            .and_then(|id| self.sessions.iter().position(|s| s.id == id))
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.sessions.len();
        self.active_session_id = Some(self.sessions[next_idx].id);
    }

    pub fn activate_prev(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let current_idx = self
            .active_session_id
            .and_then(|id| self.sessions.iter().position(|s| s.id == id))
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.sessions.len().saturating_sub(1)
        } else {
            current_idx - 1
        };
        self.active_session_id = Some(self.sessions[prev_idx].id);
    }

    pub fn close_dataset(&mut self, id: usize) -> Result<(), WorkspaceError> {
        let pos = self
            .sessions
            .iter()
            .position(|s| s.id == id)
            .ok_or(WorkspaceError::NotFound(id))?;

        self.sessions.remove(pos);

        // If closed dataset was active, select adjacent or fall back
        if self.active_session_id == Some(id) {
            if self.sessions.is_empty() {
                // Keep workspace alive with a fresh blank dataset
                let blank_ds = DataSet::blank(40, 0).unwrap_or_else(|_| DataSet::empty());
                let new_id = self.add_dataset(blank_ds, "DataSet1", None);
                self.active_session_id = Some(new_id);
            } else {
                let new_idx = pos.min(self.sessions.len().saturating_sub(1));
                self.active_session_id = Some(self.sessions[new_idx].id);
            }
        }
        Ok(())
    }

    pub fn rename_dataset(
        &mut self,
        id: usize,
        new_name: impl Into<String>,
    ) -> Result<(), WorkspaceError> {
        let name_str = new_name.into();
        if name_str.trim().is_empty() {
            return Err(WorkspaceError::InvalidName(
                "Dataset name cannot be empty".to_string(),
            ));
        }
        if let Some(sess) = self.sessions.iter_mut().find(|s| s.id == id) {
            sess.name = name_str;
            Ok(())
        } else {
            Err(WorkspaceError::NotFound(id))
        }
    }

    pub fn active_session(&self) -> Option<&DocumentSession> {
        self.active_session_id
            .and_then(|id| self.sessions.iter().find(|s| s.id == id))
            .or_else(|| self.sessions.first())
    }

    pub fn active_session_mut(&mut self) -> Option<&mut DocumentSession> {
        let active_id = self.active_session_id?;
        self.sessions.iter_mut().find(|s| s.id == active_id)
    }

    pub fn active_dataset(&self) -> Option<&DataSet> {
        self.active_session().map(|s| &s.dataset)
    }

    pub fn active_dataset_mut(&mut self) -> Option<&mut DataSet> {
        self.active_session_mut().map(|s| &mut s.dataset)
    }

    pub fn get_session(&self, id: usize) -> Option<&DocumentSession> {
        self.sessions.iter().find(|s| s.id == id)
    }

    pub fn get_session_mut(&mut self, id: usize) -> Option<&mut DocumentSession> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&DocumentSession> {
        self.sessions
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name) || s.file_name().eq_ignore_ascii_case(name))
    }

    pub fn sessions(&self) -> &[DocumentSession] {
        &self.sessions
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn has_any_unsaved(&self) -> bool {
        self.sessions.iter().any(|s| s.is_dirty)
    }

    pub fn is_any_dirty(&self) -> bool {
        self.has_any_unsaved()
    }

    pub fn mark_all_clean(&mut self) {
        for s in &mut self.sessions {
            s.mark_clean();
        }
    }

    pub fn active_session_id(&self) -> Option<usize> {
        self.active_session_id
    }

    pub fn update_active_path(&mut self, path: impl AsRef<Path>) {
        if let Some(sess) = self.active_session_mut() {
            let p = path.as_ref().to_path_buf();
            sess.file_path = Some(p);
        }
    }

    pub fn session_color_by_name(&self, name: &str) -> Option<String> {
        self.sessions
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name) || s.file_name().eq_ignore_ascii_case(name))
            .and_then(|s| s.tab_color.clone())
    }

    pub fn existing_tab_colors(&self) -> Vec<String> {
        self.sessions
            .iter()
            .filter_map(|s| s.tab_color.clone())
            .collect()
    }

    pub fn sessions_mut(&mut self) -> &mut [DocumentSession] {
        &mut self.sessions
    }
}

/// Calculates a high-contrast HEX color code (#RRGGBB) avoiding angular collisions (< 45 deg)
/// with any already open tabs.
pub fn generate_contrast_hex(existing_hex_colors: &[String], seed_str: &str) -> String {
    let mut h: u64 = 0xCBF29CE484222325;
    for &b in seed_str.as_bytes() {
        h = (h ^ (b as u64)).wrapping_mul(0x100000001B3);
    }
    // SplitMix64
    let mut sm = h.wrapping_add(0x9E3779B97F4A7C15);
    sm = (sm ^ (sm >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    sm = (sm ^ (sm >> 27)).wrapping_mul(0x94D049BB133111EB);
    let hash = sm ^ (sm >> 31);

    let mut base_hue = ((hash % 360) as f32) + ((hash >> 16) % 1000) as f32 / 1000.0;

    let existing_hues: Vec<f32> = existing_hex_colors
        .iter()
        .filter_map(|hex| hex_to_rgb(hex))
        .map(|(r, g, b)| rgb_to_hue(r, g, b))
        .collect();

    const GOLDEN_ANGLE: f32 = 137.507_77;
    if !existing_hues.is_empty() {
        let mut best_hue = base_hue;
        let mut max_min_dist = -1.0_f32;
        for _ in 0..32 {
            let min_d = existing_hues
                .iter()
                .map(|&eh| {
                    let diff = (base_hue - eh).abs().rem_euclid(360.0);
                    if diff > 180.0 {
                        360.0 - diff
                    } else {
                        diff
                    }
                })
                .fold(f32::INFINITY, f32::min);

            if min_d >= 45.0 {
                best_hue = base_hue;
                break;
            }
            if min_d > max_min_dist {
                max_min_dist = min_d;
                best_hue = base_hue;
            }
            base_hue = (base_hue + GOLDEN_ANGLE).rem_euclid(360.0);
        }
        base_hue = best_hue;
    }

    let sat_offset = ((hash >> 32) % 200) as f32 / 1000.0;
    let saturation = 0.65 + sat_offset;
    let light_offset = ((hash >> 48) % 100) as f32 / 1000.0;
    let lightness = 0.52 + light_offset;

    let (r, g, b) = hsl_to_rgb(base_hue, saturation, lightness);
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some((r, g, b))
    } else if clean.len() == 3 {
        let r = u8::from_str_radix(&clean[0..1], 16).ok()? * 17;
        let g = u8::from_str_radix(&clean[1..2], 16).ok()? * 17;
        let b = u8::from_str_radix(&clean[2..3], 16).ok()? * 17;
        Some((r, g, b))
    } else {
        None
    }
}

fn rgb_to_hue(r: u8, g: u8, b: u8) -> f32 {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;
    if delta.abs() < 1e-5 {
        return 0.0;
    }
    if (max - rf).abs() < 1e-5 {
        ((gf - bf) / delta).rem_euclid(6.0) * 60.0
    } else if (max - gf).abs() < 1e-5 {
        ((bf - rf) / delta + 2.0) * 60.0
    } else {
        ((rf - gf) / delta + 4.0) * 60.0
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = if (0.0..1.0).contains(&h_prime) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h_prime) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h_prime) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h_prime) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h_prime) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = l - c / 2.0;
    let r = ((r1 + m) * 255.0).clamp(0.0, 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).clamp(0.0, 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).clamp(0.0, 255.0).round() as u8;
    (r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_manager_lifecycle() {
        let mut ws = WorkspaceManager::new();
        assert_eq!(ws.session_count(), 0);

        let ds1 = DataSet::blank(10, 0).unwrap();
        let id1 = ws.add_dataset(ds1, "DataSet1", None);
        assert_eq!(ws.session_count(), 1);
        assert_eq!(ws.active_session_id(), Some(id1));

        let ds2 = DataSet::blank(20, 0).unwrap();
        let id2 = ws.add_dataset(ds2, "clients.csv", Some(PathBuf::from("clients.csv")));
        assert_eq!(ws.session_count(), 2);
        assert_eq!(ws.active_session_id(), Some(id2));
        assert_eq!(ws.active_dataset().unwrap().row_count(), 20);

        // Switch next and prev
        ws.activate_next();
        assert_eq!(ws.active_session_id(), Some(id1));
        assert_eq!(ws.active_dataset().unwrap().row_count(), 10);

        ws.activate_prev();
        assert_eq!(ws.active_session_id(), Some(id2));

        // Activate by name
        ws.activate_by_name("dataset1").unwrap();
        assert_eq!(ws.active_session_id(), Some(id1));

        // Mark dirty
        ws.active_session_mut().unwrap().mark_dirty();
        assert!(ws.has_any_unsaved());

        // Close dataset
        ws.close_dataset(id1).unwrap();
        assert_eq!(ws.session_count(), 1);
        assert_eq!(ws.active_session_id(), Some(id2));

        // Close last dataset: should auto-respawn DataSet1
        ws.close_dataset(id2).unwrap();
        assert_eq!(ws.session_count(), 1);
        assert_eq!(ws.active_session().unwrap().name, "DataSet1");
    }

    #[test]
    fn test_dynamic_contrast_colors_for_similar_prefixes() {
        let mut ws = WorkspaceManager::new();
        let ds1 = DataSet::empty();
        let ds2 = DataSet::empty();
        let ds3 = DataSet::empty();

        let id1 = ws.add_dataset(ds1, "merge_base.csv", None);
        let id2 = ws.add_dataset(ds2, "merge_add_cases.csv", None);
        let id3 = ws.add_dataset(ds3, "merge_add_vars.csv", None);

        let color1 = ws.get_session(id1).unwrap().tab_color.as_ref().unwrap();
        let color2 = ws.get_session(id2).unwrap().tab_color.as_ref().unwrap();
        let color3 = ws.get_session(id3).unwrap().tab_color.as_ref().unwrap();

        assert_ne!(color1, color2);
        assert_ne!(color1, color3);
        assert_ne!(color2, color3);

        let (r1, g1, b1) = hex_to_rgb(color1).unwrap();
        let (r2, g2, b2) = hex_to_rgb(color2).unwrap();
        let (r3, g3, b3) = hex_to_rgb(color3).unwrap();

        let h1 = rgb_to_hue(r1, g1, b1);
        let h2 = rgb_to_hue(r2, g2, b2);
        let h3 = rgb_to_hue(r3, g3, b3);

        let dist = |a: f32, b: f32| {
            let d = (a - b).abs().rem_euclid(360.0);
            if d > 180.0 {
                360.0 - d
            } else {
                d
            }
        };

        assert!(
            dist(h1, h2) >= 45.0,
            "Hues h1 ({h1}) and h2 ({h2}) must have distance >= 45 deg, got {}",
            dist(h1, h2)
        );
        assert!(
            dist(h2, h3) >= 45.0,
            "Hues h2 ({h2}) and h3 ({h3}) must have distance >= 45 deg, got {}",
            dist(h2, h3)
        );
        assert!(
            dist(h1, h3) >= 45.0,
            "Hues h1 ({h1}) and h3 ({h3}) must have distance >= 45 deg, got {}",
            dist(h1, h3)
        );
    }
}
