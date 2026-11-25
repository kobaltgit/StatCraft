use crate::bootstrap::{
    bootstrap_param_schema, bootstrap_to_sps, extract_strata_keys, parse_bootstrap_options,
};
use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    apply_options_filter, compute_descriptives_bootstrap, compute_descriptives_weighted,
    sort_descriptives_results, BootstrapOptions, DescriptivesMissingMode, DescriptivesOptions,
    DescriptivesResult, DescriptivesSortOrder,
};
use statcraft_core::{DataSet, DataType, Measure};
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone)]
struct ZScoreUndoEntry {
    col_name: String,
}

pub struct DescriptivesCommand {
    history: Mutex<Vec<Vec<ZScoreUndoEntry>>>,
}

impl Default for DescriptivesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl DescriptivesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(vars: &[String]) -> String {
        Self::to_sps_full(vars, &DescriptivesOptions::spps_default(), None)
    }

    pub fn to_sps_with_bootstrap(vars: &[String], bootstrap: Option<&BootstrapOptions>) -> String {
        Self::to_sps_full(vars, &DescriptivesOptions::spps_default(), bootstrap)
    }

    pub fn to_sps_full(
        vars: &[String],
        options: &DescriptivesOptions,
        bootstrap: Option<&BootstrapOptions>,
    ) -> String {
        let mut sps = format!("DESCRIPTIVES VARIABLES={}", vars.join(" "));

        let mut stat_tokens: Vec<&str> = Vec::new();
        if options.mean {
            stat_tokens.push("MEAN");
        }
        if options.std_dev {
            stat_tokens.push("STDDEV");
        }
        if options.variance {
            stat_tokens.push("VARIANCE");
        }
        if options.min {
            stat_tokens.push("MIN");
        }
        if options.max {
            stat_tokens.push("MAX");
        }
        if options.range {
            stat_tokens.push("RANGE");
        }
        if options.sum {
            stat_tokens.push("SUM");
        }
        if options.se_mean {
            stat_tokens.push("SEMEAN");
        }
        if options.skewness {
            stat_tokens.push("SKEWNESS");
        }
        if options.kurtosis {
            stat_tokens.push("KURTOSIS");
        }

        if !stat_tokens.is_empty() {
            sps.push_str(&format!("\n  /STATISTICS={}", stat_tokens.join(" ")));
        }
        if options.save_standardized {
            sps.push_str("\n  /SAVE");
        }
        if options.missing == DescriptivesMissingMode::Listwise {
            sps.push_str("\n  /MISSING=LISTWISE");
        }
        match options.sort {
            DescriptivesSortOrder::VariableList => {}
            DescriptivesSortOrder::Alphabetic => sps.push_str("\n  /SORT=NAME (A)"),
            DescriptivesSortOrder::AlphabeticDesc => sps.push_str("\n  /SORT=NAME (D)"),
            DescriptivesSortOrder::AscendingMeans => sps.push_str("\n  /SORT=MEAN (A)"),
            DescriptivesSortOrder::DescendingMeans => sps.push_str("\n  /SORT=MEAN (D)"),
        }
        sps.push('.');

        if let Some(boot) = bootstrap {
            let boot_sps = bootstrap_to_sps(boot);
            if !boot_sps.is_empty() {
                return format!("{boot_sps}\n{sps}");
            }
        }
        sps
    }
}

fn unique_z_var_name(ds: &DataSet, base: &str) -> String {
    let candidate = format!("Z{base}");
    if ds.dictionary.find_index(&candidate).is_none() {
        return candidate;
    }
    for i in 1..=9999 {
        let fallback = format!("ZSC{i:03}");
        if ds.dictionary.find_index(&fallback).is_none() {
            return fallback;
        }
    }
    format!("ZSC_{}", ds.col_count() + 1)
}

impl EngineCommand for DescriptivesCommand {
    fn id(&self) -> &'static str {
        "stat.descriptives"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Descriptive Statistics > Descriptives"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["vars"],
            "properties": {
                "vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of variable names to analyze"
                },
                "options": {
                    "type": "object",
                    "properties": {
                        "mean": { "type": "boolean" },
                        "std_dev": { "type": "boolean" },
                        "variance": { "type": "boolean" },
                        "min": { "type": "boolean" },
                        "max": { "type": "boolean" },
                        "range": { "type": "boolean" },
                        "sum": { "type": "boolean" },
                        "se_mean": { "type": "boolean" },
                        "skewness": { "type": "boolean" },
                        "kurtosis": { "type": "boolean" },
                        "save_standardized": { "type": "boolean" },
                        "sort": {
                            "type": "string",
                            "enum": ["VariableList", "Alphabetic", "AlphabeticDesc", "AscendingMeans", "DescendingMeans"]
                        },
                        "missing": {
                            "type": "string",
                            "enum": ["Variable", "Listwise"]
                        }
                    }
                },
                "bootstrap": bootstrap_param_schema()
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        match dataset {
            Some(ds) => ds.row_count() > 0 && ds.col_count() > 0,
            None => false,
        }
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
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

