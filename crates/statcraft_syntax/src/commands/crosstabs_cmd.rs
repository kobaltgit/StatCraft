use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_cmh, compute_crosstabs, CrosstabLayerKey, CrosstabOptions, CrosstabResult,
    RawCrosstabItem,
};
use statcraft_core::DataSet;
use statcraft_io::RawValue;
use std::collections::BTreeSet;

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
        let mut props = serde_json::Map::new();
        props.insert(
            "rows".into(),
            json!({ "type": "array", "items": { "type": "string" } }),
        );
        props.insert(
            "cols".into(),
            json!({ "type": "array", "items": { "type": "string" } }),
        );
        props.insert(
            "layers".into(),
            json!({ "type": "array", "items": { "type": "array", "items": { "type": "string" } } }),
        );
        for (k, def) in [
            ("observed", true),
            ("expected", true),
            ("hide_small_counts", false),
            ("compare_col_proportions", false),
            ("bonferroni_adjust", false),
            ("row_pct", true),
            ("col_pct", true),
            ("total_pct", true),
            ("residuals", true),
            ("resid_unstandardized", false),
            ("resid_standardized", false),
            ("resid_adjusted", false),
            ("apa_style", false),
            ("show_layers_in_layers", true),
            ("display_clustered_bar_charts", false),
            ("suppress_tables", false),
            ("chisq", true),
            ("correlations", false),
            ("contingency_coef", false),
            ("phi_cramer", true),
            ("lambda", false),
            ("uncertainty_coef", false),
            ("gamma", false),
            ("somers_d", false),
            ("kendall_tau_b", false),
            ("kendall_tau_c", false),
            ("eta", false),
            ("kappa", false),
            ("risk", false),
            ("mcnemar", false),
            ("cochran_mantel_haenszel", false),
        ] {
            props.insert(k.into(), json!({ "type": "boolean", "default": def }));
        }
        props.insert(
            "hide_small_threshold".into(),
            json!({ "type": "integer", "default": 5 }),
        );
        props.insert(
            "noninteger_weights".into(),
            json!({ "type": "string", "default": "RoundCell" }),
        );
        props.insert(
            "row_order".into(),
            json!({ "type": "string", "default": "Ascending" }),
        );
        props.insert("exact_mode".into(), json!({ "default": "AsymptoticOnly" }));
        props.insert(
            "cmh_common_odds_ratio".into(),
            json!({ "type": "number", "default": 1.0 }),
        );
        json!({
            "type": "object",
            "required": ["rows", "cols"],
            "properties": Value::Object(props)
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        matches!(dataset, Some(ds) if ds.row_count() > 0 && ds.col_count() >= 2)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let rows = extract_str_array(&params, "rows");
        let cols = extract_str_array(&params, "cols");
        if rows.is_empty() || cols.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Both 'rows' and 'cols' parameters must contain at least one variable"
                    .to_string(),
            });
        }

        let layers: Vec<Vec<String>> = params
            .get("layers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|l| {
                        l.as_array()
                            .map(|inner| {
                                inner
                                    .iter()
                                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default()
                    })
                    .filter(|v: &Vec<String>| !v.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let options: CrosstabOptions = serde_json::from_value(params).unwrap_or_default();
        let active_rows = ds.active_row_indices();
        let mut results: Vec<CrosstabResult> = Vec::new();

        let layer_var_combos = build_layer_var_combos(&layers);

        for r_name in &rows {
            let r_idx = find_var_idx(ds, r_name, self.id())?;
            for c_name in &cols {
                let c_idx = find_var_idx(ds, c_name, self.id())?;
                if layer_var_combos.is_empty() {
                    let items = collect_items(ds, &active_rows, r_idx, c_idx)?;
                    let res = compute_crosstabs(r_name, c_name, &items, &options)
                        .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
                    results.push(res);
                } else {
                    for layer_vars in &layer_var_combos {
                        let mut l_indices = Vec::with_capacity(layer_vars.len());
                        for lv in layer_vars {
                            l_indices.push((lv.as_str(), find_var_idx(ds, lv, self.id())?));
                        }
                        let combos = collect_layer_value_combos(ds, &active_rows, &l_indices);
                        let mut layer_results = Vec::new();
                        let mut strata_2x2: Vec<[[f64; 2]; 2]> = Vec::new();

                        for combo in combos {
                            let filtered: Vec<usize> = active_rows
                                .iter()
                                .copied()
                                .filter(|&r| {
                                    l_indices.iter().enumerate().all(|(k, &(_, col_i))| {
                                        let (v, miss) = cell_val_missing(ds, r, col_i);
                                        !miss && v == combo[k].value
                                    })
                                })
                                .collect();
                            if filtered.is_empty() {
                                continue;
                            }
                            let items = collect_items(ds, &filtered, r_idx, c_idx)?;
                            if let Ok(mut res) = compute_crosstabs(r_name, c_name, &items, &options)
                            {
                                res.layer_keys = combo;
                                if res.rows.len() == 2 && res.col_categories.len() == 2 {
                                    strata_2x2.push([
                                        [
                                            res.rows[0].cells[0].observed_f64,
                                            res.rows[0].cells[1].observed_f64,
                                        ],
                                        [
                                            res.rows[1].cells[0].observed_f64,
                                            res.rows[1].cells[1].observed_f64,
                                        ],
                                    ]);
                                }
                                layer_results.push(res);
                            }
                        }
                        if options.cochran_mantel_haenszel
                            && !strata_2x2.is_empty()
                            && strata_2x2.len() == layer_results.len()
                        {
                            if let Some(cmh) =
                                compute_cmh(&strata_2x2, options.cmh_common_odds_ratio)
                            {
                                if let Some(first) = layer_results.first_mut() {
                                    first.cmh_result = Some(cmh);
                                }
                            }
                        }
                        results.extend(layer_results);
                    }
                }
            }
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

fn extract_str_array(params: &Value, key: &str) -> Vec<String> {
    params
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn find_var_idx(ds: &DataSet, name: &str, id: &'static str) -> Result<usize, CommandError> {
    ds.dictionary
        .find_index(name)
        .ok_or_else(|| CommandError::InvalidParams {
            id,
            reason: format!("Variable '{}' not found in dataset", name),
        })
}

fn cell_val_missing(ds: &DataSet, row: usize, col_idx: usize) -> (String, bool) {
    let meta = match ds.dictionary.get(col_idx) {
        Some(m) => m,
        None => return (String::new(), true),
    };
    match ds.get_cell(row, col_idx) {
        Some(RawValue::Number(n)) => {
            let miss = meta.missing.is_missing(&RawValue::Number(*n));
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
                || meta
                    .missing
                    .is_missing(&RawValue::Text(trimmed.to_string()));
            (trimmed.to_string(), miss)
        }
        Some(RawValue::Null) | None => (String::new(), true),
    }
}

fn collect_items(
    ds: &DataSet,
    row_indices: &[usize],
    r_idx: usize,
    c_idx: usize,
) -> Result<Vec<RawCrosstabItem>, CommandError> {
    let r_meta = ds
        .dictionary
        .get(r_idx)
        .ok_or_else(|| CommandError::ExecutionFailed("Row metadata missing".to_string()))?;
    let c_meta = ds
        .dictionary
        .get(c_idx)
        .ok_or_else(|| CommandError::ExecutionFailed("Column metadata missing".to_string()))?;
    let mut items = Vec::with_capacity(row_indices.len());
    for &r in row_indices {
        let (r_val, r_miss) = cell_val_missing(ds, r, r_idx);
        let (c_val, c_miss) = cell_val_missing(ds, r, c_idx);
        let r_label = (!r_miss)
            .then(|| r_meta.values.get_label(&r_val).map(|s| s.to_string()))
            .flatten();
        let c_label = (!c_miss)
            .then(|| c_meta.values.get_label(&c_val).map(|s| s.to_string()))
            .flatten();
        items.push(RawCrosstabItem {
            row_value: r_val,
            row_label: r_label,
            col_value: c_val,
            col_label: c_label,
            is_missing: r_miss || c_miss,
            weight: ds.row_weight(r),
        });
    }
    Ok(items)
}

fn build_layer_var_combos(layers: &[Vec<String>]) -> Vec<Vec<String>> {
    if layers.is_empty() {
        return Vec::new();
    }
    let mut combos = vec![Vec::new()];
    for layer in layers {
        let mut next = Vec::new();
        for existing in &combos {
            for var in layer {
                let mut c = existing.clone();
                c.push(var.clone());
                next.push(c);
            }
        }
        combos = next;
    }
    combos
}

fn collect_layer_value_combos(
    ds: &DataSet,
    active_rows: &[usize],
    l_indices: &[(&str, usize)],
) -> Vec<Vec<CrosstabLayerKey>> {
    let mut per_layer_vals: Vec<BTreeSet<String>> = vec![BTreeSet::new(); l_indices.len()];
    for &r in active_rows {
        for (k, &(_, col_i)) in l_indices.iter().enumerate() {
            let (val, miss) = cell_val_missing(ds, r, col_i);
            if !miss {
                per_layer_vals[k].insert(val);
            }
        }
    }
    let mut combos: Vec<Vec<CrosstabLayerKey>> = vec![Vec::new()];
    for (k, &(var_name, col_i)) in l_indices.iter().enumerate() {
        let meta = ds.dictionary.get(col_i);
        let mut next = Vec::new();
        for existing in &combos {
            for val in &per_layer_vals[k] {
                let mut c = existing.clone();
                c.push(CrosstabLayerKey {
                    var_name: var_name.to_string(),
                    value: val.clone(),
                    label: meta.and_then(|m| m.values.get_label(val).map(|s| s.to_string())),
                });
                next.push(c);
            }
        }
        combos = next;
    }
    combos
}
