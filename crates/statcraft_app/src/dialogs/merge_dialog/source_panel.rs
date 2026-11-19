use super::types::MergeSourceKind;
use crate::dialogs::common::GroupBox;
use egui::Ui;
use rfd::FileDialog;
use statcraft_core::workspace::WorkspaceManager;
use statcraft_i18n::t;

pub fn render_source_panel(
    ui: &mut Ui,
    source_kind: &mut MergeSourceKind,
    external_file_path: &mut String,
    selected_session_id: &mut Option<usize>,
    workspace: Option<&WorkspaceManager>,
) -> bool {
    let mut changed = false;

    GroupBox::new(t!("dialog-merge-source-group")).show(ui, |ui| {
        // Option 1: Open dataset from workspace
        let other_sessions: Vec<(usize, String)> = workspace
            .map(|ws| {
                ws.sessions()
                    .iter()
                    .filter(|s| Some(s.id) != ws.active_session_id())
                    .map(|s| (s.id, s.name.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let has_other_sessions = !other_sessions.is_empty();

        ui.horizontal(|ui| {
            let is_open_session = matches!(source_kind, MergeSourceKind::OpenSession(_));
            let radio_open = is_open_session;

            if ui
                .add_enabled(
                    has_other_sessions,
                    egui::RadioButton::new(radio_open, t!("dialog-merge-source-open-dataset")),
                )
                .clicked()
                && !is_open_session
                && has_other_sessions
            {
                if let Some(&(first_id, _)) = other_sessions.first() {
                    *source_kind = MergeSourceKind::OpenSession(first_id);
                    *selected_session_id = Some(first_id);
                    changed = true;
                }
            }

            if has_other_sessions {
                let current_name = selected_session_id
                    .and_then(|id| {
                        other_sessions
                            .iter()
                            .find(|(sid, _)| *sid == id)
                            .map(|(_, n)| n.as_str())
                    })
                    .unwrap_or("");

                egui::ComboBox::from_id_salt("merge_source_open_session")
                    .selected_text(current_name)
                    .show_ui(ui, |ui| {
                        for &(sid, ref sname) in &other_sessions {
                            let is_sel = *selected_session_id == Some(sid);
                            if ui.selectable_label(is_sel, sname).clicked() {
                                *selected_session_id = Some(sid);
                                *source_kind = MergeSourceKind::OpenSession(sid);
                                changed = true;
                            }
                        }
                    });
            }
        });

        ui.add_space(2.0);

        // Option 2: External file
        ui.horizontal(|ui| {
            let is_ext = matches!(source_kind, MergeSourceKind::ExternalFile);
            if ui
                .radio(is_ext, t!("dialog-merge-source-external-file"))
                .clicked()
                && !is_ext
            {
                *source_kind = MergeSourceKind::ExternalFile;
                changed = true;
            }

            ui.add(
                egui::TextEdit::singleline(external_file_path)
                    .desired_width(260.0)
                    .hint_text(t!("dialog-merge-select-file")),
            );

            if ui.button(t!("dialog-merge-browse-btn")).clicked() {
                let dialog = FileDialog::new()
                    .add_filter(
                        "All Data Files",
                        &["sav", "zsav", "stc", "csv", "xlsx", "xls"],
                    )
                    .add_filter("SPSS Data (*.sav, *.zsav)", &["sav", "zsav"])
                    .add_filter("StatCraft Data (*.stc)", &["stc"])
                    .add_filter("CSV (*.csv)", &["csv"])
                    .add_filter("Excel (*.xlsx, *.xls)", &["xlsx", "xls"]);

                if let Some(picked) = dialog.pick_file() {
                    let path_str = picked.to_string_lossy().to_string();
                    if !path_str.is_empty() {
                        *external_file_path = path_str;
                        *source_kind = MergeSourceKind::ExternalFile;
                        changed = true;
                    }
                }
            }
        });
    });

    changed
}
