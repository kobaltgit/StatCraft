use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_reliability, ReliabilityModel, ReliabilityOptions, ReliabilityResult,
};
use statcraft_core::DataSet;

pub struct ReliabilityCommand;

impl EngineCommand for ReliabilityCommand {
    fn id(&self) -> &'static str {
        "stat.scale.reliability"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Scale > Reliability Analysis"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["variables"],
            "properties": {
                "variables": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of variable names to analyze (at least 2)"
                },
                "scale_name": {
                    "type": "string",
                    "default": "ALL VARIABLES",
                    "description": "Scale label"
                },
                "model": {
                    "type": "string",
                    "enum": ["alpha"],
                    "default": "alpha",
                    "description": "Reliability model"
                },
                "item_statistics": {
                    "type": "boolean",
                    "default": true,
                    "description": "Compute item statistics (Mean, Std Dev, N)"
                },
                "scale_statistics": {
                    "type": "boolean",
                    "default": true,
                    "description": "Compute scale statistics"
                },
                "scale_if_item_deleted": {
                    "type": "boolean",
                    "default": true,
                    "description": "Compute scale statistics if item deleted"
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
            .get("variables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if var_names.len() < 2 {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'variables' must contain at least 2 variables".to_string(),
            });
        }

        let scale_name = params
            .get("scale_name")
            .and_then(|v| v.as_str())
            .unwrap_or("ALL VARIABLES")
            .to_string();

        let _model_str = params
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("alpha");

        let model = ReliabilityModel::Alpha;

        let item_statistics = params
            .get("item_statistics")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let scale_statistics = params
            .get("scale_statistics")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let scale_if_item_deleted = params
            .get("scale_if_item_deleted")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let options = ReliabilityOptions {
            scale_name,
            model,
            item_statistics,
            scale_statistics,
            scale_if_item_deleted,
        };

        let active_rows = ds.active_row_indices();
        let mut var_data: Vec<(String, Vec<Option<f64>>)> = Vec::with_capacity(var_names.len());

        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;

            let mut vals: Vec<Option<f64>> = Vec::with_capacity(active_rows.len());
            for &r in &active_rows {
                let cell_str = ds.get_cell_display(r, col_idx, false);
                let v = if cell_str.trim().is_empty() || cell_str == "." || cell_str == "NA" {
                    None
                } else {
                    cell_str.parse::<f64>().ok()
                };
                vals.push(v);
            }
            var_data.push((var_name.clone(), vals));
        }

        let mut weights = Vec::with_capacity(active_rows.len());
        for &r in &active_rows {
            weights.push(ds.row_weight(r));
        }

        let items: Vec<(&str, &[Option<f64>])> = var_data
            .iter()
            .map(|(name, vals)| (name.as_str(), vals.as_slice()))
            .collect();

        let result: ReliabilityResult = compute_reliability(&items, &weights, &options);

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl ReliabilityCommand {
    pub fn to_sps(
        variables: &[String],
        scale_name: &str,
        model: ReliabilityModel,
        options: &ReliabilityOptions,
    ) -> String {
        let vars_str = variables.join(" ");
        let model_str = match model {
            ReliabilityModel::Alpha => "ALPHA",
        };

        let mut parts = vec![
            "RELIABILITY".to_string(),
            format!("  /VARIABLES={}", vars_str),
            format!("  /SCALE('{}') ALL", scale_name),
            format!("  /MODEL={}", model_str),
        ];

        let mut stats = Vec::new();
        if options.item_statistics {
            stats.push("DESCRIPTIVES");
        }
        if options.scale_statistics {
            stats.push("SCALE");
        }
        if !stats.is_empty() {
            parts.push(format!("  /STATISTICS={}", stats.join(" ")));
        }

        if options.scale_if_item_deleted {
            parts.push("  /SUMMARY=TOTAL.".to_string());
        } else if let Some(last) = parts.last_mut() {
            last.push('.');
        }

        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn create_test_dataset() -> DataSet {
        let csv_data =
            "q1,q2,q3\n1.0,2.0,1.0\n2.0,3.0,3.0\n3.0,4.0,2.0\n4.0,5.0,4.0\n5.0,6.0,5.0\n";
        let table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_reliability_command_execution() {
        let cmd = ReliabilityCommand;
        let mut ds = Some(create_test_dataset());

        let params = json!({
            "variables": ["q1", "q2", "q3"],
            "scale_name": "TEST SCALE",
            "model": "alpha",
            "item_statistics": true,
            "scale_statistics": true,
            "scale_if_item_deleted": true
        });

        let result = cmd.execute(&mut ds, params).expect("Execution failed");
        let rel_res: ReliabilityResult =
            serde_json::from_value(result).expect("Deserialization failed");

        assert_eq!(rel_res.scale_name, "TEST SCALE");
        assert_eq!(rel_res.case_processing.n_valid, 5);
        let alpha = rel_res
            .reliability_stats
            .cronbach_alpha
            .expect("Cronbach's alpha missing");
        assert!((alpha - 0.976744).abs() < 1e-4);
        assert_eq!(rel_res.item_stats.len(), 3);
        assert_eq!(rel_res.item_total_stats.len(), 3);
    }

    #[test]
    fn test_reliability_sps_syntax() {
        let vars = vec!["q1".to_string(), "q2".to_string(), "q3".to_string()];
        let options = ReliabilityOptions {
            scale_name: "ALL VARIABLES".to_string(),
            model: ReliabilityModel::Alpha,
            item_statistics: true,
            scale_statistics: true,
            scale_if_item_deleted: true,
        };

        let sps =
            ReliabilityCommand::to_sps(&vars, "ALL VARIABLES", ReliabilityModel::Alpha, &options);
        assert!(sps.contains("/VARIABLES=q1 q2 q3"));
        assert!(sps.contains("/SCALE('ALL VARIABLES') ALL"));
        assert!(sps.contains("/MODEL=ALPHA"));
        assert!(sps.contains("/STATISTICS=DESCRIPTIVES SCALE"));
        assert!(sps.contains("/SUMMARY=TOTAL."));
    }
}
