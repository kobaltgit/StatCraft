use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_ranks, RankDirection, RankType, TiesHandling};
use statcraft_core::{DataSet, DataType, Measure, VariableMetadata};
use statcraft_io::RawValue;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum RankCasesUndoItem {
    Created {
        col_idx: usize,
    },
    Overwritten {
        col_idx: usize,
        old_meta: VariableMetadata,
        old_values: Vec<RawValue>,
    },
}

pub struct RankCasesCommand {
    history: Mutex<Vec<Vec<RankCasesUndoItem>>>,
}

impl Default for RankCasesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl RankCasesCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(
        variables: &[&str],
        direction: RankDirection,
        ties: TiesHandling,
        rank_type: RankType,
        _ntiles_groups: usize,
        prefix: &str,
    ) -> String {
        if variables.is_empty() {
            return String::new();
        }

        let dir_token = match direction {
            RankDirection::Ascending => "(A)",
            RankDirection::Descending => "(D)",
        };

        let var_specs: Vec<String> = variables
            .iter()
            .map(|v| format!("{} {}", v, dir_token))
            .collect();

        let mut sps = format!("RANK VARIABLES={}\n", var_specs.join(" "));

        let ties_clause = match ties {
            TiesHandling::Mean => "/TIES=MEAN",
            TiesHandling::Low => "/TIES=LOW",
            TiesHandling::High => "/TIES=HIGH",
            TiesHandling::Seq => "/TIES=CONDENSE",
        };
        sps.push_str(&format!("  {}\n", ties_clause));

        let type_clause = match rank_type {
            RankType::SimpleRank => "/RANK".to_string(),
            RankType::FractionalRank => "/RFRACTION".to_string(),
            RankType::FractionalPercent | RankType::Percentiles => "/PERCENT".to_string(),
            RankType::Ntiles(k) => format!("/NTILES({})", k.max(2)),
            RankType::NormalBlom => "/NORMAL (BLOM)".to_string(),
            RankType::NormalTukey => "/NORMAL (TUKEY)".to_string(),
            RankType::NormalRankit => "/NORMAL (RANKIT)".to_string(),
            RankType::NormalVanderWaerden => "/NORMAL (VW)".to_string(),
        };
        sps.push_str(&format!("  {}\n", type_clause));

        if prefix != "R_" && !prefix.is_empty() {
            let target_names: Vec<String> = variables
                .iter()
                .map(|v| format!("{}{}", prefix, v))
                .collect();
            sps.push_str(&format!("  /INTO {}\n", target_names.join(" ")));
        }

        sps.push_str("  /PRINT.\nEXECUTE.");
        sps
    }
}

