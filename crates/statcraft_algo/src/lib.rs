pub mod anova_oneway;
pub mod descriptives;
pub mod frequencies;
pub mod ttest_indep;
pub mod ttest_onesample;
pub mod ttest_paired;

pub use anova_oneway::{
    compute_oneway_anova, compute_oneway_anova_from_slices, f_distribution_sf, AlgoError,
    AnovaGroupDescriptives, AnovaHomogeneityTest, AnovaTableRow, OneWayAnovaOptions,
    OneWayAnovaResult,
};
pub use descriptives::{compute_descriptives, DescriptivesOptions, DescriptivesResult};
pub use frequencies::{
    compute_frequencies, compute_frequencies_detailed, compute_percentile, compute_quartiles,
    FrequencyCategory, FrequencyInputItem, FrequencyResult, FrequencyTable,
};
pub use ttest_indep::{
    compute_indep_ttest, compute_indep_ttest_from_slices, GroupStats, IndepTTestOptions,
    IndepTTestResult, LeveneResult, TTestTwoGroupDetails,
};
pub use ttest_onesample::{compute_onesample_ttest, OneSampleTTestOptions, OneSampleTTestResult};
pub use ttest_paired::{
    compute_paired_ttest, compute_paired_ttest_from_slices, PairedCorrelation, PairedDataInput,
    PairedDifferences, PairedSliceInput, PairedTTestResult, PairedVarStats,
};