        if var_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'vars' must contain at least one variable name".to_string(),
            });
        }

        let effective_options: DescriptivesOptions = match params.get("options") {
            Some(v) if !v.is_null() => serde_json::from_value::<DescriptivesOptions>(v.clone())
                .map_err(|e| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Invalid 'options': {e}"),
                })?,
            _ => DescriptivesOptions::spps_default(),
        };
        let boot_options = parse_bootstrap_options(&params, self.id())?;

        let mut col_indices: Vec<(String, usize)> = Vec::with_capacity(var_names.len());
        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: "stat.descriptives",
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;
            col_indices.push((var_name.clone(), col_idx));
        }

        let active_rows = ds.active_row_indices();
        let listwise_rows: Vec<usize> = active_rows
            .iter()
            .copied()
            .filter(|&r| {
                col_indices.iter().all(|&(_, col_idx)| {
                    ds.get_cell_numeric(r, col_idx).is_some_and(f64::is_finite)
                })
            })
            .collect();

        let listwise_w_sum: f64 = listwise_rows
            .iter()
            .map(|&r| ds.row_weight(r))
            .filter(|&w| w > 0.0 && w.is_finite())
            .sum();
        let listwise_n = listwise_w_sum.round() as usize;

        let eval_rows: &[usize] = match effective_options.missing {
            DescriptivesMissingMode::Listwise => &listwise_rows,
            DescriptivesMissingMode::Variable => &active_rows,
        };
        let strata_keys = extract_strata_keys(ds, eval_rows, boot_options.as_ref(), self.id())?;

        let mut results: Vec<DescriptivesResult> = Vec::with_capacity(col_indices.len());
        let mut z_params: Vec<(String, usize, Option<f64>, Option<f64>)> = Vec::new();

        for (var_name, col_idx) in &col_indices {
            let mut values: Vec<Option<f64>> = Vec::with_capacity(eval_rows.len());
            let mut weights: Vec<f64> = Vec::with_capacity(eval_rows.len());
            for &r in eval_rows {
                values.push(ds.get_cell_numeric(r, *col_idx));
                weights.push(ds.row_weight(r));
            }

            let mut res = compute_descriptives_weighted(var_name, &values, &weights);
            res.listwise_n = Some(listwise_n);

            if effective_options.save_standardized {
                z_params.push((var_name.clone(), *col_idx, res.mean, res.std_dev));
            }

            if let Some(ref boot) = boot_options {
                let boot_res = compute_descriptives_bootstrap(
                    var_name,
                    &values,
                    &weights,
                    strata_keys.as_deref(),
                    Some(&effective_options),
                    boot,
                )
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
                res.bootstrap = Some(boot_res);
            }
            results.push(res);
        }

        // Sort before filtering so AscendingMeans/DescendingMeans still has mean available.
        sort_descriptives_results(&mut results, effective_options.sort);
        for res in &mut results {
            apply_options_filter(res, &effective_options);
        }

        let mut undo_entries: Vec<ZScoreUndoEntry> = Vec::new();
        if effective_options.save_standardized {
            let row_count = ds.row_count();
            for (var_name, src_col_idx, mean_opt, sd_opt) in z_params {
                let z_name = unique_z_var_name(ds, &var_name);
                let z_col_idx =
                    ds.add_variable(&z_name, DataType::Numeric { decimals: 2 }, Measure::Scale)?;

                let z_values: Vec<RawValue> = match (mean_opt, sd_opt) {
                    (Some(mean), Some(sd)) if mean.is_finite() && sd.is_finite() && sd != 0.0 => (0
                        ..row_count)
                        .map(|r| match ds.get_cell_numeric(r, src_col_idx) {
                            Some(x) if x.is_finite() => RawValue::Number((x - mean) / sd),
                            _ => RawValue::Null,
                        })
                        .collect(),
                    _ => vec![RawValue::Null; row_count],
                };
                if let Some(col) = ds.table.columns.get_mut(z_col_idx) {
                    col.values = z_values;
                }
                undo_entries.push(ZScoreUndoEntry { col_name: z_name });
            }
        }

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_entries);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let entry = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        if let Some(entries) = entry {
            if !entries.is_empty() {
                let ds = dataset
                    .as_mut()
                    .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;
                for z_entry in entries.into_iter().rev() {
                    if let Some(col_idx) = ds.dictionary.find_index(&z_entry.col_name) {
                        ds.remove_variable(col_idx)?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "descriptives_cmd_tests.rs"]
mod tests;
