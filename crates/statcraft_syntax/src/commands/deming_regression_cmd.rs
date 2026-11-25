use crate::bootstrap::{
    bootstrap_param_schema, bootstrap_to_sps, extract_strata_keys, parse_bootstrap_options,
};
use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_deming_regression, compute_deming_regression_bootstrap, BootstrapOptions,
    DemingRegressionOptions, DemingRegressionResult,
};
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use std::sync::Mutex;

#[derive(Debug, Clone)]
struct DemingUndoEntry {
    col_name: String,
}

pub struct DemingRegressionCommand {
    history: Mutex<Vec<Vec<DemingUndoEntry>>>,
}

impl Default for DemingRegressionCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl DemingRegressionCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_sps(
        dep_var: &str,
        indep_var: &str,
        lambda: f64,
        ci_level: f64,
        include_scatter_plot: bool,
        include_identity_line: bool,
        include_residual_plot: bool,
        save_predicted: bool,
        save_residuals: bool,
    ) -> String {
        let mut lines = vec![
            "ORTHOGONAL REGRESSION".to_string(),
            format!("  /DEPENDENT {dep_var}"),
            format!("  /INDEPENDENT {indep_var}"),
            format!("  /LAMBDA {lambda:.4}"),
            format!("  /CI {ci_level:.2}"),
        ];

        let mut plots = Vec::new();
        if include_scatter_plot {
            plots.push("SCATTER");
        }
        if include_identity_line {
            plots.push("IDENTITY");
        }
        if include_residual_plot {
            plots.push("RESIDUAL");
        }
        if plots.is_empty() {
            lines.push("  /PLOT NONE".to_string());
        } else {
            lines.push(format!("  /PLOT {}", plots.join(" ")));
        }

        let mut saves = Vec::new();
        if save_predicted {
            saves.push("PRE");
        }
        if save_residuals {
            saves.push("RES");
        }
        if !saves.is_empty() {
            lines.push(format!("  /SAVE {}", saves.join(" ")));
        }

        let last_idx = lines.len() - 1;
        lines[last_idx].push('.');
        lines.join("\n")
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_sps_with_bootstrap(
        dep_var: &str,
        indep_var: &str,
        lambda: f64,
        ci_level: f64,
        include_scatter_plot: bool,
        include_identity_line: bool,
        include_residual_plot: bool,
        save_predicted: bool,
        save_residuals: bool,
        bootstrap: Option<&BootstrapOptions>,
    ) -> String {
        let base = Self::to_sps(
            dep_var,
            indep_var,
            lambda,
            ci_level,
            include_scatter_plot,
            include_identity_line,
            include_residual_plot,
            save_predicted,
            save_residuals,
        );
        if let Some(boot) = bootstrap {
            let sps = bootstrap_to_sps(boot);
            if !sps.is_empty() {
                return format!("{sps}\n{base}");
            }
        }
        base
    }
}

fn unique_var_name(ds: &DataSet, prefix: &str) -> String {
    let mut idx = 1_usize;
    loop {
        let candidate = format!("{prefix}_{idx}");
        if ds.dictionary.find_index(&candidate).is_none() {
            return candidate;
        }
        idx += 1;
    }
}

