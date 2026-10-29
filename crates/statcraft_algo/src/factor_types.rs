use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FactorExtractionMethod {
    #[default]
    PC,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FactorExtractionCriteria {
    Mineigen(f64),
    Factors(usize),
}

impl Default for FactorExtractionCriteria {
    fn default() -> Self {
        Self::Mineigen(1.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactorOptions {
    pub extraction: FactorExtractionMethod,
    pub criteria: FactorExtractionCriteria,
    pub kmo_bartlett: bool,
}

impl Default for FactorOptions {
    fn default() -> Self {
        Self {
            extraction: FactorExtractionMethod::PC,
            criteria: FactorExtractionCriteria::Mineigen(1.0),
            kmo_bartlett: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KmoBartlettResult {
    pub kmo: f64,
    pub bartlett_approx_chi_square: f64,
    pub bartlett_df: usize,
    pub bartlett_sig: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommunalityRow {
    pub variable: String,
    pub initial: f64,
    pub extraction: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarianceExplainedRow {
    pub component: usize,
    pub initial_total: f64,
    pub initial_pct_variance: f64,
    pub initial_cumulative_pct: f64,
    pub extraction_total: Option<f64>,
    pub extraction_pct_variance: Option<f64>,
    pub extraction_cumulative_pct: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentMatrixRow {
    pub variable: String,
    pub loadings: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactorAnalysisResult {
    pub variables: Vec<String>,
    pub n_cases: usize,
    pub n_total: usize,
    pub kmo_bartlett: Option<KmoBartlettResult>,
    pub communalities: Vec<CommunalityRow>,
    pub total_variance_explained: Vec<VarianceExplainedRow>,
    pub component_matrix: Vec<ComponentMatrixRow>,
    pub n_components_extracted: usize,
}
