use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_crosstabs, CrosstabOptions, CrosstabResult, RawCrosstabItem};
use statcraft_core::DataSet;
use statcraft_io::RawValue;

pub struct CrosstabsCommand;

impl EngineCommand for CrosstabsCommand {
    fn id(&self) -> &'static str {
        "stat.crosstabs"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Descriptive Statistics > Crosstabs"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["rows", "cols"],
            "properties": {
                "rows": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of row variable names"
                },
                "cols": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of column variable names"
                },
                "expected": { "type": "boolean", "default": true },
                "row_pct": { "type": "boolean", "default": true },
                "col_pct": { "type": "boolean", "default": true },
                "total_pct": { "type": "boolean", "default": true },
                "residuals": { "type": "boolean", "default": true },
                "chisq": { "type": "boolean", "default": true },
                "phi_cramer": { "type": "boolean", "default": true }
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

        let rows: Vec<String> = params
            .get("rows")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let cols: Vec<String> = params
            .get("cols")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if rows.is_empty() || cols.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Both 'rows' and 'cols' parameters must contain at least one variable"
                    .to_string(),
            });
        }

        let options = CrosstabOptions {
            expected: params
                .get("expected")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            row_pct: params
                .get("row_pct")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            col_pct: params
                .get("col_pct")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            total_pct: params
                .get("total_pct")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            residuals: params
                .get("residuals")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            chisq: params
                .get("chisq")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            phi_cramer: params
                .get("phi_cramer")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        };

        let active_rows = ds.active_row_indices();
        let mut results: Vec<CrosstabResult> = Vec::new();

        for r_name in &rows {
            let r_col_idx =
                ds.dictionary
                    .find_index(r_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Row variable '{}' not found in dataset", r_name),
                    })?;
            let r_meta = ds
                .dictionary
                .get(r_col_idx)
                .ok_or_else(|| CommandError::ExecutionFailed("Row metadata missing".to_string()))?;

            for c_name in &cols {
                let c_col_idx = ds.dictionary.find_index(c_name).ok_or_else(|| {
                    CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Column variable '{}' not found in dataset", c_name),
                    }
                })?;
                let c_meta = ds.dictionary.get(c_col_idx).ok_or_else(|| {
                    CommandError::ExecutionFailed("Column metadata missing".to_string())
                })?;

                let mut items: Vec<RawCrosstabItem> = Vec::with_capacity(active_rows.len());

                for &r in &active_rows {
                    let r_cell = ds.get_cell(r, r_col_idx);
                    let c_cell = ds.get_cell(r, c_col_idx);

                    let (r_val_str, r_is_missing) = match r_cell {
                        Some(RawValue::Number(n)) => {
                            let miss = r_meta.missing.is_missing(&RawValue::Number(*n));
                            let s = if n.fract() == 0.0 {
                                format!("{:.0}", n)
                            } else {
                                n.to_string()
                            };
                            (s, miss)
                        }
                        Some(RawValue::Text(s)) => {
                            let trimmed = s.trim();
                            let miss = trimmed.is_empty()
                                || trimmed == "."
                                || trimmed.eq_ignore_ascii_case("NA")
                                || r_meta
                                    .missing
                                    .is_missing(&RawValue::Text(trimmed.to_string()));
                            (trimmed.to_string(), miss)
                        }
                        Some(RawValue::Null) | None => (String::new(), true),
                    };

                    let (c_val_str, c_is_missing) = match c_cell {
                        Some(RawValue::Number(n)) => {
                            let miss = c_meta.missing.is_missing(&RawValue::Number(*n));
                            let s = if n.fract() == 0.0 {
                                format!("{:.0}", n)
                            } else {
                                n.to_string()
                            };
                            (s, miss)
                        }
                        Some(RawValue::Text(s)) => {
                            let trimmed = s.trim();
                            let miss = trimmed.is_empty()
                                || trimmed == "."
                                || trimmed.eq_ignore_ascii_case("NA")
                                || c_meta
                                    .missing
                                    .is_missing(&RawValue::Text(trimmed.to_string()));
                            (trimmed.to_string(), miss)
                        }
                        Some(RawValue::Null) | None => (String::new(), true),
                    };

                    let r_label = if !r_is_missing {
                        r_meta.values.get_label(&r_val_str).map(|s| s.to_string())
                    } else {
                        None
                    };

                    let c_label = if !c_is_missing {
                        c_meta.values.get_label(&c_val_str).map(|s| s.to_string())
                    } else {
                        None
                    };

                    items.push(RawCrosstabItem {
                        row_value: r_val_str,
                        row_label: r_label,
                        col_value: c_val_str,
                        col_label: c_label,
                        is_missing: r_is_missing || c_is_missing,
                    });
                }

                let res = compute_crosstabs(r_name, c_name, &items, &options)
                    .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
                results.push(res);
            }
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

