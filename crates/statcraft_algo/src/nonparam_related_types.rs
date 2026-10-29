use crate::nonparam_twosample_types::RankGroupStats;
use serde::{Deserialize, Serialize};

/// Detailed rank counts for Wilcoxon signed-rank test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WilcoxonRanksSummary {
    pub negative_ranks: RankGroupStats,
    pub positive_ranks: RankGroupStats,
    pub ties_count: usize,
    pub total_n: usize,
}

/// Result of Wilcoxon signed-rank test for paired samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WilcoxonSignedRankResult {
    pub var1: String,
    pub var2: String,
    pub ranks: WilcoxonRanksSummary,
    pub test_statistic_w: f64,
    pub z_stat: f64,
    pub asymp_sig_2tailed: f64,
    pub based_on_negative: bool,
}

/// Frequency counts of differences for Sign Test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignTestDifferences {
    pub negative_count: usize,
    pub positive_count: usize,
    pub ties_count: usize,
    pub total_n: usize,
}

/// Result of Sign Test for paired samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignTestResult {
    pub var1: String,
    pub var2: String,
    pub diffs: SignTestDifferences,
    pub exact_sig_2tailed: f64,
}

/// Combined result for two related samples analysis on a pair of variables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedPairTestResult {
    pub var1: String,
    pub var2: String,
    pub wilcoxon: Option<WilcoxonSignedRankResult>,
    pub sign: Option<SignTestResult>,
}
