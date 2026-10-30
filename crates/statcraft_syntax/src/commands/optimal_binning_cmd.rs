use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::compute_optimal_binning;
use statcraft_core::{DataSet, DataType, Measure};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct OptimalBinningUndoEntry {
    pub created_cols: Vec<usize>,
}

pub struct OptimalBinningCommand {
    history: Mutex<Vec<OptimalBinningUndoEntry>>,
}

impl Default for OptimalBinningCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimalBinningCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(variables: &[&str], target_var: &str, max_bins: usize) -> String {
        let vars_str = variables.join(" ");
        if max_bins == 5 {
            format!(
                "OPTIMAL BINNING\n  /VARIABLES = {} BY {}\n  /CRITERIA METHOD = MDLP.\nEXECUTE.",
                vars_str, target_var
            )
        } else {
            format!(
                "OPTIMAL BINNING\n  /VARIABLES = {} BY {}\n  /CRITERIA METHOD = MDLP MAXBINS = {}.\nEXECUTE.",
                vars_str, target_var, max_bins
            )
        }
    }
}

impl EngineCommand for OptimalBinningCommand {
    fn id(&self) -> &'static str {
        "transform.optimal_binning"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Optimal Binning"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["variables", "target_var"],
            "properties": {
                "variables": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "target_var": { "type": "string" },
                "max_bins": { "type": "integer", "minimum": 2, "default": 5 },
                "prefix": { "type": "string", "default": "OB_" }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|d| d.col_count() >= 2).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let var_names: Vec<String> = params
            .get("variables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        if var_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Select at least one variable to bin".to_string(),
            });
        }

        let target_var = params
            .get("target_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Guide/target variable is required".to_string(),
            })?;

        let target_col_idx =
            ds.dictionary
                .find_index(&target_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Guide variable '{}' not found in dataset", target_var),
                })?;

        let target_values = ds
            .table
            .columns
            .get(target_col_idx)
            .map(|c| c.values.clone())
            .ok_or_else(|| {
                CommandError::ExecutionFailed(format!(
                    "Failed to read guide variable '{}' values",
                    target_var
                ))
            })?;

        let prefix = params
            .get("prefix")
            .and_then(|v| v.as_str())
            .unwrap_or("OB_")
            .trim()
            .to_string();

        let max_bins = params.get("max_bins").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

        for var in &var_names {
            if ds.dictionary.find_index(var).is_none() {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Variable '{}' not found in dataset", var),
                });
            }
        }

        let mut batch_created_cols: Vec<usize> = Vec::new();
        let mut created_names: Vec<String> = Vec::new();
        let mut cutpoints_summary = serde_json::Map::new();

        for var in &var_names {
            let col_idx = match ds.dictionary.find_index(var) {
                Some(idx) => idx,
                None => continue,
            };

            let feature_values = match ds.table.columns.get(col_idx) {
                Some(c) => c.values.clone(),
                None => continue,
            };

            let binning_result = compute_optimal_binning(&feature_values, &target_values, max_bins)
                .map_err(|e| {
                    CommandError::ExecutionFailed(format!("Binning error for '{}': {}", var, e))
                })?;

            cutpoints_summary.insert(var.clone(), json!(binning_result.cutpoints));

            let base_target_name = format!("{}{}", prefix, var);
            let mut target_name = base_target_name.clone();
            let mut counter = 1;
            while ds.dictionary.find_index(&target_name).is_some() {
                target_name = format!("{}_{}", base_target_name, counter);
                counter += 1;
            }

            let new_idx = ds.add_variable(
                &target_name,
                DataType::Numeric { decimals: 0 },
                Measure::Ordinal,
            )?;

            if let Some(m) = ds.dictionary.get_mut(new_idx) {
                m.label = format!("Optimal Binning ({})", var);
                m.values = binning_result.value_labels;
            }

            if let Some(col) = ds.table.columns.get_mut(new_idx) {
                col.values = binning_result.binned_values;
            }

            batch_created_cols.push(new_idx);
            created_names.push(target_name);
        }

        let count = created_names.len();

        if let Ok(mut hist) = self.history.lock() {
            hist.push(OptimalBinningUndoEntry {
                created_cols: batch_created_cols,
            });
        }

        Ok(json!({
            "status": "success",
            "created_variables": created_names,
            "count": count,
            "cutpoints": cutpoints_summary
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
        };

        if let Some(mut undo_entry) = entry {
            undo_entry.created_cols.sort_unstable_by(|a, b| b.cmp(a));
            for col_idx in undo_entry.created_cols {
                ds.remove_variable(col_idx)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::RawValue;

    fn create_test_dataset() -> DataSet {
        let mut ds = DataSet::empty();
        ds.add_variable("income", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
        ds.add_variable(
            "default",
            DataType::Numeric { decimals: 0 },
            Measure::Nominal,
        )
        .unwrap();

        // 8 rows
        let incomes = [10.0, 15.0, 20.0, 25.0, 100.0, 120.0, 130.0, 150.0];
        let defaults = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];

        for _ in 0..8 {
            ds.add_empty_row();
        }

        for (r, &inc) in incomes.iter().enumerate() {
            ds.set_cell(r, 0, RawValue::Number(inc)).unwrap();
            ds.set_cell(r, 1, RawValue::Number(defaults[r])).unwrap();
        }

        ds
    }

    #[test]
    fn test_optimal_binning_command_execute_and_undo() {
        let mut ds_opt = Some(create_test_dataset());
        let cmd = OptimalBinningCommand::new();

        let params = json!({
            "variables": ["income"],
            "target_var": "default",
            "max_bins": 5,
            "prefix": "OB_"
        });

        let res = cmd.execute(&mut ds_opt, params).unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(res["count"], 1);

        let ds = ds_opt.as_ref().unwrap();
        assert_eq!(ds.col_count(), 3);
        let ob_idx = ds.dictionary.find_index("OB_income").unwrap();
        let meta = ds.dictionary.get(ob_idx).unwrap();
        assert_eq!(meta.measure, Measure::Ordinal);
        assert!(!meta.values.is_empty());

        // Undo
        cmd.undo(&mut ds_opt).unwrap();
        let ds = ds_opt.as_ref().unwrap();
        assert_eq!(ds.col_count(), 2);
        assert!(ds.dictionary.find_index("OB_income").is_none());
    }

    #[test]
    fn test_to_sps() {
        let sps = OptimalBinningCommand::to_sps(&["income", "age"], "default_status", 5);
        assert!(sps.contains("OPTIMAL BINNING"));
        assert!(sps.contains("/VARIABLES = income age BY default_status"));
        assert!(sps.contains("/CRITERIA METHOD = MDLP."));
    }
}
