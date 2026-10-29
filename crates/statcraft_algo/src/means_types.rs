use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeansCellStats {
    pub group_label: String,
    pub count: usize,
    pub weighted_count: f64,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub variance: Option<f64>,
    pub median: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub sum: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeansCaseProcessingSummary {
    pub dep_var: String,
    pub factor_var: String,
    pub included_n: usize,
    pub included_percent: f64,
    pub excluded_n: usize,
    pub excluded_percent: f64,
    pub total_n: usize,
    pub total_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeansAnovaRow {
    pub source: String,
    pub sum_of_squares: f64,
    pub df: usize,
    pub mean_square: Option<f64>,
    pub f_stat: Option<f64>,
    pub p_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeansAnovaTable {
    pub between_groups: MeansAnovaRow,
    pub within_groups: MeansAnovaRow,
    pub total: MeansAnovaRow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeansAssociationMeasures {
    pub eta: f64,
    pub eta_squared: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeansTableResult {
    pub dep_var: String,
    pub factor_var: String,
    pub summary: MeansCaseProcessingSummary,
    pub cells: Vec<MeansCellStats>,
    pub total: MeansCellStats,
    pub anova: Option<MeansAnovaTable>,
    pub association: Option<MeansAssociationMeasures>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeansProcedureResult {
    pub tables: Vec<MeansTableResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeansOptions {
    pub cell_mean: bool,
    pub cell_count: bool,
    pub cell_stddev: bool,
    pub cell_variance: bool,
    pub cell_median: bool,
    pub cell_min: bool,
    pub cell_max: bool,
    pub anova: bool,
}

impl Default for MeansOptions {
    fn default() -> Self {
        Self {
            cell_mean: true,
            cell_count: true,
            cell_stddev: true,
            cell_variance: false,
            cell_median: true,
            cell_min: false,
            cell_max: false,
            anova: true,
        }
    }
}
