use serde::{Deserialize, Serialize};

/// Options for binary logistic regression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogisticRegressionOptions {
    /// Include intercept (constant) in the model
    pub include_constant: bool,
    /// Probability cutoff for classification table (default 0.5)
    pub classification_cutoff: f64,
    /// Confidence level for Exp(B) interval (default 0.95)
    pub confidence_level: f64,
    /// Compute Hosmer-Lemeshow goodness-of-fit test
    pub hosmer_lemeshow: bool,
    /// Maximum number of IRLS iterations (default 20)
    pub max_iterations: usize,
    /// Convergence tolerance (default 1e-6)
    pub convergence_tol: f64,
}

impl Default for LogisticRegressionOptions {
    fn default() -> Self {
        Self {
            include_constant: true,
            classification_cutoff: 0.5,
            confidence_level: 0.95,
            hosmer_lemeshow: true,
            max_iterations: 20,
            convergence_tol: 1e-6,
        }
    }
}

/// A row in the Variables in the Equation table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogisticCoefficientRow {
    pub param_name: String,
    pub b: f64,
    pub se: f64,
    pub wald: f64,
    pub df: usize,
    pub sig: f64,
    pub exp_b: f64,
    pub lower_ci: f64,
    pub upper_ci: f64,
}

/// Omnibus tests of model coefficients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OmnibusTestsRow {
    pub chi_square: f64,
    pub df: usize,
    pub sig: f64,
}

/// Model summary statistics (-2LL, Cox & Snell, Nagelkerke).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogisticModelSummary {
    pub minus_2ll: f64,
    pub cox_snell_r2: f64,
    pub nagelkerke_r2: f64,
}

/// A row in the Classification Table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClassificationTableRow {
    pub observed_label: String,
    pub pred_zero: usize,
    pub pred_one: usize,
    pub percent_correct: f64,
}

/// Complete Classification Table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClassificationTable {
    pub rows: Vec<ClassificationTableRow>,
    pub overall_percent_correct: f64,
    pub cutoff: f64,
}

/// Contingency table group for Hosmer-Lemeshow test.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HosmerLemeshowGroup {
    pub group: usize,
    pub total: usize,
    pub observed_one: usize,
    pub expected_one: f64,
    pub observed_zero: usize,
    pub expected_zero: f64,
}

/// Hosmer-Lemeshow goodness-of-fit test result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HosmerLemeshowResult {
    pub chi_square: f64,
    pub df: usize,
    pub sig: f64,
    pub contingency_table: Vec<HosmerLemeshowGroup>,
}

/// Complete result of binary logistic regression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogisticRegressionResult {
    pub dependent_var: String,
    pub covariates: Vec<String>,
    pub coefficients: Vec<LogisticCoefficientRow>,
    pub omnibus: OmnibusTestsRow,
    pub model_summary: LogisticModelSummary,
    pub classification_table: ClassificationTable,
    pub hosmer_lemeshow: Option<HosmerLemeshowResult>,
    pub n_cases: usize,
    pub iterations: usize,
    pub converged: bool,
    pub warnings: Vec<String>,
}
