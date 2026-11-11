use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::{variable::Alignment, DataSet, Measure};
use std::sync::{Mutex, MutexGuard};

fn lock_hist<T>(m: &Mutex<Vec<T>>) -> MutexGuard<'_, Vec<T>> {
    match m.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}

pub struct VarSetAlignCommand {
    history: Mutex<Vec<(usize, Alignment)>>,
}

impl Default for VarSetAlignCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl VarSetAlignCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

impl EngineCommand for VarSetAlignCommand {
    fn id(&self) -> &'static str {
        "var.set_align"
    }

    fn menu_path(&self) -> &'static str {
        "Variable > Set Alignment"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 },
                "align": { "type": "string", "enum": ["left", "right", "center"] }
            },
            "required": ["col_index", "align"]
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

        let align_str = params
            .get("align")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'align'".to_string(),
            })?;

        let align = match align_str.to_lowercase().as_str() {
            "left" => Alignment::Left,
            "right" => Alignment::Right,
            "center" => Alignment::Center,
            _ => {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Unknown align: '{}'", align_str),
                })
            }
        };

        let old_align = ds
            .dictionary
            .get(col_index)
            .map(|m| m.align)
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Column index {} out of bounds", col_index),
            })?;

        ds.set_variable_align(col_index, align)?;
        lock_hist(&self.history).push((col_index, old_align));

        Ok(json!({ "status": "success", "col_index": col_index, "align": align_str }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        if let Some((col_index, old_align)) = lock_hist(&self.history).pop() {
            if let Some(ds) = dataset.as_mut() {
                ds.set_variable_align(col_index, old_align)?;
            }
        }
        Ok(())
    }
}

pub struct SetLabelCommand {
    history: Mutex<Vec<Vec<(usize, String)>>>,
}

impl Default for SetLabelCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SetLabelCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

impl EngineCommand for SetLabelCommand {
    fn id(&self) -> &'static str {
        "var.set_label"
    }

    fn menu_path(&self) -> &'static str {
        "Variable > Set Label"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 },
                "label": { "type": "string" }
            },
            "required": ["col_index", "label"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|d| d.col_count() > 0).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let mut batch = Vec::new();

        if let Some(labels_map) = params.get("labels").and_then(|v| v.as_object()) {
            for (var_name, lbl_val) in labels_map {
                if let Some(lbl) = lbl_val.as_str() {
                    let target_idx = if let Some(idx) = ds.dictionary.find_index(var_name) {
                        Some(idx)
                    } else if let Ok(idx) = var_name.parse::<usize>() {
                        if idx < ds.col_count() {
                            Some(idx)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(idx) = target_idx {
                        if let Some(meta) = ds.dictionary.get(idx) {
                            batch.push((idx, meta.label.clone()));
                        }
                        ds.dictionary.set_label(idx, lbl)?;
                    }
                }
            }
            lock_hist(&self.history).push(batch);
            return Ok(json!({ "status": "success", "updated": labels_map.len() }));
        }

        let col_index = if let Some(v) = params.get("col_index").and_then(|v| v.as_u64()) {
            v as usize
        } else if let Some(name) = params.get("var_name").and_then(|v| v.as_str()) {
            ds.dictionary
                .find_index(name)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Variable '{}' not found", name),
                })?
        } else {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'col_index' or 'var_name'".to_string(),
            });
        };

        let label = params
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'label'".to_string(),
            })?;

        let old_label = ds
            .dictionary
            .get(col_index)
            .map(|m| m.label.clone())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Column index {} out of bounds", col_index),
            })?;

        batch.push((col_index, old_label));
        ds.dictionary.set_label(col_index, label)?;
        lock_hist(&self.history).push(batch);

        Ok(json!({ "status": "success", "col_index": col_index, "label": label }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        if let Some(batch) = lock_hist(&self.history).pop() {
            if let Some(ds) = dataset.as_mut() {
                for (col_index, old_label) in batch {
                    ds.dictionary.set_label(col_index, &old_label)?;
                }
            }
        }
        Ok(())
    }
}

pub struct SetMeasureCommand {
    history: Mutex<Vec<Vec<(usize, Measure)>>>,
}

impl Default for SetMeasureCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SetMeasureCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }
}

