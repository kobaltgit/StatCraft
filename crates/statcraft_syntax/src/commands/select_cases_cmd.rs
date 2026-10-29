use crate::command::{CommandError, EngineCommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use statcraft_algo::CompiledExpr;
use statcraft_core::{
    DataSet, DataType, Measure, RawTable, RawValue, ValueLabels, VariableDictionary,
    VariableMetadata,
};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SelectCasesMode {
    All,
    Condition(String),
    FilterVar(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SelectCasesAction {
    Filter,
    Delete,
}

#[derive(Debug, Clone)]
pub struct SelectCasesUndoEntry {
    pub previous_filter_var: Option<String>,
    pub previous_table: Option<RawTable>,
    pub previous_dict: Option<VariableDictionary>,
    pub created_col: Option<usize>,
    pub overwritten_col: Option<(usize, VariableMetadata, Vec<RawValue>)>,
}

pub struct SelectCasesCommand {
    history: Mutex<Vec<SelectCasesUndoEntry>>,
}

impl Default for SelectCasesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectCasesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(mode: &SelectCasesMode, action: &SelectCasesAction) -> String {
        match mode {
            SelectCasesMode::All => "FILTER OFF.\nEXECUTE.".to_string(),
            SelectCasesMode::FilterVar(var) => match action {
                SelectCasesAction::Filter => format!("FILTER BY {}.\nEXECUTE.", var),
                SelectCasesAction::Delete => format!("SELECT IF ({} = 1).\nEXECUTE.", var),
            },
            SelectCasesMode::Condition(cond) => match action {
                SelectCasesAction::Filter => format!(
                    "COMPUTE filter_$ = ({}).\nVARIABLE LABELS filter_$ '{} (FILTER)'.\nVALUE LABELS filter_$ 0 'Not Selected' 1 'Selected'.\nFORMATS filter_$ (f1.0).\nFILTER BY filter_$.\nEXECUTE.",
                    cond, cond
                ),
                SelectCasesAction::Delete => format!("SELECT IF ({}).\nEXECUTE.", cond),
            },
        }
    }
}

impl EngineCommand for SelectCasesCommand {
    fn id(&self) -> &'static str {
        "data.select_cases"
    }

    fn menu_path(&self) -> &'static str {
        "Data > Select Cases"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "enum": ["all", "condition", "variable"]
                },
                "condition": { "type": "string" },
                "filter_var": { "type": "string" },
                "action": {
                    "type": "string",
                    "enum": ["filter", "delete"]
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        match dataset {
            Some(ds) => ds.col_count() > 0 && ds.row_count() > 0,
            None => false,
        }
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let mode_str = params.get("mode").and_then(|v| v.as_str()).unwrap_or("all");

        let action_str = params
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("filter");

        let action = if action_str.eq_ignore_ascii_case("delete") {
            SelectCasesAction::Delete
        } else {
            SelectCasesAction::Filter
        };

        match mode_str {
            "all" => {
                let undo_entry = SelectCasesUndoEntry {
                    previous_filter_var: ds.active_filter_var.clone(),
                    previous_table: None,
                    previous_dict: None,
                    created_col: None,
                    overwritten_col: None,
                };
                if let Ok(mut hist) = self.history.lock() {
                    hist.push(undo_entry);
                }
                ds.set_filter_variable(None)?;
                Ok(json!({ "mode": "all", "status": "filter_off" }))
            }
            "variable" => {
                let var_name = params
                    .get("filter_var")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: "Missing 'filter_var' parameter".to_string(),
                    })?;

                let col_idx = ds.dictionary.find_index(var_name).ok_or_else(|| {
                    CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Filter variable '{}' not found in dataset", var_name),
                    }
                })?;

                match action {
                    SelectCasesAction::Filter => {
                        let undo_entry = SelectCasesUndoEntry {
                            previous_filter_var: ds.active_filter_var.clone(),
                            previous_table: None,
                            previous_dict: None,
                            created_col: None,
                            overwritten_col: None,
                        };
                        if let Ok(mut hist) = self.history.lock() {
                            hist.push(undo_entry);
                        }
                        ds.set_filter_variable(Some(var_name.to_string()))?;
                        Ok(json!({ "mode": "variable", "filter_var": var_name }))
                    }
                    SelectCasesAction::Delete => {
                        let undo_entry = SelectCasesUndoEntry {
                            previous_filter_var: ds.active_filter_var.clone(),
                            previous_table: Some(ds.table.clone()),
                            previous_dict: Some(ds.dictionary.clone()),
                            created_col: None,
                            overwritten_col: None,
                        };
                        if let Ok(mut hist) = self.history.lock() {
                            hist.push(undo_entry);
                        }

                        let total_rows = ds.row_count();
                        let mut keep_indices = Vec::new();
                        for r in 0..total_rows {
                            let cell = ds.get_cell(r, col_idx);
                            let active = match cell {
                                Some(RawValue::Number(n)) => !n.is_nan() && *n != 0.0,
                                Some(RawValue::Text(s)) => {
                                    let trimmed = s.trim();
                                    trimmed == "1" || trimmed.eq_ignore_ascii_case("true")
                                }
                                Some(RawValue::Null) | None => false,
                            };
                            if active {
                                keep_indices.push(r);
                            }
                        }

                        for col in &mut ds.table.columns {
                            let new_vals: Vec<RawValue> = keep_indices
                                .iter()
                                .map(|&idx| col.values.get(idx).cloned().unwrap_or(RawValue::Null))
                                .collect();
                            col.values = new_vals;
                        }
                        ds.table.row_count = keep_indices.len();
                        ds.set_filter_variable(None)?;

                        Ok(json!({
                            "mode": "variable",
                            "action": "delete",
                            "kept_rows": keep_indices.len()
                        }))
                    }
                }
            }
            "condition" => {
                let condition = params
                    .get("condition")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: "Missing 'condition' parameter".to_string(),
                    })?;

                let compiled =
                    CompiledExpr::parse(condition).map_err(|e| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Failed to parse condition: {}", e),
                    })?;

                let total_rows = ds.row_count();
                let total_cols = ds.col_count();
                let mut condition_mask: Vec<bool> = Vec::with_capacity(total_rows);

                for r in 0..total_rows {
                    let mut row_map = HashMap::new();
                    for c in 0..total_cols {
                        if let Some(meta) = ds.dictionary.get(c) {
                            let cell_str = ds.get_cell_display(r, c, false);
                            if let Ok(num) = cell_str.trim().parse::<f64>() {
                                row_map.insert(meta.name.clone(), num);
                            }
                        }
                    }

                    let is_active = match compiled.eval_row(&row_map) {
                        Ok(v) => !v.is_nan() && v != 0.0,
                        Err(_) => false,
                    };
                    condition_mask.push(is_active);
                }

                match action {
                    SelectCasesAction::Delete => {
                        let undo_entry = SelectCasesUndoEntry {
                            previous_filter_var: ds.active_filter_var.clone(),
                            previous_table: Some(ds.table.clone()),
                            previous_dict: Some(ds.dictionary.clone()),
                            created_col: None,
                            overwritten_col: None,
                        };
                        if let Ok(mut hist) = self.history.lock() {
                            hist.push(undo_entry);
                        }

                        let keep_indices: Vec<usize> = condition_mask
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, &keep)| if keep { Some(idx) } else { None })
                            .collect();

                        for col in &mut ds.table.columns {
                            let new_vals: Vec<RawValue> = keep_indices
                                .iter()
                                .map(|&idx| col.values.get(idx).cloned().unwrap_or(RawValue::Null))
                                .collect();
                            col.values = new_vals;
                        }
                        ds.table.row_count = keep_indices.len();
                        ds.set_filter_variable(None)?;

                        Ok(json!({
                            "mode": "condition",
                            "action": "delete",
                            "kept_rows": keep_indices.len()
                        }))
                    }
                    SelectCasesAction::Filter => {
                        let filter_var_name = "filter_$";
                        let existing_col = ds.dictionary.find_index(filter_var_name);

                        let undo_entry = if let Some(c_idx) = existing_col {
                            let old_meta = ds.dictionary.get(c_idx).cloned().ok_or_else(|| {
                                CommandError::ExecutionFailed("Missing column metadata".into())
                            })?;
                            let old_vals = ds
                                .table
                                .columns
                                .get(c_idx)
                                .map(|c| c.values.clone())
                                .unwrap_or_default();
                            SelectCasesUndoEntry {
                                previous_filter_var: ds.active_filter_var.clone(),
                                previous_table: None,
                                previous_dict: None,
                                created_col: None,
                                overwritten_col: Some((c_idx, old_meta, old_vals)),
                            }
                        } else {
                            SelectCasesUndoEntry {
                                previous_filter_var: ds.active_filter_var.clone(),
                                previous_table: None,
                                previous_dict: None,
                                created_col: Some(ds.col_count()),
                                overwritten_col: None,
                            }
                        };

                        if let Ok(mut hist) = self.history.lock() {
                            hist.push(undo_entry);
                        }

                        let target_col_idx = if let Some(c_idx) = existing_col {
                            c_idx
                        } else {
                            let col_idx = ds.add_variable(
                                filter_var_name,
                                DataType::Numeric { decimals: 0 },
                                Measure::Nominal,
                            )?;
                            if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                                meta.label = format!("{} (FILTER)", condition);
                                let mut val_labels = ValueLabels::new();
                                val_labels.insert("0", "Not Selected");
                                val_labels.insert("1", "Selected");
                                meta.values = val_labels;
                            }
                            col_idx
                        };

                        for (r, &active) in condition_mask.iter().enumerate() {
                            let val = RawValue::Number(if active { 1.0 } else { 0.0 });
                            ds.set_cell(r, target_col_idx, val)?;
                        }

                        ds.set_filter_variable(Some(filter_var_name.to_string()))?;

                        let selected_count = condition_mask.iter().filter(|&&b| b).count();
                        Ok(json!({
                            "mode": "condition",
                            "filter_var": filter_var_name,
                            "selected": selected_count,
                            "total": total_rows
                        }))
                    }
                }
            }
            _ => Err(CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Unknown mode: {}", mode_str),
            }),
        }
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let entry = {
            let mut hist = self
                .history
                .lock()
                .map_err(|_| CommandError::ExecutionFailed("Mutex poisoned".into()))?;
            hist.pop()
        };

        let entry = match entry {
            Some(e) => e,
            None => return Ok(()),
        };

        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        if let (Some(tbl), Some(dict)) = (entry.previous_table, entry.previous_dict) {
            ds.table = tbl;
            ds.dictionary = dict;
        } else if let Some(created_idx) = entry.created_col {
            let _ = ds.remove_variable(created_idx);
        } else if let Some((col_idx, old_meta, old_vals)) = entry.overwritten_col {
            if col_idx < ds.col_count() {
                if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                    *meta = old_meta;
                }
                if let Some(col) = ds.table.columns.get_mut(col_idx) {
                    col.values = old_vals;
                }
            }
        }

        ds.active_filter_var = entry.previous_filter_var;
        Ok(())
    }
}