impl EngineCommand for RankCasesCommand {
    fn id(&self) -> &'static str {
        "transform.rank_cases"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Rank Cases"
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
                    "description": "List of source variable names to rank"
                },
                "direction": {
                    "type": "string",
                    "enum": ["ascending", "descending"],
                    "default": "ascending",
                    "description": "Direction: smallest or largest value gets rank 1"
                },
                "ties": {
                    "type": "string",
                    "enum": ["mean", "low", "high", "sequential"],
                    "default": "mean",
                    "description": "Method for handling tied values"
                },
                "rank_type": {
                    "type": "string",
                    "enum": ["rank", "fractional", "percent", "ntiles", "blom", "tukey", "rankit", "vw"],
                    "default": "rank",
                    "description": "Type of assigned rank"
                },
                "ntiles_groups": {
                    "type": "integer",
                    "minimum": 2,
                    "default": 4,
                    "description": "Number of groups for Ntiles rank type"
                },
                "prefix": {
                    "type": "string",
                    "default": "R_",
                    "description": "Prefix for newly created rank variables"
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

        let var_array = params
            .get("variables")
            .and_then(|v| v.as_array())
            .ok_or_else(|| CommandError::InvalidParams {
                id: self.id(),
                reason: "Expected 'variables' to be an array of strings".to_string(),
            })?;

        if var_array.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Variable list cannot be empty".to_string(),
            });
        }

        let mut var_names = Vec::with_capacity(var_array.len());
        for val in var_array {
            let name = val
                .as_str()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| CommandError::InvalidParams {
                    id: self.id(),
                    reason: "Variable names in 'variables' array must be non-empty strings"
                        .to_string(),
                })?;
            var_names.push(name.to_string());
        }

        let direction = match params
            .get("direction")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("descending") => RankDirection::Descending,
            _ => RankDirection::Ascending,
        };

        let ties = match params
            .get("ties")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("low") => TiesHandling::Low,
            Some("high") => TiesHandling::High,
            Some("sequential") => TiesHandling::Seq,
            _ => TiesHandling::Mean,
        };

        let ntiles_groups = params
            .get("ntiles_groups")
            .and_then(|v| v.as_u64())
            .unwrap_or(4) as usize;

        let rank_type = match params
            .get("rank_type")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("fractional") => RankType::FractionalRank,
            Some("percent") => RankType::FractionalPercent,
            Some("ntiles") => RankType::Ntiles(ntiles_groups.max(2)),
            Some("blom") => RankType::NormalBlom,
            Some("tukey") => RankType::NormalTukey,
            Some("rankit") => RankType::NormalRankit,
            Some("vw") => RankType::NormalVanderWaerden,
            _ => RankType::SimpleRank,
        };

        let prefix = params
            .get("prefix")
            .and_then(|v| v.as_str())
            .unwrap_or("R_");

        let decimals: u8 = match rank_type {
            RankType::SimpleRank => {
                if ties == TiesHandling::Mean {
                    2
                } else {
                    0
                }
            }
            RankType::Ntiles(_) => 0,
            _ => 2,
        };

        let mut batch_undo = Vec::with_capacity(var_names.len());
        let mut target_names = Vec::with_capacity(var_names.len());

        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: self.id(),
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;

            let values = ds
                .table
                .columns
                .get(col_idx)
                .map(|c| c.values.as_slice())
                .ok_or_else(|| {
                    CommandError::ExecutionFailed("Column values missing".to_string())
                })?;

            let ranked_values = compute_ranks(values, direction, ties, rank_type);
            let target_var = format!("{}{}", prefix, var_name);

            let undo_item = if let Some(existing_idx) = ds.dictionary.find_index(&target_var) {
                let old_meta = ds.dictionary.get(existing_idx).cloned().ok_or_else(|| {
                    CommandError::ExecutionFailed("Corrupt dictionary".to_string())
                })?;
                let old_values = ds
                    .table
                    .columns
                    .get(existing_idx)
                    .map(|c| c.values.clone())
                    .ok_or_else(|| {
                        CommandError::ExecutionFailed("Target column missing".to_string())
                    })?;

                if let Some(col) = ds.table.columns.get_mut(existing_idx) {
                    col.values = ranked_values;
                }
                if let Some(meta) = ds.dictionary.get_mut(existing_idx) {
                    meta.var_type = DataType::Numeric { decimals };
                    meta.measure = Measure::Ordinal;
                }

                RankCasesUndoItem::Overwritten {
                    col_idx: existing_idx,
                    old_meta,
                    old_values,
                }
            } else {
                let target_idx = ds.add_variable(
                    &target_var,
                    DataType::Numeric { decimals },
                    Measure::Ordinal,
                )?;
                if let Some(col) = ds.table.columns.get_mut(target_idx) {
                    col.values = ranked_values;
                }

                RankCasesUndoItem::Created {
                    col_idx: target_idx,
                }
            };

            batch_undo.push(undo_item);
            target_names.push(target_var);
        }

        if let Ok(mut hist) = self.history.lock() {
            hist.push(batch_undo);
        }

        Ok(json!({
            "status": "success",
            "targets": target_names,
            "rows": ds.row_count()
        }))
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
                    RankCasesUndoItem::Created { col_idx } => {
                        ds.remove_variable(col_idx)?;
                    }
                    RankCasesUndoItem::Overwritten {
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
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataset() -> DataSet {
        let mut ds = DataSet::empty();
        let c0 = ds
            .add_variable("score", DataType::Numeric { decimals: 0 }, Measure::Scale)
            .unwrap();
        let _ = ds
            .add_variable("grade", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
            .unwrap();

        ds.table.columns[c0].values = vec![
            RawValue::Number(20.0),
            RawValue::Number(10.0),
            RawValue::Number(30.0),
        ];
        ds.table.row_count = 3;
        ds
    }

    #[test]
    fn test_rank_cases_command_execute_and_undo() {
        let cmd = RankCasesCommand::new();
        let mut ds = Some(create_test_dataset());

        let params = json!({
            "variables": ["score"],
            "direction": "ascending",
            "ties": "mean",
            "rank_type": "rank",
            "prefix": "R_"
        });

        let res = cmd.execute(&mut ds, params).unwrap();
        assert_eq!(res["status"], "success");
        assert_eq!(res["targets"], json!(["R_score"]));

        let ds_ref = ds.as_ref().unwrap();
        let r_idx = ds_ref.dictionary.find_index("R_score").unwrap();
        let values = &ds_ref.table.columns[r_idx].values;
        assert_eq!(
            values,
            &vec![
                RawValue::Number(2.0),
                RawValue::Number(1.0),
                RawValue::Number(3.0),
            ]
        );

        // Undo
        cmd.undo(&mut ds).unwrap();
        let ds_after = ds.as_ref().unwrap();
        assert!(ds_after.dictionary.find_index("R_score").is_none());
    }

    #[test]
    fn test_rank_cases_to_sps() {
        let sps = RankCasesCommand::to_sps(
            &["salary", "bonus"],
            RankDirection::Ascending,
            TiesHandling::Mean,
            RankType::SimpleRank,
            4,
            "R_",
        );
        assert!(sps.contains("RANK VARIABLES=salary (A) bonus (A)"));
        assert!(sps.contains("/TIES=MEAN"));
        assert!(sps.contains("/RANK"));
    }
}
