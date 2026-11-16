use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_cox_regression, CoxDataPoint, CoxRegressionOptions, TieMethod};
use statcraft_core::{DataSet, RawValue};

pub struct CoxRegressionCommand;

impl CoxRegressionCommand {
    pub fn to_sps(
        time_var: &str,
        status_var: &str,
        event_val: f64,
        covariates: &[String],
        strata_var: Option<&str>,
        tie_method: TieMethod,
    ) -> String {
        let cov_str = covariates.join(" ");
        let ties_str = match tie_method {
            TieMethod::Breslow => "BRESLOW",
            TieMethod::Efron => "EFRON",
        };

        let mut sps = format!("COXREG {}", time_var);
        if let Some(strata) = strata_var {
            sps.push_str(&format!(" /STRATA = {}", strata));
        }
        sps.push_str(&format!(
            "\n  /STATUS = {}({})\n  /PATTERN = BY {}\n  /METHOD = ENTER {}\n  /CRITERIA = ITERATE(20) TIES({})\n  /PRINT = CI(95) BASELINE\n  /PLOT = SURVIVAL.\nEXECUTE.",
            status_var, event_val, cov_str, cov_str, ties_str
        ));
        sps
    }
}

impl EngineCommand for CoxRegressionCommand {
    fn id(&self) -> &'static str {
        "stat.survival.cox"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze -> Survival -> Cox Regression..."
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn category(&self) -> &'static str {
        "Survival"
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["COXREG", "COX REGRESSION"]
    }

    fn specs(&self) -> Value {
        json!({
            "command": "stat.survival.cox",
            "required": ["time_var", "status_var", "covariates"],
            "properties": {
                "time_var": { "type": "string" },
                "status_var": { "type": "string" },
                "event_values": { "type": "array", "items": { "type": "number" }, "default": [1.0] },
                "covariates": { "type": "array", "items": { "type": "string" } },
                "strata_var": { "type": "string" },
                "tie_method": { "type": "string", "enum": ["Breslow", "Efron"], "default": "Breslow" },
                "confidence_level": { "type": "number", "default": 0.95 }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|ds| ds.col_count() >= 3).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let time_var = params
            .get("time_var")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'time_var'".to_string(),
            })?;

        let status_var = params
            .get("status_var")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing 'status_var'".to_string(),
            })?;

        let covariates: Vec<String> = params
            .get("covariates")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if covariates.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "At least one covariate must be specified".to_string(),
            });
        }

        let strata_var = params.get("strata_var").and_then(|v| v.as_str());

        let event_values = params
            .get("event_values")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_f64()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec![1.0]);

        let tie_method = match params
            .get("tie_method")
            .and_then(|v| v.as_str())
            .unwrap_or("Breslow")
        {
            "Efron" | "EFRON" => TieMethod::Efron,
            _ => TieMethod::Breslow,
        };

        let conf_level = params
            .get("confidence_level")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.95);

        // Find variable indices
        let time_idx =
            ds.dictionary
                .find_index(time_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Time variable '{}' not found", time_var),
                })?;

        let status_idx =
            ds.dictionary
                .find_index(status_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Status variable '{}' not found", status_var),
                })?;

        let strata_idx =
            if let Some(st_var) = strata_var {
                Some(ds.dictionary.find_index(st_var).ok_or_else(|| {
                    CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Strata variable '{}' not found", st_var),
                    }
                })?)
            } else {
                None
            };

        let mut cov_indices = Vec::with_capacity(covariates.len());
        for c_var in &covariates {
            let idx =
                ds.dictionary
                    .find_index(c_var)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Covariate '{}' not found", c_var),
                    })?;
            cov_indices.push(idx);
        }

        // Collect rows
        let mut data_points = Vec::new();
        let row_count = ds.row_count();

        for row in 0..row_count {
            let t_val = match ds.table.columns[time_idx].values.get(row) {
                Some(RawValue::Number(n)) if !n.is_nan() && *n >= 0.0 => *n,
                _ => continue,
            };

            let s_val = match ds.table.columns[status_idx].values.get(row) {
                Some(RawValue::Number(n)) if !n.is_nan() => *n,
                _ => continue,
            };

            let is_event = event_values.iter().any(|&ev| (s_val - ev).abs() < 1e-6);

            let stratum = if let Some(s_idx) = strata_idx {
                match ds.table.columns[s_idx].values.get(row) {
                    Some(RawValue::Text(s)) => Some(s.clone()),
                    Some(RawValue::Number(n)) if !n.is_nan() => Some(n.to_string()),
                    _ => continue,
                }
            } else {
                None
            };

            let mut row_covs = Vec::with_capacity(cov_indices.len());
            let mut valid_covs = true;
            for &c_idx in &cov_indices {
                match ds.table.columns[c_idx].values.get(row) {
                    Some(RawValue::Number(n)) if !n.is_nan() => row_covs.push(*n),
                    _ => {
                        valid_covs = false;
                        break;
                    }
                }
            }

            if !valid_covs {
                continue;
            }

            data_points.push(CoxDataPoint {
                time: t_val,
                is_event,
                stratum,
                covariates: row_covs,
            });
        }

        let options = CoxRegressionOptions {
            confidence_level: conf_level,
            tie_method,
            ..Default::default()
        };

        let result = compute_cox_regression(
            &data_points,
            time_var,
            status_var,
            strata_var,
            &covariates,
            &options,
        )
        .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
