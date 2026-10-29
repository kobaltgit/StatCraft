use serde::{Deserialize, Serialize};

/// Statistics for a single group in rank-based tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankGroupStats {
    pub group_val: String,
    pub n: usize,
    pub mean_rank: f64,
    pub sum_ranks: f64,
}

/// Result of Mann-Whitney U test for two independent samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MannWhitneyResult {
    pub variable: String,
    pub grouping_var: String,
    pub group1_stats: RankGroupStats,
    pub group2_stats: RankGroupStats,
    pub total_n: usize,
    pub mann_whitney_u: f64,
    pub wilcoxon_w: f64,
    pub z_stat: f64,
    pub asymp_sig_2tailed: f64,
}

pub use crate::nonparam_related_types::{WilcoxonRanksSummary, WilcoxonSignedRankResult};
