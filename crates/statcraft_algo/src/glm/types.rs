use crate::anova_oneway::{AnovaGroupDescriptives, AnovaHomogeneityTest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SumOfSquaresType {
    Type1,
    Type2,
    #[default]
    Type3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GlmModelType {
    #[default]
    FullFactorial,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnivariateGlmOptions {
    pub dependent: String,
    pub fixed_factors: Vec<String>,
    pub covariates: Vec<String>,
    pub model_type: GlmModelType,
    pub custom_terms: Vec<Vec<String>>,
    pub sum_of_squares: SumOfSquaresType,
    pub intercept: bool,
    pub descriptives: bool,
    pub partial_eta_squared: bool,
    pub homogeneity_levene: bool,
    pub confidence_level: f64,
}

impl Default for UnivariateGlmOptions {
    fn default() -> Self {
        Self {
            dependent: String::new(),
            fixed_factors: Vec::new(),
            covariates: Vec::new(),
            model_type: GlmModelType::FullFactorial,
            custom_terms: Vec::new(),
            sum_of_squares: SumOfSquaresType::Type3,
            intercept: true,
            descriptives: true,
            partial_eta_squared: true,
            homogeneity_levene: false,
            confidence_level: 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BetweenSubjectsRow {
    pub source: String,
    pub type3_ss: f64,
    pub df: usize,
    pub mean_square: Option<f64>,
    pub f_stat: Option<f64>,
    pub p_value: Option<f64>,
    pub partial_eta_squared: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnivariateGlmResult {
    pub dependent: String,
    pub n_valid: usize,
    pub r_squared: f64,
    pub adjusted_r_squared: f64,
    pub tests: Vec<BetweenSubjectsRow>,
    pub levene_test: Option<AnovaHomogeneityTest>,
    pub group_descriptives: Vec<AnovaGroupDescriptives>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultivariateGlmOptions {
    pub dependents: Vec<String>,
    pub factors: Vec<String>,
    pub covariates: Vec<String>,
    pub intercept: bool,
    pub sum_of_squares: SumOfSquaresType,
    pub partial_eta_squared: bool,
}

impl Default for MultivariateGlmOptions {
    fn default() -> Self {
        Self {
            dependents: Vec::new(),
            factors: Vec::new(),
            covariates: Vec::new(),
            intercept: true,
            sum_of_squares: SumOfSquaresType::Type3,
            partial_eta_squared: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultivariateStatName {
    PillaiTrace,
    WilksLambda,
    HotellingTrace,
    RoyLargestRoot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultivariateEffectRow {
    pub effect: String,
    pub stat_name: MultivariateStatName,
    pub value: f64,
    pub f_stat: f64,
    pub hypothesis_df: f64,
    pub error_df: f64,
    pub p_value: f64,
    pub partial_eta_squared: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultivariateGlmResult {
    pub dependents: Vec<String>,
    pub n_valid: usize,
    pub effects: Vec<MultivariateEffectRow>,
    pub between_subjects: Vec<UnivariateGlmResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatedGlmOptions {
    pub factor_name: String,
    pub measure_variables: Vec<String>,
    pub between_factors: Vec<String>,
    pub sum_of_squares: SumOfSquaresType,
    pub partial_eta_squared: bool,
}

impl Default for RepeatedGlmOptions {
    fn default() -> Self {
        Self {
            factor_name: "time".to_string(),
            measure_variables: Vec::new(),
            between_factors: Vec::new(),
            sum_of_squares: SumOfSquaresType::Type3,
            partial_eta_squared: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SphericityTestResult {
    pub within_subject_effect: String,
    pub mauchly_w: f64,
    pub approx_chi_square: f64,
    pub df: usize,
    pub p_value: f64,
    pub greenhouse_geisser: f64,
    pub huynh_feldt: f64,
    pub lower_bound: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WithinSubjectsEffectRow {
    pub source: String,
    pub sphericity_assumed_ss: f64,
    pub sphericity_assumed_df: f64,
    pub sphericity_assumed_ms: f64,
    pub sphericity_assumed_f: Option<f64>,
    pub sphericity_assumed_p: Option<f64>,
    pub greenhouse_geisser_ss: f64,
    pub greenhouse_geisser_df: f64,
    pub greenhouse_geisser_ms: f64,
    pub greenhouse_geisser_f: Option<f64>,
    pub greenhouse_geisser_p: Option<f64>,
    pub huynh_feldt_ss: f64,
    pub huynh_feldt_df: f64,
    pub huynh_feldt_ms: f64,
    pub huynh_feldt_f: Option<f64>,
    pub huynh_feldt_p: Option<f64>,
    pub lower_bound_ss: f64,
    pub lower_bound_df: f64,
    pub lower_bound_ms: f64,
    pub lower_bound_f: Option<f64>,
    pub lower_bound_p: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepeatedGlmResult {
    pub factor_name: String,
    pub measure_variables: Vec<String>,
    pub n_valid: usize,
    pub sphericity: SphericityTestResult,
    pub within_subjects_tests: Vec<WithinSubjectsEffectRow>,
    pub error_row: WithinSubjectsEffectRow,
}
