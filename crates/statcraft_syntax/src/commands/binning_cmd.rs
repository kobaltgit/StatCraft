use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_visual_binning_with_labels, format_bound, Cutpoint};
use statcraft_core::{DataSet, DataType, Measure, VariableMetadata};
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum BinningUndoItem {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

pub struct VisualBinningCommand {
    history: Mutex<Vec<BinningUndoItem>>,
}

impl Default for VisualBinningCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualBinningCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(
        source_var: &str,
        target_var: &str,
        target_label: Option<&str>,
        cutpoints: &[Cutpoint],
        labels: &[String],
    ) -> String {
        let mut sorted = cutpoints.to_vec();
        sorted.sort_by(|a, b| {
            a.upper_bound
                .partial_cmp(&b.upper_bound)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut recode_parts = Vec::new();
        recode_parts.push("(MISSING=COPY)".to_string());

        let k = sorted.len();
        if k == 0 {
            recode_parts.push("(ELSE = 1)".to_string());
        } else {
            for (idx, cp) in sorted.iter().enumerate() {
                let bin = idx + 1;
                let bound_str = format_bound(cp.upper_bound);
                if cp.inclusive {
                    recode_parts.push(format!("(LO THRU {} = {})", bound_str, bin));
                } else {
                    recode_parts.push(format!("(LO THRU < {} = {})", bound_str, bin));
                }
            }
            recode_parts.push(format!("(ELSE = {})", k + 1));
        }

        let mut sps = format!(
            "RECODE {} {}\n  INTO {}.\n",
            source_var,
            recode_parts.join(" "),
            target_var
        );

        if let Some(lbl) = target_label {
            let trimmed = lbl.trim();
            if !trimmed.is_empty() {
                let escaped_lbl = trimmed.replace('\'', "''");
                sps.push_str(&format!(
                    "VARIABLE LABELS {} '{}'.\n",
                    target_var, escaped_lbl
                ));
            }
        }

        if !labels.is_empty() {
            let mut label_parts = Vec::new();
            for (idx, lbl) in labels.iter().enumerate() {
                let bin = idx + 1;
                let escaped = lbl.trim().replace('\'', "''");
                label_parts.push(format!("{} '{}'", bin, escaped));
            }
            sps.push_str(&format!(
                "VALUE LABELS {} {}.\n",
                target_var,
                label_parts.join(" ")
            ));
        }

        sps.push_str("EXECUTE.");
        sps
    }
}

impl EngineCommand for VisualBinningCommand {
    fn id(&self) -> &'static str {
        "transform.visual_binning"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Visual Binning"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["source_var", "target_var", "cutpoints"],
            "properties": {
                "source_var": { "type": "string" },
                "target_var": { "type": "string" },
                "target_label": { "type": "string" },
                "cutpoints": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["upper_bound"],
                        "properties": {
                            "upper_bound": { "type": "number" },
                            "inclusive": { "type": "boolean", "default": true }
                        }
                    }
                },
                "labels": {
                    "type": "array",
                    "items": { "type": "string" }
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

        let source_var = params
            .get("source_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing or invalid 'source_var'".to_string(),
            })?;

        let target_var = params
            .get("target_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Missing or invalid 'target_var'".to_string(),
            })?;

        let target_label = params
            .get("target_label")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string());

        let mut cutpoints = Vec::new();
        if let Some(cp_array) = params.get("cutpoints").and_then(|v| v.as_array()) {
            for item in cp_array {
                if let Some(bound) = item.get("upper_bound").and_then(|v| v.as_f64()) {
                    let inclusive = item
                        .get("inclusive")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    cutpoints.push(Cutpoint::new(bound, inclusive));
                }
            }
        }

        let custom_labels: Option<Vec<String>> =
            params.get("labels").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .collect()
            });

        let src_idx =
            ds.dictionary
                .find_index(&source_var)
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: format!("Source variable '{}' not found in dataset", source_var),
                })?;

        let input_vals = ds
            .table
            .columns
            .get(src_idx)
            .map(|c| c.values.clone())
            .unwrap_or_default();

        let result =
            compute_visual_binning_with_labels(&input_vals, &cutpoints, custom_labels.as_deref());

        let existing_idx = ds.dictionary.find_index(&target_var);
        let undo_item = if let Some(target_idx) = existing_idx {
            let old_meta = ds.dictionary.get(target_idx).cloned().ok_or_else(|| {
                CommandError::ExecutionFailed("Dictionary metadata missing".to_string())
            })?;
            let old_values = ds.table.columns[target_idx].values.clone();

            ds.table.columns[target_idx].values = result.binned_values;
            if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                meta.var_type = DataType::Numeric { decimals: 0 };
                meta.measure = Measure::Ordinal;
                meta.values = result.value_labels;
                if let Some(ref lbl) = target_label {
                    if !lbl.is_empty() {
                        meta.label = lbl.clone();
                    }
                }
            }

            BinningUndoItem::Overwritten {
                col_idx: target_idx,
                old_meta,
                old_values,
            }
        } else {
            let target_idx = ds.add_variable(
                &target_var,
                DataType::Numeric { decimals: 0 },
                Measure::Ordinal,
            )?;
            if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                meta.values = result.value_labels;
                if let Some(ref lbl) = target_label {
                    if !lbl.is_empty() {
                        meta.label = lbl.clone();
                    }
                }
            }
            ds.table.columns[target_idx].values = result.binned_values;

            BinningUndoItem::Created {
                col_idx: target_idx,
            }
        };

        if let Ok(mut hist) = self.history.lock() {
            hist.push(undo_item);
        }

        Ok(json!({
            "status": "success",
            "target": target_var,
            "rows": ds.row_count()
        }))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let item = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        if let Some(undo_item) = item {
            match undo_item {
                BinningUndoItem::Created { col_idx } => {
                    ds.remove_variable(col_idx)?;
                }
                BinningUndoItem::Overwritten {
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

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::RawTable;

    fn make_test_dataset() -> DataSet {
        let mut ds = DataSet::empty();
        let idx = ds
            .add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
        ds.table = RawTable {
            columns: vec![statcraft_io::RawColumn {
                name: "salary".to_string(),
                data_type: statcraft_io::RawDataType::Numeric,
                values: vec![
                    RawValue::Number(18000.0),
                    RawValue::Number(25000.0),
                    RawValue::Number(45000.0),
                    RawValue::Number(70000.0),
                    RawValue::Null,
                ],
            }],
            row_count: 5,
        };
        let _ = idx;
        ds
    }

    #[test]
    fn test_visual_binning_command_execution_and_undo() {
        let mut ds = Some(make_test_dataset());
        let cmd = VisualBinningCommand::new();

        let params = json!({
            "source_var": "salary",
            "target_var": "salary_binned",
            "target_label": "Binned Salary",
            "cutpoints": [
                { "upper_bound": 25000.0, "inclusive": true },
                { "upper_bound": 50000.0, "inclusive": true }
            ]
        });

        let res = cmd.execute(&mut ds, params);
        assert!(res.is_ok());

        let d = ds.as_ref().unwrap();
        assert_eq!(d.col_count(), 2);
        let tgt_meta = d.dictionary.find_by_name("salary_binned").unwrap();
        assert_eq!(tgt_meta.label, "Binned Salary");
        assert_eq!(tgt_meta.measure, Measure::Ordinal);
        assert_eq!(tgt_meta.values.get_label("1"), Some("<= 25000"));
        assert_eq!(tgt_meta.values.get_label("2"), Some("25000 - 50000"));
        assert_eq!(tgt_meta.values.get_label("3"), Some("> 50000"));

        let vals = &d.table.columns[1].values;
        assert_eq!(
            vals,
            &vec![
                RawValue::Number(1.0),
                RawValue::Number(1.0),
                RawValue::Number(2.0),
                RawValue::Number(3.0),
                RawValue::Null,
            ]
        );

        // Undo
        let undo_res = cmd.undo(&mut ds);
        assert!(undo_res.is_ok());
        let d_after = ds.as_ref().unwrap();
        assert_eq!(d_after.col_count(), 1);
        assert!(d_after.dictionary.find_by_name("salary_binned").is_none());
    }

    #[test]
    fn test_visual_binning_sps_syntax() {
        let cutpoints = vec![Cutpoint::new(25000.0, true), Cutpoint::new(50000.0, true)];
        let labels = vec![
            "<= 25000".to_string(),
            "25001 - 50000".to_string(),
            "> 50000".to_string(),
        ];

        let sps = VisualBinningCommand::to_sps(
            "salary",
            "salary_binned",
            Some("Salary (Binned)"),
            &cutpoints,
            &labels,
        );

        assert!(sps.contains("RECODE salary (MISSING=COPY) (LO THRU 25000 = 1) (LO THRU 50000 = 2) (ELSE = 3)\n  INTO salary_binned."));
        assert!(sps.contains("VARIABLE LABELS salary_binned 'Salary (Binned)'."));
        assert!(
            sps.contains("VALUE LABELS salary_binned 1 '<= 25000' 2 '25001 - 50000' 3 '> 50000'.")
        );
        assert!(sps.contains("EXECUTE."));
    }
}
