use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrelationType {
    Pearson,
    Spearman,
    KendallTauB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignificanceTest {
    TwoTailed,
    OneTailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingHandling {
    Pairwise,
    Listwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignificanceFlag {
    None,
    SingleStar,
    DoubleStar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorrelationCell {
    pub r: Option<f64>,
    pub sig: Option<f64>,
    pub n: usize,
    pub ss_cross_prod: Option<f64>,
    pub covariance: Option<f64>,
    pub flag: SignificanceFlag,
}

impl CorrelationCell {
    pub fn empty() -> Self {
        Self {
            r: None,
            sig: None,
            n: 0,
            ss_cross_prod: None,
            covariance: None,
            flag: SignificanceFlag::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VarDescriptives {
    pub var_name: String,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub n: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorrelationMatrixResult {
    pub corr_type: CorrelationType,
    pub variables: Vec<String>,
    pub cells: Vec<Vec<CorrelationCell>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BivariateCorrelationsOptions {
    pub pearson: bool,
    pub spearman: bool,
    pub kendall: bool,
    pub significance: SignificanceTest,
    pub flag_significant: bool,
    pub descriptives: bool,
    pub cross_prod_cov: bool,
    pub missing: MissingHandling,
}

impl Default for BivariateCorrelationsOptions {
    fn default() -> Self {
        Self {
            pearson: true,
            spearman: false,
            kendall: false,
            significance: SignificanceTest::TwoTailed,
            flag_significant: true,
            descriptives: false,
            cross_prod_cov: false,
            missing: MissingHandling::Pairwise,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BivariateCorrelationsResult {
    pub descriptives: Option<Vec<VarDescriptives>>,
    pub matrices: Vec<CorrelationMatrixResult>,
    pub options: BivariateCorrelationsOptions,
}

pub type VariableInput<'a> = (&'a str, &'a [Option<f64>]);
