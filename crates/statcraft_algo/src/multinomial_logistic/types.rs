use serde::{Deserialize, Serialize};

/// Policy for choosing the reference (baseline) category for the multinomial dependent variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReferenceCategory {
    /// Use the last (highest) category value as reference (IBM SPSS default).
    Last,
    /// Use the first (lowest) category value as reference.
    First,
    /// Use an explicit category value as reference.
    Value(f64),
}

/// Configuration options for multinomial logistic regression (NOMREG).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultinomialLogisticOptions {
    pub reference_category: ReferenceCategory,
    pub include_constant: bool,
    pub confidence_level: f64,
    pub max_iterations: usize,
    pub convergence_tol: f64,
    pub print_classification: bool,
    pub print_goodness_of_fit: bool,
    pub print_lrt: bool,
}

impl Default for MultinomialLogisticOptions {
    fn default() -> Self {
        Self {
            reference_category: ReferenceCategory::Last,
            include_constant: true,
            confidence_level: 0.95,
            max_iterations: 50,
            convergence_tol: 1e-6,
            print_classification: true,
            print_goodness_of_fit: true,
            print_lrt: true,
        }
    }
}

/// Case processing summary for a single outcome category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseProcessingCategory {
    pub category_value: f64,
    pub category_label: Option<String>,
    pub n: usize,
    pub percent: f64,
}

/// Case processing summary table (N and marginal percentage for each category).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultinomialCaseProcessingSummary {
    pub categories: Vec<CaseProcessingCategory>,
    pub valid_n: usize,
    pub valid_percent: f64,
    pub missing_n: usize,
    pub total_n: usize,
}

/// Model fitting information (-2 Log Likelihood, Chi-Square, df, Sig., AIC, BIC).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelFittingInfo {
    pub intercept_only_minus_2ll: f64,
    pub final_minus_2ll: f64,
    pub chi_square: f64,
    pub df: usize,
    pub sig: f64,
    pub aic: f64,
    pub bic: f64,
}

/// Goodness-of-Fit tests (Pearson Chi-Square and Deviance).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoodnessOfFit {
    pub pearson_chi_square: f64,
    pub pearson_df: usize,
    pub pearson_sig: f64,
    pub deviance_chi_square: f64,
    pub deviance_df: usize,
    pub deviance_sig: f64,
}

/// Pseudo R-Square measures (Cox and Snell, Nagelkerke, McFadden).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PseudoRSquared {
    pub cox_and_snell: f64,
    pub nagelkerke: f64,
    pub mcfadden: f64,
}

/// Likelihood Ratio Test for an individual effect (factor or covariate).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikelihoodRatioTestRow {
    pub effect_name: String,
    pub model_fitting_criteria_minus_2ll: f64,
    pub chi_square: f64,
    pub df: usize,
    pub sig: f64,
}

/// Single parameter estimate row in the Multinomial Logistic Parameter Estimates table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterEstimateRow {
    pub category_value: f64,
    pub category_label: Option<String>,
    pub param_name: String,
    pub b: f64,
    pub std_err: f64,
    pub wald: f64,
    pub df: usize,
    pub sig: f64,
    pub exp_b: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
    pub is_reference_category: bool,
}

/// Row in the classification table (observed vs predicted outcomes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultinomialClassificationTableRow {
    pub observed_category: f64,
    pub observed_label: Option<String>,
    pub predicted_counts: Vec<usize>,
    pub percent_correct: f64,
}

/// Overall classification table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultinomialClassificationTable {
    pub categories: Vec<f64>,
    pub category_labels: Vec<Option<String>>,
    pub rows: Vec<MultinomialClassificationTableRow>,
    pub overall_percent: f64,
}

/// Complete result structure for Multinomial Logistic Regression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultinomialLogisticResult {
    pub dependent_var: String,
    pub factors: Vec<String>,
    pub covariates: Vec<String>,
    pub reference_category_val: f64,
    pub categories: Vec<f64>,
    pub case_processing: MultinomialCaseProcessingSummary,
    pub model_fitting: ModelFittingInfo,
    pub goodness_of_fit: Option<GoodnessOfFit>,
    pub pseudo_r_squared: PseudoRSquared,
    pub likelihood_ratio_tests: Vec<LikelihoodRatioTestRow>,
    pub parameter_estimates: Vec<ParameterEstimateRow>,
    pub classification_table: Option<MultinomialClassificationTable>,
    pub n_cases: usize,
    pub iterations: usize,
    pub converged: bool,
    pub warnings: Vec<String>,
}
