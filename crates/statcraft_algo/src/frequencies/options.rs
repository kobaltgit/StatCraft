use serde::{Deserialize, Serialize};

/// Options for which statistics to compute in the FREQUENCIES procedure.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FrequenciesStatisticsOptions {
    pub mean: bool,
    pub se_mean: bool,
    pub median: bool,
    pub mode: bool,
    pub sum: bool,
    pub std_dev: bool,
    pub variance: bool,
    pub range: bool,
    pub min: bool,
    pub max: bool,
    pub skewness: bool,
    pub kurtosis: bool,
    pub percentiles: PercentilesOptions,
    pub group_midpoints: bool,
}

impl FrequenciesStatisticsOptions {
    /// Enables all 14 univariate statistics and quartiles.
    pub fn all() -> Self {
        Self {
            mean: true,
            se_mean: true,
            median: true,
            mode: true,
            sum: true,
            std_dev: true,
            variance: true,
            range: true,
            min: true,
            max: true,
            skewness: true,
            kurtosis: true,
            percentiles: PercentilesOptions {
                quartiles: true,
                ntiles: None,
                custom: Vec::new(),
            },
            group_midpoints: false,
        }
    }

    /// Default empty options (no extra statistics computed).
    pub fn none() -> Self {
        Self::default()
    }

    /// Checks if any statistic or percentile is requested.
    pub fn any_requested(&self) -> bool {
        self.mean
            || self.se_mean
            || self.median
            || self.mode
            || self.sum
            || self.std_dev
            || self.variance
            || self.range
            || self.min
            || self.max
            || self.skewness
            || self.kurtosis
            || self.percentiles.any_requested()
    }
}

/// Options for percentile computation.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PercentilesOptions {
    pub quartiles: bool,
    pub ntiles: Option<usize>, // e.g., 10 for deciles
    pub custom: Vec<f64>,      // e.g., [5.0, 10.0, 90.0, 95.0]
}

impl PercentilesOptions {
    pub fn any_requested(&self) -> bool {
        self.quartiles || self.ntiles.is_some() || !self.custom.is_empty()
    }
}

/// Chart types supported by the FREQUENCIES procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    #[default]
    None,
    Bar,
    Pie,
    Histogram,
}

/// Chart values mode: counts/frequencies vs percentages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChartValueMode {
    #[default]
    Frequencies,
    Percentages,
}

/// Specification for frequency chart generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FrequencyChartSpec {
    pub chart_type: ChartType,
    pub value_mode: ChartValueMode,
    pub normal_curve: bool,
}

impl FrequencyChartSpec {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn is_active(&self) -> bool {
        self.chart_type != ChartType::None
    }
}
