use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{apply_recode, RecodeMapping, RecodePlan};
use statcraft_core::{DataSet, DataType, Measure, VariableMetadata};
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum DiffUndoItem {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

pub struct RecodeIntoDifferentCommand {
    history: Mutex<Vec<Vec<DiffUndoItem>>>,
}

impl Default for RecodeIntoDifferentCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl RecodeIntoDifferentCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(pairs: &[(&str, &str, Option<&str>)], plan: &RecodePlan) -> String {
        let mut sps = String::new();
        let plan_sps = plan.to_sps();
        for (source, target, label) in pairs {
            sps.push_str(&format!(
                "RECODE {} {} INTO {}.\n",
                source, plan_sps, target
            ));
            if let Some(lbl) = label {
                let trimmed = lbl.trim();
                if !trimmed.is_empty() {
                    sps.push_str(&format!("VARIABLE LABELS {} \"{}\".\n", target, trimmed));
                }
            }
        }
        sps.push_str("EXECUTE.");
        sps
    }
}

pub(crate) fn parse_recode_plan(
    params: &Value,
    cmd_id: &'static str,
) -> Result<RecodePlan, CommandError> {
    if let Some(m_val) = params.get("mappings") {
        if m_val.is_array() {
            let mappings: Vec<RecodeMapping> =
                serde_json::from_value(m_val.clone()).map_err(|e| CommandError::InvalidParams {
                    id: cmd_id,
                    reason: format!("Failed to parse recode mappings array: {}", e),
                })?;
            Ok(RecodePlan::new(mappings))
        } else if let Ok(plan) = serde_json::from_value::<RecodePlan>(m_val.clone()) {
            Ok(plan)
        } else {
            Err(CommandError::InvalidParams {
                id: cmd_id,
                reason: "Invalid recode mappings specification".to_string(),
            })
        }
    } else {
        Ok(RecodePlan::default())
    }
}

impl EngineCommand for RecodeIntoDifferentCommand {
    fn id(&self) -> &'static str {
        "transform.recode.diff"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Recode into Different Variables"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["mappings"],
            "properties": {
                "source": { "type": "string" },
                "target": { "type": "string" },
                "label": { "type": "string" },
                "pairs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["source", "target"],
                        "properties": {
                            "source": { "type": "string" },
                            "target": { "type": "string" },
                            "label": { "type": "string" }
                        }
                    }
                },
                "mappings": { "type": "array" }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|d| d.col_count() > 0).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let plan = parse_recode_plan(&params, self.id())?;

        let mut pairs_to_process: Vec<(String, String, Option<String>)> = Vec::new();
        if let Some(pairs_arr) = params.get("pairs").and_then(|v| v.as_array()) {
            for item in pairs_arr {
                let src = item
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let tgt = item
                    .get("target")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let lbl = item
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                if let (Some(s), Some(t)) = (src, tgt) {
                    pairs_to_process.push((s, t, lbl));
                }
            }
        } else if let (Some(s), Some(t)) = (
            params.get("source").and_then(|v| v.as_str()),
            params.get("target").and_then(|v| v.as_str()),
        ) {
            let lbl = params
                .get("label")
                .and_then(|v| v.as_str())
                .map(|l| l.trim().to_string());
            pairs_to_process.push((s.trim().to_string(), t.trim().to_string(), lbl));
        }

        if pairs_to_process.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "No valid source -> target pairs provided".to_string(),
            });
        }

        let mut batch_undo = Vec::new();
        let mut recoded_targets = Vec::new();

        for (source_var, target_var, label_opt) in pairs_to_process {
            let src_idx = ds.dictionary.find_index(&source_var).ok_or_else(|| {
                CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Source variable '{}' not found", source_var),
                }
            })?;

            let src_meta = ds.dictionary.get(src_idx).cloned().ok_or_else(|| {
                CommandError::ExecutionFailed("Dictionary metadata missing".to_string())
            })?;

            let row_count = ds.row_count();
            let mut input_vals = Vec::with_capacity(row_count);
            for row in 0..row_count {
                let cell = ds.get_cell(row, src_idx);
                let num_val = match cell {
                    Some(RawValue::Number(n)) => {
                        if src_meta.missing.is_missing(&RawValue::Number(*n)) {
                            None
                        } else {
                            Some(*n)
                        }
                    }
                    Some(RawValue::Text(s)) => s.parse::<f64>().ok(),
                    _ => None,
                };
                input_vals.push(num_val);
            }

            let is_user_missing = |val: f64| src_meta.missing.is_missing(&RawValue::Number(val));
            let transformed = apply_recode(&input_vals, &plan, is_user_missing);

            let new_raw_vals: Vec<RawValue> = transformed
                .into_iter()
                .map(|opt| match opt {
                    Some(v) => RawValue::Number(v),
                    None => RawValue::Null,
                })
                .collect();

            let existing_idx = ds.dictionary.find_index(&target_var);
            let undo_item = if let Some(target_idx) = existing_idx {
                let old_meta = ds.dictionary.get(target_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Dictionary metadata missing".to_string())
                })?;
                let old_values = ds.table.columns[target_idx].values.clone();

                ds.table.columns[target_idx].values = new_raw_vals;
                if let Some(lbl) = label_opt {
                    if !lbl.is_empty() {
                        if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                            meta.label = lbl;
                        }
                    }
                }

                DiffUndoItem::Overwritten {
                    col_idx: target_idx,
                    old_meta,
                    old_values,
                }
            } else {
                let target_idx = ds.add_variable(
                    &target_var,
                    DataType::Numeric { decimals: 2 },
                    Measure::Scale,
                )?;
                if let Some(lbl) = label_opt {
                    if !lbl.is_empty() {
                        if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                            meta.label = lbl;
                        }
                    }
                }
                ds.table.columns[target_idx].values = new_raw_vals;

                DiffUndoItem::Created {
                    col_idx: target_idx,
                }
            };

            batch_undo.push(undo_item);
            recoded_targets.push(target_var);
        }

        if let Ok(mut hist) = self.history.lock() {
            hist.push(batch_undo);
        }

        Ok(json!({
            "status": "success",
            "targets": recoded_targets,
            "rows": ds.row_count()
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let batch = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        if let Some(items) = batch {
            for item in items.into_iter().rev() {
                match item {
                    DiffUndoItem::Created { col_idx } => {
                        ds.remove_variable(col_idx)?;
                    }
                    DiffUndoItem::Overwritten {
                        col_idx,
                        old_meta,
                        old_values,
                    } => {
                        if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                            *meta = old_meta;
                        }
                        if let Some(col) = ds.table.columns.get_mut(col_idx) {
                            col.values = old_values;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
