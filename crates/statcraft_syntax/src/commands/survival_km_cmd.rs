use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_kaplan_meier, KMDataPoint, KMOptions};
use statcraft_core::{DataSet, RawValue};

pub struct KaplanMeierCommand;

impl KaplanMeierCommand {
    pub fn to_sps(
        time_var: &str,
        status_var: &str,
        event_val: f64,
        factor_var: Option<&str>,
    ) -> String {
        let mut sps = format!("KM {}", time_var);
        if let Some(factor) = factor_var {
            sps.push_str(&format!(" BY {}", factor));
        }
        sps.push_str(&format!(
            "\n  /STATUS = {}({})\n  /PRINT = TABLE MEAN\n  /TEST = LOGRANK\n  /COMPARE = OVERALL.\nEXECUTE.",
            status_var, event_val
        ));
        sps
    }
}

impl EngineCommand for KaplanMeierCommand {
    fn id(&self) -> &'static str {
        "stat.survival.km"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze -> Survival -> Kaplan-Meier..."
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn category(&self) -> &'static str {
        "Survival"
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["KM", "KAPLAN-MEIER"]
    }

    fn specs(&self) -> Value {
        json!({
            "command": "stat.survival.km",
            "required": ["time_var", "status_var"],
            "properties": {
                "time_var": { "type": "string" },
                "status_var": { "type": "string" },
                "event_values": { "type": "array", "items": { "type": "number" }, "default": [1.0] },
                "factor_var": { "type": "string" }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|ds| ds.col_count() >= 2).unwrap_or(false)
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

        let factor_var = params.get("factor_var").and_then(|v| v.as_str());

        let event_values = params
            .get("event_values")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_f64()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec![1.0]);

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

        let factor_idx =
            if let Some(f_name) = factor_var {
                Some(ds.dictionary.find_index(f_name).ok_or_else(|| {
                    CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Factor variable '{}' not found", f_name),
                    }
                })?)
            } else {
                None
            };

        let mut data_points = Vec::new();
        let n_rows = ds.row_count();

        for r in 0..n_rows {
            let t_val = match ds.table.columns[time_idx].values.get(r) {
                Some(RawValue::Number(num)) if !num.is_nan() && *num >= 0.0 => *num,
                _ => continue,
            };

            let is_event = match ds.table.columns[status_idx].values.get(r) {
                Some(RawValue::Number(num)) => {
                    event_values.iter().any(|&ev| (ev - num).abs() < 1e-6)
                }
                _ => false,
            };

            let grp_str = if let Some(f_idx) = factor_idx {
                match ds.table.columns[f_idx].values.get(r) {
                    Some(RawValue::Number(num)) => format!("{}", num),
                    Some(RawValue::Text(s)) => s.clone(),
                    _ => "(Missing)".to_string(),
                }
            } else {
                "Overall".to_string()
            };

            data_points.push(KMDataPoint {
                time: t_val,
                is_event,
                group: grp_str,
            });
        }

        let result = compute_kaplan_meier(
            &data_points,
            time_var,
            status_var,
            factor_var,
            &KMOptions::default(),
        )
        .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
