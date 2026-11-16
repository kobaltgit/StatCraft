use serde::{Deserialize, Serialize};

/// Method for handling tied event times in Cox regression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TieMethod {
    /// Breslow approximation (default in IBM SPSS Statistics).
    #[default]
    Breslow,
    /// Efron approximation (closer to continuous time partial likelihood).
    Efron,
}

/// Options for configuring Cox Proportional Hazards Regression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxRegressionOptions {
    /// Confidence level for hazard ratio intervals (default: 0.95).
    pub confidence_level: f64,
    /// Method for handling tied survival times.
    pub tie_method: TieMethod,
    /// Maximum number of Newton-Raphson iterations (default: 20, matching SPSS).
    pub max_iterations: usize,
    /// Convergence tolerance for parameter changes (default: 1e-6).
    pub convergence_tol: f64,
    /// Whether to compute baseline hazard and survival functions.
    pub print_baseline: bool,
    /// Whether to compute confidence intervals for Exp(B).
    pub print_ci: bool,
}

impl Default for CoxRegressionOptions {
    fn default() -> Self {
        Self {
            confidence_level: 0.95,
            tie_method: TieMethod::Breslow,
            max_iterations: 20,
            convergence_tol: 1e-6,
            print_baseline: true,
            print_ci: true,
        }
    }
}

/// A single observation input for Cox regression.
#[derive(Debug, Clone, PartialEq)]
pub struct CoxDataPoint {
    pub time: f64,
    pub is_event: bool,
    pub stratum: Option<String>,
    pub covariates: Vec<f64>,
}

/// Summary of cases (total, events, censored) for a stratum or overall.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxCaseSummary {
    pub stratum: Option<String>,
    pub total_n: usize,
    pub events_n: usize,
    pub censored_n: usize,
    pub censored_percent: f64,
}

/// Omnibus tests of model coefficients (-2 Log Likelihood, Chi-Square, df, Sig.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OmnibusTestsResult {
    pub initial_minus_2ll: f64,
    pub final_minus_2ll: f64,
    pub chi_square: f64,
    pub df: usize,
    pub p_value: f64,
}

/// Estimated regression coefficient and statistics for one covariate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxVariableInEquation {
    pub name: String,
    pub b: f64,
    pub se: f64,
    pub wald: f64,
    pub df: usize,
    pub p_value: f64,
    pub exp_b: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
}

/// Estimated baseline cumulative hazard and survival at a specific time point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineSurvivalPoint {
    pub stratum: Option<String>,
    pub time: f64,
    pub baseline_cum_hazard: f64,
    pub baseline_survival: f64,
    pub mean_covariates_survival: f64,
}

/// Full output of the Cox Proportional Hazards Regression procedure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxRegressionResult {
    pub time_variable: String,
    pub status_variable: String,
    pub strata_variable: Option<String>,
    pub covariate_names: Vec<String>,
    pub covariate_means: Vec<f64>,
    pub case_summaries: Vec<CoxCaseSummary>,
    pub overall_summary: CoxCaseSummary,
    pub omnibus_tests: OmnibusTestsResult,
    pub variables: Vec<CoxVariableInEquation>,
    pub baseline_survival: Vec<BaselineSurvivalPoint>,
    pub tie_method: TieMethod,
    pub iterations: usize,
    pub converged: bool,
}
