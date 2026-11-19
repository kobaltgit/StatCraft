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
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
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

        let session = DocumentSession::new(id, final_name, dataset, path);
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
}
