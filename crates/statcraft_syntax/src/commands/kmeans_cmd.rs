use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_kmeans, KMeansOptions};
use statcraft_core::{DataSet, DataType, Measure, RawValue, VariableMetadata};
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum KMeansUndoItem {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

pub struct KMeansClusterCommand {
    history: Mutex<Vec<Vec<KMeansUndoItem>>>,
}

impl Default for KMeansClusterCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl KMeansClusterCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(
        variables: &[String],
        k: usize,
        max_iter: usize,
        converge: f64,
        save_cluster: bool,
        save_distance: bool,
    ) -> String {
        let mut sps = format!("QUICK CLUSTER {}\n", variables.join(" "));
        sps.push_str(&format!(
            "  /CRITERIA = CLUSTERS({}) MXITER({}) CONVERGE({})\n",
            k, max_iter, converge
        ));
        sps.push_str("  /PRINT = INITIAL FINAL DISTAN ANOVA\n");
        if save_cluster || save_distance {
            let mut save_parts = Vec::new();
            if save_cluster {
                save_parts.push("CLUSTER");
            }
            if save_distance {
                save_parts.push("DISTANCE");
            }
            sps.push_str(&format!("  /SAVE = {}.\n", save_parts.join(" ")));
        } else {
            sps.push_str("  /METHOD = CLUSTER.\n");
        }
        sps.push_str("EXECUTE.");
        sps
    }
}

impl EngineCommand for KMeansClusterCommand {
    fn id(&self) -> &'static str {
        "stat.cluster.kmeans"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze -> Classify -> K-Means Cluster..."
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn category(&self) -> &'static str {
        "Classify"
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["QUICK CLUSTER", "KMEANS"]
    }

    fn specs(&self) -> Value {
        json!({
            "command": "stat.cluster.kmeans",
            "required": ["variables"],
            "properties": {
                "variables": { "type": "array", "items": { "type": "string" } },
                "clusters": { "type": "integer", "default": 3, "minimum": 2 },
                "max_iterations": { "type": "integer", "default": 10, "minimum": 1 },
                "convergence_criterion": { "type": "number", "default": 0.0001 },
                "save_cluster": { "type": "boolean", "default": false },
                "save_distance": { "type": "boolean", "default": false }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset
            .map(|ds| ds.row_count() >= 2 && ds.col_count() >= 1)
            .unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
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

        let k = params.get("clusters").and_then(|v| v.as_u64()).unwrap_or(3) as usize;

        let max_iter = params
            .get("max_iterations")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let converge = params
            .get("convergence_criterion")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0001);

        let save_cluster = params
            .get("save_cluster")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let save_distance = params
            .get("save_distance")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Find column indices
        let mut col_indices = Vec::with_capacity(var_names.len());
        for name in &var_names {
            let idx =
                ds.dictionary
                    .find_index(name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dictionary", name),
                    })?;
            col_indices.push(idx);
        }

        // Collect rows (listwise deletion of missing values)
        let total_rows = ds.row_count();
        let mut valid_row_indices = Vec::new();
        let mut data_matrix = Vec::new();

        for r in 0..total_rows {
            let mut row = Vec::with_capacity(col_indices.len());
            let mut has_missing = false;

            for &c in &col_indices {
                match ds.table.columns[c].values.get(r) {
                    Some(RawValue::Number(val)) if !val.is_nan() => row.push(*val),
                    _ => {
                        has_missing = true;
                        break;
                    }
                }
            }

            if !has_missing {
                valid_row_indices.push(r);
                data_matrix.push(row);
            }
        }

        let options = KMeansOptions {
            clusters: k,
            max_iterations: max_iter,
            convergence_criterion: converge,
        };

        let result = compute_kmeans(&data_matrix, &var_names, &options)
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        // Handle saving columns QCL_1 and QCL_2
        let mut batch_undo = Vec::new();

        if save_cluster {
            let cluster_col_name = "QCL_1";
            let mut cluster_vals = vec![RawValue::Null; total_rows];
            for (valid_idx, &cluster_id) in result.cluster_assignments.iter().enumerate() {
                let orig_r = valid_row_indices[valid_idx];
                cluster_vals[orig_r] = RawValue::Number(cluster_id as f64);
            }

            let undo_item = if let Some(target_idx) = ds.dictionary.find_index(cluster_col_name) {
                let old_meta = ds.dictionary.get(target_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Missing existing column metadata".to_string())
                })?;
                let old_values = ds.table.columns[target_idx].values.clone();

                ds.table.columns[target_idx].values = cluster_vals;
                if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                    meta.var_type = DataType::Numeric { decimals: 0 };
                    meta.measure = Measure::Nominal;
                    meta.label = "Cluster Membership".to_string();
                    for c_i in 1..=k {
                        meta.values
                            .insert(&c_i.to_string(), &format!("Cluster {}", c_i));
                    }
                }

                KMeansUndoItem::Overwritten {
                    col_idx: target_idx,
                    old_meta,
                    old_values,
                }
            } else {
                let target_idx = ds.add_variable(
                    cluster_col_name,
                    DataType::Numeric { decimals: 0 },
                    Measure::Nominal,
                )?;
                if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                    meta.label = "Cluster Membership".to_string();
                    for c_i in 1..=k {
                        meta.values
                            .insert(&c_i.to_string(), &format!("Cluster {}", c_i));
                    }
                }
                ds.table.columns[target_idx].values = cluster_vals;

                KMeansUndoItem::Created {
                    col_idx: target_idx,
                }
            };
            batch_undo.push(undo_item);
        }

        if save_distance {
            let dist_col_name = "QCL_2";
            let mut dist_vals = vec![RawValue::Null; total_rows];
            for (valid_idx, &d) in result.distances_to_center.iter().enumerate() {
                let orig_r = valid_row_indices[valid_idx];
                dist_vals[orig_r] = RawValue::Number(d);
            }

            let undo_item = if let Some(target_idx) = ds.dictionary.find_index(dist_col_name) {
                let old_meta = ds.dictionary.get(target_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Missing existing column metadata".to_string())
                })?;
                let old_values = ds.table.columns[target_idx].values.clone();

                ds.table.columns[target_idx].values = dist_vals;
                if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                    meta.var_type = DataType::Numeric { decimals: 4 };
                    meta.measure = Measure::Scale;
                    meta.label = "Distance from Cluster Center".to_string();
                }

                KMeansUndoItem::Overwritten {
                    col_idx: target_idx,
                    old_meta,
                    old_values,
                }
            } else {
                let target_idx = ds.add_variable(
                    dist_col_name,
                    DataType::Numeric { decimals: 4 },
                    Measure::Scale,
                )?;
                if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                    meta.label = "Distance from Cluster Center".to_string();
                }
                ds.table.columns[target_idx].values = dist_vals;

                KMeansUndoItem::Created {
                    col_idx: target_idx,
                }
            };
            batch_undo.push(undo_item);
        }

        if let Ok(mut hist) = self.history.lock() {
            hist.push(batch_undo);
        }

        serde_json::to_value(&result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let batch = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        if let Some(items) = batch {
            for item in items.into_iter().rev() {
                match item {
                    KMeansUndoItem::Created { col_idx } => {
                        ds.remove_variable(col_idx)?;
                    }
                    KMeansUndoItem::Overwritten {
                        col_idx,
                        old_meta,
                        old_values,
                    } => {
                        if let Some(meta) = ds.dictionary.get_mut(col_idx) {
                            *meta = old_meta;
                        }
                        if col_idx < ds.table.columns.len() {
                            ds.table.columns[col_idx].values = old_values;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