impl EngineCommand for DemingRegressionCommand {
    fn id(&self) -> &'static str {
        "stat.regression.orthogonal"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Regression > Orthogonal / Deming"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependent_var", "independent_var"],
            "properties": {
                "dependent_var": { "type": "string" },
                "independent_var": { "type": "string" },
                "lambda": { "type": "number", "default": 1.0 },
                "ci": { "type": "number", "default": 0.95 },
                "include_scatter_plot": { "type": "boolean", "default": true },
                "include_identity_line": { "type": "boolean", "default": true },
                "include_residual_plot": { "type": "boolean", "default": false },
                "save_predicted": { "type": "boolean", "default": false },
                "save_residuals": { "type": "boolean", "default": false },
                "table_styles": { "type": "object" },
                "bootstrap": bootstrap_param_schema()
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
            .as_mut()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let dep_var = params
            .get("dependent_var")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'dependent_var' is required".to_string(),
            })?;

        let indep_var = params
            .get("independent_var")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'independent_var' is required".to_string(),
            })?;

        let dep_col_idx = ds.dictionary.find_index(dep_var).ok_or_else(|| {
            CommandError::ExecutionFailed(format!("Dependent variable '{dep_var}' not found"))
        })?;
        let indep_col_idx = ds.dictionary.find_index(indep_var).ok_or_else(|| {
            CommandError::ExecutionFailed(format!("Independent variable '{indep_var}' not found"))
        })?;

        let lambda = params.get("lambda").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let ci_level = params.get("ci").and_then(|v| v.as_f64()).unwrap_or(0.95);
        let save_predicted = params
            .get("save_predicted")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let save_residuals = params
            .get("save_residuals")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let options = DemingRegressionOptions { lambda, ci_level };
        let boot_options = parse_bootstrap_options(&params, self.id())?;

        let active_rows = ds.active_row_indices();
        let strata_keys = extract_strata_keys(ds, &active_rows, boot_options.as_ref(), self.id())?;

        let mut dep_vals = Vec::with_capacity(active_rows.len());
        let mut indep_vals = Vec::with_capacity(active_rows.len());
        for &r in &active_rows {
            dep_vals.push(ds.get_cell_numeric(r, dep_col_idx));
            indep_vals.push(ds.get_cell_numeric(r, indep_col_idx));
        }

        let mut result: DemingRegressionResult =
            compute_deming_regression(dep_var, &dep_vals, indep_var, &indep_vals, &options)
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        if let Some(ref boot) = boot_options {
            let boot_res = compute_deming_regression_bootstrap(
                dep_var,
                &dep_vals,
                indep_var,
                &indep_vals,
                &options,
                strata_keys.as_deref(),
                boot,
            )
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            result.bootstrap = Some(boot_res);
        }

        let mut undo_entries = Vec::new();
        let row_count = ds.row_count();

        if save_predicted {
            let pre_name = unique_var_name(ds, "PRE");
            let pre_col_idx =
                ds.add_variable(&pre_name, DataType::Numeric { decimals: 2 }, Measure::Scale)?;
            let pre_values: Vec<RawValue> = (0..row_count)
                .map(|r| match ds.get_cell_numeric(r, indep_col_idx) {
                    Some(x) if x.is_finite() => {
                        RawValue::Number(result.slope * x + result.intercept)
                    }
                    _ => RawValue::Null,
                })
                .collect();
            if let Some(col) = ds.table.columns.get_mut(pre_col_idx) {
                col.values = pre_values;
            }
            result.saved_predicted_var = Some(pre_name.clone());
            undo_entries.push(DemingUndoEntry { col_name: pre_name });
        }

        if save_residuals {
            let res_name = unique_var_name(ds, "RES");
            let res_col_idx =
                ds.add_variable(&res_name, DataType::Numeric { decimals: 2 }, Measure::Scale)?;
            let scale = (result.lambda / (result.lambda + result.slope * result.slope)).sqrt();
            let res_values: Vec<RawValue> = (0..row_count)
                .map(|r| {
                    match (
                        ds.get_cell_numeric(r, dep_col_idx),
                        ds.get_cell_numeric(r, indep_col_idx),
                    ) {
                        (Some(y), Some(x)) if y.is_finite() && x.is_finite() => {
                            RawValue::Number(scale * (y - (result.slope * x + result.intercept)))
                        }
                        _ => RawValue::Null,
                    }
                })
                .collect();
            if let Some(col) = ds.table.columns.get_mut(res_col_idx) {
                col.values = res_values;
            }
            result.saved_residuals_var = Some(res_name.clone());
            undo_entries.push(DemingUndoEntry { col_name: res_name });
        }

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_entries);
        }

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
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
                for item in entries.into_iter().rev() {
                    if let Some(col_idx) = ds.dictionary.find_index(&item.col_name) {
                        ds.remove_variable(col_idx)?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax_parser::lexer::SyntaxLexer;
    use crate::syntax_parser::mappers::map_stat_command;
    use crate::syntax_parser::SyntaxParser;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn create_test_dataset() -> DataSet {
        let csv_data = "y,x\n2.1,1.0\n3.9,2.0\n6.2,3.0\n7.8,4.0\n10.1,5.0\n11.9,6.0\n";
        let table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_deming_command_and_save_undo() {
        let cmd = DemingRegressionCommand::new();
        let mut ds = Some(create_test_dataset());

        let params = json!({
            "dependent_var": "y",
            "independent_var": "x",
            "lambda": 1.0,
            "ci": 0.95,
            "save_predicted": true,
            "save_residuals": true
        });

        let res_val = cmd.execute(&mut ds, params).expect("execute");
        let res: DemingRegressionResult = serde_json::from_value(res_val).unwrap();
        assert!((res.slope - 1.9796948320595082).abs() < 1e-6);
        assert_eq!(res.saved_predicted_var.as_deref(), Some("PRE_1"));
        assert_eq!(res.saved_residuals_var.as_deref(), Some("RES_1"));
        assert!(ds
            .as_ref()
            .unwrap()
            .dictionary
            .find_index("PRE_1")
            .is_some());
        assert!(ds
            .as_ref()
            .unwrap()
            .dictionary
            .find_index("RES_1")
            .is_some());

        cmd.undo(&mut ds).expect("undo");
        assert!(ds
            .as_ref()
            .unwrap()
            .dictionary
            .find_index("PRE_1")
            .is_none());
        assert!(ds
            .as_ref()
            .unwrap()
            .dictionary
            .find_index("RES_1")
            .is_none());
    }

    #[test]
    fn test_deming_sps_round_trip() {
        let ds = create_test_dataset();
        let sps =
            DemingRegressionCommand::to_sps("y", "x", 2.5, 0.95, true, true, false, true, false);
        let blocks = SyntaxLexer::split_commands(&sps).expect("lex");
        let parsed_cmd = SyntaxParser::parse_command(&blocks[0]).expect("parse");
        let (id, params) = map_stat_command(&parsed_cmd, Some(&ds))
            .expect("map")
            .expect("some");
        assert_eq!(id, "stat.regression.orthogonal");
        assert_eq!(params["dependent_var"], "y");
        assert_eq!(params["independent_var"], "x");
        assert!((params["lambda"].as_f64().unwrap() - 2.5).abs() < 1e-9);
        assert_eq!(params["include_scatter_plot"], true);
        assert_eq!(params["include_identity_line"], true);
        assert_eq!(params["include_residual_plot"], false);
        assert_eq!(params["save_predicted"], true);
        assert_eq!(params["save_residuals"], false);
    }
}
