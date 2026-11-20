use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::glm::{compute_repeated_glm, RepeatedGlmOptions, SumOfSquaresType};
use statcraft_core::DataSet;

pub struct RepeatedGlmCommand;

impl RepeatedGlmCommand {
    pub fn to_sps(factor_name: &str, measures: &[String]) -> String {
        format!(
            "GLM {}\n  /WSFACTOR = {} {} Polynomial\n  /METHOD = SSTYPE(3)\n  /PRINT = TEST(MMSPH).",
            measures.join(" "),
            factor_name,
            measures.len()
        )
    }
}

impl EngineCommand for RepeatedGlmCommand {
    fn id(&self) -> &'static str {
        "stat.glm.repeated"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > General Linear Model > Repeated Measures"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["measures"],
            "properties": {
                "factor_name": {
                    "type": "string",
                    "default": "time",
                    "description": "Within-subject factor name"
                },
                "measures": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Repeated measurement columns (at least 2)"
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        match dataset {
            Some(ds) => ds.row_count() > 0 && ds.col_count() >= 2,
            None => false,
        }
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let measure_names: Vec<String> = params
            .get("measures")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if measure_names.len() < 2 {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'measures' must contain at least 2 variables".to_string(),
            });
        }

        let factor_name = params
            .get("factor_name")
            .and_then(|v| v.as_str())
            .unwrap_or("time")
            .to_string();

        let active_rows = ds.active_row_indices();
        let mut measure_vectors: Vec<Vec<f64>> = Vec::with_capacity(measure_names.len());
        for name in &measure_names {
            let col_idx = ds.dictionary.find_index(name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Measure variable '{name}' not found"))
            })?;
            let mut vals = Vec::with_capacity(active_rows.len());
            for &r in &active_rows {
                let val = ds.get_cell_numeric(r, col_idx).unwrap_or(f64::NAN);
                vals.push(val);
            }
            measure_vectors.push(vals);
        }

        let measure_slices: Vec<(&str, &[f64])> = measure_names
            .iter()
            .enumerate()
            .map(|(idx, name)| (name.as_str(), measure_vectors[idx].as_slice()))
            .collect();

        let options = RepeatedGlmOptions {
            factor_name: factor_name.clone(),
            measure_variables: measure_names.clone(),
            between_factors: Vec::new(),
            sum_of_squares: SumOfSquaresType::Type3,
            partial_eta_squared: true,
        };

        let result = compute_repeated_glm(&measure_slices, &options)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(result)
            .map_err(|e| CommandError::ExecutionFailed(format!("Serialization error: {e}")))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
