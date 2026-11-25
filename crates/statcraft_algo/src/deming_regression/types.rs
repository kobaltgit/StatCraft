use crate::bootstrap::{BootstrapOptions, BootstrapSummary};
use serde::{Deserialize, Serialize};

/// Options for Orthogonal / Deming regression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemingRegressionOptions {
    /// Error variance ratio lambda = sigma_y^2 / sigma_x^2 (> 0).
    /// lambda = 1.0 corresponds to Orthogonal (Total Least Squares) regression.
    pub lambda: f64,
    /// Confidence interval level in (0, 1), e.g. 0.95.
    pub ci_level: f64,
}

impl Default for DemingRegressionOptions {
    fn default() -> Self {
        Self {
            lambda: 1.0,
            ci_level: 0.95,
        }
    }
}

/// Single valid observation with fitted value and orthogonal residual for plotting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemingPoint {
    pub x: f64,
    pub y: f64,
    pub fitted: f64,
    pub orthogonal_residual: f64,
}

/// Bootstrap summary for Deming regression parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemingBootstrap {
    pub options: BootstrapOptions,
    pub slope: Option<BootstrapSummary>,
    pub intercept: Option<BootstrapSummary>,
}

/// Complete output of Orthogonal / Deming regression with Jackknife SEs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemingRegressionResult {
    pub dep_var: String,
    pub indep_var: String,
    pub n_valid: usize,
    pub n_missing: usize,
    pub lambda: f64,
    pub slope: f64,
    pub intercept: f64,
    pub slope_se: f64,
    pub intercept_se: f64,
    pub slope_ci_lower: f64,
    pub slope_ci_upper: f64,
    pub intercept_ci_lower: f64,
    pub intercept_ci_upper: f64,
    pub orthogonal_residual_sd: f64,
    pub pearson_r: Option<f64>,
    #[serde(default)]
    pub points: Vec<DemingPoint>,
    #[serde(default)]
    pub saved_predicted_var: Option<String>,
    #[serde(default)]
    pub saved_residuals_var: Option<String>,
    pub options: DemingRegressionOptions,
    #[serde(default)]
    pub bootstrap: Option<DemingBootstrap>,
}
