use super::options::{ChartType, ChartValueMode, FrequencyChartSpec};
use super::table::FrequencyCategory;
use serde::{Deserialize, Serialize};

/// Aggregated data for a single bar or pie chart category slice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BarCategoryData {
    pub label: String,
    pub value_repr: String,
    pub count: f64,
    pub percent: f64,
}

/// Aggregated data for a single histogram bin (interval).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistogramBinData {
    pub left: f64,
    pub right: f64,
    pub midpoint: f64,
    pub count: usize,
    pub percent: f64,
}

/// Complete statistical summary and binning data for a histogram.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistogramSummary {
    pub mean: f64,
    pub std_dev: f64,
    pub n_valid: usize,
    pub bin_width: f64,
    pub bins: Vec<HistogramBinData>,
    pub normal_curve_points: Option<Vec<(f64, f64)>>, // (x, y_frequency)
}

/// Calculated chart data ready for rendering in Output Viewer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FrequencyChartResult {
    Bar {
        var_name: String,
        var_label: Option<String>,
        is_percentages: bool,
        categories: Vec<BarCategoryData>,
    },
    Pie {
        var_name: String,
        var_label: Option<String>,
        is_percentages: bool,
        slices: Vec<BarCategoryData>,
    },
    Histogram {
        var_name: String,
        var_label: Option<String>,
        summary: HistogramSummary,
    },
}

impl FrequencyChartResult {
    pub fn var_name(&self) -> &str {
        match self {
            Self::Bar { var_name, .. }
            | Self::Pie { var_name, .. }
            | Self::Histogram { var_name, .. } => var_name.as_str(),
        }
    }

    pub fn var_label(&self) -> Option<&str> {
        match self {
            Self::Bar { var_label, .. }
            | Self::Pie { var_label, .. }
            | Self::Histogram { var_label, .. } => var_label.as_deref(),
        }
    }

    pub fn display_title(&self) -> String {
        if let Some(lbl) = self.var_label() {
            if !lbl.trim().is_empty() {
                return format!("{} [{}]", lbl.trim(), self.var_name());
            }
        }
        self.var_name().to_string()
    }
}

/// Calculates a "nice" human-readable bin step (1, 2, 5, 10 * 10^k).
pub fn calculate_nice_step(raw_step: f64) -> f64 {
    if raw_step <= 0.0 || !raw_step.is_finite() {
        return 1.0;
    }
    let exponent = raw_step.log10().floor();
    let fraction = raw_step / 10.0_f64.powf(exponent);
    let nice_fraction = if fraction < 1.5 {
        1.0
    } else if fraction < 3.0 {
        2.0
    } else if fraction < 7.0 {
        5.0
    } else {
        10.0
    };
    nice_fraction * 10.0_f64.powf(exponent)
}

