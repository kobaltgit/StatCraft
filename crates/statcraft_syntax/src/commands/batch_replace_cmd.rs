use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::DataSet;
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub struct ReplaceCellRecord {
    pub row: usize,
    pub col: usize,
    pub old_val: RawValue,
    pub new_val: RawValue,
}

pub struct BatchReplaceCommand {
    history: Mutex<Vec<Vec<ReplaceCellRecord>>>,
}

impl Default for BatchReplaceCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchReplaceCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

fn replace_case_insensitive(haystack: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return haystack.to_string();
    }
    let haystack_lower = haystack.to_ascii_lowercase();
    let needle_lower = needle.to_ascii_lowercase();
    let mut result = String::with_capacity(haystack.len());
    let mut last_idx = 0;

    while let Some(idx) = haystack_lower[last_idx..].find(&needle_lower) {
        let abs_idx = last_idx + idx;
        result.push_str(&haystack[last_idx..abs_idx]);
        result.push_str(replacement);
        last_idx = abs_idx + needle.len();
    }
    result.push_str(&haystack[last_idx..]);
    result
}

impl EngineCommand for BatchReplaceCommand {
    fn id(&self) -> &'static str {
        "data.replace_values"
    }

    fn menu_path(&self) -> &'static str {
        "Edit > Replace"
    }

    fn default_shortcut(&self) -> &'static str {
        "Ctrl+H"
    }

    fn category(&self) -> &'static str {
        "Transform"
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "find": { "type": "string" },
                "replace": { "type": "string" },
                "match_case": { "type": "boolean", "default": false },
                "match_entire_cell": { "type": "boolean", "default": false },
                "target_col": { "type": "integer" },
                "replacements": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "row": { "type": "integer", "minimum": 0 },
                            "col": { "type": "integer", "minimum": 0 },
                            "new_value": { "type": "string" }
                        },
                        "required": ["row", "col", "new_value"]
                    }
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset
            .map(|d| d.row_count() > 0 && d.col_count() > 0)
            .unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let mut applied_records = Vec::new();

        if let Some(explicit_replacements) = params.get("replacements").and_then(|v| v.as_array()) {
            for item in explicit_replacements {
                let row = item.get("row").and_then(|v| v.as_u64()).ok_or_else(|| {
                    CommandError::InvalidParams {
                        id: self.id(),
                        reason: "Missing 'row' in replacement item".to_string(),
                    }
                })? as usize;
                let col = item.get("col").and_then(|v| v.as_u64()).ok_or_else(|| {
                    CommandError::InvalidParams {
                        id: self.id(),
                        reason: "Missing 'col' in replacement item".to_string(),
                    }
                })? as usize;
                let new_val_str =
                    item.get("new_value")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| CommandError::InvalidParams {
                            id: self.id(),
                            reason: "Missing 'new_value' in replacement item".to_string(),
                        })?;

                let old_val = ds.get_cell(row, col).cloned().unwrap_or(RawValue::Null);
                ds.set_cell_parsed(row, col, new_val_str)?;
                let new_val = ds.get_cell(row, col).cloned().unwrap_or(RawValue::Null);

                applied_records.push(ReplaceCellRecord {
                    row,
                    col,
                    old_val,
                    new_val,
                });
            }
        } else {
            let find_text = params.get("find").and_then(|v| v.as_str()).ok_or_else(|| {
                CommandError::InvalidParams {
                    id: self.id(),
                    reason: "Missing 'find'".to_string(),
                }
            })?;
            let replace_text = params.get("replace").and_then(|v| v.as_str()).unwrap_or("");
            let match_case = params
                .get("match_case")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let match_entire = params
                .get("match_entire_cell")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let target_col = params
                .get("target_col")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);

            let (col_start, col_end) = match target_col {
                Some(c) => (c, (c + 1).min(ds.col_count())),
                None => (0, ds.col_count()),
            };

            for c in col_start..col_end {
                for r in 0..ds.row_count() {
                    let cell_str = ds.get_cell_display(r, c, false);
                    let is_match = if match_entire {
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
                        let new_cell_str = if match_entire {
                            replace_text.to_string()
                        } else if match_case {
                            cell_str.replace(find_text, replace_text)
                        } else {
                            replace_case_insensitive(&cell_str, find_text, replace_text)
                        };

                        let old_val = ds.get_cell(r, c).cloned().unwrap_or(RawValue::Null);
                        ds.set_cell_parsed(r, c, &new_cell_str)?;
                        let new_val = ds.get_cell(r, c).cloned().unwrap_or(RawValue::Null);

                        applied_records.push(ReplaceCellRecord {
                            row: r,
                            col: c,
                            old_val,
                            new_val,
                        });
                    }
                }
            }
        }

        let count = applied_records.len();
        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        hist.push(applied_records);

        Ok(json!({
            "status": "success",
            "replaced_count": count
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let mut hist = match self.history.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if let Some(records) = hist.pop() {
            if let Some(ds) = dataset.as_mut() {
                for rec in records.into_iter().rev() {
                    let _ = ds.set_cell(rec.row, rec.col, rec.old_val);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::{DataType, Measure};

    fn make_test_dataset() -> DataSet {
        let mut ds = DataSet::blank(3, 0).unwrap();
        ds.add_variable("fruit", DataType::String { max_len: 20 }, Measure::Nominal)
            .unwrap();
        ds.set_cell_parsed(0, 0, "apple").unwrap();
        ds.set_cell_parsed(1, 0, "Apple Pie").unwrap();
        ds.set_cell_parsed(2, 0, "banana").unwrap();
        ds
    }

    #[test]
    fn test_batch_replace_substring_case_insensitive() {
        let mut ds = Some(make_test_dataset());
        let cmd = BatchReplaceCommand::new();
        let params = json!({
            "find": "apple",
            "replace": "orange",
            "match_case": false,
            "match_entire_cell": false
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["replaced_count"], 2);

        let d = ds.as_ref().unwrap();
        assert_eq!(d.get_cell_display(0, 0, false), "orange");
        assert_eq!(d.get_cell_display(1, 0, false), "orange Pie");
        assert_eq!(d.get_cell_display(2, 0, false), "banana");

        // Test undo
        cmd.undo(&mut ds).unwrap();
        let d = ds.as_ref().unwrap();
        assert_eq!(d.get_cell_display(0, 0, false), "apple");
        assert_eq!(d.get_cell_display(1, 0, false), "Apple Pie");
    }

    #[test]
    fn test_batch_replace_entire_cell_case_sensitive() {
        let mut ds = Some(make_test_dataset());
        let cmd = BatchReplaceCommand::new();
        let params = json!({
            "find": "apple",
            "replace": "grape",
            "match_case": true,
            "match_entire_cell": true
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["replaced_count"], 1);

        let d = ds.as_ref().unwrap();
        assert_eq!(d.get_cell_display(0, 0, false), "grape");
        assert_eq!(d.get_cell_display(1, 0, false), "Apple Pie");
    }

    #[test]
    fn test_batch_replace_explicit_list() {
        let mut ds = Some(make_test_dataset());
        let cmd = BatchReplaceCommand::new();
        let params = json!({
            "replacements": [
                { "row": 0, "col": 0, "new_value": "pear" },
                { "row": 2, "col": 0, "new_value": "kiwi" }
            ]
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["replaced_count"], 2);

        let d = ds.as_ref().unwrap();
        assert_eq!(d.get_cell_display(0, 0, false), "pear");
        assert_eq!(d.get_cell_display(2, 0, false), "kiwi");

        cmd.undo(&mut ds).unwrap();
        let d = ds.as_ref().unwrap();
        assert_eq!(d.get_cell_display(0, 0, false), "apple");
        assert_eq!(d.get_cell_display(2, 0, false), "banana");
    }
}
