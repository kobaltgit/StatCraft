use crate::bootstrap::{BootstrapOptions, BootstrapSummary};
use serde::{Deserialize, Serialize};

/// Maximum sample size for exact O(N^2) pairwise slope calculation.
pub const MAX_EXACT_PAIRS_N: usize = 5000;

/// Options for Theil-Sen robust simple linear regression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TheilSenOptions {
    pub ci_level: f64,
    pub mann_kendall: bool,
}

impl Default for TheilSenOptions {
    fn default() -> Self {
        Self {
            ci_level: 0.95,
            mann_kendall: true,
        }
    }
}

/// Single valid observation with fitted value and residual for plotting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TheilSenPoint {
    pub x: f64,
    pub y: f64,
    pub fitted: f64,
    pub residual: f64,
}

/// Bootstrap summary for Theil-Sen regression parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TheilSenBootstrap {
    pub options: BootstrapOptions,
    pub slope: Option<BootstrapSummary>,
    pub intercept: Option<BootstrapSummary>,
}

/// Complete output of Theil-Sen regression and optional Mann-Kendall test.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TheilSenResult {
    pub dep_var: String,
    pub indep_var: String,
    pub n_valid: usize,
    pub n_missing: usize,
    pub slope: f64,
    pub intercept: f64,
    pub slope_ci_lower: f64,
    pub slope_ci_upper: f64,
    pub ci_level: f64,
    pub kendall_tau: Option<f64>,
    pub z_stat: Option<f64>,
    pub p_value: Option<f64>,
    pub mad_residuals: f64,
    #[serde(default)]
    pub points: Vec<TheilSenPoint>,
    #[serde(default)]
    pub saved_predicted_var: Option<String>,
    #[serde(default)]
    pub saved_residuals_var: Option<String>,
    pub options: TheilSenOptions,
    #[serde(default)]
    pub bootstrap: Option<TheilSenBootstrap>,
}
