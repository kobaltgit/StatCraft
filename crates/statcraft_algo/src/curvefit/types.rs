use serde::{Deserialize, Serialize};

/// Canonical curve models supported by CURVEFIT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurveModelType {
    Linear,
    Logarithmic,
    Inverse,
    Quadratic,
    Power,
    Exponential,
}

impl CurveModelType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Logarithmic => "Logarithmic",
            Self::Inverse => "Inverse",
            Self::Quadratic => "Quadratic",
            Self::Power => "Power",
            Self::Exponential => "Exponential",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Linear,
            Self::Logarithmic,
            Self::Inverse,
            Self::Quadratic,
            Self::Power,
            Self::Exponential,
        ]
    }
}

/// Options for curve estimation procedure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurveFitOptions {
    /// Selected models to fit
    pub models: Vec<CurveModelType>,
    /// Include intercept (constant) in equations
    pub include_constant: bool,
    /// Confidence level (default 0.95)
    pub confidence_level: f64,
}

impl Default for CurveFitOptions {
    fn default() -> Self {
        Self {
            models: CurveModelType::all(),
            include_constant: true,
            confidence_level: 0.95,
        }
    }
}

/// A row in the Model Description & Parameter Estimates summary table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurveModelSummaryRow {
    pub model_type: CurveModelType,
    pub r_squared: f64,
    pub f_stat: f64,
    pub df1: usize,
    pub df2: usize,
    pub sig: f64,
    pub b0: f64,
    pub b1: f64,
    pub b2: Option<f64>,
    pub n_cases: usize,
}

/// Complete result of curve estimation analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurveFitResult {
    pub dependent_var: String,
    pub independent_var: String,
    pub models: Vec<CurveModelSummaryRow>,
    pub warnings: Vec<String>,
}
