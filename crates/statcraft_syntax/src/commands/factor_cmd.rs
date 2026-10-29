use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_factor_analysis, FactorExtractionCriteria, FactorExtractionMethod, FactorOptions,
};
use statcraft_core::DataSet;

pub struct FactorCommand;

impl EngineCommand for FactorCommand {
    fn id(&self) -> &'static str {
        "stat.factor"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Dimension Reduction > Factor"
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
                "extraction": {
                    "type": "string",
                    "enum": ["pc"],
                    "default": "pc",
                    "description": "Extraction method: pc (Principal Components)"
                },
                "mineigen": {
                    "type": "number",
                    "default": 1.0,
                    "description": "Eigenvalue cutoff for factor extraction (Kaiser criterion)"
                },
                "nfactors": {
                    "type": "integer",
                    "description": "Fixed number of factors to extract (optional)"
                },
                "kmo_bartlett": {
                    "type": "boolean",
                    "default": true,
                    "description": "Compute KMO and Bartlett's test of sphericity"
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
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Dataset is empty or not loaded".to_string(),
            })?;

        let var_names = params
            .get("variables")
            .and_then(|v| v.as_array())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'variables' must be an array of variable names".to_string(),
            })?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<String>>();

        if var_names.len() < 2 {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Factor analysis requires at least 2 variables".to_string(),
            });
        }

        let kmo_bartlett = params
            .get("kmo_bartlett")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let criteria = if let Some(n_fact) = params.get("nfactors").and_then(|v| v.as_u64()) {
            FactorExtractionCriteria::Factors(n_fact as usize)
        } else {
            let mineigen = params
                .get("mineigen")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);
            FactorExtractionCriteria::Mineigen(mineigen)
        };

        let options = FactorOptions {
            extraction: FactorExtractionMethod::PC,
            criteria,
            kmo_bartlett,
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

        let result = compute_factor_analysis(&items, &weights, &options)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl FactorCommand {
    pub fn to_sps(variables: &[String], options: &FactorOptions) -> String {
        let vars_str = variables.join(" ");
        let crit_str = match options.criteria {
            FactorExtractionCriteria::Mineigen(val) => format!("MINEIGEN({})", val),
            FactorExtractionCriteria::Factors(k) => format!("FACTORS({})", k),
        };
        let print_kmo = if options.kmo_bartlett { " KMO" } else { "" };

        format!(
            "FACTOR\n  /VARIABLES {}\n  /MISSING LISTWISE\n  /ANALYSIS {}\n  /PRINT INITIAL EXTRACTION{}\n  /CRITERIA {}\n  /EXTRACTION PC\n  /ROTATION NOROTATE.",
            vars_str, vars_str, print_kmo, crit_str
        )
    }

    pub fn from_sps(sps: &str) -> Option<(Vec<String>, FactorOptions)> {
        let mut variables = Vec::new();
        let mut criteria = FactorExtractionCriteria::Mineigen(1.0);
        let mut kmo_bartlett = false;

        let upper = sps.to_uppercase();
        if !upper.contains("FACTOR") {
            return None;
        }

        for line in sps.lines() {
            let trimmed = line.trim();
            if trimmed.to_uppercase().starts_with("/VARIABLES") {
                let parts: Vec<&str> = trimmed[10..]
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .collect();
                for p in parts {
                    let cleaned = p.trim_matches('.').to_string();
                    if !cleaned.is_empty() {
                        variables.push(cleaned);
                    }
                }
            } else if trimmed.to_uppercase().starts_with("/PRINT") {
                if trimmed.to_uppercase().contains("KMO") {
                    kmo_bartlett = true;
                }
            } else if trimmed.to_uppercase().starts_with("/CRITERIA") {
                let rest = trimmed[9..].to_uppercase();
                if let Some(start) = rest.find("MINEIGEN(") {
                    if let Some(end) = rest[start..].find(')') {
                        if let Ok(v) = rest[start + 9..start + end].trim().parse::<f64>() {
                            criteria = FactorExtractionCriteria::Mineigen(v);
                        }
                    }
                } else if let Some(start) = rest.find("FACTORS(") {
                    if let Some(end) = rest[start..].find(')') {
                        if let Ok(k) = rest[start + 8..start + end].trim().parse::<usize>() {
                            criteria = FactorExtractionCriteria::Factors(k);
                        }
                    }
                }
            }
        }

        if variables.len() >= 2 {
            Some((
                variables,
                FactorOptions {
                    extraction: FactorExtractionMethod::PC,
                    criteria,
                    kmo_bartlett,
                },
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_algo::FactorAnalysisResult;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn create_test_dataset() -> DataSet {
        let csv_data = "v1,v2,v3\n1,2,5\n2,3,4\n3,4,3\n4,5,2\n5,6,1\n";
        let table = load_csv_from_reader(csv_data.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_factor_command_execution() {
        let ds = create_test_dataset();
        let cmd = FactorCommand;
        assert!(cmd.enabled(Some(&ds)));

        let params = json!({
            "variables": ["v1", "v2", "v3"],
            "mineigen": 1.0,
            "kmo_bartlett": true
        });

        let mut opt_ds = Some(ds);
        let res_val = cmd.execute(&mut opt_ds, params).unwrap();
        let parsed: FactorAnalysisResult = serde_json::from_value(res_val).unwrap();

        assert_eq!(parsed.n_cases, 5);
        assert_eq!(parsed.variables.len(), 3);
        assert!(parsed.n_components_extracted >= 1);
        assert!(parsed.kmo_bartlett.is_some());
    }

    #[test]
    fn test_factor_sps_syntax() {
        let vars = vec!["v1".to_string(), "v2".to_string(), "v3".to_string()];
        let options = FactorOptions {
            extraction: FactorExtractionMethod::PC,
            criteria: FactorExtractionCriteria::Mineigen(1.0),
            kmo_bartlett: true,
        };

        let sps = FactorCommand::to_sps(&vars, &options);
        assert!(sps.contains("/VARIABLES v1 v2 v3"));
        assert!(sps.contains("/PRINT INITIAL EXTRACTION KMO"));
        assert!(sps.contains("/CRITERIA MINEIGEN(1)"));
        assert!(sps.contains("/EXTRACTION PC"));

        let (parsed_vars, parsed_opts) = FactorCommand::from_sps(&sps).unwrap();
        assert_eq!(parsed_vars, vars);
        assert!(parsed_opts.kmo_bartlett);
        assert_eq!(
            parsed_opts.criteria,
            FactorExtractionCriteria::Mineigen(1.0)
        );
    }
}
