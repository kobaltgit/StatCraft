use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_adp, AdpBinningOptions, AdpCategoricalOptions, AdpInteractionOptions, AdpObjective,
    AdpOptions, AdpOutlierOptions, AdpPartitionOptions, AdpRescaleMethod, AdpResult,
    AdpScreeningOptions, OutlierMethod, OutlierTreatment,
};
use statcraft_core::{DataSet, DataType, Measure};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct AdpUndoEntry {
    pub created_cols: Vec<usize>,
}

pub struct PrepareDataModelingCommand {
    history: Mutex<Vec<AdpUndoEntry>>,
}

impl Default for PrepareDataModelingCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PrepareDataModelingCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(opts: &AdpOptions) -> String {
        let mut sps = String::from("ADP\n");

        // /FIELDS
        let mut fields_clause = String::from("  /FIELDS");
        if let Some(ref tgt) = opts.target_variable {
            fields_clause.push_str(&format!(" TARGET={}", tgt));
        }
        if !opts.input_variables.is_empty() {
            fields_clause.push_str(&format!(" INPUTS={}", opts.input_variables.join(" ")));
        }
        if let Some(ref w) = opts.weight_variable {
            fields_clause.push_str(&format!(" WEIGHT={}", w));
        }
        fields_clause.push('\n');
        sps.push_str(&fields_clause);

        // /OBJECTIVE
        let obj_str = match opts.objective {
            AdpObjective::BalanceSpeedAccuracy => "BALANCE",
            AdpObjective::OptimizeAccuracy => "ACCURACY",
            AdpObjective::OptimizeSpeed => "SPEED",
            AdpObjective::Custom => "CUSTOM",
        };
        sps.push_str(&format!("  /OBJECTIVE {}\n", obj_str));

        // /PREPAREINPUTS
        sps.push_str(&format!(
            "  /PREPAREINPUTS SCREENING=YES(PCTMISSING={:.1} PCTUNIQUEMAX={:.1} PCTFREQMIN={:.1})\n",
            opts.screening.max_missing_pct,
            opts.screening.max_unique_pct,
            opts.screening.max_category_pct
        ));

        // /OUTLIERS
        match opts.outliers.method {
            OutlierMethod::None => {
                sps.push_str("  /OUTLIERS METHOD=NONE\n");
            }
            OutlierMethod::ZScore { threshold } => {
                let action = match opts.outliers.treatment {
                    OutlierTreatment::Winsorize => "WINSORIZE",
                    OutlierTreatment::SetSysmis => "SYSMIS",
                };
                sps.push_str(&format!(
                    "  /OUTLIERS METHOD=ZSCORE({:.1}) ACTION={}\n",
                    threshold, action
                ));
            }
            OutlierMethod::Iqr { multiplier } => {
                let action = match opts.outliers.treatment {
                    OutlierTreatment::Winsorize => "WINSORIZE",
                    OutlierTreatment::SetSysmis => "SYSMIS",
                };
                sps.push_str(&format!(
                    "  /OUTLIERS METHOD=IQR({:.1}) ACTION={}\n",
                    multiplier, action
                ));
            }
        }

        // /RESCALE
        match opts.rescale {
            AdpRescaleMethod::None => sps.push_str("  /RESCALE METHOD=NONE\n"),
            AdpRescaleMethod::ZScore => sps.push_str("  /RESCALE METHOD=ZSCORE\n"),
            AdpRescaleMethod::MinMax { min, max } => sps.push_str(&format!(
                "  /RESCALE METHOD=MINMAX(MIN={} MAX={})\n",
                min, max
            )),
        }

        // /CATEGORICAL
        if opts.categorical.merge_sparse {
            sps.push_str(&format!(
                "  /CATEGORICAL MERGE=YES(PCTMIN={:.1})\n",
                opts.categorical.sparse_threshold_pct
            ));
        }

        // /TRANSFORM
        if opts.binning.bin_continuous_inputs {
            sps.push_str(&format!(
                "  /TRANSFORM BINNING=MDLP(MAXBINS={})\n",
                opts.binning.max_bins
            ));
        }

        // /CONSTRUCT
        if opts.interactions.create_interactions {
            sps.push_str(&format!(
                "  /CONSTRUCT INTERACTIONS=YES(MAXPAIRS={})\n",
                opts.interactions.max_pairs
            ));
        }

