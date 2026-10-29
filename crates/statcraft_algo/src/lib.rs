pub mod anova_oneway;
pub mod correlations;
#[cfg(test)]
mod correlations_tests;
pub mod correlations_types;
pub mod crosstabs;
#[cfg(test)]
mod crosstabs_tests;
pub mod crosstabs_types;
pub mod descriptives;
pub mod expr_eval;
pub mod frequencies;
pub mod kruskal_wallis;
#[cfg(test)]
mod kruskal_wallis_tests;
pub mod nonparam_related;
#[cfg(test)]
mod nonparam_related_tests;
pub mod nonparam_related_types;
pub mod nonparam_twosample;
#[cfg(test)]
mod nonparam_twosample_tests;
pub mod nonparam_twosample_types;
pub mod recode;
pub mod regression;
#[cfg(test)]
mod regression_tests;
pub mod regression_types;
pub mod reliability;
#[cfg(test)]
mod reliability_tests;
pub mod reliability_types;
pub mod sort_cases;
pub mod split_groups;
pub mod ttest_indep;
pub mod ttest_onesample;
pub mod ttest_paired;

pub use anova_oneway::{
    compute_oneway_anova, compute_oneway_anova_from_slices, f_distribution_sf, AlgoError,
    AnovaGroupDescriptives, AnovaHomogeneityTest, AnovaTableRow, OneWayAnovaOptions,
    OneWayAnovaResult,
};
pub use correlations::{
    compute_bivariate_correlations, BivariateCorrelationsOptions, BivariateCorrelationsResult,
    CorrelationCell, CorrelationMatrixResult, CorrelationType, MissingHandling, SignificanceFlag,
    SignificanceTest, VarDescriptives, VariableInput,
};
pub use crosstabs::{chi_square_sf, compute_crosstabs, fisher_exact_2x2};
pub use crosstabs_types::{
    AssociationMeasure, ChiSquareTestRow, CrosstabCategory, CrosstabCell, CrosstabOptions,
    CrosstabResult, CrosstabRow, RawCrosstabItem,
};
pub use descriptives::{
    compute_descriptives, compute_descriptives_weighted, DescriptivesOptions, DescriptivesResult,
};
pub use expr_eval::{CompiledExpr, EvalError, EvalNode, Op};
pub use frequencies::{
    compute_frequencies, compute_frequencies_detailed, compute_percentile, compute_quartiles,
    FrequencyCategory, FrequencyInputItem, FrequencyResult, FrequencyTable,
};
pub use kruskal_wallis::{
    compute_kruskal_wallis, compute_kruskal_wallis_from_groups, KruskalWallisGroupStats,
    KruskalWallisResult,
};
pub use nonparam_related::{
    compute_related_samples, compute_sign_test, compute_sign_test_from_slices,
    compute_wilcoxon_signed_rank, compute_wilcoxon_signed_rank_from_slices,
};
pub use nonparam_related_types::{
    RelatedPairTestResult, SignTestDifferences, SignTestResult, WilcoxonRanksSummary,
    WilcoxonSignedRankResult,
};
pub use nonparam_twosample::{
    compute_mann_whitney, compute_mann_whitney_from_slices, compute_midranks_with_ties,
};
pub use nonparam_twosample_types::{MannWhitneyResult, RankGroupStats};
pub use recode::{apply_recode, NewValueSpec, OldValueRule, RecodeMapping, RecodePlan};
pub use regression::{compute_linear_regression, invert_matrix};
pub use regression_types::{
    LinearRegressionOptions, LinearRegressionResult, RegressionAnova, RegressionAnovaRow,
    RegressionCoefficient,
};
pub use reliability::compute_reliability;
pub use reliability_types::{
    CaseProcessingSummary, ItemStatistic, ItemTotalStatistic, ReliabilityModel, ReliabilityOptions,
    ReliabilityResult, ReliabilityStatistics, ScaleStatistics,
};
pub use sort_cases::{
    compare_raw_values, compute_sort_permutation, invert_permutation, SortDirection, SortKeySpec,
};
pub use split_groups::extract_split_groups;
pub use ttest_indep::{
    compute_indep_ttest, compute_indep_ttest_from_slices, GroupStats, IndepTTestOptions,
    IndepTTestResult, LeveneResult, TTestTwoGroupDetails,
};
pub use ttest_onesample::{compute_onesample_ttest, OneSampleTTestOptions, OneSampleTTestResult};
pub use ttest_paired::{
    compute_paired_ttest, compute_paired_ttest_from_slices, PairedCorrelation, PairedDataInput,
    PairedDifferences, PairedSliceInput, PairedTTestResult, PairedVarStats,
};