/// Computes a histogram with automatic adaptive binning (Scott's rule) and optional normal curve.
pub fn compute_histogram(
    values: &[f64],
    var_name: String,
    var_label: Option<String>,
    with_normal_curve: bool,
) -> Option<FrequencyChartResult> {
    let n = values.len();
    if n == 0 {
        return None;
    }

    let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !min_val.is_finite() || !max_val.is_finite() {
        return None;
    }

    let sum = values.iter().sum::<f64>();
    let mean = sum / (n as f64);
    let variance = if n > 1 {
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / ((n - 1) as f64)
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    // Determine bin width using Scott's rule with fallback for zero variance
    let raw_h = if std_dev > 1e-12 {
        3.49 * std_dev * (n as f64).powf(-1.0 / 3.0)
    } else {
        1.0
    };
    let bin_width = calculate_nice_step(raw_h);
    let start = (min_val / bin_width).floor() * bin_width;
    let end = (max_val / bin_width).ceil() * bin_width;

    let num_bins = if (end - start).abs() < 1e-12 {
        1
    } else {
        (((end - start) / bin_width).round() as usize).clamp(1, 500)
    };
    let mut bin_counts = vec![0usize; num_bins];

    for &v in values {
        let raw_idx = ((v - start) / bin_width).floor();
        let idx = if raw_idx <= 0.0 {
            0
        } else {
            (raw_idx as usize).min(num_bins - 1)
        };
        bin_counts[idx] += 1;
    }

    let bins: Vec<HistogramBinData> = bin_counts
        .into_iter()
        .enumerate()
        .map(|(i, count)| {
            let left = start + (i as f64) * bin_width;
            let right = left + bin_width;
            let midpoint = left + bin_width * 0.5;
            let percent = (count as f64) * 100.0 / (n as f64);
            HistogramBinData {
                left,
                right,
                midpoint,
                count,
                percent,
            }
        })
        .collect();

    // Gaussian normal density curve scaled to match histogram area
    let normal_curve_points = if with_normal_curve && std_dev > 1e-12 && n >= 2 {
        let mut pts = Vec::with_capacity(101);
        let x_min = start;
        let x_max = start + (num_bins as f64) * bin_width;
        let step = (x_max - x_min) / 100.0;
        let scale = (n as f64) * bin_width;

        for i in 0..=100 {
            let x = x_min + (i as f64) * step;
            let z = (x - mean) / std_dev;
            let pdf = (-0.5 * z * z).exp() / (std_dev * (2.0 * std::f64::consts::PI).sqrt());
            pts.push((x, pdf * scale));
        }
        Some(pts)
    } else {
        None
    };

    Some(FrequencyChartResult::Histogram {
        var_name,
        var_label,
        summary: HistogramSummary {
            mean,
            std_dev,
            n_valid: n,
            bin_width,
            bins,
            normal_curve_points,
        },
    })
}

/// Computes bar chart categories filtering out missing cases.
pub fn compute_bar_chart(
    categories: &[FrequencyCategory],
    var_name: String,
    var_label: Option<String>,
    is_percentages: bool,
) -> Option<FrequencyChartResult> {
    let valid_cats: Vec<BarCategoryData> = categories
        .iter()
        .filter(|c| c.valid_percent.is_some())
        .map(|c| BarCategoryData {
            label: c.display_label().to_string(),
            value_repr: c.value.clone(),
            count: c.count as f64,
            percent: c.valid_percent.unwrap_or(c.percent),
        })
        .collect();

    if valid_cats.is_empty() {
        return None;
    }

    Some(FrequencyChartResult::Bar {
        var_name,
        var_label,
        is_percentages,
        categories: valid_cats,
    })
}

/// Computes pie chart slices filtering out missing cases.
pub fn compute_pie_chart(
    categories: &[FrequencyCategory],
    var_name: String,
    var_label: Option<String>,
    is_percentages: bool,
) -> Option<FrequencyChartResult> {
    let valid_cats: Vec<BarCategoryData> = categories
        .iter()
        .filter(|c| c.valid_percent.is_some())
        .map(|c| BarCategoryData {
            label: c.display_label().to_string(),
            value_repr: c.value.clone(),
            count: c.count as f64,
            percent: c.valid_percent.unwrap_or(c.percent),
        })
        .collect();

    if valid_cats.is_empty() {
        return None;
    }

    Some(FrequencyChartResult::Pie {
        var_name,
        var_label,
        is_percentages,
        slices: valid_cats,
    })
}

/// Unified entry point to compute a frequency chart from spec and calculated distributions.
pub fn compute_frequency_chart(
    spec: &FrequencyChartSpec,
    var_name: &str,
    var_label: Option<&str>,
    categories: &[FrequencyCategory],
    numeric_values: &[f64],
) -> Option<FrequencyChartResult> {
    match spec.chart_type {
        ChartType::None => None,
        ChartType::Bar => {
            let is_percentages = spec.value_mode == ChartValueMode::Percentages;
            compute_bar_chart(
                categories,
                var_name.to_string(),
                var_label.map(|s| s.to_string()),
                is_percentages,
            )
        }
        ChartType::Pie => {
            let is_percentages = spec.value_mode == ChartValueMode::Percentages;
            compute_pie_chart(
                categories,
                var_name.to_string(),
                var_label.map(|s| s.to_string()),
                is_percentages,
            )
        }
        ChartType::Histogram => {
            if numeric_values.is_empty() {
                return None;
            }
            compute_histogram(
                numeric_values,
                var_name.to_string(),
                var_label.map(|s| s.to_string()),
                spec.normal_curve,
            )
        }
    }
}
