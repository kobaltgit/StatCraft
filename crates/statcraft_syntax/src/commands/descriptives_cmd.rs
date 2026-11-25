use crate::bootstrap::{
    bootstrap_param_schema, bootstrap_to_sps, extract_strata_keys, parse_bootstrap_options,
};
use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_descriptives_bootstrap, compute_descriptives_weighted, BootstrapOptions,
    DescriptivesOptions, DescriptivesResult,
};
use statcraft_core::DataSet;

pub struct DescriptivesCommand;

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
                        "se_mean": { "type": "boolean" },
                        "skewness": { "type": "boolean" },
                        "kurtosis": { "type": "boolean" }
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

        if var_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'vars' must contain at least one variable name".to_string(),
            });
        }

        let desc_options: Option<DescriptivesOptions> = params
            .get("options")
            .and_then(|v| serde_json::from_value::<DescriptivesOptions>(v.clone()).ok());
        let boot_options = parse_bootstrap_options(&params, self.id())?;

        let mut results: Vec<DescriptivesResult> = Vec::new();
        let active_rows = ds.active_row_indices();
        let strata_keys = extract_strata_keys(ds, &active_rows, boot_options.as_ref(), self.id())?;

        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: "stat.descriptives",
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;

            // Extract numeric values and weights for this column
            let mut values: Vec<Option<f64>> = Vec::with_capacity(active_rows.len());
            let mut weights: Vec<f64> = Vec::with_capacity(active_rows.len());
            for &r in &active_rows {
                let val = ds.get_cell_numeric(r, col_idx);
                values.push(val);
                weights.push(ds.row_weight(r));
            }

            let mut res = compute_descriptives_weighted(var_name, &values, &weights);
            if let Some(ref boot) = boot_options {
                let boot_res = compute_descriptives_bootstrap(
                    var_name,
                    &values,
                    &weights,
                    strata_keys.as_deref(),
                    desc_options.as_ref(),
                    boot,
                )
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
                res.bootstrap = Some(boot_res);
            }
            results.push(res);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        // Read-only statistical procedure: nothing to mutate in the dataset
        Ok(())
    }
}

impl DescriptivesCommand {
    pub fn to_sps(vars: &[String]) -> String {
        format!(
            "DESCRIPTIVES VARIABLES={}\n  /STATISTICS=MEAN STDDEV MIN MAX.",
            vars.join(" ")
        )
    }

    pub fn to_sps_with_bootstrap(vars: &[String], bootstrap: Option<&BootstrapOptions>) -> String {
        let base = Self::to_sps(vars);
        if let Some(boot) = bootstrap {
            let sps = bootstrap_to_sps(boot);
            if !sps.is_empty() {
                return format!("{sps}\n{base}");
            }
        }
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv = "age,income\n20,25000\n30,35000\n40,45000\n50,55000\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_descriptives_command_execution() {
        let cmd = DescriptivesCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "vars": ["age", "income"]
        });

        assert!(cmd.enabled(ds.as_ref()));

        let res = cmd
            .execute(&mut ds, params)
            .expect("Execution should succeed");
        let results: Vec<DescriptivesResult> = serde_json::from_value(res).expect("Valid JSON");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].variable, "age");
        assert_eq!(results[0].n_valid, 4);
        assert_eq!(results[0].mean, Some(35.0));
        assert_eq!(results[0].min, Some(20.0));
        assert_eq!(results[0].max, Some(50.0));

        assert_eq!(results[1].variable, "income");
        assert_eq!(results[1].mean, Some(40000.0));
    }

    #[test]
    fn test_descriptives_command_missing_variable() {
        let cmd = DescriptivesCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "vars": ["non_existent"]
        });

        let err = cmd.execute(&mut ds, params).unwrap_err();
        match err {
            CommandError::InvalidParams { id, .. } => assert_eq!(id, "stat.descriptives"),
            _ => panic!("Expected InvalidParams error"),
        }
    }

    #[test]
    fn test_descriptives_sps_syntax() {
        let vars = vec!["age".to_string(), "income".to_string()];
        let sps = DescriptivesCommand::to_sps(&vars);
        assert_eq!(
            sps,
            "DESCRIPTIVES VARIABLES=age income\n  /STATISTICS=MEAN STDDEV MIN MAX."
        );
    }
}
