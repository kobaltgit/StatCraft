use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_date_operation, DatePart, DateTimeOperation, DateUnit};
use statcraft_core::{DataSet, DataType, Measure, RawValue, VariableMetadata};
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum DateTimeUndoEntry {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

pub struct DateTimeWizardCommand {
    history: Mutex<Vec<DateTimeUndoEntry>>,
}

impl Default for DateTimeWizardCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl DateTimeWizardCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_sps(
        operation_str: &str,
        target_var: &str,
        primary_var: &str,
        secondary_var: Option<&str>,
        part_str: Option<&str>,
        unit_str: Option<&str>,
        duration_value: Option<f64>,
        target_label: Option<&str>,
    ) -> String {
        let mut sps = match operation_str.to_ascii_lowercase().as_str() {
            "extract" => {
                let part_name = match part_str.unwrap_or("year").to_ascii_lowercase().as_str() {
                    "year" => "YEAR",
                    "month" => "MONTH",
                    "day" => "MDAY",
                    "hour" => "HOUR",
                    "minute" => "MINUTE",
                    "second" => "SECOND",
                    "day_of_week" | "dayofweek" => "WKDAY",
                    "quarter" => "QUARTER",
                    _ => "YEAR",
                };
                format!(
                    "COMPUTE {} = XDATE.{}({}).\n",
                    target_var, part_name, primary_var
                )
            }
            "diff" => {
                let sec = secondary_var.unwrap_or("");
                let unit = unit_str.unwrap_or("days").to_ascii_lowercase();
                format!(
                    "COMPUTE {} = DATEDIFF({}, {}, \"{}\").\n",
                    target_var, primary_var, sec, unit
                )
            }
            "add" => {
                let amt = duration_value.unwrap_or(0.0);
                let unit = unit_str.unwrap_or("days").to_ascii_lowercase();
                format!(
                    "COMPUTE {} = DATESUM({}, {}, \"{}\").\n",
                    target_var, primary_var, amt, unit
                )
            }
            _ => {
                format!("COMPUTE {} = {}.\n", target_var, primary_var)
            }
        };

        if let Some(lbl) = target_label {
            let trimmed = lbl.trim();
            if !trimmed.is_empty() {
                sps.push_str(&format!(
                    "VARIABLE LABELS {} \"{}\".\n",
                    target_var, trimmed
                ));
            }
        }
        sps.push_str("EXECUTE.");
        sps
    }
}