impl CrosstabsCommand {
    pub fn to_sps(rows: &[String], cols: &[String], options: &CrosstabOptions) -> String {
        let mut stats = Vec::new();
        if options.chisq {
            stats.push("CHISQ");
        }
        if options.phi_cramer {
            stats.push("PHI");
        }
        let stat_clause = if !stats.is_empty() {
            format!("\n  /STATISTICS={}", stats.join(" "))
        } else {
            String::new()
        };

        let mut cells = vec!["COUNT"];
        if options.expected {
            cells.push("EXPECTED");
        }
        if options.row_pct {
            cells.push("ROW");
        }
        if options.col_pct {
            cells.push("COLUMN");
        }
        if options.total_pct {
            cells.push("TOTAL");
        }
        if options.residuals {
            cells.push("RESID");
        }

        format!(
            "CROSSTABS\n  /TABLES={} BY {}{}\n  /CELLS={}.",
            rows.join(" "),
            cols.join(" "),
            stat_clause,
            cells.join(" ")
        )
    }

    pub fn to_sps_syntax(
        &self,
        rows: &[String],
        cols: &[String],
        options: &CrosstabOptions,
    ) -> String {
        Self::to_sps(rows, cols, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::{DataType, Measure};

    #[test]
    fn test_crosstabs_sps_syntax() {
        let opts = CrosstabOptions::default();
        let sps = CrosstabsCommand::to_sps(&["gender".to_string()], &["jobcat".to_string()], &opts);
        assert!(sps.contains("CROSSTABS\n  /TABLES=gender BY jobcat"));
        assert!(sps.contains("/STATISTICS=CHISQ PHI"));
        assert!(sps.contains("/CELLS=COUNT EXPECTED ROW COLUMN TOTAL RESID."));
    }

    #[test]
    fn test_crosstabs_command_execution() {
        let mut ds = DataSet::empty();
        let r_col = ds
            .add_variable(
                "gender",
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )
            .unwrap();
        let c_col = ds
            .add_variable(
                "jobcat",
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )
            .unwrap();

        for i in 0..20 {
            ds.add_empty_row();
            let row = i;
            let g = if i < 10 { 1.0 } else { 2.0 };
            let j = if i % 2 == 0 { 1.0 } else { 2.0 };
            ds.set_cell(row, r_col, RawValue::Number(g)).unwrap();
            ds.set_cell(row, c_col, RawValue::Number(j)).unwrap();
        }

        let mut opt_ds = Some(ds);
        let cmd = CrosstabsCommand;
        let params = json!({
            "rows": ["gender"],
            "cols": ["jobcat"],
            "chisq": true
        });

        let res = cmd
            .execute(&mut opt_ds, params)
            .expect("Command must succeed");
        let arr = res.as_array().expect("Result must be array");
        assert_eq!(arr.len(), 1);
        let first = &arr[0];
        assert_eq!(first["row_var"], "gender");
        assert_eq!(first["col_var"], "jobcat");
        assert_eq!(first["grand_total"], 20);
    }
}
