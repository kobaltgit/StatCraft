use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ReliabilityModel {
    #[default]
    Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReliabilityOptions {
    pub scale_name: String,
    pub model: ReliabilityModel,
    pub item_statistics: bool,
    pub scale_statistics: bool,
    pub scale_if_item_deleted: bool,
}

impl Default for ReliabilityOptions {
    fn default() -> Self {
        Self {
            scale_name: "ALL VARIABLES".to_string(),
            model: ReliabilityModel::Alpha,
            item_statistics: true,
            scale_statistics: true,
            scale_if_item_deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseProcessingSummary {
    pub n_valid: usize,
    pub percent_valid: f64,
    pub n_excluded: usize,
    pub percent_excluded: f64,
    pub n_total: usize,
    pub percent_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReliabilityStatistics {
    pub cronbach_alpha: Option<f64>,
    pub cronbach_alpha_standardized: Option<f64>,
    pub n_of_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ItemStatistic {
    pub item_name: String,
    pub mean: f64,
    pub std_dev: f64,
    pub n: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ItemTotalStatistic {
    pub item_name: String,
    pub scale_mean_if_deleted: f64,
    pub scale_variance_if_deleted: f64,
    pub corrected_item_total_correlation: Option<f64>,
    pub cronbach_alpha_if_deleted: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScaleStatistics {
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub n_of_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReliabilityResult {
    pub scale_name: String,
    pub model: ReliabilityModel,
    pub case_processing: CaseProcessingSummary,
    pub reliability_stats: ReliabilityStatistics,
    pub item_stats: Vec<ItemStatistic>,
    pub item_total_stats: Vec<ItemTotalStatistic>,
    pub scale_stats: Option<ScaleStatistics>,
}
