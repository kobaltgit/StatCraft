use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_related_samples, RelatedPairTestResult};
use statcraft_core::DataSet;

pub struct RelatedSamplesTestCommand;

impl EngineCommand for RelatedSamplesTestCommand {
    fn id(&self) -> &'static str {
        "stat.npar.related2"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Nonparametric Tests > Two Related Samples"
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
                "wilcoxon": {
                    "type": "boolean",
                    "default": true,
                    "description": "Compute Wilcoxon signed-ranks test"
                },
                "sign": {
                    "type": "boolean",
                    "default": false,
                    "description": "Compute Sign test"
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
                reason: "Parameter 'pairs' must be an array of variable pairs".to_string(),
            })?;

        if raw_pairs.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'pairs' must contain at least one pair of variables".to_string(),
            });
        }

        let mut pairs: Vec<(String, String)> = Vec::with_capacity(raw_pairs.len());
        for p in raw_pairs {
            if let Some(arr) = p.as_array() {
                if arr.len() == 2 {
                    if let (Some(v1), Some(v2)) = (arr[0].as_str(), arr[1].as_str()) {
                        pairs.push((v1.to_string(), v2.to_string()));
                        continue;
                    }
                }
            }
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Each pair in 'pairs' must be an array of exactly 2 variable names"
                    .to_string(),
            });
        }

        let do_wilcoxon = params
            .get("wilcoxon")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let do_sign = params
            .get("sign")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !do_wilcoxon && !do_sign {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "At least one test (Wilcoxon or Sign) must be selected".to_string(),
            });
        }

        let active_rows = ds.active_row_indices();
        let mut results: Vec<RelatedPairTestResult> = Vec::with_capacity(pairs.len());

        for (var1, var2) in &pairs {
            let col1_idx =
                ds.dictionary
                    .find_index(var1)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dataset", var1),
                    })?;
            let col2_idx =
                ds.dictionary
                    .find_index(var2)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dataset", var2),
                    })?;

            let mut paired_data: Vec<(Option<f64>, Option<f64>)> =
                Vec::with_capacity(active_rows.len());

            for &r in &active_rows {
                let cell1 = ds.get_cell_display(r, col1_idx, false);
                let val1 = if cell1.trim().is_empty() || cell1 == "." || cell1 == "NA" {
                    None
                } else {
                    cell1.trim().parse::<f64>().ok()
                };

                let cell2 = ds.get_cell_display(r, col2_idx, false);
                let val2 = if cell2.trim().is_empty() || cell2 == "." || cell2 == "NA" {
                    None
                } else {
                    cell2.trim().parse::<f64>().ok()
                };

                paired_data.push((val1, val2));
            }

            let pair_res = compute_related_samples(var1, var2, &paired_data, do_wilcoxon, do_sign);
            results.push(pair_res);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl RelatedSamplesTestCommand {
    pub fn to_sps(pairs: &[(String, String)], wilcoxon: bool, sign: bool) -> String {
        let pairs_clause = pairs
            .iter()
            .map(|(v1, v2)| format!("{} WITH {}", v1, v2))
            .collect::<Vec<_>>()
            .join(" ");

        let mut lines = vec!["NPAR TESTS".to_string()];
        if wilcoxon {
            lines.push(format!("  /WILCOXON= {} (PAIRED)", pairs_clause));
        }
        if sign {
            lines.push(format!("  /SIGN= {} (PAIRED)", pairs_clause));
        }
        lines.push("  /MISSING=ANALYSIS.".to_string());
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv_data = "pre,post\n15,18\n16,16\n17,20\n25,22\n12,15\n";
        let table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_related_samples_command_execution() {
        let ds = make_test_dataset();
        let cmd = RelatedSamplesTestCommand;

        let params = json!({
            "pairs": [["pre", "post"]],
            "wilcoxon": true,
            "sign": true
        });

        let mut opt_ds = Some(ds);
        let res_val = cmd.execute(&mut opt_ds, params).unwrap();
        let results: Vec<RelatedPairTestResult> = serde_json::from_value(res_val).unwrap();

        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert_eq!(r.var1, "pre");
        assert_eq!(r.var2, "post");

        let w = r.wilcoxon.as_ref().unwrap();
        assert_eq!(w.ranks.total_n, 5);
        assert_eq!(w.ranks.ties_count, 1);
        assert_eq!(w.ranks.negative_ranks.n, 1);
        assert_eq!(w.ranks.positive_ranks.n, 3);
        assert_eq!(w.test_statistic_w, 2.5);

        let s = r.sign.as_ref().unwrap();
        assert_eq!(s.diffs.total_n, 5);
        assert_eq!(s.diffs.ties_count, 1);
        assert_eq!(s.diffs.negative_count, 1);
        assert_eq!(s.diffs.positive_count, 3);
        assert!((s.exact_sig_2tailed - 0.625).abs() < 1e-6);
    }

    #[test]
    fn test_related_samples_sps_syntax() {
        let pairs = vec![
            ("pre".to_string(), "post".to_string()),
            ("t1".to_string(), "t2".to_string()),
        ];
        let sps_both = RelatedSamplesTestCommand::to_sps(&pairs, true, true);
        assert_eq!(
            sps_both,
            "NPAR TESTS\n  /WILCOXON= pre WITH post t1 WITH t2 (PAIRED)\n  /SIGN= pre WITH post t1 WITH t2 (PAIRED)\n  /MISSING=ANALYSIS."
        );

        let sps_wilcoxon = RelatedSamplesTestCommand::to_sps(&pairs, true, false);
        assert_eq!(
            sps_wilcoxon,
            "NPAR TESTS\n  /WILCOXON= pre WITH post t1 WITH t2 (PAIRED)\n  /MISSING=ANALYSIS."
        );
    }

    #[test]
    fn test_related_samples_missing_variable() {
        let ds = make_test_dataset();
        let cmd = RelatedSamplesTestCommand;
        let mut opt_ds = Some(ds);

        let params = json!({
            "pairs": [["pre", "nonexistent"]],
            "wilcoxon": true,
            "sign": false
        });

        assert!(cmd.execute(&mut opt_ds, params).is_err());
    }

    #[test]
    fn test_related_samples_no_tests_selected() {
        let ds = make_test_dataset();
        let cmd = RelatedSamplesTestCommand;
        let mut opt_ds = Some(ds);

        let params = json!({
            "pairs": [["pre", "post"]],
            "wilcoxon": false,
            "sign": false
        });

        assert!(cmd.execute(&mut opt_ds, params).is_err());
    }
}
