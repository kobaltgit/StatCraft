use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_count_across_columns, ValueCriterion};
use statcraft_core::{DataSet, DataType, Measure, VariableMetadata};
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum CountUndoEntry {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

pub struct CountValuesCommand {
    history: Mutex<Vec<CountUndoEntry>>,
}

impl Default for CountValuesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CountValuesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(
        target: &str,
        variables: &[String],
        criteria: &[ValueCriterion],
        label: Option<&str>,
    ) -> String {
        let vars_str = variables.join(" ");
        let crit_str = criteria
            .iter()
            .map(|c| format!("({})", c.to_sps()))
            .collect::<Vec<_>>()
            .join(" ");
        let mut sps = format!("COUNT {} = {} {}.\n", target, vars_str, crit_str);
        if let Some(lbl) = label {
            let trimmed = lbl.trim();
            if !trimmed.is_empty() {
                sps.push_str(&format!("VARIABLE LABELS {} \"{}\".\n", target, trimmed));
            }
        }
        sps.push_str("EXECUTE.");
        sps
    }
}

impl EngineCommand for CountValuesCommand {
    fn id(&self) -> &'static str {
        "transform.count_values"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Count Values within Cases"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["target_var", "variables", "criteria"],
            "properties": {
                "target_var": {
                    "type": "string",
                    "description": "Target variable name to hold counts"
                },
                "target_label": {
                    "type": "string",
                    "description": "Optional label for target variable"
                },
                "variables": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Input variable names to count occurrences across"
                },
                "criteria": {
                    "type": "array",
                    "items": { "type": "object" },
                    "description": "Criteria matching values, ranges, or system-missing"
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

        let target_var = params
            .get("target_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Target variable name cannot be empty".to_string(),
            })?;

        let target_label = params
            .get("target_label")
            .and_then(|v| v.as_str())
            .map(|s| s.trim());