impl EngineCommand for DateTimeWizardCommand {
    fn id(&self) -> &'static str {
        "transform.date_time_wizard"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Date and Time Wizard"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["operation", "target_var", "primary_var"],
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["extract", "diff", "add", "construct"],
                    "description": "Date/Time operation"
                },
                "target_var": {
                    "type": "string",
                    "description": "Result variable name"
                },
                "target_label": {
                    "type": "string",
                    "description": "Optional label for result variable"
                },
                "primary_var": {
                    "type": "string",
                    "description": "First date/time variable"
                },
                "secondary_var": {
                    "type": "string",
                    "description": "Second date variable (for diff)"
                },
                "part": {
                    "type": "string",
                    "enum": ["year", "month", "day", "hour", "minute", "second", "day_of_week", "quarter"],
                    "description": "Part to extract"
                },
                "unit": {
                    "type": "string",
                    "enum": ["years", "months", "weeks", "days", "hours", "minutes", "seconds"],
                    "description": "Time unit for difference or duration"
                },
                "duration_value": {
                    "type": "number",
                    "description": "Duration amount to add or subtract"
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset.map(|d| d.col_count() > 0).unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let op_str = params
            .get("operation")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_ascii_lowercase())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Operation must be specified ('extract', 'diff', 'add', 'construct')"
                    .to_string(),
            })?;

        let target_var = params
            .get("target_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Target variable name cannot be empty".to_string(),
            })?;

        let primary_var = params
            .get("primary_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Primary date variable must be specified".to_string(),
            })?;

        let prim_idx =
            ds.dictionary
                .find_index(primary_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Variable '{}' not found", primary_var),
                })?;

        let prim_values = ds
            .table
            .columns
            .get(prim_idx)
            .map(|c| c.values.as_slice())
            .ok_or_else(|| {
                CommandError::ExecutionFailed("Primary column values missing".to_string())
            })?;

        let sec_var_opt = params
            .get("secondary_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        let sec_values_holder: Option<Vec<RawValue>> = if let Some(sec_var) = sec_var_opt {
            let sec_idx =
                ds.dictionary
                    .find_index(sec_var)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Secondary variable '{}' not found", sec_var),
                    })?;
            let col = ds.table.columns.get(sec_idx).ok_or_else(|| {
                CommandError::ExecutionFailed("Secondary column values missing".to_string())
            })?;
            Some(col.values.clone())
        } else {
            None
        };

        let parse_unit =
            |u_str: Option<&str>| match u_str.unwrap_or("days").to_ascii_lowercase().as_str() {
                "years" => DateUnit::Years,
                "months" => DateUnit::Months,
                "weeks" => DateUnit::Weeks,
                "hours" => DateUnit::Hours,
                "minutes" => DateUnit::Minutes,
                "seconds" => DateUnit::Seconds,
                _ => DateUnit::Days,
            };

        let (date_op, out_dtype) = match op_str.as_str() {
            "extract" => {
                let part_name = params
                    .get("part")
                    .and_then(|v| v.as_str())
                    .unwrap_or("year");
                let date_part = match part_name.to_ascii_lowercase().as_str() {
                    "month" => DatePart::Month,
                    "day" => DatePart::Day,
                    "hour" => DatePart::Hour,
                    "minute" => DatePart::Minute,
                    "second" => DatePart::Second,
                    "day_of_week" | "dayofweek" => DatePart::DayOfWeek,
                    "quarter" => DatePart::Quarter,
                    _ => DatePart::Year,
                };
                (
                    DateTimeOperation::ExtractPart(date_part),
                    DataType::Numeric { decimals: 0 },
                )
            }
            "diff" => {
                let unit = parse_unit(params.get("unit").and_then(|v| v.as_str()));
                let decimals = match unit {
                    DateUnit::Weeks | DateUnit::Hours | DateUnit::Minutes => 2,
                    _ => 0,
                };
                (
                    DateTimeOperation::Difference { unit },
                    DataType::Numeric { decimals },
                )
            }
            "add" => {
                let unit = parse_unit(params.get("unit").and_then(|v| v.as_str()));
                let amount = params
                    .get("duration_value")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                (
                    DateTimeOperation::AddDuration { amount, unit },
                    DataType::Date {
                        format: "YYYY-MM-DD".to_string(),
                    },
                )
            }
            "construct" => (
                DateTimeOperation::ConstructDate,
                DataType::Date {
                    format: "YYYY-MM-DD".to_string(),
                },
            ),
            other => {
                return Err(CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Unknown date/time operation: '{}'", other),
                });
            }
        };

        let result_values =
            compute_date_operation(date_op, prim_values, sec_values_holder.as_deref())
                .map_err(CommandError::ExecutionFailed)?;

        let target_label = params
            .get("target_label")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let existing_idx = ds.dictionary.find_index(target_var);
        let undo_entry = match existing_idx {
            Some(idx) => {
                let old_meta = ds.dictionary.get(idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Dictionary slot missing".to_string())
                })?;
                let old_values = ds
                    .table
                    .columns
                    .get(idx)
                    .map(|c| c.values.clone())
                    .ok_or_else(|| {
                        CommandError::ExecutionFailed("Column values missing".to_string())
                    })?;

                if let Some(col) = ds.table.columns.get_mut(idx) {
                    col.values = result_values;
                }
                if let Some(meta) = ds.dictionary.get_mut(idx) {
                    meta.var_type = out_dtype;
                    meta.measure = Measure::Scale;
                    if let Some(lbl) = target_label {
                        meta.label = lbl;
                    }
                }

                DateTimeUndoEntry::Overwritten {
                    col_idx: idx,
                    old_meta,
                    old_values,
                }
            }
            None => {
                let col_idx = ds.add_variable(target_var, out_dtype, Measure::Scale)?;
                if let Some(lbl) = target_label {
                    if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                        meta.label = lbl;
                    }
                }
                if let Some(col) = ds.table.columns.get_mut(col_idx) {
                    col.values = result_values;
                }

                DateTimeUndoEntry::Created { col_idx }
            }
        };

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_entry);
        }

        Ok(json!({
            "status": "success",
            "target_var": target_var,
            "operation": op_str
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let entry = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        if let Some(entry) = entry {
            match entry {
                DateTimeUndoEntry::Created { col_idx } => {
                    ds.remove_variable(col_idx)?;
                }
                DateTimeUndoEntry::Overwritten {
                    col_idx,
                    old_meta,
                    old_values,
                } => {
                    if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                        *meta = old_meta;
                    }
                    if let Some(col) = ds.table.columns.get_mut(col_idx) {
                        col.values = old_values;
                    }
                }
            }
        }

        Ok(())
    }
}
