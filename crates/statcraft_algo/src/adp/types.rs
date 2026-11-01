use serde::{Deserialize, Serialize};
use statcraft_core::{RawValue, ValueLabels};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AdpObjective {
    #[default]
    BalanceSpeedAccuracy,
    OptimizeAccuracy,
    OptimizeSpeed,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdpScreeningOptions {
    pub max_missing_pct: f64,
    pub max_category_pct: f64,
    pub max_unique_pct: f64,
}

impl Default for AdpScreeningOptions {
    fn default() -> Self {
        Self {
            max_missing_pct: 50.0,
            max_category_pct: 95.0,
            max_unique_pct: 95.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutlierMethod {
    None,
    ZScore { threshold: f64 },
    Iqr { multiplier: f64 },
}

impl Default for OutlierMethod {
    fn default() -> Self {
        Self::ZScore { threshold: 3.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutlierTreatment {
    #[default]
    Winsorize,
    SetSysmis,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AdpOutlierOptions {
    pub method: OutlierMethod,
    pub treatment: OutlierTreatment,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum AdpRescaleMethod {
    #[default]
    None,
    ZScore,
    MinMax {
        min: f64,
        max: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdpCategoricalOptions {
    pub merge_sparse: bool,
    pub sparse_threshold_pct: f64,
}

impl Default for AdpCategoricalOptions {
    fn default() -> Self {
        Self {
            merge_sparse: true,
            sparse_threshold_pct: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdpBinningOptions {
    pub bin_continuous_inputs: bool,
    pub max_bins: usize,
}

impl Default for AdpBinningOptions {
    fn default() -> Self {
        Self {
            bin_continuous_inputs: false,
            max_bins: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdpInteractionOptions {
    pub create_interactions: bool,
    pub max_pairs: usize,
}

impl Default for AdpInteractionOptions {
    fn default() -> Self {
        Self {
            create_interactions: false,
            max_pairs: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdpPartitionOptions {
    pub enabled: bool,
    pub train_pct: f64,
    pub test_pct: f64,
    pub validate_pct: f64,
    pub seed: Option<u64>,
}

impl Default for AdpPartitionOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            train_pct: 70.0,
            test_pct: 20.0,
            validate_pct: 10.0,
            seed: Some(2026),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AdpOptions {
    pub objective: AdpObjective,
    pub target_variable: Option<String>,
    pub input_variables: Vec<String>,
    pub weight_variable: Option<String>,
    pub screening: AdpScreeningOptions,
    pub outliers: AdpOutlierOptions,
    pub rescale: AdpRescaleMethod,
    pub categorical: AdpCategoricalOptions,
    pub binning: AdpBinningOptions,
    pub interactions: AdpInteractionOptions,
    pub partition: AdpPartitionOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdpFieldAction {
    pub variable_name: String,
    pub is_screened_out: bool,
    pub screening_reason: Option<String>,
    pub action_type: String,
    pub new_field_name: Option<String>,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdpPredictivePower {
    pub variable_name: String,
    pub method: String,
    pub score: f64,
    pub p_value: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PartitionCounts {
    pub train_count: usize,
    pub test_count: usize,
    pub validate_count: usize,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformedColumn {
    pub name: String,
    pub original_name: String,
    pub values: Vec<RawValue>,
    pub labels: Option<ValueLabels>,
    pub is_partition: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdpResult {
    pub target_variable: Option<String>,
    pub objective: AdpObjective,
    pub total_inputs: usize,
    pub screened_out_count: usize,
    pub recommended_count: usize,
    pub field_actions: Vec<AdpFieldAction>,
    pub predictive_powers: Vec<AdpPredictivePower>,
    pub partition_counts: Option<PartitionCounts>,
    pub transformed_columns: Vec<TransformedColumn>,
}