        let variables: Vec<String> = params
            .get("variables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'variables' must be a non-empty array of variable names"
                    .to_string(),
            })?;

        if variables.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "At least one input variable must be selected".to_string(),
            });
        }

        let criteria_val = params
            .get("criteria")
            .and_then(|v| v.as_array())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'criteria' must be an array of ValueCriterion".to_string(),
            })?;

        if criteria_val.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "At least one value criterion must be defined".to_string(),
            });
        }

        let criteria: Vec<ValueCriterion> =
            serde_json::from_value(Value::Array(criteria_val.clone())).map_err(|e| {
                CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Failed to parse criteria array: {}", e),
                }
            })?;

        // Locate input column indices
        let mut col_indices = Vec::with_capacity(variables.len());
        for var in &variables {
            let idx = ds
                .dictionary
                .find_index(var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Variable '{}' not found in dataset", var),
                })?;
            col_indices.push(idx);
        }

        let row_count = ds.row_count();
        let col_slices: Vec<&[RawValue]> = col_indices
            .iter()
            .map(|&idx| ds.table.columns[idx].values.as_slice())
            .collect();

        let computed_values = compute_count_across_columns(row_count, &col_slices, &criteria);

        let existing_idx = ds.dictionary.find_index(target_var);

        let undo_entry = if let Some(col_idx) = existing_idx {
            let old_meta =
                ds.dictionary.get(col_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Corrupt dictionary".to_string())
                })?;
            let old_values = ds.table.columns[col_idx].values.clone();

            ds.table.columns[col_idx].values = computed_values;
            if let Some(lbl) = target_label {
                if !lbl.is_empty() {
                    if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                        meta.label = lbl.to_string();
                    }
                }
            }

            CountUndoEntry::Overwritten {
                col_idx,
                old_meta,
                old_values,
            }
        } else {
            let col_idx = ds.add_variable(
                target_var,
                DataType::Numeric { decimals: 0 },
                Measure::Scale,
            )?;
            if let Some(lbl) = target_label {
                if !lbl.is_empty() {
                    if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                        meta.label = lbl.to_string();
                    }
                }
            }
            ds.table.columns[col_idx].values = computed_values;

            CountUndoEntry::Created { col_idx }
        };

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_entry);
        }

        Ok(json!({
            "status": "success",
            "target": target_var,
            "rows": row_count,
            "variables_count": variables.len(),
            "criteria_count": criteria.len()
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let entry = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
                .ok_or_else(|| CommandError::ExecutionFailed("Nothing to undo".to_string()))?
        };

        match entry {
            CountUndoEntry::Created { col_idx } => {
                ds.remove_variable(col_idx)?;
            }
            CountUndoEntry::Overwritten {
                col_idx,
                old_meta,
                old_values,
            } => {
                if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                    *meta = old_meta;
                }
                if col_idx < ds.table.columns.len() {
                    ds.table.columns[col_idx].values = old_values;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_dataset() -> DataSet {
        let mut ds = DataSet::empty();
        let c0 = ds
            .add_variable("q1", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
            .unwrap();
        let c1 = ds
            .add_variable("q2", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
            .unwrap();
        let c2 = ds
            .add_variable("q3", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
            .unwrap();

        // 4 cases:
        // Case 0: 1, 2, 1
        // Case 1: 5, 5, 2
        // Case 2: 3, Null, 1
        // Case 3: 1, 1, 1
        for _ in 0..4 {
            ds.add_empty_row();
        }

        let _ = ds.set_cell(0, c0, RawValue::Number(1.0));
        let _ = ds.set_cell(0, c1, RawValue::Number(2.0));
        let _ = ds.set_cell(0, c2, RawValue::Number(1.0));

        let _ = ds.set_cell(1, c0, RawValue::Number(5.0));
        let _ = ds.set_cell(1, c1, RawValue::Number(5.0));
        let _ = ds.set_cell(1, c2, RawValue::Number(2.0));

        let _ = ds.set_cell(2, c0, RawValue::Number(3.0));
        let _ = ds.set_cell(2, c1, RawValue::Null);
        let _ = ds.set_cell(2, c2, RawValue::Number(1.0));

        let _ = ds.set_cell(3, c0, RawValue::Number(1.0));
        let _ = ds.set_cell(3, c1, RawValue::Number(1.0));
        let _ = ds.set_cell(3, c2, RawValue::Number(1.0));

        ds
    }

    #[test]
    fn test_count_values_execution_new_var_and_undo() {
        let mut dataset = Some(make_test_dataset());
        let cmd = CountValuesCommand::new();

        let params = json!({
            "target_var": "q_ones",
            "target_label": "Count of Ones",
            "variables": ["q1", "q2", "q3"],
            "criteria": [
                { "ExactNumber": 1.0 }
            ]
        });

        let res = cmd.execute(&mut dataset, params).unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(res["target"], "q_ones");

        let ds = dataset.as_ref().unwrap();
        assert_eq!(ds.col_count(), 4);
        let col_idx = ds.dictionary.find_index("q_ones").unwrap();
        assert_eq!(ds.dictionary.get(col_idx).unwrap().label, "Count of Ones");

        // Row 0: q1=1, q2=2, q3=1 -> count of 1s is 2
        // Row 1: q1=5, q2=5, q3=2 -> count is 0
        // Row 2: q1=3, q2=Null, q3=1 -> count is 1
        // Row 3: q1=1, q2=1, q3=1 -> count is 3
        assert_eq!(ds.get_cell(0, col_idx), Some(&RawValue::Number(2.0)));
        assert_eq!(ds.get_cell(1, col_idx), Some(&RawValue::Number(0.0)));
        assert_eq!(ds.get_cell(2, col_idx), Some(&RawValue::Number(1.0)));
        assert_eq!(ds.get_cell(3, col_idx), Some(&RawValue::Number(3.0)));

        // Undo
        cmd.undo(&mut dataset).unwrap();
        let ds = dataset.as_ref().unwrap();
        assert_eq!(ds.col_count(), 3);
        assert!(ds.dictionary.find_index("q_ones").is_none());
    }

    #[test]
    fn test_count_values_overwrite_existing_and_undo() {
        let mut dataset = Some(make_test_dataset());
        let cmd = CountValuesCommand::new();

        let params = json!({
            "target_var": "q3",
            "target_label": "Overwritten Q3",
            "variables": ["q1", "q2"],
            "criteria": [
                { "ExactNumber": 5.0 }
            ]
        });

        cmd.execute(&mut dataset, params).unwrap();
        let ds = dataset.as_ref().unwrap();
        assert_eq!(ds.col_count(), 3);
        let c3 = ds.dictionary.find_index("q3").unwrap();
        // Row 1 has q1=5, q2=5 -> count = 2
        assert_eq!(ds.get_cell(1, c3), Some(&RawValue::Number(2.0)));

        // Undo should restore previous values
        cmd.undo(&mut dataset).unwrap();
        let ds = dataset.as_ref().unwrap();
        assert_eq!(ds.get_cell(1, c3), Some(&RawValue::Number(2.0))); // original value of q3 at row 1 was 2.0
    }

    #[test]
    fn test_count_values_sps_syntax() {
        let sps = CountValuesCommand::to_sps(
            "high_scores",
            &["q1".to_string(), "q2".to_string()],
            &[
                ValueCriterion::ExactNumber(5.0),
                ValueCriterion::Range {
                    low: 8.0,
                    high: 10.0,
                },
            ],
            Some("High score count"),
        );
        assert!(sps.contains("COUNT high_scores = q1 q2 (5) (8 THRU 10)."));
        assert!(sps.contains("VARIABLE LABELS high_scores \"High score count\"."));
        assert!(sps.ends_with("EXECUTE."));
    }

    #[test]
    fn test_count_values_validation_errors() {
        let mut dataset = Some(make_test_dataset());
        let cmd = CountValuesCommand::new();

        // Empty target
        let res = cmd.execute(
            &mut dataset,
            json!({
                "target_var": "",
                "variables": ["q1"],
                "criteria": [{ "ExactNumber": 1.0 }]
            }),
        );
        assert!(res.is_err());

        // Empty variables
        let res = cmd.execute(
            &mut dataset,
            json!({
                "target_var": "target",
                "variables": [],
                "criteria": [{ "ExactNumber": 1.0 }]
            }),
        );
        assert!(res.is_err());

        // Unknown variable
        let res = cmd.execute(
            &mut dataset,
            json!({
                "target_var": "target",
                "variables": ["nonexistent"],
                "criteria": [{ "ExactNumber": 1.0 }]
            }),
        );
        assert!(res.is_err());

        // Empty criteria
        let res = cmd.execute(
            &mut dataset,
            json!({
                "target_var": "target",
                "variables": ["q1"],
                "criteria": []
            }),
        );
        assert!(res.is_err());
    }
}
