use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_codebook, CodebookOptions, RawColumnData};
use statcraft_core::{DataSet, RawValue};

pub struct CodebookCommand;

impl CodebookCommand {
    pub fn to_sps(variables: &[String], max_categories: usize) -> String {
        let vars_str = if variables.is_empty() {
            "ALL".to_string()
        } else {
            variables.join(" ")
        };
        format!(
            "CODEBOOK {}\n  /VARINFO = POSITION LABEL TYPE FORMAT MEASURE MISSING\n  /OPTIONS = MAXCATS({}).\nEXECUTE.",
            vars_str, max_categories
        )
    }
}

impl EngineCommand for CodebookCommand {
    fn id(&self) -> &'static str {
        "stat.reports.codebook"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze -> Reports -> Codebook..."
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn category(&self) -> &'static str {
        "Reports"
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["CODEBOOK"]
    }

    fn specs(&self) -> Value {
        json!({
            "command": "stat.reports.codebook",
            "properties": {
                "variables": { "type": "array", "items": { "type": "string" } },
                "max_categories": { "type": "integer", "default": 200 },
                "include_counts": { "type": "boolean", "default": true },
                "include_stats": { "type": "boolean", "default": true }
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

        let var_names = if let Some(arr) = params.get("variables").and_then(|v| v.as_array()) {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        } else {
            ds.dictionary
                .variables
                .iter()
                .map(|v| v.name.clone())
                .collect()
        };

        let selected_vars = if var_names.is_empty() {
            ds.dictionary
                .variables
                .iter()
                .map(|v| v.name.clone())
                .collect()
        } else {
            var_names
        };

        let max_cats = params
            .get("max_categories")
            .and_then(|v| v.as_u64())
            .unwrap_or(200) as usize;

        let inc_counts = params
            .get("include_counts")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let inc_stats = params
            .get("include_stats")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut meta_refs = Vec::new();
        let mut col_strings: Vec<Vec<Option<String>>> = Vec::new();
        let mut col_numbers: Vec<Vec<Option<f64>>> = Vec::new();

        for name in &selected_vars {
            let idx =
                ds.dictionary
                    .find_index(name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found", name),
                    })?;
            let meta = ds.dictionary.get(idx).ok_or_else(|| {
                CommandError::ExecutionFailed("Corrupt variable dictionary".to_string())
            })?;
            meta_refs.push(meta);

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

            col_strings.push(str_vals);
            col_numbers.push(num_vals);
        }

        let mut raw_data = Vec::with_capacity(meta_refs.len());
        for i in 0..meta_refs.len() {
            raw_data.push(RawColumnData {
                values: &col_strings[i],
                numeric_values: &col_numbers[i],
            });
        }

        let options = CodebookOptions {
            max_categories: max_cats,
            include_counts: inc_counts,
            include_stats: inc_stats,
        };

        let result = compute_codebook(&meta_refs, &raw_data, &options)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
