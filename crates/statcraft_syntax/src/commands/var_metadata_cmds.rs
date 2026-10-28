use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::{DataSet, MissingValues, ValueLabels};
use std::sync::Mutex;

pub struct VarSetValueLabelsCommand {
    history: Mutex<Vec<(usize, ValueLabels)>>,
}

impl Default for VarSetValueLabelsCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl VarSetValueLabelsCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

impl EngineCommand for VarSetValueLabelsCommand {
    fn id(&self) -> &'static str {
        "var.set_value_labels"
    }

    fn menu_path(&self) -> &'static str {
        "Variable > Set Value Labels"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["col_index", "labels"],
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 },
                "labels": {
                    "type": "object",
                    "additionalProperties": { "type": "string" },
                    "description": "Mapping from code to label string"
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

        let col_index = params
            .get("col_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'col_index'".to_string(),
            })? as usize;

        let labels_map = params
            .get("labels")
            .and_then(|v| v.as_object())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing or invalid 'labels' object".to_string(),
            })?;

        let var_meta = ds
            .dictionary
            .get(col_index)
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Variable index {} out of bounds", col_index),
            })?;

        let old_labels = var_meta.values.clone();

        let mut new_labels = ValueLabels::new();
        for (code, label_val) in labels_map {
            if let Some(lbl) = label_val.as_str() {
                new_labels.insert(code, lbl);
            }
        }

        ds.dictionary.set_value_labels(col_index, new_labels)?;

        let mut history = match self.history.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        history.push((col_index, old_labels));

        Ok(json!({
            "status": "success",
            "col_index": col_index
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let mut history = match self.history.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Some((col_index, prev_labels)) = history.pop() {
            if let Some(ds) = dataset.as_mut() {
                ds.dictionary.set_value_labels(col_index, prev_labels)?;
            }
        }
        Ok(())
    }
}

pub struct VarSetMissingValuesCommand {
    history: Mutex<Vec<(usize, MissingValues)>>,
}

impl Default for VarSetMissingValuesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl VarSetMissingValuesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

impl EngineCommand for VarSetMissingValuesCommand {
    fn id(&self) -> &'static str {
        "var.set_missing_values"
    }

    fn menu_path(&self) -> &'static str {
        "Variable > Set Missing Values"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["col_index"],
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 },
                "discrete": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Up to 3 discrete missing values"
                },
                "range": {
                    "type": "array",
                    "items": { "type": "number" },
                    "minItems": 2,
                    "maxItems": 2,
                    "description": "Range of missing values [low, high]"
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

        let col_index = params
            .get("col_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'col_index'".to_string(),
            })? as usize;

        let var_meta = ds
            .dictionary
            .get(col_index)
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Variable index {} out of bounds", col_index),
            })?;

        let old_missing = var_meta.missing.clone();

        let discrete: Vec<String> = params
            .get("discrete")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let range = params
            .get("range")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                if arr.len() == 2 {
                    let low = arr[0].as_f64()?;
                    let high = arr[1].as_f64()?;
                    Some((low, high))
                } else {
                    None
                }
            });

        let new_missing = MissingValues { discrete, range };
        ds.dictionary.set_missing_values(col_index, new_missing)?;

        let mut history = match self.history.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        history.push((col_index, old_missing));

        Ok(json!({
            "status": "success",
            "col_index": col_index
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let mut history = match self.history.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Some((col_index, prev_missing)) = history.pop() {
            if let Some(ds) = dataset.as_mut() {
                ds.dictionary.set_missing_values(col_index, prev_missing)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv = "gender,age\n1,25\n2,30\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_set_value_labels_command_and_undo() {
        let cmd = VarSetValueLabelsCommand::new();
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "col_index": 0,
            "labels": {
                "1": "Male",
                "2": "Female"
            }
        });

        assert!(cmd.enabled(ds.as_ref()));
        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");

        assert_eq!(
            ds.as_ref()
                .unwrap()
                .dictionary
                .get(0)
                .unwrap()
                .values
                .get_label("1"),
            Some("Male")
        );

        // Test undo
        cmd.undo(&mut ds).unwrap();
        assert!(ds
            .as_ref()
            .unwrap()
            .dictionary
            .get(0)
            .unwrap()
            .values
            .is_empty());
    }

    #[test]
    fn test_set_missing_values_command_and_undo() {
        let cmd = VarSetMissingValuesCommand::new();
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "col_index": 1,
            "discrete": ["99", "-1"],
            "range": [900.0, 999.0]
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");

        let missing = &ds.as_ref().unwrap().dictionary.get(1).unwrap().missing;
        assert_eq!(missing.discrete, vec!["99", "-1"]);
        assert_eq!(missing.range, Some((900.0, 999.0)));

        // Test undo
        cmd.undo(&mut ds).unwrap();
        let restored_missing = &ds.as_ref().unwrap().dictionary.get(1).unwrap().missing;
        assert!(restored_missing.discrete.is_empty());
        assert!(restored_missing.range.is_none());
    }
}
