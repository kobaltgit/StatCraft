use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_positive_values() -> Vec<String> {
    vec!["1".to_string()]
}

fn default_num_to_display() -> Option<usize> {
    Some(10)
}

fn default_true() -> bool {
    true
}

fn default_max_operations() -> usize {
    5_000_000
}

fn default_advanced_size() -> usize {
    18
}

fn default_advanced_winners() -> usize {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurfOptions {
    pub variables: Vec<String>,
    #[serde(default = "default_positive_values")]
    pub positive_values: Vec<String>,
    #[serde(default)]
    pub max_combinations: Option<usize>,
    #[serde(default = "default_num_to_display")]
    pub num_to_display: Option<usize>,
    #[serde(default)]
    pub min_positive_pct: Option<f64>,
    #[serde(default)]
    pub always_required: usize,
    #[serde(default)]
    pub removal_cycles: usize,
    #[serde(default)]
    pub show_only_max_combo: bool,
    #[serde(default = "default_true")]
    pub limit_exceeded_abort: bool,
    #[serde(default = "default_max_operations")]
    pub max_operations: usize,
    #[serde(default)]
    pub importance_weights: HashMap<String, f64>,
    #[serde(default = "default_true")]
    pub weights_strict: bool,
    #[serde(default = "default_true")]
    pub use_advanced_selection: bool,
    #[serde(default = "default_true")]
    pub advanced_auto: bool,
    #[serde(default = "default_advanced_size")]
    pub advanced_size: usize,
    #[serde(default = "default_advanced_winners")]
    pub advanced_winners: usize,
    #[serde(default = "default_true")]
    pub reach_and_frequency_chart: bool,
}

impl Default for TurfOptions {
    fn default() -> Self {
        Self {
            variables: Vec::new(),
            positive_values: default_positive_values(),
            max_combinations: None,
            num_to_display: default_num_to_display(),
            min_positive_pct: None,
            always_required: 0,
            removal_cycles: 0,
            show_only_max_combo: false,
            limit_exceeded_abort: true,
            max_operations: default_max_operations(),
            importance_weights: HashMap::new(),
            weights_strict: true,
            use_advanced_selection: true,
            advanced_auto: true,
            advanced_size: default_advanced_size(),
            advanced_winners: default_advanced_winners(),
            reach_and_frequency_chart: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurfCombinationRow {
    pub combo_size: usize,
    pub variables: Vec<String>,
    pub reach_count: f64,
    pub reach_pct: f64,
    pub frequency_count: f64,
    pub frequency_pct: f64,
    pub mean_responses_per_reached: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weighted_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurfCycleResult {
    pub cycle_index: usize,
    pub removed_variables: Vec<String>,
    pub rows_by_size: Vec<(usize, Vec<TurfCombinationRow>)>,
    pub used_heats: bool,
    pub estimated_operations: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurfResult {
    pub total_valid_cases: f64,
    pub total_cases_unweighted: usize,
    pub excluded_variables: Vec<(String, f64)>,
    pub cycles: Vec<TurfCycleResult>,
}
