use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_mann_whitney, MannWhitneyResult};
use statcraft_core::DataSet;

pub struct MannWhitneyCommand;

impl EngineCommand for MannWhitneyCommand {
    fn id(&self) -> &'static str {
        "stat.npar.mann_whitney"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Nonparametric Tests > Two Independent Samples"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["vars", "grouping_var", "group1", "group2"],
            "properties": {
                "vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of test variable names to compare across groups"
                },
                "grouping_var": {
                    "type": "string",
                    "description": "Name of the binary grouping variable"
                },
                "group1": {
                    "type": "string",
                    "description": "Value defining the first group"
                },
                "group2": {
                    "type": "string",
                    "description": "Value defining the second group"
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
                reason: "Parameter 'vars' must contain at least one test variable name".to_string(),
            });
        }

        let grouping_var = params
            .get("grouping_var")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'grouping_var' is required".to_string(),
            })?;

        let group1_val = params
            .get("group1")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'group1' is required".to_string(),
            })?;

        let group2_val = params
            .get("group2")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'group2' is required".to_string(),
            })?;

        let group_col_idx =
            ds.dictionary
                .find_index(grouping_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Grouping variable '{}' not found in dataset", grouping_var),
                })?;

        let active_rows = ds.active_row_indices();
        let mut group_values: Vec<Option<String>> = Vec::with_capacity(active_rows.len());
        for &r in &active_rows {
            let cell_str = ds.get_cell_display(r, group_col_idx, false);
            let val = if cell_str.trim().is_empty() || cell_str == "." || cell_str == "NA" {
                None
            } else {
                Some(cell_str.trim().to_string())
            };
            group_values.push(val);
        }

        let mut results: Vec<MannWhitneyResult> = Vec::new();

        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;

            let mut var_values: Vec<Option<f64>> = Vec::with_capacity(active_rows.len());
            for &r in &active_rows {
                let cell_str = ds.get_cell_display(r, col_idx, false);
                let val = if cell_str.trim().is_empty() || cell_str == "." || cell_str == "NA" {
                    None
                } else {
                    cell_str.parse::<f64>().ok()
                };
                var_values.push(val);
            }

            let res = compute_mann_whitney(
                var_name,
                grouping_var,
                &var_values,
                &group_values,
                group1_val,
                group2_val,
            );
            results.push(res);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl MannWhitneyCommand {
    pub fn to_sps(vars: &[String], grouping_var: &str, group1: &str, group2: &str) -> String {
        format!(
            "NPAR TESTS\n  /M-W= {} BY {}({} {})\n  /MISSING=ANALYSIS.",
            vars.join(" "),
            grouping_var,
            group1,
            group2
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv_data = "score,gender\n10,1\n15,1\n15,1\n20,1\n25,1\n15,2\n20,2\n25,2\n30,2\n35,2\n";
        let table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_mann_whitney_command_execution() {
        let ds = make_test_dataset();
        let cmd = MannWhitneyCommand;

        let params = json!({
            "vars": ["score"],
            "grouping_var": "gender",
            "group1": "1",
            "group2": "2"
        });

        let mut opt_ds = Some(ds);
        let res_val = cmd.execute(&mut opt_ds, params).unwrap();
        let results: Vec<MannWhitneyResult> = serde_json::from_value(res_val).unwrap();

        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert_eq!(r.variable, "score");
        assert_eq!(r.grouping_var, "gender");
        assert_eq!(r.group1_stats.n, 5);
        assert_eq!(r.group2_stats.n, 5);
        assert_eq!(r.mann_whitney_u, 5.0);
        assert_eq!(r.wilcoxon_w, 20.0);
    }

    #[test]
    fn test_mann_whitney_sps_syntax() {
        let sps = MannWhitneyCommand::to_sps(
            &["salary".to_string(), "age".to_string()],
            "gender",
            "1",
            "2",
        );
        assert_eq!(
            sps,
            "NPAR TESTS\n  /M-W= salary age BY gender(1 2)\n  /MISSING=ANALYSIS."
        );
    }

    #[test]
    fn test_mann_whitney_command_missing_vars() {
        let ds = make_test_dataset();
        let cmd = MannWhitneyCommand;
        let mut opt_ds = Some(ds);

        let params = json!({
            "vars": [],
            "grouping_var": "gender",
            "group1": "1",
            "group2": "2"
        });

        assert!(cmd.execute(&mut opt_ds, params).is_err());
    }
}
