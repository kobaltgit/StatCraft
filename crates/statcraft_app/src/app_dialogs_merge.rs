use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_add_cases_dialog, render_add_vars_dialog, MergeCasesAction, MergeVarsAction,
};
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_merge_dialogs(&mut self, ctx: &egui::Context) {
        // 1. Add Cases Dialog
        let cases_action = render_add_cases_dialog(
            ctx,
            self.dataset.as_ref(),
            Some(&self.workspace),
            &mut self.merge_cases_dialog_state,
        );

        match cases_action {
            MergeCasesAction::Run {
                file_path,
                in_variable,
                renames,
                interleave_by,
            } => {
                let params = json!({
                    "file_path": file_path,
                    "in_variable": in_variable,
                    "renames": renames,
                    "interleave_by": interleave_by,
                });

                match self
                    .command_bus
                    .execute("data.merge_cases", &mut self.dataset, params)
                {
                    Ok(val) => {
                        let sps = val.get("sps").and_then(|s| s.as_str()).unwrap_or("");
                        self.output_view_state
                            .add_entry(t!("output-log"), sps.to_string());
                        let rows = val.get("rows").and_then(|r| r.as_u64()).unwrap_or(0) as usize;
                        let cols = val.get("cols").and_then(|c| c.as_u64()).unwrap_or(0) as usize;
                        self.status_message = StatusMessage::MergeCasesExecuted { rows, cols };
                        if let Some(active_session) = self.workspace.active_session_mut() {
                            if let Some(ref ds) = self.dataset {
                                active_session.dataset = ds.clone();
                                active_session.mark_dirty();
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            MergeCasesAction::Paste {
                file_path,
                in_variable,
                renames,
                interleave_by,
            } => {
                let sps = statcraft_syntax::commands::MergeCasesCommand::to_sps(
                    &file_path,
                    in_variable.as_deref(),
                    &renames,
                    interleave_by.as_deref(),
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            MergeCasesAction::None => {}
        }

        // 2. Add Variables Dialog
        let vars_action = render_add_vars_dialog(
            ctx,
            self.dataset.as_ref(),
            Some(&self.workspace),
            &mut self.merge_vars_dialog_state,
        );

        match vars_action {
            MergeVarsAction::Run {
                file_path,
                method,
                keys,
                renames,
                drop_second,
                in_second,
            } => {
                let method_str = match method {
                    statcraft_core::dataset::merge::MergeMethod::OneToManyTable => "OneToManyTable",
                    statcraft_core::dataset::merge::MergeMethod::Sequential => "Sequential",
                    _ => "OneToOneWithKeys",
                };

                let params = json!({
                    "file_path": file_path,
                    "method": method_str,
                    "keys": keys,
                    "renames": renames,
                    "drop_second": drop_second,
                    "in_second_variable": in_second,
                });

                match self
                    .command_bus
                    .execute("data.merge_vars", &mut self.dataset, params)
                {
                    Ok(val) => {
                        let sps = val.get("sps").and_then(|s| s.as_str()).unwrap_or("");
                        self.output_view_state
                            .add_entry(t!("output-log"), sps.to_string());
                        let rows = val.get("rows").and_then(|r| r.as_u64()).unwrap_or(0) as usize;
                        let cols = val.get("cols").and_then(|c| c.as_u64()).unwrap_or(0) as usize;
                        self.status_message = StatusMessage::MergeVarsExecuted { rows, cols };
                        if let Some(active_session) = self.workspace.active_session_mut() {
                            if let Some(ref ds) = self.dataset {
                                active_session.dataset = ds.clone();
                                active_session.mark_dirty();
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            MergeVarsAction::Paste {
                file_path,
                method,
                keys,
                renames,
                drop_second,
                in_second,
            } => {
                let sps = statcraft_syntax::commands::MergeVarsCommand::to_sps(
                    &file_path,
                    method,
                    &keys,
                    &renames,
                    &drop_second,
                    in_second.as_deref(),
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            MergeVarsAction::None => {}
        }
    }
}
