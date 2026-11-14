use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_life_table, LifeTableCase, LifeTableOptions};
use statcraft_core::{DataSet, RawValue};

pub struct LifeTablesCommand;

impl LifeTablesCommand {
    pub fn to_sps(
        time_var: &str,
        status_var: &str,
        event_val: f64,
        end: f64,
        width: f64,
    ) -> String {
        format!(
            "SURVIVAL\n  /TABLE = {}\n  /INTERVAL = THRU {} BY {}\n  /STATUS = {}({}).\nEXECUTE.",
            time_var, end, width, status_var, event_val
        )
    }
}

impl EngineCommand for LifeTablesCommand {
    fn id(&self) -> &'static str {
        "stat.survival.lifetables"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze -> Survival -> Life Tables..."
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn category(&self) -> &'static str {
        "Survival"
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["SURVIVAL", "LIFETABLES"]
    }

    fn specs(&self) -> Value {
        json!({
            "command": "stat.survival.lifetables",
            "required": ["time_var", "status_var"],
            "properties": {
                "time_var": { "type": "string" },
                "status_var": { "type": "string" },
                "event_values": { "type": "array", "items": { "type": "number" }, "default": [1.0] },
                "interval_start": { "type": "number", "default": 0.0 },
                "interval_end": { "type": "number", "default": 60.0 },
                "interval_width": { "type": "number", "default": 10.0 }
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

        let event_values = params
            .get("event_values")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_f64()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec![1.0]);

        let start = params
            .get("interval_start")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let end = params
            .get("interval_end")
            .and_then(|v| v.as_f64())
            .unwrap_or(60.0);

        let width = params
            .get("interval_width")
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0);

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

        let mut cases = Vec::new();
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

            cases.push(LifeTableCase {
                time: t_val,
                is_event,
            });
        }

        let options = LifeTableOptions {
            interval_start: start,
            interval_end: end,
            interval_width: width,
        };

        let result = compute_life_table(&cases, time_var, status_var, &options)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}
