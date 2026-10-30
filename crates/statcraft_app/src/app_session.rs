use super::app::{StatCraftApp, Tab};
use super::dialogs::{
    render_confirm_save_dialog, ConfirmSaveAction, SelectCasesDialogState, SplitFileDialogState,
    WeightCasesDialogState,
};
use super::status_message::StatusMessage;
use super::views::{DataViewState, OutputViewerState, VariableViewState};
use super::widgets::{render_menu_item, MenuItemStatus};
use egui::Ui;
use serde_json::json;
use statcraft_core::stc_codec::{
    load_project_stc_from_path, save_csv_to_path, save_project_stc_to_path,
};
use statcraft_core::DataSet;
use statcraft_i18n::t;
use statcraft_syntax::CommandBus;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingSessionAction {
    NewDataset,
    NewProject,
    CloseProject,
    OpenPath(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SavedCheckpoint {
    pub col_count: usize,
    pub row_count: usize,
    pub delta_undo_len: usize,
    pub command_history_len: usize,
    pub syntax_len: usize,
    pub output_procedures_count: usize,
}

impl StatCraftApp {
    pub fn current_checkpoint(&self) -> SavedCheckpoint {
        let (col_count, row_count, delta_undo_len) = if let Some(ds) = &self.dataset {
            (ds.col_count(), ds.row_count(), ds.delta.undo_stack.len())
        } else {
            (0, 0, 0)
        };
        SavedCheckpoint {
            col_count,
            row_count,
            delta_undo_len,
            command_history_len: self.command_bus.history().len(),
            syntax_len: self.syntax_editor_state.text.len(),
            output_procedures_count: self.output_view_state.procedures.len(),
        }
    }

    pub fn update_saved_checkpoint(&mut self) {
        self.saved_checkpoint = Some(self.current_checkpoint());
    }

    pub fn has_unsaved_changes(&self) -> bool {
        let current = self.current_checkpoint();
        match &self.saved_checkpoint {
            Some(saved) => saved != &current,
            None => {
                current.col_count > 0
                    || current.command_history_len > 0
                    || current.delta_undo_len > 0
                    || current.syntax_len > 0
                    || current.output_procedures_count > 0
            }
        }
    }

    pub fn request_session_action(&mut self, action: PendingSessionAction) {
        if self.has_unsaved_changes() {
            self.pending_session_action = Some(action);
            self.confirm_save_dialog_state.is_open = true;
        } else {
            self.execute_session_action(action);
        }
    }

    pub fn execute_session_action(&mut self, action: PendingSessionAction) {
        match action {
            PendingSessionAction::NewDataset => self.execute_new_dataset(),
            PendingSessionAction::NewProject => self.execute_new_project(),
            PendingSessionAction::CloseProject => self.execute_close_project(),
            PendingSessionAction::OpenPath(path) => self.load_file_from_path(path),
        }
    }

    pub fn execute_new_dataset(&mut self) {
        self.dataset = DataSet::blank(40, 0).ok();
        self.current_file_path = None;
        self.command_bus = CommandBus::with_defaults();
        self.data_view_state = DataViewState::default();
        self.variable_view_state = VariableViewState::default();
        self.select_cases_dialog_state = SelectCasesDialogState::default();
        self.split_file_dialog_state = SplitFileDialogState::default();
        self.weight_cases_dialog_state = WeightCasesDialogState::default();

        if let Some(pos) = self.dock_state.find_tab(&Tab::DataView) {
            self.dock_state.set_active_tab(pos);
        }
        self.status_message = StatusMessage::NewDatasetCreated;
        self.output_view_state.add_log("NEW DATASET.");
        self.update_saved_checkpoint();
    }

    pub fn execute_new_project(&mut self) {
        self.dataset = DataSet::blank(40, 0).ok();
        self.current_file_path = None;
        self.command_bus = CommandBus::with_defaults();
        self.data_view_state = DataViewState::default();
        self.variable_view_state = VariableViewState::default();
        self.output_view_state = OutputViewerState::new();
        self.syntax_editor_state.clear();
        self.select_cases_dialog_state = SelectCasesDialogState::default();
        self.split_file_dialog_state = SplitFileDialogState::default();
        self.weight_cases_dialog_state = WeightCasesDialogState::default();

        if let Some(pos) = self.dock_state.find_tab(&Tab::DataView) {
            self.dock_state.set_active_tab(pos);
        }
        self.status_message = StatusMessage::NewProjectCreated;
        self.update_saved_checkpoint();
    }

    pub fn execute_close_project(&mut self) {
        self.dataset = DataSet::blank(40, 0).ok();
        self.current_file_path = None;
        self.command_bus = CommandBus::with_defaults();
        self.data_view_state = DataViewState::default();
        self.variable_view_state = VariableViewState::default();
        self.output_view_state = OutputViewerState::new();
        self.syntax_editor_state.clear();
        self.select_cases_dialog_state = SelectCasesDialogState::default();
        self.split_file_dialog_state = SplitFileDialogState::default();
        self.weight_cases_dialog_state = WeightCasesDialogState::default();

        if let Some(pos) = self.dock_state.find_tab(&Tab::DataView) {
            self.dock_state.set_active_tab(pos);
        }
        self.status_message = StatusMessage::Ready;
        self.update_saved_checkpoint();
    }

    pub fn open_project_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(t!("dialog-open-project-title"))
            .add_filter(t!("dialog-filter-stc"), &["stc"])
            .pick_file()
        {
            self.request_session_action(PendingSessionAction::OpenPath(path));
        }
    }

    pub fn open_data_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(t!("dialog-open-data-title"))
            .add_filter(
                t!("dialog-filter-all-data"),
                &["stc", "sav", "zsav", "csv", "tsv", "txt", "xlsx", "xls"],
            )
            .add_filter(t!("dialog-filter-spss"), &["sav", "zsav"])
            .add_filter(t!("dialog-filter-stc"), &["stc"])
            .add_filter(t!("dialog-filter-csv"), &["csv", "tsv", "txt"])
            .pick_file()
        {
            self.request_session_action(PendingSessionAction::OpenPath(path));
        }
    }

    pub fn open_spss_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(t!("dialog-open-spss-title"))
            .add_filter(t!("dialog-filter-spss"), &["sav", "zsav"])
            .pick_file()
        {
            self.request_session_action(PendingSessionAction::OpenPath(path));
        }
    }

    pub fn open_excel_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(t!("dialog-open-excel-title"))
            .add_filter("Excel Spreadsheets", &["xlsx", "xls", "ods", "xlsm"])
            .pick_file()
        {
            self.request_session_action(PendingSessionAction::OpenPath(path));
        }
    }

    pub fn save_project(&mut self) {
        if let Some(path) = self.current_file_path.clone() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if ext == "stc" {
                self.save_to_path(path);
                return;
            }
        }
        self.save_project_as();
    }

    pub fn save_project_as(&mut self) {
        let initial_name = self
            .current_file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Dataset.stc");

        let mut dialog = rfd::FileDialog::new()
            .set_title(t!("dialog-save-project-title"))
            .add_filter(t!("dialog-filter-stc"), &["stc"])
            .set_file_name(initial_name);

        if let Some(parent) = self.current_file_path.as_ref().and_then(|p| p.parent()) {
            dialog = dialog.set_directory(parent);
        }

        if let Some(mut path) = dialog.save_file() {
            if path.extension().is_none() {
                path.set_extension("stc");
            }
            self.save_to_path(path);
        }
    }

    pub fn save_data_as(&mut self) {
        let initial_name = self
            .current_file_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|n| n.to_str())
            .map(|s| format!("{}.csv", s))
            .unwrap_or_else(|| "Dataset.csv".to_string());

        let mut dialog = rfd::FileDialog::new()
            .set_title(t!("dialog-save-data-title"))
            .add_filter(t!("dialog-filter-csv"), &["csv", "tsv", "txt"])
            .set_file_name(initial_name);

        if let Some(parent) = self.current_file_path.as_ref().and_then(|p| p.parent()) {
            dialog = dialog.set_directory(parent);
        }

        if let Some(mut path) = dialog.save_file() {
            if path.extension().is_none() {
                path.set_extension("csv");
            }
            self.save_to_path(path);
        }
    }

    pub fn save_to_path(&mut self, path: PathBuf) {
        let ds = match &self.dataset {
            Some(ds) => ds,
            None => {
                self.status_message = StatusMessage::Error(t!("status-no-dataset-to-save"));
                return;
            }
        };

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let is_csv = ext == "csv" || ext == "tsv" || ext == "txt";
        let save_res = if is_csv {
            save_csv_to_path(&path, ds)
        } else {
            let output_json = self.output_view_state.to_json().ok();
            let syntax_text = if self.syntax_editor_state.is_empty() {
                None
            } else {
                Some(self.syntax_editor_state.text.clone())
            };
            save_project_stc_to_path(&path, ds, output_json, syntax_text)
        };

        match save_res {
            Ok(()) => {
                let path_display = path.to_string_lossy().to_string();
                self.current_file_path = Some(path.clone());
                self.recent_files.add(path.clone());
                self.recent_files.save_default();

                if is_csv {
                    self.status_message = StatusMessage::DataSaved {
                        path: path_display.clone(),
                    };
                } else {
                    self.status_message = StatusMessage::ProjectSaved {
                        path: path_display.clone(),
                    };
                }

                self.output_view_state
                    .add_log(format!("SAVE OUTFILE='{}'.", path_display));
                self.update_saved_checkpoint();
            }
            Err(e) => {
                self.status_message = StatusMessage::Error(e.to_string());
            }
        }
    }

    pub fn load_file_from_path(&mut self, path: PathBuf) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Dataset")
            .to_string();

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if ext == "stc" {
            match load_project_stc_from_path(&path) {
                Ok((ds, output_viewer_json, syntax_text)) => {
                    let rows = ds.row_count();
                    let cols = ds.col_count();
                    self.dataset = Some(ds);
                    self.current_file_path = Some(path.clone());
                    self.recent_files.add(path.clone());
                    self.recent_files.save_default();

                    if let Some(json_str) = output_viewer_json {
                        let _ = self.output_view_state.load_from_json(&json_str);
                    }
                    if let Some(syn) = syntax_text {
                        self.syntax_editor_state.text = syn;
                    }

                    self.status_message = StatusMessage::ProjectOpened {
                        name: file_name.clone(),
                        rows,
                        cols,
                    };
                    self.output_view_state
                        .add_log(format!("GET FILE='{}'.", path.display()));
                    if let Some(pos) = self.dock_state.find_tab(&Tab::DataView) {
                        self.dock_state.set_active_tab(pos);
                    }
                    self.update_saved_checkpoint();
                }
                Err(e) => {
                    self.status_message = StatusMessage::Error(e.to_string());
                }
            }
            return;
        }

        let is_spss = matches!(ext.as_str(), "sav" | "zsav");
        let is_excel = matches!(ext.as_str(), "xlsx" | "xls" | "ods" | "xlsm");

        let res = if is_spss {
            let params = json!({
                "file_path": path.to_string_lossy(),
                "dataset_name": file_name
            });
            self.command_bus
                .execute("data.import_spss", &mut self.dataset, params)
        } else if is_excel {
            let params = json!({
                "file_path": path.to_string_lossy(),
                "dataset_name": file_name
            });
            self.command_bus
                .execute("data.import_excel", &mut self.dataset, params)
        } else {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    let params = json!({
                        "csv_data": content,
                        "dataset_name": file_name
                    });
                    self.command_bus
                        .execute("data.import_csv", &mut self.dataset, params)
                }
                Err(e) => Err(statcraft_syntax::CommandError::Io(e.to_string())),
            }
        };

        match res {
            Ok(res) => {
                let rows = res["rows"].as_u64().unwrap_or(0) as usize;
                let cols = res["cols"].as_u64().unwrap_or(0) as usize;
                self.current_file_path = Some(path.clone());
                self.recent_files.add(path.clone());
                self.recent_files.save_default();
                self.status_message = StatusMessage::DatasetImported {
                    name: file_name.clone(),
                    rows,
                    cols,
                };
                if is_spss {
                    self.output_view_state
                        .add_log(format!("GET FILE='{}'.", path.display()));
                } else if is_excel {
                    let type_str = if ext == "xls" { "XLS" } else { "XLSX" };
                    self.output_view_state.add_log(format!(
                        "GET DATA /TYPE={} /FILE='{}' /FIRSTROW=YES.",
                        type_str,
                        path.display()
                    ));
                }
                self.output_view_state
                    .add_dataset_imported(file_name, rows, cols);
                if let Some(pos) = self.dock_state.find_tab(&Tab::DataView) {
                    self.dock_state.set_active_tab(pos);
                }
                self.update_saved_checkpoint();
            }
            Err(e) => {
                self.status_message = StatusMessage::Error(e.to_string());
            }
        }
    }

    pub fn render_recent_files_menu(&mut self, ui: &mut Ui) {
        ui.menu_button(t!("menu-file-recent"), |ui| {
            if self.recent_files.is_empty() {
                let reason = t!("menu-file-recent-empty");
                render_menu_item(
                    ui,
                    t!("menu-file-recent-empty"),
                    MenuItemStatus::ContextDisabled { reason: &reason },
                );
            } else {
                let paths = self.recent_files.paths.clone();
                for path in paths {
                    let display_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");
                    let path_str = path.to_string_lossy().to_string();

                    if render_menu_item(ui, display_name, MenuItemStatus::Ready { shortcut: None })
                        .on_hover_text(path_str)
                        .clicked()
                    {
                        ui.close_menu();
                        self.request_session_action(PendingSessionAction::OpenPath(path));
                    }
                }

                ui.separator();
                if render_menu_item(
                    ui,
                    t!("menu-file-recent-clear"),
                    MenuItemStatus::Ready { shortcut: None },
                )
                .clicked()
                {
                    ui.close_menu();
                    self.recent_files.clear();
                    self.recent_files.save_default();
                }
            }
        });
    }

    pub fn handle_session_dialogs(&mut self, ctx: &egui::Context) {
        self.handle_confirm_save_dialog(ctx);
    }

    pub fn handle_confirm_save_dialog(&mut self, ctx: &egui::Context) {
        let action = render_confirm_save_dialog(ctx, &mut self.confirm_save_dialog_state);
        match action {
            ConfirmSaveAction::DontSave => {
                if let Some(act) = self.pending_session_action.take() {
                    self.execute_session_action(act);
                }
            }
            ConfirmSaveAction::Save => {
                if self.current_file_path.is_some() {
                    self.save_project();
                    if let Some(act) = self.pending_session_action.take() {
                        self.execute_session_action(act);
                    }
                } else {
                    self.save_project_as();
                    if self.current_file_path.is_some() {
                        if let Some(act) = self.pending_session_action.take() {
                            self.execute_session_action(act);
                        }
                    } else {
                        self.pending_session_action = None;
                    }
                }
            }
            ConfirmSaveAction::Cancel => {
                self.pending_session_action = None;
            }
            ConfirmSaveAction::None => {}
        }
    }
}