        // /PARTITION
        if opts.partition.enabled {
            let seed_str = match opts.partition.seed {
                Some(s) => format!(" SEED={}", s),
                None => String::new(),
            };
            sps.push_str(&format!(
                "  /PARTITION TRAIN={:.0} TEST={:.0} VALIDATE={:.0}{}.\n",
                opts.partition.train_pct,
                opts.partition.test_pct,
                opts.partition.validate_pct,
                seed_str
            ));
        } else {
            // End sentence with dot
            if let Some(idx) = sps.rfind('\n') {
                sps.insert(idx, '.');
            }
        }

        sps.push_str("EXECUTE.");
        sps
    }
}

impl EngineCommand for PrepareDataModelingCommand {
    fn id(&self) -> &'static str {
        "transform.adp"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Prepare Data for Modeling (ADP)"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "inputs": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Predictor candidate input variables"
                },
                "target": {
                    "type": ["string", "null"],
                    "description": "Target variable name"
                },
                "weight": {
                    "type": ["string", "null"],
                    "description": "Optional weighting variable name"
                },
                "objective": {
                    "type": "string",
                    "enum": ["balance", "accuracy", "speed", "custom"],
                    "default": "balance"
                },
                "max_missing_pct": { "type": "number", "default": 50.0 },
                "max_category_pct": { "type": "number", "default": 95.0 },
                "max_unique_pct": { "type": "number", "default": 95.0 },
                "outlier_method": {
                    "type": "string",
                    "enum": ["none", "zscore", "iqr"],
                    "default": "zscore"
                },
                "outlier_action": {
                    "type": "string",
                    "enum": ["winsorize", "sysmis"],
                    "default": "winsorize"
                },
                "rescale_method": {
                    "type": "string",
                    "enum": ["none", "zscore", "minmax"],
                    "default": "none"
                },
                "merge_sparse": { "type": "boolean", "default": true },
                "binning": { "type": "boolean", "default": false },
                "interactions": { "type": "boolean", "default": false },
                "partition": { "type": "boolean", "default": false },
                "train_pct": { "type": "number", "default": 70.0 },
                "test_pct": { "type": "number", "default": 20.0 },
                "validate_pct": { "type": "number", "default": 10.0 },
                "seed": { "type": ["integer", "null"], "default": 2026 }
            },
            "required": ["inputs"]
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        dataset
            .map(|d| d.col_count() > 0 && d.row_count() > 0)
            .unwrap_or(false)
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let input_names: Vec<String> = params
            .get("inputs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if input_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "At least one input variable must be specified".to_string(),
            });
        }

        let target_name = params
            .get("target")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let weight_name = params
            .get("weight")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // Parse options
        let objective = match params.get("objective").and_then(|v| v.as_str()) {
            Some("accuracy") => AdpObjective::OptimizeAccuracy,
            Some("speed") => AdpObjective::OptimizeSpeed,
            Some("custom") => AdpObjective::Custom,
            _ => AdpObjective::BalanceSpeedAccuracy,
        };

        let screening = AdpScreeningOptions {
            max_missing_pct: params
                .get("max_missing_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(50.0),
            max_category_pct: params
                .get("max_category_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(95.0),
            max_unique_pct: params
                .get("max_unique_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(95.0),
        };

        let outlier_method = match params.get("outlier_method").and_then(|v| v.as_str()) {
            Some("none") => OutlierMethod::None,
            Some("iqr") => OutlierMethod::Iqr { multiplier: 1.5 },
            _ => OutlierMethod::ZScore { threshold: 3.0 },
        };

        let outlier_treatment = match params.get("outlier_action").and_then(|v| v.as_str()) {
            Some("sysmis") => OutlierTreatment::SetSysmis,
            _ => OutlierTreatment::Winsorize,
        };

        let rescale = match params.get("rescale_method").and_then(|v| v.as_str()) {
            Some("zscore") => AdpRescaleMethod::ZScore,
            Some("minmax") => AdpRescaleMethod::MinMax { min: 0.0, max: 1.0 },
            _ => AdpRescaleMethod::None,
        };

        let categorical = AdpCategoricalOptions {
            merge_sparse: params
                .get("merge_sparse")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            sparse_threshold_pct: 5.0,
        };

        let binning = AdpBinningOptions {
            bin_continuous_inputs: params
                .get("binning")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            max_bins: 5,
        };

        let interactions = AdpInteractionOptions {
            create_interactions: params
                .get("interactions")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            max_pairs: 5,
        };

        let partition_enabled = params
            .get("partition")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let partition = AdpPartitionOptions {
            enabled: partition_enabled,
            train_pct: params
                .get("train_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(70.0),
            test_pct: params
                .get("test_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(20.0),
            validate_pct: params
                .get("validate_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(10.0),
            seed: params.get("seed").and_then(|v| v.as_u64()),
        };

        let adp_opts = AdpOptions {
            objective,
            target_variable: target_name.clone(),
            input_variables: input_names.clone(),
            weight_variable: weight_name.clone(),
            screening,
            outliers: AdpOutlierOptions {
                method: outlier_method,
                treatment: outlier_treatment,
            },
            rescale,
            categorical,
            binning,
            interactions,
            partition,
        };

        // Extract column slices from DataSet
        let mut inputs_data = Vec::with_capacity(input_names.len());
        for name in &input_names {
            let col_idx = ds.dictionary.find_index(name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Input variable '{}' not found", name))
            })?;
            let meta = ds.dictionary.get(col_idx).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Metadata for '{}' not found", name))
            })?;
            let col = ds.table.columns.get(col_idx).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Column data for '{}' not found", name))
            })?;
            inputs_data.push((
                name.clone(),
                col.values.as_slice(),
                meta.var_type.clone(),
                meta.measure,
            ));
        }

        let target_data = if let Some(ref tgt) = target_name {
            let col_idx = ds.dictionary.find_index(tgt).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Target variable '{}' not found", tgt))
            })?;
            let meta = ds.dictionary.get(col_idx).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Metadata for '{}' not found", tgt))
            })?;
            let col = ds.table.columns.get(col_idx).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Column data for '{}' not found", tgt))
            })?;
            Some((
                tgt.as_str(),
                col.values.as_slice(),
                meta.var_type.clone(),
                meta.measure,
            ))
        } else {
            None
        };

        let weights_data: Option<Vec<f64>> = if let Some(ref w_name) = weight_name {
            let col_idx = ds.dictionary.find_index(w_name).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Weight variable '{}' not found", w_name))
            })?;
            let col = ds.table.columns.get(col_idx).ok_or_else(|| {
                CommandError::ExecutionFailed(format!("Weight data for '{}' not found", w_name))
            })?;
            let ws: Vec<f64> = col
                .values
                .iter()
                .map(|v| match v {
                    statcraft_core::RawValue::Number(n) if n.is_finite() && *n > 0.0 => *n,
                    _ => 0.0,
                })
                .collect();
            Some(ws)
        } else {
            None
        };

        // Compute ADP
        let adp_result: AdpResult = compute_adp(
            &inputs_data,
            target_data,
            weights_data.as_deref(),
            &adp_opts,
        )
        .map_err(CommandError::ExecutionFailed)?;

        // Apply transformations into DataSet
        let mut created_indices = Vec::new();

        for col in &adp_result.transformed_columns {
            let (dtype, measure) = if col.is_partition || col.labels.is_some() {
                (DataType::Numeric { decimals: 0 }, Measure::Nominal)
            } else {
                (DataType::Numeric { decimals: 4 }, Measure::Scale)
            };

            let target_idx = ds.add_variable(&col.name, dtype, measure)?;
            if let Some(labels) = &col.labels {
                if let Some(meta) = ds.dictionary.get_mut(target_idx) {
                    meta.values = labels.clone();
                }
            }
            ds.table.columns[target_idx].values = col.values.clone();
            created_indices.push(target_idx);
        }

        if let Ok(mut hist) = self.history.lock() {
            hist.push(AdpUndoEntry {
                created_cols: created_indices,
            });
        }

        serde_json::to_value(&adp_result).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let ds = dataset
            .as_mut()
            .ok_or_else(|| CommandError::Disabled(self.id().to_string()))?;

        let entry = {
            let mut hist = self
                .history
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            hist.pop()
        };

        if let Some(mut undo_entry) = entry {
            undo_entry.created_cols.sort_unstable_by(|a, b| b.cmp(a));
            for col_idx in undo_entry.created_cols {
                ds.remove_variable(col_idx)?;
            }
        }

        Ok(())
    }
}
