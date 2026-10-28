use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrosstabOptions {
    pub expected: bool,
    pub row_pct: bool,
    pub col_pct: bool,
    pub total_pct: bool,
    pub residuals: bool,
    pub chisq: bool,
    pub phi_cramer: bool,
}

impl Default for CrosstabOptions {
    fn default() -> Self {
        Self {
            expected: true,
            row_pct: true,
            col_pct: true,
            total_pct: true,
            residuals: true,
            chisq: true,
            phi_cramer: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrosstabCell {
    pub observed: usize,
    pub expected: Option<f64>,
    pub row_pct: Option<f64>,
    pub col_pct: Option<f64>,
    pub total_pct: Option<f64>,
    pub residual: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrosstabCategory {
    pub value: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrosstabRow {
    pub category: CrosstabCategory,
    pub cells: Vec<CrosstabCell>,
    pub row_total: usize,
    pub row_total_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChiSquareTestRow {
    pub name: String,
    pub value: f64,
    pub df: Option<usize>,
    pub asymp_sig: Option<f64>,
    pub exact_sig: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssociationMeasure {
    pub name: String,
    pub value: f64,
    pub approx_sig: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrosstabResult {
    pub row_var: String,
    pub col_var: String,
    pub n_valid: usize,
    pub n_missing: usize,
    pub col_categories: Vec<CrosstabCategory>,
    pub rows: Vec<CrosstabRow>,
    pub col_totals: Vec<usize>,
    pub col_total_pcts: Option<Vec<f64>>,
    pub grand_total: usize,
    pub chi_square_tests: Vec<ChiSquareTestRow>,
    pub association_measures: Vec<AssociationMeasure>,
    pub options: CrosstabOptions,
}

#[derive(Debug, Clone)]
pub struct RawCrosstabItem {
    pub row_value: String,
    pub row_label: Option<String>,
    pub col_value: String,
    pub col_label: Option<String>,
    pub is_missing: bool,
}
