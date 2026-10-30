use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_auto_recode, AutoRecodeOrder};
use statcraft_core::{DataSet, DataType, Measure, VariableMetadata};
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum AutoRecodeUndoItem {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

pub struct AutoRecodeCommand {
    history: Mutex<Vec<Vec<AutoRecodeUndoItem>>>,
}

impl Default for AutoRecodeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoRecodeCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(pairs: &[(&str, &str)], order: AutoRecodeOrder) -> String {
        if pairs.is_empty() {
            return String::new();
        }

        let mut sources = Vec::with_capacity(pairs.len());
        let mut targets = Vec::with_capacity(pairs.len());

        for (src, tgt) in pairs {
            sources.push(*src);
            targets.push(*tgt);
        }

        let mut sps = format!(
            "AUTORECODE VARIABLES={} /INTO {}\n",
            sources.join(" "),
            targets.join(" ")
        );

        if order == AutoRecodeOrder::HighestValue {
            sps.push_str("  /DESCENDING\n");
        }
        sps.push_str("  /PRINT.\nEXECUTE.");
        sps
    }
}

impl EngineCommand for AutoRecodeCommand {
    fn id(&self) -> &'static str {
        "transform.auto_recode"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Automatic Recode"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["pairs"],
            "properties": {
                "pairs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["source_var", "target_var"],
                        "properties": {
                            "source_var": { "type": "string" },
                            "target_var": { "type": "string" }
                        }
                    }
                },
                "order": {
                    "type": "string",
                    "enum": ["lowest", "highest"],
                    "default": "lowest"
                }
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

        let order = match params.get("order").and_then(|v| v.as_str()) {
            Some("highest") => AutoRecodeOrder::HighestValue,
            _ => AutoRecodeOrder::LowestValue,
        };

        let mut pairs_to_process: Vec<(String, String)> = Vec::new();

        if let Some(arr) = params.get("pairs").and_then(|v| v.as_array()) {
            for item in arr {
                let src = item
                    .get("source_var")
                    .or_else(|| item.get("source"))
                    .and_then(|v| v.as_str());
                let tgt = item
                    .get("target_var")
                    .or_else(|| item.get("target"))
                    .and_then(|v| v.as_str());

                if let (Some(s), Some(t)) = (src, tgt) {
                    let s_trimmed = s.trim().to_string();
                    let t_trimmed = t.trim().to_string();
                    if !s_trimmed.is_empty() && !t_trimmed.is_empty() {
                        pairs_to_process.push((s_trimmed, t_trimmed));
                    }
                }
            }
        }

        if pairs_to_process.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "No valid source -> target pairs provided".to_string(),
            });
        }

        let mut batch_undo = Vec::new();
        let mut recoded_targets = Vec::new();

        for (source_var, target_var) in pairs_to_process {
            let src_idx = ds.dictionary.find_index(&source_var).ok_or_else(|| {
                CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Source variable '{}' not found", source_var),
                }
            })?;

            let input_vals = ds
                .table
                .columns
                .get(src_idx)
                .map(|c| c.values.clone())
                .unwrap_or_default();

            let result = compute_auto_recode(&input_vals, order);

            let existing_idx = ds.dictionary.find_index(&target_var);
            let undo_item = if let Some(target_idx) = existing_idx {
                let old_meta = ds.dictionary.get(target_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Dictionary metadata missing".to_string())
                })?;
                let old_values = ds.table.columns[target_idx].values.clone();

                ds.table.columns[target_idx].values = result.recoded_values;
                if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                    meta.var_type = DataType::Numeric { decimals: 0 };
                    meta.measure = Measure::Nominal;
                    meta.values = result.value_labels;
                }

                AutoRecodeUndoItem::Overwritten {
                    col_idx: target_idx,
                    old_meta,
                    old_values,
                }
            } else {
                let target_idx = ds.add_variable(
                    &target_var,
                    DataType::Numeric { decimals: 0 },
                    Measure::Nominal,
                )?;
                if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                    meta.values = result.value_labels;
                }
                ds.table.columns[target_idx].values = result.recoded_values;

                AutoRecodeUndoItem::Created {
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
                    AutoRecodeUndoItem::Created { col_idx } => {
                        ds.remove_variable(col_idx)?;
                    }
                    AutoRecodeUndoItem::Overwritten {
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

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::RawTable;

    fn make_test_dataset() -> DataSet {
        let mut ds = DataSet::empty();
        ds.add_variable(
            "country",
            DataType::String { max_len: 20 },
            Measure::Nominal,
        )
        .unwrap();
        ds.table = RawTable::empty();
        ds.table.row_count = 5;
        ds.table.columns.push(statcraft_io::RawColumn {
            name: "country".to_string(),
            data_type: statcraft_io::RawDataType::Text,
            values: vec![
                RawValue::Text("USA".to_string()),
                RawValue::Text("Canada".to_string()),
                RawValue::Text("USA".to_string()),
                RawValue::Null,
                RawValue::Text("Mexico".to_string()),
            ],
        });
        ds
    }

    #[test]
    fn test_auto_recode_command_execution_and_undo() {
        let mut dataset = Some(make_test_dataset());
        let cmd = AutoRecodeCommand::new();

        let params = json!({
            "pairs": [
                {
                    "source_var": "country",
                    "target_var": "country_code"
                }
            ],
            "order": "lowest"
        });

        let res = cmd.execute(&mut dataset, params).unwrap();
        assert_eq!(res["status"], "success");

        let ds = dataset.as_ref().unwrap();
        assert_eq!(ds.col_count(), 2);
        let target_idx = ds.dictionary.find_index("country_code").unwrap();
        let target_meta = ds.dictionary.get(target_idx).unwrap();
        assert_eq!(target_meta.var_type, DataType::Numeric { decimals: 0 });
        assert_eq!(target_meta.measure, Measure::Nominal);

        // Canada=1, Mexico=2, USA=3
        assert_eq!(target_meta.values.get_label("1"), Some("Canada"));
        assert_eq!(target_meta.values.get_label("2"), Some("Mexico"));
        assert_eq!(target_meta.values.get_label("3"), Some("USA"));

        let vals = &ds.table.columns[target_idx].values;
        assert_eq!(
            vals,
            &vec![
                RawValue::Number(3.0),
                RawValue::Number(1.0),
                RawValue::Number(3.0),
                RawValue::Null,
                RawValue::Number(2.0),
            ]
        );

        // Test undo
        cmd.undo(&mut dataset).unwrap();
        let ds_undone = dataset.as_ref().unwrap();
        assert_eq!(ds_undone.col_count(), 1);
        assert!(ds_undone.dictionary.find_index("country_code").is_none());
    }

    #[test]
    fn test_auto_recode_to_sps() {
        let pairs = vec![("country", "country_code"), ("city", "city_code")];
        let sps_lowest = AutoRecodeCommand::to_sps(&pairs, AutoRecodeOrder::LowestValue);
        assert_eq!(
            sps_lowest,
            "AUTORECODE VARIABLES=country city /INTO country_code city_code\n  /PRINT.\nEXECUTE."
        );

        let sps_highest = AutoRecodeCommand::to_sps(&pairs, AutoRecodeOrder::HighestValue);
        assert_eq!(
            sps_highest,
            "AUTORECODE VARIABLES=country city /INTO country_code city_code\n  /DESCENDING\n  /PRINT.\nEXECUTE."
        );
    }
}