impl EngineCommand for SetMeasureCommand {
    fn id(&self) -> &'static str {
        "var.set_measure"
    }

    fn menu_path(&self) -> &'static str {
        "Variable > Set Measure"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "col_index": { "type": "integer", "minimum": 0 },
                "measure": { "type": "string", "enum": ["scale", "ordinal", "nominal"] }
            },
            "required": ["col_index", "measure"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|d| d.col_count() > 0).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let mut batch = Vec::new();

        if let Some(assignments) = params.get("assignments").and_then(|v| v.as_array()) {
            for item in assignments {
                let m_str = item
                    .get("measure")
                    .and_then(|v| v.as_str())
                    .unwrap_or("scale");
                let m = match m_str.to_lowercase().as_str() {
                    "scale" => Measure::Scale,
                    "ordinal" => Measure::Ordinal,
                    "nominal" => Measure::Nominal,
                    _ => Measure::Scale,
                };
                if let Some(vars) = item.get("variables").and_then(|v| v.as_array()) {
                    for v in vars {
                        if let Some(name) = v.as_str() {
                            if let Some(idx) = ds.dictionary.find_index(name) {
                                if let Some(meta) = ds.dictionary.get(idx) {
                                    batch.push((idx, meta.measure));
                                }
                                ds.dictionary.set_measure(idx, m)?;
                            }
                        }
                    }
                }
            }
            lock_hist(&self.history).push(batch);
            return Ok(json!({ "status": "success" }));
        }

        let measure_str = params
            .get("measure")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'measure'".to_string(),
            })?;

        let measure = match measure_str.to_lowercase().as_str() {
            "scale" => Measure::Scale,
            "ordinal" => Measure::Ordinal,
            "nominal" => Measure::Nominal,
            _ => {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Unknown measure: '{}'", measure_str),
                })
            }
        };

        if let Some(vars) = params.get("variables").and_then(|v| v.as_array()) {
            for v in vars {
                if let Some(name) = v.as_str() {
                    let idx = ds.dictionary.find_index(name).ok_or_else(|| {
                        CommandError::InvalidParams {
                            id: self.id(),
                            reason: format!("Variable '{}' not found", name),
                        }
                    })?;
                    if let Some(meta) = ds.dictionary.get(idx) {
                        batch.push((idx, meta.measure));
                    }
                    ds.dictionary.set_measure(idx, measure)?;
                }
            }
            lock_hist(&self.history).push(batch);
            return Ok(json!({ "status": "success", "measure": measure_str }));
        }

        let col_index = if let Some(v) = params.get("col_index").and_then(|v| v.as_u64()) {
            v as usize
        } else if let Some(name) = params.get("var_name").and_then(|v| v.as_str()) {
            ds.dictionary
                .find_index(name)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Variable '{}' not found", name),
                })?
        } else {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'col_index', 'var_name', or 'variables'".to_string(),
            });
        };

        let old_measure = ds
            .dictionary
            .get(col_index)
            .map(|m| m.measure)
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: format!("Column index {} out of bounds", col_index),
            })?;

        batch.push((col_index, old_measure));
        ds.dictionary.set_measure(col_index, measure)?;
        lock_hist(&self.history).push(batch);

        Ok(json!({ "status": "success", "col_index": col_index, "measure": measure_str }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        if let Some(batch) = lock_hist(&self.history).pop() {
            if let Some(ds) = dataset.as_mut() {
                for (col_index, old_measure) in batch {
                    ds.dictionary.set_measure(col_index, old_measure)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::variable::{DataType, VariableMetadata};

    fn make_test_ds() -> Option<DataSet> {
        let mut ds = DataSet::empty();
        let var = VariableMetadata::new("age", DataType::Numeric { decimals: 0 }, Measure::Scale)
            .unwrap();
        ds.dictionary.add(var).unwrap();
        Some(ds)
    }

    #[test]
    fn test_set_label_command() {
        let mut ds = make_test_ds();
        let initial_ds = ds.clone().unwrap();
        let cmd = SetLabelCommand::default();

        let params = json!({
            "col_index": 0,
            "label": "Age in years"
        });

        cmd.execute(&mut ds, params).unwrap();
        assert_eq!(
            ds.as_ref().unwrap().dictionary.get(0).unwrap().label,
            "Age in years"
        );

        cmd.undo(&mut ds).unwrap();
        assert_eq!(ds.as_ref().unwrap(), &initial_ds);
    }

    #[test]
    fn test_set_measure_command() {
        let mut ds = make_test_ds();
        let initial_ds = ds.clone().unwrap();
        let cmd = SetMeasureCommand::default();

        let params = json!({
            "col_index": 0,
            "measure": "ordinal"
        });

        cmd.execute(&mut ds, params).unwrap();
        assert_eq!(
            ds.as_ref().unwrap().dictionary.get(0).unwrap().measure,
            Measure::Ordinal
        );

        cmd.undo(&mut ds).unwrap();
        assert_eq!(ds.as_ref().unwrap(), &initial_ds);
    }
}
