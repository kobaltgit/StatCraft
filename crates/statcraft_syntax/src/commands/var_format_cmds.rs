use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::{variable::Alignment, DataSet, Measure};

pub struct VarSetAlignCommand;

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
                });
            }
        };

        ds.set_variable_align(col_index, align)?;

        Ok(json!({
            "status": "success",
            "col_index": col_index,
            "align": align_str
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

pub struct SetLabelCommand;

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

        let col_index = params
            .get("col_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'col_index'".to_string(),
            })? as usize;

        let label = params
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'label'".to_string(),
            })?;

        ds.dictionary.set_label(col_index, label)?;

        Ok(json!({
            "status": "success",
            "col_index": col_index,
            "label": label
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

pub struct SetMeasureCommand;

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

        let col_index = params
            .get("col_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'col_index'".to_string(),
            })? as usize;

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
                });
            }
        };

        ds.dictionary.set_measure(col_index, measure)?;

        Ok(json!({
            "status": "success",
            "col_index": col_index,
            "measure": measure_str
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
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
        let cmd = SetLabelCommand;

        let params = json!({
            "col_index": 0,
            "label": "Age in years"
        });

        cmd.execute(&mut ds, params).unwrap();
        assert_eq!(
            ds.as_ref().unwrap().dictionary.get(0).unwrap().label,
            "Age in years"
        );
    }

    #[test]
    fn test_set_measure_command() {
        let mut ds = make_test_ds();
        let cmd = SetMeasureCommand;

        let params = json!({
            "col_index": 0,
            "measure": "ordinal"
        });

        cmd.execute(&mut ds, params).unwrap();
        assert_eq!(
            ds.as_ref().unwrap().dictionary.get(0).unwrap().measure,
            Measure::Ordinal
        );
    }
}
