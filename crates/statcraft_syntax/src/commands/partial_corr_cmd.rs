use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_partial_correlations, MissingHandling, PartialCorrOptions, SignificanceTest,
};
use statcraft_core::DataSet;

pub struct PartialCorrCommand;

impl PartialCorrCommand {
    pub fn to_sps(vars: &[String], controls: &[String], two_tailed: bool, missing: &str) -> String {
        let sig_str = if two_tailed { "TWOTAIL" } else { "ONETAIL" };
        let miss_str = if missing.eq_ignore_ascii_case("pairwise") {
            "PAIRWISE"
        } else {
            "LISTWISE"
        };

        format!(
            "PARTIAL CORR\n  /VARIABLES = {} WITH {}\n  /SIGNIFICANCE = {}\n  /MISSING = {}.",
            vars.join(" "),
            controls.join(" "),
            sig_str,
            miss_str
        )
    }
}

impl EngineCommand for PartialCorrCommand {
    fn id(&self) -> &'static str {
        "stat.correlations.partial"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Correlate > Partial"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["vars", "controls"],
            "properties": {
                "vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of analyzed variable names (at least 2)"
                },
                "controls": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of controlling variable names (at least 1)"
                },
                "two_tailed": {
                    "type": "boolean",
                    "default": true,
                    "description": "Two-tailed test of significance"
                },
                "missing": {
                    "type": "string",
                    "enum": ["listwise", "pairwise"],
                    "default": "listwise",
                    "description": "Missing data handling mode"
                },
                "descriptives": {
                    "type": "boolean",
                    "default": false,
                    "description": "Include descriptive statistics"
                },
                "zero_order": {
                    "type": "boolean",
                    "default": false,
                    "description": "Include zero-order Pearson correlations"
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        match dataset {
            Some(ds) => ds.row_count() > 0 && ds.col_count() >= 3,
            None => false,
        }
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let var_names: Vec<String> = params
            .get("vars")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let control_names: Vec<String> = params
            .get("controls")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if var_names.len() < 2 {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'vars' must contain at least 2 variables".to_string(),
            });
        }
        if control_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'controls' must contain at least 1 variable".to_string(),
            });
        }

        let mut var_col_indices = Vec::with_capacity(var_names.len());
        for name in &var_names {
            let col_idx = ds.dictionary.find_index(name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Variable '{name}' not found in dataset"))
            })?;
            var_col_indices.push(col_idx);
        }

        let mut control_col_indices = Vec::with_capacity(control_names.len());
        for name in &control_names {
            let col_idx = ds.dictionary.find_index(name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!(
                    "Control variable '{name}' not found in dataset"
                ))
            })?;
            control_col_indices.push(col_idx);
        }

        let two_tailed = params
            .get("two_tailed")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let missing_str = params
            .get("missing")
            .and_then(|v| v.as_str())
            .unwrap_or("listwise");
        let descriptives = params
            .get("descriptives")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let zero_order = params
            .get("zero_order")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let active_rows = ds.active_row_indices();
        let n_active = active_rows.len();

        let mut var_data: Vec<Vec<Option<f64>>> =
            vec![Vec::with_capacity(n_active); var_names.len()];
        for &r in &active_rows {
            for (v_idx, &col_idx) in var_col_indices.iter().enumerate() {
                let val = ds.get_cell_numeric(r, col_idx);
                var_data[v_idx].push(val);
            }
        }

        let mut control_data: Vec<Vec<Option<f64>>> =
            vec![Vec::with_capacity(n_active); control_names.len()];
        for &r in &active_rows {
            for (c_idx, &col_idx) in control_col_indices.iter().enumerate() {
                let val = ds.get_cell_numeric(r, col_idx);
                control_data[c_idx].push(val);
            }
        }

        let var_inputs: Vec<(&str, &[Option<f64>])> = var_names
            .iter()
            .zip(var_data.iter())
            .map(|(name, vals)| (name.as_str(), vals.as_slice()))
            .collect();

        let control_inputs: Vec<(&str, &[Option<f64>])> = control_names
            .iter()
            .zip(control_data.iter())
            .map(|(name, vals)| (name.as_str(), vals.as_slice()))
            .collect();

        let options = PartialCorrOptions {
            significance: if two_tailed {
                SignificanceTest::TwoTailed
            } else {
                SignificanceTest::OneTailed
            },
            missing: if missing_str.eq_ignore_ascii_case("pairwise") {
                MissingHandling::Pairwise
            } else {
                MissingHandling::Listwise
            },
            descriptives,
            zero_order_correlations: zero_order,
        };

        let result = compute_partial_correlations(&var_inputs, &control_inputs, &options)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(result)
            .map_err(|e| CommandError::ExecutionFailed(format!("Serialization error: {e}")))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
