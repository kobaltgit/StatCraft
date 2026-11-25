use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RowOrder {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ExactMode {
    #[default]
    AsymptoticOnly,
    MonteCarlo {
        confidence: f64,
        samples: usize,
    },
    Exact {
        time_limit_min: Option<u32>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum NonIntegerWeights {
    #[default]
    RoundCell,
    TruncateCell,
    RoundCase,
    TruncateCase,
    NoAdjust,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CrosstabOptions {
    pub observed: bool,
    pub expected: bool,
    pub hide_small_counts: bool,
    pub hide_small_threshold: usize,
    pub compare_col_proportions: bool,
    pub bonferroni_adjust: bool,
    pub row_pct: bool,
    pub col_pct: bool,
    pub total_pct: bool,
    pub residuals: bool,
    pub resid_unstandardized: bool,
    pub resid_standardized: bool,
    pub resid_adjusted: bool,
    pub apa_style: bool,
    pub noninteger_weights: NonIntegerWeights,
    pub row_order: RowOrder,
    pub show_layers_in_layers: bool,
    pub display_clustered_bar_charts: bool,
    pub suppress_tables: bool,
    pub exact_mode: ExactMode,
    pub chisq: bool,
    pub correlations: bool,
    pub contingency_coef: bool,
    pub phi_cramer: bool,
    pub lambda: bool,
    pub uncertainty_coef: bool,
    pub gamma: bool,
    pub somers_d: bool,
    pub kendall_tau_b: bool,
    pub kendall_tau_c: bool,
    pub eta: bool,
    pub kappa: bool,
    pub risk: bool,
    pub mcnemar: bool,
    pub cochran_mantel_haenszel: bool,
    pub cmh_common_odds_ratio: f64,
}

impl Default for CrosstabOptions {
    fn default() -> Self {
        Self {
            observed: true,
            expected: true,
            hide_small_counts: false,
            hide_small_threshold: 5,
            compare_col_proportions: false,
            bonferroni_adjust: false,
            row_pct: true,
            col_pct: true,
            total_pct: true,
            residuals: true,
            resid_unstandardized: false,
            resid_standardized: false,
            resid_adjusted: false,
            apa_style: false,
            noninteger_weights: NonIntegerWeights::RoundCell,
            row_order: RowOrder::Ascending,
            show_layers_in_layers: true,
            display_clustered_bar_charts: false,
            suppress_tables: false,
            exact_mode: ExactMode::AsymptoticOnly,
            chisq: true,
            correlations: false,
            contingency_coef: false,
            phi_cramer: true,
            lambda: false,
            uncertainty_coef: false,
            gamma: false,
            somers_d: false,
            kendall_tau_b: false,
            kendall_tau_c: false,
            eta: false,
            kappa: false,
            risk: false,
            mcnemar: false,
            cochran_mantel_haenszel: false,
            cmh_common_odds_ratio: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct CrosstabCell {
    pub observed: usize,
    pub observed_f64: f64,
    pub suppressed: bool,
    pub expected: Option<f64>,
    pub row_pct: Option<f64>,
    pub col_pct: Option<f64>,
    pub total_pct: Option<f64>,
    pub residual: Option<f64>,
    pub std_residual: Option<f64>,
    pub adj_residual: Option<f64>,
    pub col_prop_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CrosstabCategory {
    pub value: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CrosstabRow {
    pub category: CrosstabCategory,
    pub cells: Vec<CrosstabCell>,
    pub row_total: usize,
    pub row_total_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct ChiSquareTestRow {
    pub name: String,
    pub value: f64,
    pub df: Option<usize>,
    pub asymp_sig: Option<f64>,
    pub exact_sig: Option<f64>,
    pub monte_carlo_sig: Option<f64>,
    pub mc_ci_lower: Option<f64>,
    pub mc_ci_upper: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct AssociationMeasure {
    pub name: String,
    pub category: String,
    pub value: f64,
    pub asymp_std_error: Option<f64>,
    pub approx_t: Option<f64>,
    pub approx_sig: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DirectionalMeasure {
    pub name: String,
    pub direction: String,
    pub value: f64,
    pub asymp_std_error: Option<f64>,
    pub approx_t: Option<f64>,
    pub approx_sig: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RiskEstimate {
    pub name: String,
    pub value: f64,
    pub ci_lower: Option<f64>,
    pub ci_upper: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CmhResult {
    pub cochran_stat: f64,
    pub cochran_df: usize,
    pub cochran_sig: f64,
    pub mh_stat: f64,
    pub mh_df: usize,
    pub mh_sig: f64,
    pub breslow_day_stat: f64,
    pub breslow_day_df: usize,
    pub breslow_day_sig: f64,
    pub tarone_stat: f64,
    pub tarone_df: usize,
    pub tarone_sig: f64,
    pub common_or_estimate: f64,
    pub common_or_ln_se: f64,
    pub common_or_ci_lower: f64,
    pub common_or_ci_upper: f64,
    pub common_or_test_val: f64,
    pub common_or_sig: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CrosstabLayerKey {
    pub var_name: String,
    pub value: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct CrosstabResult {
    pub row_var: String,
    pub col_var: String,
    pub layer_keys: Vec<CrosstabLayerKey>,
    pub n_valid: usize,
    pub n_missing: usize,
    pub col_categories: Vec<CrosstabCategory>,
    pub rows: Vec<CrosstabRow>,
    pub col_totals: Vec<usize>,
    pub col_total_pcts: Option<Vec<f64>>,
    pub grand_total: usize,
    pub chi_square_tests: Vec<ChiSquareTestRow>,
    pub association_measures: Vec<AssociationMeasure>,
    pub directional_measures: Vec<DirectionalMeasure>,
    pub risk_estimates: Vec<RiskEstimate>,
    pub mcnemar_sig: Option<f64>,
    pub cmh_result: Option<CmhResult>,
    pub options: CrosstabOptions,
}

#[derive(Debug, Clone)]
pub struct RawCrosstabItem {
    pub row_value: String,
    pub row_label: Option<String>,
    pub col_value: String,
    pub col_label: Option<String>,
    pub is_missing: bool,
    pub weight: f64,
}
