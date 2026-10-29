use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinearRegressionOptions {
    pub ci_level: f64,
    pub collinearity: bool,
}

impl Default for LinearRegressionOptions {
    fn default() -> Self {
        Self {
            ci_level: 0.95,
            collinearity: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionAnovaRow {
    pub sum_of_squares: f64,
    pub df: usize,
    pub mean_square: Option<f64>,
    pub f_stat: Option<f64>,
    pub p_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionAnova {
    pub regression: RegressionAnovaRow,
    pub residual: RegressionAnovaRow,
    pub total: RegressionAnovaRow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionCoefficient {
    pub var_name: String,
    pub b: f64,
    pub std_error: f64,
    pub beta: Option<f64>,
    pub t_stat: f64,
    pub p_value: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
    pub tolerance: Option<f64>,
    pub vif: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinearRegressionResult {
    pub dep_var: String,
    pub indep_vars: Vec<String>,
    pub method: String,
    pub n_valid: usize,
    pub n_missing: usize,
    pub r: f64,
    pub r_squared: f64,
    pub adj_r_squared: f64,
    pub std_error_estimate: f64,
    pub anova: RegressionAnova,
    pub coefficients: Vec<RegressionCoefficient>,
    pub options: LinearRegressionOptions,
}
