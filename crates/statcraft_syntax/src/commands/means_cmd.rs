use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_means_breakdown, MeansOptions, MeansProcedureResult, MeansTableResult,
};
use statcraft_core::DataSet;

pub struct MeansCommand;

impl MeansCommand {
    pub fn to_sps(
        dependent_vars: &[String],
        factor_vars: &[String],
        options: &MeansOptions,
    ) -> String {
        let deps = dependent_vars.join(" ");
        let factors = factor_vars.join(" ");
        let mut cells = Vec::new();
        if options.cell_mean {
            cells.push("MEAN");
        }
        if options.cell_count {
            cells.push("COUNT");
        }
        if options.cell_stddev {
            cells.push("STDDEV");
        }
        if options.cell_variance {
            cells.push("VARIANCE");
        }
        if options.cell_median {
            cells.push("MEDIAN");
        }
        if options.cell_min {
            cells.push("MIN");
        }
        if options.cell_max {
            cells.push("MAX");
        }
        let cells_str = cells.join(" ");

        let mut lines = vec![
            format!("MEANS TABLES={} BY {}", deps, factors),
            format!("  /CELLS={}", cells_str),
        ];

        if options.anova {
            lines.push("  /STATISTICS ANOVA.".to_string());
        } else {
            lines.push("  /STATISTICS NONE.".to_string());
        }

        lines.join("\n")
    }
}

impl EngineCommand for MeansCommand {
    fn id(&self) -> &'static str {
        "stat.means"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Compare Means > Means"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependent_vars", "factor_vars"],
            "properties": {
                "dependent_vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of quantitative dependent variable names"
                },
                "factor_vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of categorical grouping factor variables"
                },
                "cell_mean": { "type": "boolean", "default": true },
                "cell_count": { "type": "boolean", "default": true },
                "cell_stddev": { "type": "boolean", "default": true },
                "cell_variance": { "type": "boolean", "default": false },
                "cell_median": { "type": "boolean", "default": true },
                "cell_min": { "type": "boolean", "default": false },
                "cell_max": { "type": "boolean", "default": false },
                "anova": { "type": "boolean", "default": true }
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

        let dependent_vars: Vec<String> = params
            .get("dependent_vars")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if dependent_vars.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'dependent_vars' must contain at least one variable".to_string(),
            });
        }

        let factor_vars: Vec<String> = params
            .get("factor_vars")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if factor_vars.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'factor_vars' must contain at least one variable".to_string(),
            });
        }

        let options = MeansOptions {
            cell_mean: params
                .get("cell_mean")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            cell_count: params
                .get("cell_count")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            cell_stddev: params
                .get("cell_stddev")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            cell_variance: params
                .get("cell_variance")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            cell_median: params
                .get("cell_median")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            cell_min: params
                .get("cell_min")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            cell_max: params
                .get("cell_max")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            anova: params
                .get("anova")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        };

        let active_rows = ds.active_row_indices();
        let mut weights = Vec::with_capacity(active_rows.len());
        for &r in &active_rows {
            weights.push(ds.row_weight(r));
        }

        let mut tables: Vec<MeansTableResult> = Vec::new();

        for factor_var in &factor_vars {
            let factor_col_idx = ds.dictionary.find_index(factor_var).ok_or_else(|| {
                CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Factor variable '{}' not found in dataset", factor_var),
                }
            })?;

            let mut factor_values: Vec<Option<String>> = Vec::with_capacity(active_rows.len());
            for &r in &active_rows {
                let cell = ds.get_cell_display(r, factor_col_idx, false);
                let val = if cell.trim().is_empty() || cell == "." || cell == "NA" {
                    None
                } else {
                    Some(cell)
                };
                factor_values.push(val);
            }

            for dep_var in &dependent_vars {
                let dep_col_idx = ds.dictionary.find_index(dep_var).ok_or_else(|| {
                    CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Dependent variable '{}' not found in dataset", dep_var),
                    }
                })?;

                let mut dep_values: Vec<Option<f64>> = Vec::with_capacity(active_rows.len());
                for &r in &active_rows {
                    let cell = ds.get_cell_display(r, dep_col_idx, false);
                    let val = cell.trim().parse::<f64>().ok();
                    dep_values.push(val);
                }

                match compute_means_breakdown(
                    dep_var,
                    factor_var,
                    &dep_values,
                    &factor_values,
                    Some(&weights),
                    &options,
                ) {
                    Ok(table) => tables.push(table),
                    Err(e) => {
                        return Err(CommandError::ExecutionFailed(format!(
                            "Failed computing means for '{}' BY '{}': {}",
                            dep_var, factor_var, e
                        )));
                    }
                }
            }
        }

        let proc_result = MeansProcedureResult { tables };
        serde_json::to_value(&proc_result)
            .map_err(|e| CommandError::ExecutionFailed(format!("Serialization error: {}", e)))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv_data = "group,score\n1,10\n1,12\n2,20\n2,22\n3,30\n3,32\n";
        let table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_means_sps_syntax() {
        let options = MeansOptions::default();
        let sps = MeansCommand::to_sps(
            &["salary".to_string(), "bonus".to_string()],
            &["jobcat".to_string()],
            &options,
        );
        assert_eq!(
            sps,
            "MEANS TABLES=salary bonus BY jobcat\n  /CELLS=MEAN COUNT STDDEV MEDIAN\n  /STATISTICS ANOVA."
        );
    }

    #[test]
    fn test_means_command_execution() {
        let cmd = MeansCommand;
        let mut dataset = Some(make_test_dataset());

        let params = json!({
            "dependent_vars": ["score"],
            "factor_vars": ["group"],
            "cell_mean": true,
            "cell_count": true,
            "cell_stddev": true,
            "anova": true
        });

        let res = cmd.execute(&mut dataset, params).expect("Should succeed");
        let proc_result: MeansProcedureResult =
            serde_json::from_value(res).expect("Should deserialize");

        assert_eq!(proc_result.tables.len(), 1);
        let table = &proc_result.tables[0];
        assert_eq!(table.dep_var, "score");
        assert_eq!(table.factor_var, "group");
        assert_eq!(table.cells.len(), 3);
        assert!(table.anova.is_some());
        assert!(table.association.is_some());
    }
}
