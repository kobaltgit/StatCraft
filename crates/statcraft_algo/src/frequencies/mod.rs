pub mod charts;
pub mod options;
pub mod percentiles;
pub mod stats;
pub mod table;

pub use charts::{
    calculate_nice_step, compute_bar_chart, compute_frequency_chart, compute_histogram,
    compute_pie_chart, BarCategoryData, FrequencyChartResult, HistogramBinData, HistogramSummary,
};
pub use options::{
    CategoryOrder, ChartType, ChartValueMode, FrequenciesFormatOptions,
    FrequenciesStatisticsOptions, FrequencyChartSpec, MultipleVarsMode, PercentilesOptions,
};
pub use percentiles::{
    compute_custom_percentiles, compute_grouped_percentile, compute_ntiles, compute_percentile,
    compute_percentiles_with_options, compute_quartiles, PercentileValue,
};
pub use stats::{
    compute_frequencies_statistics, compute_statistics_from_slice, FrequenciesStatisticsResult,
};
pub use table::{
    apply_format_options, compute_frequencies, compute_frequencies_detailed,
    compute_frequencies_full, recalculate_cumulative_percents, sort_categories, FrequencyCategory,
    FrequencyInputItem, FrequencyResult, FrequencyTable,
};
