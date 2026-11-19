use crate::app::StatCraftApp;
use crate::app_session::PendingSessionAction;
use statcraft_core::DataSet;

impl StatCraftApp {
    #[inline]
    pub fn current_dataset(&self) -> Option<&DataSet> {
        self.workspace.active_dataset()
    }

    #[inline]
    pub fn current_dataset_mut(&mut self) -> Option<&mut DataSet> {
        self.workspace.active_dataset_mut()
    }

    #[inline]
    pub fn active_dataset_display_name(&self) -> String {
        self.workspace
            .active_session()
            .map(|s| s.file_name().to_string())
            .unwrap_or_else(|| "DataSet1".to_string())
    }

    pub fn sync_active_dataset_to_workspace(&mut self) {
        if let Some(sess) = self.workspace.active_session_mut() {
            if let Some(ref ds) = self.dataset {
                if sess.dataset != *ds {
                    sess.dataset = ds.clone();
                    sess.mark_dirty();
                }
            }
        }
    }

    pub fn switch_to_dataset(&mut self, id: usize) {
        self.sync_active_dataset_to_workspace();
        if self.workspace.activate_dataset(id).is_ok() {
            self.dataset = self.workspace.active_dataset().cloned();
            self.current_file_path = self
                .workspace
                .active_session()
                .and_then(|s| s.file_path.clone());
        }
    }

    pub fn switch_to_next_dataset(&mut self) {
        self.sync_active_dataset_to_workspace();
        self.workspace.activate_next();
        self.dataset = self.workspace.active_dataset().cloned();
        self.current_file_path = self
            .workspace
            .active_session()
            .and_then(|s| s.file_path.clone());
    }

    pub fn switch_to_prev_dataset(&mut self) {
        self.sync_active_dataset_to_workspace();
        self.workspace.activate_prev();
        self.dataset = self.workspace.active_dataset().cloned();
        self.current_file_path = self
            .workspace
            .active_session()
            .and_then(|s| s.file_path.clone());
    }

    pub fn request_close_dataset(&mut self, id: usize) {
        self.sync_active_dataset_to_workspace();
        let is_dirty = self
            .workspace
            .get_session(id)
            .map(|s| s.is_dirty)
            .unwrap_or(false);
        if is_dirty {
            self.request_session_action(PendingSessionAction::CloseDataset(id));
        } else {
            self.execute_close_dataset(id);
        }
    }

    pub fn execute_close_dataset(&mut self, id: usize) {
        let _ = self.workspace.close_dataset(id);
        self.dataset = self.workspace.active_dataset().cloned();
        self.current_file_path = self
            .workspace
            .active_session()
            .and_then(|s| s.file_path.clone());
    }

    pub fn add_new_blank_dataset(&mut self) {
        self.sync_active_dataset_to_workspace();
        let blank_ds = DataSet::blank(40, 0).unwrap_or_else(|_| DataSet::empty());
        let name = self.workspace.next_default_name();
        self.workspace.add_dataset(blank_ds.clone(), name, None);
        self.dataset = Some(blank_ds);
        self.current_file_path = None;
    }
}
