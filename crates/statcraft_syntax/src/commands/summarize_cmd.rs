use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_summarize, SummarizeOptions};
use statcraft_core::{DataSet, RawValue};

pub struct SummarizeCommand;

impl SummarizeCommand {
    pub fn to_sps(variables: &[String], group_by: &[String], limit_cases: Option<usize>) -> String {
        let mut sps = format!("SUMMARIZE\n  /TABLES = {}", variables.join(" "));
        if !group_by.is_empty() {
            sps.push_str(&format!(" BY {}", group_by.join(" ")));
        }
        sps.push('\n');
        if let Some(limit) = limit_cases {
            sps.push_str(&format!(
                "  /FORMAT = VALIDLIST NOCASENUM TOTAL LIMIT({})\n",
                limit
            ));
        } else {
            sps.push_str("  /FORMAT = VALIDLIST NOCASENUM TOTAL\n");
        }
        sps.push_str("  /TITLE = 'Case Summaries'\n  /CELLS = COUNT MEAN STDDEV SUM.\nEXECUTE.");
        sps
    }
}

impl EngineCommand for SummarizeCommand {
    fn id(&self) -> &'static str {
        "stat.reports.summarize"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze -> Reports -> Case Summaries..."
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn category(&self) -> &'static str {
        "Reports"
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["SUMMARIZE", "CASES"]
    }

    fn specs(&self) -> Value {
        json!({
            "command": "stat.reports.summarize",
            "required": ["variables"],
            "properties": {
                "variables": { "type": "array", "items": { "type": "string" } },
                "group_by": { "type": "array", "items": { "type": "string" }, "default": [] },
                "limit_cases": { "type": "integer", "default": 100 },
                "display_cases": { "type": "boolean", "default": true }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|ds| ds.col_count() >= 1).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let var_names = params
            .get("variables")
            .and_then(|v| v.as_array())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'variables' array".to_string(),
            })?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();

        if var_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "At least one variable must be specified".to_string(),
            });
        }

        let group_names = params
            .get("group_by")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let limit = params
            .get("limit_cases")
            .and_then(|v| v.as_u64())
            .map(|u| u as usize);

        let display_cases = params
            .get("display_cases")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Extract variables data
        let mut var_strings = Vec::with_capacity(var_names.len());
        let mut var_numbers = Vec::with_capacity(var_names.len());

        for name in &var_names {
            let idx =
                ds.dictionary
                    .find_index(name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found", name),
                    })?;

            let mut str_vals = Vec::with_capacity(ds.row_count());
            let mut num_vals = Vec::with_capacity(ds.row_count());

            for val in &ds.table.columns[idx].values {
                match val {
                    RawValue::Number(n) => {
                        str_vals.push(Some(format!("{}", n)));
                        num_vals.push(Some(*n));
                    }
                    RawValue::Text(s) => {
                        str_vals.push(Some(s.clone()));
                        num_vals.push(s.trim().parse::<f64>().ok());
                    }
                    _ => {
                        str_vals.push(None);
                        num_vals.push(None);
                    }
                }
            }
            var_strings.push(str_vals);
            var_numbers.push(num_vals);
        }

        // Extract grouping data
        let mut grp_strings = Vec::with_capacity(group_names.len());
        for name in &group_names {
            let idx =
                ds.dictionary
                    .find_index(name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Grouping variable '{}' not found", name),
                    })?;

            let mut str_vals = Vec::with_capacity(ds.row_count());
            for val in &ds.table.columns[idx].values {
                match val {
                    RawValue::Number(n) => str_vals.push(Some(format!("{}", n))),
                    RawValue::Text(s) => str_vals.push(Some(s.clone())),
                    _ => str_vals.push(None),
                }
            }
            grp_strings.push(str_vals);
        }

        let options = SummarizeOptions {
            limit_cases: limit,
            display_cases,
            compute_mean: true,
            compute_stddev: true,
            compute_sum: true,
            compute_count: true,
        };

        let result = compute_summarize(
            &var_names,
            &var_strings,
            &var_numbers,
            &group_names,
            &grp_strings,
            &options,
        )
        .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
