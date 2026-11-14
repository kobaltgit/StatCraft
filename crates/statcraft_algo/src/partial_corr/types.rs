use crate::correlations_types::{
    CorrelationMatrixResult, MissingHandling, SignificanceTest, VarDescriptives,
};
use serde::{Deserialize, Serialize};

/// Represents a single cell in a partial correlation matrix.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartialCorrCell {
    /// Partial correlation coefficient r
    pub r: Option<f64>,
    /// Significance (p-value)
    pub p_val: Option<f64>,
    /// Degrees of freedom (N - k - 2)
    pub df: usize,
    /// Sample size used for this cell
    pub n: usize,
}

impl PartialCorrCell {
    pub fn empty() -> Self {
        Self {
            r: None,
            p_val: None,
            df: 0,
            n: 0,
        }
    }
}

/// Options for partial correlations analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartialCorrOptions {
    /// Two-tailed or one-tailed test of significance
    pub significance: SignificanceTest,
    /// Missing value treatment (Listwise or Pairwise)
    pub missing: MissingHandling,
    /// Include descriptive statistics table
    pub descriptives: bool,
    /// Include zero-order (Pearson) correlations table
    pub zero_order_correlations: bool,
}

impl Default for PartialCorrOptions {
    fn default() -> Self {
        Self {
            significance: SignificanceTest::TwoTailed,
            missing: MissingHandling::Listwise,
            descriptives: false,
            zero_order_correlations: false,
        }
    }
}

/// Complete result of partial correlations analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartialCorrResult {
    /// Names of analyzed variables
    pub variables: Vec<String>,
    /// Names of controlling variables
    pub controls: Vec<String>,
    /// Matrix of partial correlation cells (variables x variables)
    pub cells: Vec<Vec<PartialCorrCell>>,
    /// Optional descriptive statistics for all variables
    pub descriptives: Option<Vec<VarDescriptives>>,
    /// Optional zero-order Pearson correlation matrix
    pub zero_order: Option<CorrelationMatrixResult>,
}
