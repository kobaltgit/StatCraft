pub mod options;
pub mod percentiles;
pub mod stats;
pub mod table;

pub use options::{FrequenciesStatisticsOptions, PercentilesOptions};
pub use percentiles::{
    compute_custom_percentiles, compute_grouped_percentile, compute_ntiles, compute_percentile,
    compute_percentiles_with_options, compute_quartiles, PercentileValue,
};
pub use stats::{
    compute_frequencies_statistics, compute_statistics_from_slice, FrequenciesStatisticsResult,
};
pub use table::{
    compute_frequencies, compute_frequencies_detailed, compute_frequencies_full, FrequencyCategory,
    FrequencyInputItem, FrequencyResult, FrequencyTable,
};
