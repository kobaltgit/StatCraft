use super::app::{StatCraftApp, StatusMessage, Tab};
use super::dialogs::{
    render_find_replace_dialog, render_goto_dialog, FindReplaceAction, GoToAction,
};
use egui::Context;
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_edit_dialogs(&mut self, ctx: &Context) {
        // 1. Find & Replace Dialog
        let find_action = render_find_replace_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.find_replace_dialog_state,
        );

        match find_action {
            FindReplaceAction::FindNext {
                find_text,
                match_case,
                match_entire_cell,
                search_down,
                target_col,
            } => {
                self.execute_find_next(
                    &find_text,
                    match_case,
                    match_entire_cell,
                    search_down,
                    target_col,
                );
            }
            FindReplaceAction::Replace {
                find_text,
                replace_text,
                match_case,
                match_entire_cell,
                search_down,
                target_col,
            } => {
                self.execute_single_replace(
                    &find_text,
                    &replace_text,
                    match_case,
                    match_entire_cell,
                    search_down,
                    target_col,
                );
            }
            FindReplaceAction::ReplaceAll {
                find_text,
                replace_text,
                match_case,
                match_entire_cell,
                target_col,
            } => {
                let mut params = json!({
                    "find": find_text,
                    "replace": replace_text,
                    "match_case": match_case,
                    "match_entire_cell": match_entire_cell
                });
                if let Some(col) = target_col {
                    params["target_col"] = json!(col);
                }

                match self
                    .command_bus
                    .execute("data.replace_values", &mut self.dataset, params)
                {
                    Ok(res) => {
                        let count = res
                            .get("replaced_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        self.find_replace_dialog_state.status_message =
                            Some(t!("dialog-find-status-replaced", count = count));
                        self.status_message = StatusMessage::Ready;
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            FindReplaceAction::Close | FindReplaceAction::None => {}
        }

        // 2. Go To Dialog
        let goto_action =
            render_goto_dialog(ctx, self.dataset.as_ref(), &mut self.goto_dialog_state);

        match goto_action {
            GoToAction::GoToCase(row) => {
                self.data_view_state.selected_cell = Some((row, 0));
                self.data_view_state.scroll_to_row = Some(row);
                if let Some(pos) = self.dock_state.find_tab(&Tab::DataView) {
                    self.dock_state.set_active_tab(pos);
                }
                self.status_message = StatusMessage::Ready;
            }
            GoToAction::GoToVariable(col) => {
                self.data_view_state.selected_cell = Some((0, col));
                self.data_view_state.scroll_to_col = Some(col);
                self.variable_view_state.selected_var = Some(col);
                self.variable_view_state.scroll_to_var = Some(col);
                if let Some(pos) = self.dock_state.find_tab(&Tab::DataView) {
                    self.dock_state.set_active_tab(pos);
                }
                self.status_message = StatusMessage::Ready;
            }
            GoToAction::Close | GoToAction::None => {}
        }
    }

    fn execute_find_next(
        &mut self,
        find_text: &str,
        match_case: bool,
        match_entire_cell: bool,
        search_down: bool,
        target_col: Option<usize>,
    ) {
        let ds = match self.dataset.as_ref() {
            Some(d) if d.row_count() > 0 && d.col_count() > 0 => d,
            _ => {
                self.find_replace_dialog_state.status_message =
                    Some(t!("dialog-find-status-not-found"));
                return;
            }
        };

        let total_rows = ds.row_count();
        let total_cols = ds.col_count();

        let start_pos = self
            .data_view_state
            .selected_cell
            .or(self.find_replace_dialog_state.last_match)
            .unwrap_or((0, 0));

        let cols_to_search: Vec<usize> = match target_col {
            Some(c) if c < total_cols => vec![c],
            _ => (0..total_cols).collect(),
        };

        let total_cells = total_rows * cols_to_search.len();
        let start_col_idx = cols_to_search
            .iter()
            .position(|&c| c == start_pos.1)
            .unwrap_or(0);
        let start_flat = (start_pos.0 * cols_to_search.len() + start_col_idx) as isize;

        let mut found = None;

        for step in 1..=total_cells {
            let current_flat = if search_down {
                (start_flat + step as isize).rem_euclid(total_cells as isize) as usize
            } else {
                (start_flat - step as isize).rem_euclid(total_cells as isize) as usize
            };

            let row = current_flat / cols_to_search.len();
            let col = cols_to_search[current_flat % cols_to_search.len()];

            let cell_str = ds.get_cell_display(row, col, false);
            let is_match = if match_entire_cell {
                if match_case {
                    cell_str == find_text
                } else {
                    cell_str.eq_ignore_ascii_case(find_text)
                }
            } else if match_case {
                cell_str.contains(find_text)
            } else {
                cell_str
                    .to_ascii_lowercase()
                    .contains(&find_text.to_ascii_lowercase())
            };

            if is_match {
                found = Some((row, col));
                break;
            }
        }

        if let Some((r, c)) = found {
            self.data_view_state.selected_cell = Some((r, c));
            self.data_view_state.scroll_to_row = Some(r);
            self.data_view_state.scroll_to_col = Some(c);
            self.find_replace_dialog_state.last_match = Some((r, c));

            let var_name = ds
                .dictionary
                .get(c)
                .map(|v| v.name.clone())
                .unwrap_or_else(|| format!("VAR{}", c + 1));
            self.find_replace_dialog_state.status_message =
                Some(t!("dialog-find-status-found", row = r + 1, var = var_name));
        } else {
            self.find_replace_dialog_state.status_message =
                Some(t!("dialog-find-status-not-found"));
        }
    }

    fn execute_single_replace(
        &mut self,
        find_text: &str,
        replace_text: &str,
        match_case: bool,
        match_entire_cell: bool,
        search_down: bool,
        target_col: Option<usize>,
    ) {
        if let Some((r, c)) = self.data_view_state.selected_cell {
            if let Some(ds) = self.dataset.as_ref() {
                let cell_str = ds.get_cell_display(r, c, false);
                let is_match = if match_entire_cell {
                    if match_case {
                        cell_str == find_text
                    } else {
                        cell_str.eq_ignore_ascii_case(find_text)
                    }
                } else if match_case {
                    cell_str.contains(find_text)
                } else {
                    cell_str
                        .to_ascii_lowercase()
                        .contains(&find_text.to_ascii_lowercase())
                };

                if is_match {
                    let new_val_str = if match_entire_cell {
                        replace_text.to_string()
                    } else if match_case {
                        cell_str.replace(find_text, replace_text)
                    } else {
                        let lower_cell = cell_str.to_ascii_lowercase();
                        let lower_find = find_text.to_ascii_lowercase();
                        let mut res = String::new();
                        let mut last = 0;
                        while let Some(idx) = lower_cell[last..].find(&lower_find) {
                            let abs = last + idx;
                            res.push_str(&cell_str[last..abs]);
                            res.push_str(replace_text);
                            last = abs + find_text.len();
                        }
                        res.push_str(&cell_str[last..]);
                        res
                    };

                    let params = json!({
                        "replacements": [
                            { "row": r, "col": c, "new_value": new_val_str }
                        ]
                    });

                    let _ =
                        self.command_bus
                            .execute("data.replace_values", &mut self.dataset, params);
                }
            }
        }

        self.execute_find_next(
            find_text,
            match_case,
            match_entire_cell,
            search_down,
            target_col,
        );
    }

    pub fn copy_selected_cell(&mut self, ctx: &Context) {
        if let Some((r, c)) = self.data_view_state.selected_cell {
            if let Some(ds) = &self.dataset {
                let val = ds.get_cell_display(r, c, self.data_view_state.show_value_labels);
                self.clipboard_cache = Some(val.clone());
                ctx.output_mut(|o| o.copied_text = val);
                self.status_message = StatusMessage::Custom(t!("status-cell-copied"));
            }
        }
    }

    pub fn copy_with_variable_names(&mut self, ctx: &Context) {
        if let Some((r, c)) = self.data_view_state.selected_cell {
            if let Some(ds) = &self.dataset {
                let var_name = ds
                    .dictionary
                    .get(c)
                    .map(|v| v.name.as_str())
                    .unwrap_or("VAR");
                let val = ds.get_cell_display(r, c, self.data_view_state.show_value_labels);
                let text = format!("{}\n{}", var_name, val);
                self.clipboard_cache = Some(text.clone());
                ctx.output_mut(|o| o.copied_text = text);
                self.status_message = StatusMessage::Custom(t!("status-cell-copied"));
            }
        }
    }

    pub fn cut_selected_cell(&mut self, ctx: &Context) {
        self.copy_selected_cell(ctx);
        self.clear_selected_cell();
        self.status_message = StatusMessage::Custom(t!("status-cell-cut"));
    }

    pub fn paste_selected_cell(&mut self, _ctx: &Context) {
        if let Some((r, c)) = self.data_view_state.selected_cell {
            if let Some(ref text) = self.clipboard_cache {
                let params = json!({
                    "row": r,
                    "col": c,
                    "value": text
                });
                let _ = self
                    .command_bus
                    .execute("data.cell_edit", &mut self.dataset, params);
                self.status_message = StatusMessage::Custom(t!("status-cell-pasted"));
            }
        }
    }

    pub fn clear_selected_cell(&mut self) {
        if let Some((r, c)) = self.data_view_state.selected_cell {
            let params = json!({
                "row": r,
                "col": c,
                "value": ""
            });
            let _ = self
                .command_bus
                .execute("data.cell_edit", &mut self.dataset, params);
            self.status_message = StatusMessage::Custom(t!("status-cell-cleared"));
        }
    }

    pub fn insert_case_action(&mut self) {
        let row_idx = self
            .data_view_state
            .selected_cell
            .map(|(r, _)| r)
            .unwrap_or_else(|| self.dataset.as_ref().map(|d| d.row_count()).unwrap_or(0));

        let params = json!({ "row_index": row_idx });
        if self
            .command_bus
            .execute("data.add_row", &mut self.dataset, params)
            .is_ok()
        {
            self.data_view_state.selected_cell = Some((row_idx, 0));
            self.data_view_state.scroll_to_row = Some(row_idx);
            self.status_message =
                StatusMessage::Custom(t!("status-case-inserted", row = row_idx + 1));
        }
    }

    pub fn insert_variable_action(&mut self) {
        let (col_idx, var_name) = {
            let col = self
                .data_view_state
                .selected_cell
                .map(|(_, c)| c)
                .unwrap_or_else(|| self.dataset.as_ref().map(|d| d.col_count()).unwrap_or(0));

            let total_cols = self.dataset.as_ref().map(|d| d.col_count()).unwrap_or(0);
            let mut name = format!("VAR{:05}", total_cols + 1);
            let mut attempt = 1;
            while self
                .dataset
                .as_ref()
                .map(|d| d.dictionary.find_by_name(&name).is_some())
                .unwrap_or(false)
            {
                attempt += 1;
                name = format!("VAR{:05}", total_cols + attempt);
            }
            (col, name)
        };

        let params = json!({
            "name": var_name,
            "col_index": col_idx
        });

        if self
            .command_bus
            .execute("var.add", &mut self.dataset, params)
            .is_ok()
        {
            self.data_view_state.selected_cell = Some((0, col_idx));
            self.data_view_state.scroll_to_col = Some(col_idx);
            self.status_message = StatusMessage::Custom(t!("status-var-inserted", name = var_name));
        }
    }

    pub fn open_find_dialog(&mut self) {
        self.find_replace_dialog_state.open_find();
    }

    pub fn open_replace_dialog(&mut self) {
        self.find_replace_dialog_state.open_replace();
    }

    pub fn open_goto_case_dialog(&mut self) {
        let current_case = self.data_view_state.selected_cell.map(|(r, _)| r);
        self.goto_dialog_state.open_case(current_case);
    }

    pub fn open_goto_var_dialog(&mut self) {
        let current_var = self.data_view_state.selected_cell.map(|(_, c)| c);
        self.goto_dialog_state.open_variable(current_var);
    }
}
