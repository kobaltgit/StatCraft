#![allow(clippy::needless_range_loop)]

pub mod adp;
pub mod anova_oneway;
pub mod auto_recode;
pub mod binning;
pub mod cluster_kmeans;
pub mod codebook_core;
pub mod correlations;
#[cfg(test)]
mod correlations_tests;
pub mod correlations_types;
pub mod count_values;
pub mod cox;
pub mod crosstabs;
#[cfg(test)]
mod crosstabs_tests;
pub mod crosstabs_types;
pub mod curvefit;
pub mod date_time;
pub mod descriptives;
pub mod dummies;
pub mod explore;
pub mod expr_eval;
pub mod factor;
#[cfg(test)]
mod factor_tests;
pub mod factor_types;
pub mod frequencies;
pub mod glm;
pub mod kruskal_wallis;
#[cfg(test)]
mod kruskal_wallis_tests;
pub mod logistic;
pub mod means;
#[cfg(test)]
mod means_tests;
pub mod means_types;
pub mod multinomial_logistic;
pub mod nonparam_related;
#[cfg(test)]
mod nonparam_related_tests;
pub mod nonparam_related_types;
pub mod nonparam_twosample;
#[cfg(test)]
mod nonparam_twosample_tests;
pub mod nonparam_twosample_types;
pub mod optimal_binning;
pub mod partial_corr;
pub mod programmability;
pub mod rank_cases;
#[cfg(test)]
mod rank_cases_tests;
pub mod recode;
pub mod regression;
#[cfg(test)]
mod regression_tests;
pub mod regression_types;
pub mod reliability;
#[cfg(test)]
mod reliability_tests;
pub mod reliability_types;
pub mod replace_missing;
pub mod shift;
pub mod sort_cases;
pub mod split_groups;
pub mod summarize_core;
pub mod survival_km;
pub mod survival_lifetables;
pub mod time_series;
pub mod ttest_indep;
pub mod ttest_onesample;
pub mod ttest_paired;

pub use adp::*;
pub use anova_oneway::{
    compute_oneway_anova, compute_oneway_anova_from_slices, f_distribution_sf, AlgoError,
    AnovaGroupDescriptives, AnovaHomogeneityTest, AnovaTableRow, OneWayAnovaOptions,
    OneWayAnovaResult,
};
pub use auto_recode::{compute_auto_recode, AutoRecodeOrder, AutoRecodeResult};
pub use binning::{
    compute_visual_binning, compute_visual_binning_with_labels, format_bound, generate_bin_labels,
    generate_equal_percentile_cutpoints, generate_equal_width_cutpoints, generate_stddev_cutpoints,
    BinningResult, Cutpoint,
};
pub use cluster_kmeans::*;
pub use codebook_core::*;
pub use correlations::{
    compute_bivariate_correlations, BivariateCorrelationsOptions, BivariateCorrelationsResult,
    CorrelationCell, CorrelationMatrixResult, CorrelationType, MissingHandling, SignificanceFlag,
    SignificanceTest, VarDescriptives, VariableInput,
};
pub use count_values::{compute_count_across_columns, ValueCriterion};
pub use cox::{
    compute_baseline_survival, compute_cox_regression, eval_partial_likelihood, fit_cox_engine,
    normal_quantile, BaselineSurvivalPoint, CoxCaseSummary, CoxDataPoint, CoxEngineOutput,
    CoxRegressionOptions, CoxRegressionResult, CoxVariableInEquation, OmnibusTestsResult,
    TieMethod,
};
pub use crosstabs::{chi_square_sf, compute_crosstabs, fisher_exact_2x2};
pub use crosstabs_types::{
    AssociationMeasure, ChiSquareTestRow, CrosstabCategory, CrosstabCell, CrosstabOptions,
    CrosstabResult, CrosstabRow, RawCrosstabItem,
};
pub use curvefit::{
    compute_curve_fit, CurveFitOptions, CurveFitResult, CurveModelSummaryRow, CurveModelType,
};
pub use date_time::{
    compute_date_operation, format_date_output, parse_date_value, DatePart, DateTimeOperation,
    DateUnit,
};
pub use descriptives::{
    compute_descriptives, compute_descriptives_weighted, DescriptivesOptions, DescriptivesResult,
};
pub use dummies::{generate_dummies_for_column, DummySpec, GeneratedDummyColumn};
pub use explore::*;
pub use expr_eval::{CompiledExpr, EvalError, EvalNode, Op};
pub use factor::{compute_factor_analysis, jacobi_eigen_symmetric};
pub use factor_types::{
    CommunalityRow, ComponentMatrixRow, FactorAnalysisResult, FactorExtractionCriteria,
    FactorExtractionMethod, FactorOptions, KmoBartlettResult, VarianceExplainedRow,
};
pub use frequencies::{
    compute_frequencies, compute_frequencies_detailed, compute_percentile, compute_quartiles,
    FrequencyCategory, FrequencyInputItem, FrequencyResult, FrequencyTable,
};
pub use kruskal_wallis::{
    compute_kruskal_wallis, compute_kruskal_wallis_from_groups, KruskalWallisGroupStats,
    KruskalWallisResult,
};
pub use logistic::{
    compute_logistic_regression, ClassificationTable, ClassificationTableRow, HosmerLemeshowGroup,
    HosmerLemeshowResult, LogisticCoefficientRow, LogisticModelSummary, LogisticRegressionOptions,
    LogisticRegressionResult, OmnibusTestsRow,
};
pub use means::compute_means_breakdown;
pub use means_types::{
    MeansAnovaRow, MeansAnovaTable, MeansAssociationMeasures, MeansCaseProcessingSummary,
    MeansCellStats, MeansOptions, MeansProcedureResult, MeansTableResult,
};
pub use multinomial_logistic::{
    compute_multinomial_logistic, CaseProcessingCategory, GoodnessOfFit, LikelihoodRatioTestRow,
    ModelFittingInfo as MultinomialModelFittingInfo, MultinomialCaseProcessingSummary,
    MultinomialClassificationTable, MultinomialClassificationTableRow, MultinomialLogisticOptions,
    MultinomialLogisticResult, ParameterEstimateRow, PseudoRSquared as MultinomialPseudoRSquared,
    ReferenceCategory,
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
pub use optimal_binning::{compute_optimal_binning, OptimalBinningResult};
pub use partial_corr::{
    compute_partial_correlations, PartialCorrCell, PartialCorrOptions, PartialCorrResult,
};
pub use programmability::*;
pub use rank_cases::{compute_ranks, inv_normal_cdf, RankDirection, RankType, TiesHandling};
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
pub use replace_missing::{impute_missing_values, ImputationMethod};
pub use shift::{compute_shift_raw, ShiftDirection};
pub use sort_cases::{
    compare_raw_values, compute_sort_permutation, invert_permutation, SortDirection, SortKeySpec,
};
pub use split_groups::extract_split_groups;
pub use summarize_core::*;
pub use survival_km::*;
pub use survival_lifetables::*;
pub use time_series::{compute_time_series_transform, TimeSeriesFunction};
pub use ttest_indep::{
    compute_indep_ttest, compute_indep_ttest_from_slices, GroupStats, IndepTTestOptions,
    IndepTTestResult, LeveneResult, TTestTwoGroupDetails,
};
pub use ttest_onesample::{compute_onesample_ttest, OneSampleTTestOptions, OneSampleTTestResult};
pub use ttest_paired::{
    compute_paired_ttest, compute_paired_ttest_from_slices, PairedCorrelation, PairedDataInput,
    PairedDifferences, PairedSliceInput, PairedTTestResult, PairedVarStats,
};
