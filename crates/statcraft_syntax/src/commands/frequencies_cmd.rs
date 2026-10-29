use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_frequencies_detailed, FrequencyInputItem, FrequencyResult};
use statcraft_core::DataSet;
use statcraft_io::RawValue;

pub struct FrequenciesCommand;

impl EngineCommand for FrequenciesCommand {
    fn id(&self) -> &'static str {
        "stat.frequencies"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Descriptive Statistics > Frequencies"
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
                "display_tables": {
                    "type": "boolean",
                    "description": "Display frequency tables",
                    "default": true
                }
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

        let mut results: Vec<FrequencyResult> = Vec::new();
        let active_rows = ds.active_row_indices();

        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: "stat.frequencies",
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;

            let var_meta = ds
                .dictionary
                .get(col_idx)
                .ok_or_else(|| CommandError::ExecutionFailed("Metadata missing".to_string()))?;

            let mut items: Vec<FrequencyInputItem> = Vec::with_capacity(active_rows.len());

            for &r in &active_rows {
                let cell_val = ds.get_cell(r, col_idx);
                let (value_str, is_missing, num_val) = match cell_val {
                    Some(RawValue::Number(n)) => {
                        let is_miss = var_meta.missing.is_missing(&RawValue::Number(*n));
                        let s = if n.fract() == 0.0 {
                            format!("{:.0}", n)
                        } else {
                            n.to_string()
                        };
                        (s, is_miss, Some(*n))
                    }
                    Some(RawValue::Text(s)) => {
                        let trimmed = s.trim();
                        let is_miss = trimmed.is_empty()
                            || trimmed == "."
                            || trimmed.eq_ignore_ascii_case("NA")
                            || var_meta
                                .missing
                                .is_missing(&RawValue::Text(trimmed.to_string()));
                        let num = trimmed.parse::<f64>().ok();
                        (trimmed.to_string(), is_miss, num)
                    }
                    Some(RawValue::Null) | None => (String::new(), true, None),
                };

                let label = if !is_missing {
                    var_meta.values.get_label(&value_str).map(|s| s.to_string())
                } else {
                    None
                };

                items.push(FrequencyInputItem {
                    value: value_str,
                    label,
                    is_missing,
                    numeric_value: num_val,
                    weight: ds.row_weight(r),
                });
            }

            let mut res = compute_frequencies_detailed(var_name, &items);
            // In SPSS, percentiles/quartiles are only computed for quantitative / non-nominal scales
            if var_meta.measure == statcraft_core::Measure::Nominal {
                res.quartiles = None;
            }
            results.push(res);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        // Read-only statistical procedure
        Ok(())
    }
}

impl FrequenciesCommand {
    pub fn to_sps(vars: &[String]) -> String {
        format!(
            "FREQUENCIES VARIABLES={}\n  /STATISTICS=STDDEV MEAN MEDIAN\n  /ORDER=ANALYSIS.",
            vars.join(" ")
        )
    }

    pub fn to_sps_syntax(&self, vars: &[String]) -> String {
        Self::to_sps(vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv = "gender,score\n1,85\n2,90\n1,95\n2,85\n1,100\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        let mut ds = DataSet::from_raw_table(table).unwrap();
        let mut labels = statcraft_core::ValueLabels::new();
        labels.insert("1", "Male");
        labels.insert("2", "Female");
        ds.dictionary.set_value_labels(0, labels).unwrap();
        ds.dictionary
            .set_measure(0, statcraft_core::Measure::Nominal)
            .unwrap();
        ds
    }

    #[test]
    fn test_frequencies_command_execution() {
        let cmd = FrequenciesCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "vars": ["gender", "score"]
        });

        assert!(cmd.enabled(ds.as_ref()));

        let res = cmd
            .execute(&mut ds, params)
            .expect("Execution should succeed");
        let results: Vec<FrequencyResult> = serde_json::from_value(res).expect("Valid JSON");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].variable, "gender");
        assert_eq!(results[0].n_valid, 5);
        assert_eq!(results[0].n_missing, 0);
        assert_eq!(results[0].categories.len(), 2);
        assert_eq!(results[0].categories[0].value, "1");
        assert_eq!(results[0].categories[0].label, Some("Male".to_string()));
        assert_eq!(results[0].categories[0].count, 3);
        assert!(results[0].quartiles.is_none());

        assert_eq!(results[1].variable, "score");
        assert_eq!(results[1].n_valid, 5);
        assert!(results[1].quartiles.is_some());
    }

    #[test]
    fn test_frequencies_sps_syntax() {
        let vars = vec!["gender".to_string(), "score".to_string()];
        let sps = FrequenciesCommand::to_sps(&vars);
        assert_eq!(
            sps,
            "FREQUENCIES VARIABLES=gender score\n  /STATISTICS=STDDEV MEAN MEDIAN\n  /ORDER=ANALYSIS."
        );
    }
}
