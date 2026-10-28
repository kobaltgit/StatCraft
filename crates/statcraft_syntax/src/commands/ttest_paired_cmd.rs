use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_paired_ttest, PairedDataInput, PairedTTestResult};
use statcraft_core::DataSet;

pub struct PairedSamplesTTestCommand;

impl EngineCommand for PairedSamplesTTestCommand {
    fn id(&self) -> &'static str {
        "stat.ttest.paired"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Compare Means > Paired-Samples T Test"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["pairs"],
            "properties": {
                "pairs": {
                    "type": "array",
                    "items": {
                        "type": "array",
                        "items": { "type": "string" },
                        "minItems": 2,
                        "maxItems": 2
                    },
                    "description": "List of pairs of quantitative variable names, e.g. [[\"before\", \"after\"]]"
                },
                "ci": {
                    "type": "number",
                    "default": 0.95,
                    "description": "Confidence interval level (e.g. 0.95 for 95%)"
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

        let raw_pairs = params
            .get("pairs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'pairs' must be an array of [var1, var2] pairs".to_string(),
            })?;

        if raw_pairs.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'pairs' must contain at least one pair".to_string(),
            });
        }

        let mut parsed_pairs: Vec<(String, String)> = Vec::with_capacity(raw_pairs.len());
        for p in raw_pairs {
            if let Some(arr) = p.as_array() {
                if arr.len() >= 2 {
                    if let (Some(v1), Some(v2)) = (arr[0].as_str(), arr[1].as_str()) {
                        parsed_pairs.push((v1.to_string(), v2.to_string()));
                        continue;
                    }
                }
            }
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Each pair in 'pairs' must be a 2-element array of strings [var1, var2]"
                    .to_string(),
            });
        }

        let ci = params.get("ci").and_then(|v| v.as_f64()).unwrap_or(0.95);

        let total_rows = ds.row_count();
        let mut col_data_cache: std::collections::HashMap<String, Vec<Option<f64>>> =
            std::collections::HashMap::new();

        // Extract column values for all unique variables involved
        for (v1, v2) in &parsed_pairs {
            for v_name in [v1, v2] {
                if !col_data_cache.contains_key(v_name) {
                    let col_idx = ds.dictionary.find_index(v_name).ok_or_else(|| {
                        CommandError::InvalidParams {
                            id: self.id(),
                            reason: format!("Variable '{}' not found in dataset", v_name),
                        }
                    })?;

                    let mut values: Vec<Option<f64>> = Vec::with_capacity(total_rows);
                    for r in 0..total_rows {
                        let cell = ds.get_cell_display(r, col_idx, false);
                        let val = if cell.trim().is_empty() || cell == "." || cell == "NA" {
                            None
                        } else {
                            cell.parse::<f64>().ok()
                        };
                        values.push(val);
                    }
                    col_data_cache.insert(v_name.clone(), values);
                }
            }
        }

        // Prepare pairs for algo
        let mut algo_pairs: Vec<PairedDataInput<'_>> = Vec::with_capacity(parsed_pairs.len());

        for (v1, v2) in &parsed_pairs {
            let data1 = col_data_cache.get(v1).map(|v| v.as_slice()).unwrap_or(&[]);
            let data2 = col_data_cache.get(v2).map(|v| v.as_slice()).unwrap_or(&[]);
            algo_pairs.push((v1.as_str(), data1, v2.as_str(), data2));
        }

        let results: Vec<PairedTTestResult> = compute_paired_ttest(&algo_pairs, ci);

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl PairedSamplesTTestCommand {
    pub fn to_sps(pairs: &[(String, String)], ci: f64) -> String {
        let mut pairs_strs = Vec::new();
        for (v1, v2) in pairs {
            pairs_strs.push(format!("{} WITH {}", v1, v2));
        }
        format!(
            "T-TEST PAIRS={} (PAIRED)\n  /CRITERIA=CI({:.2}).",
            pairs_strs.join(" "),
            ci
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv = "id,before,after\n1,10.0,8.0\n2,12.0,11.0\n3,15.0,13.0\n4,11.0,10.0\n5,14.0,15.0\n6,16.0,14.0\n7,13.0,12.0\n8,18.0,16.0\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_ttest_paired_command_execution() {
        let cmd = PairedSamplesTTestCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "pairs": [["before", "after"]],
            "ci": 0.95
        });

        let res = cmd
            .execute(&mut ds, params)
            .expect("execution should succeed");
        let results: Vec<PairedTTestResult> =
            serde_json::from_value(res).expect("should deserialize to PairedTTestResult");

        assert_eq!(results.len(), 1);
        let pair = &results[0];
        assert_eq!(pair.pair_num, 1);
        assert_eq!(pair.var1_stats.var_name, "before");
        assert_eq!(pair.var2_stats.var_name, "after");
        assert_eq!(pair.var1_stats.n, 8);
        assert!((pair.differences.mean_diff.unwrap() - 1.25).abs() < 1e-6);
        assert!((pair.differences.t_stat.unwrap() - 3.415650255).abs() < 1e-6);
    }

    #[test]
    fn test_ttest_paired_sps_syntax() {
        let pairs = vec![
            ("pre_test".to_string(), "post_test".to_string()),
            ("weight_1".to_string(), "weight_2".to_string()),
        ];
        let sps = PairedSamplesTTestCommand::to_sps(&pairs, 0.95);
        assert_eq!(
            sps,
            "T-TEST PAIRS=pre_test WITH post_test weight_1 WITH weight_2 (PAIRED)\n  /CRITERIA=CI(0.95)."
        );
    }

    #[test]
    fn test_ttest_paired_command_missing_variable() {
        let cmd = PairedSamplesTTestCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "pairs": [["before", "nonexistent_var"]],
            "ci": 0.95
        });

        let err = cmd.execute(&mut ds, params).unwrap_err();
        match err {
            CommandError::InvalidParams { reason, .. } => {
                assert!(reason.contains("nonexistent_var"));
            }
            _ => panic!("Expected InvalidParams error"),
        }
    }
}
