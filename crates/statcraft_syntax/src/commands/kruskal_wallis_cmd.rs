use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_kruskal_wallis, KruskalWallisResult};
use statcraft_core::DataSet;

pub struct KruskalWallisCommand;

impl EngineCommand for KruskalWallisCommand {
    fn id(&self) -> &'static str {
        "stat.npar.kruskal_wallis"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Nonparametric Tests > K Independent Samples"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["vars", "grouping_var", "min_range", "max_range"],
            "properties": {
                "vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of test variable names to compare across groups"
                },
                "grouping_var": {
                    "type": "string",
                    "description": "Name of the grouping variable"
                },
                "min_range": {
                    "type": "number",
                    "description": "Minimum value defining the range of the grouping variable"
                },
                "max_range": {
                    "type": "number",
                    "description": "Maximum value defining the range of the grouping variable"
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

        let min_range = params
            .get("min_range")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'min_range' is required".to_string(),
            })?;

        let max_range = params
            .get("max_range")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'max_range' is required".to_string(),
            })?;

        if min_range > max_range {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: format!(
                    "min_range ({min_range}) cannot be greater than max_range ({max_range})"
                ),
            });
        }

        let group_col_idx =
            ds.dictionary
                .find_index(grouping_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Grouping variable '{grouping_var}' not found in dataset"),
                })?;

        let active_rows = ds.active_row_indices();
        let mut group_values: Vec<Option<String>> = Vec::with_capacity(active_rows.len());
        for &row_idx in &active_rows {
            let cell_str = ds.get_cell_display(row_idx, group_col_idx, false);
            let val = if cell_str.trim().is_empty() || cell_str == "." || cell_str == "NA" {
                None
            } else {
                Some(cell_str.trim().to_string())
            };
            group_values.push(val);
        }

        let mut results: Vec<KruskalWallisResult> = Vec::with_capacity(var_names.len());

        for var_name in &var_names {
            let var_col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Test variable '{var_name}' not found in dataset"),
                    })?;

            let mut var_values: Vec<Option<f64>> = Vec::with_capacity(active_rows.len());
            for &row_idx in &active_rows {
                let cell_str = ds.get_cell_display(row_idx, var_col_idx, false);
                let cell_num = if cell_str.trim().is_empty() || cell_str == "." || cell_str == "NA"
                {
                    None
                } else {
                    cell_str.parse::<f64>().ok()
                };
                var_values.push(cell_num);
            }

            let res = compute_kruskal_wallis(
                var_name,
                grouping_var,
                &var_values,
                &group_values,
                min_range,
                max_range,
            );
            results.push(res);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl KruskalWallisCommand {
    pub fn to_sps(vars: &[String], grouping_var: &str, min_range: f64, max_range: f64) -> String {
        let min_str = if (min_range.fract()).abs() < 1e-9 {
            format!("{}", min_range as i64)
        } else {
            format!("{min_range}")
        };
        let max_str = if (max_range.fract()).abs() < 1e-9 {
            format!("{}", max_range as i64)
        } else {
            format!("{max_range}")
        };
        format!(
            "NPAR TESTS\n  /K-W= {} BY {}({} {})\n  /MISSING=ANALYSIS.",
            vars.join(" "),
            grouping_var,
            min_str,
            max_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv_data = "score,category\n10,1\n12,1\n20,2\n22,2\n30,3\n32,3\n";
        let table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_kruskal_wallis_command_execution() {
        let ds = make_test_dataset();
        let cmd = KruskalWallisCommand;

        let params = json!({
            "vars": ["score"],
            "grouping_var": "category",
            "min_range": 1.0,
            "max_range": 3.0
        });

        let mut opt_ds = Some(ds);
        let res_val = cmd.execute(&mut opt_ds, params).unwrap();
        let results: Vec<KruskalWallisResult> = serde_json::from_value(res_val).unwrap();

        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert_eq!(r.variable, "score");
        assert_eq!(r.grouping_var, "category");
        assert_eq!(r.total_n, 6);
        assert_eq!(r.groups.len(), 3);
        assert_eq!(r.df, 2);
        assert!(r.kruskal_wallis_h > 0.0);
    }

    #[test]
    fn test_kruskal_wallis_command_missing_vars() {
        let ds = make_test_dataset();
        let cmd = KruskalWallisCommand;

        let params = json!({
            "vars": [],
            "grouping_var": "category",
            "min_range": 1.0,
            "max_range": 3.0
        });

        let mut opt_ds = Some(ds);
        let res = cmd.execute(&mut opt_ds, params);
        assert!(res.is_err());
    }

    #[test]
    fn test_kruskal_wallis_sps_syntax() {
        let vars = vec!["salary".to_string(), "age".to_string()];
        let sps = KruskalWallisCommand::to_sps(&vars, "jobcat", 1.0, 3.0);
        assert_eq!(
            sps,
            "NPAR TESTS\n  /K-W= salary age BY jobcat(1 3)\n  /MISSING=ANALYSIS."
        );
    }
}
