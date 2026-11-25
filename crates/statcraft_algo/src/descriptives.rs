pub use crate::bootstrap::compute_descriptives_bootstrap;
use crate::bootstrap::{BootstrapOptions, BootstrapSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DescriptivesBootstrap {
    pub options: BootstrapOptions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub std_dev: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variance: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skewness: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kurtosis: Option<BootstrapSummary>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DescriptivesSortOrder {
    #[default]
    VariableList,
    Alphabetic,
    AlphabeticDesc,
    AscendingMeans,
    DescendingMeans,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DescriptivesMissingMode {
    #[default]
    Variable,
    Listwise,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DescriptivesResult {
    pub variable: String,
    pub n_valid: usize,
    pub n_missing: usize,
    pub mean: Option<f64>,
    pub se_mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub variance: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub range: Option<f64>,
    pub sum: Option<f64>,
    pub skewness: Option<f64>,
    pub se_skewness: Option<f64>,
    pub kurtosis: Option<f64>,
    pub se_kurtosis: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<DescriptivesBootstrap>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listwise_n: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_options: Option<DescriptivesOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DescriptivesOptions {
    #[serde(default)]
    pub mean: bool,
    #[serde(default)]
    pub se_mean: bool,
    #[serde(default)]
    pub std_dev: bool,
    #[serde(default)]
    pub variance: bool,
    #[serde(default)]
    pub min: bool,
    #[serde(default)]
    pub max: bool,
    #[serde(default)]
    pub range: bool,
    #[serde(default)]
    pub sum: bool,
    #[serde(default)]
    pub skewness: bool,
    #[serde(default)]
    pub kurtosis: bool,
    #[serde(default)]
    pub save_standardized: bool,
    #[serde(default)]
    pub sort: DescriptivesSortOrder,
    #[serde(default)]
    pub missing: DescriptivesMissingMode,
}

impl DescriptivesOptions {
    pub fn all() -> Self {
        Self {
            mean: true,
            se_mean: true,
            std_dev: true,
            variance: true,
            min: true,
            max: true,
            range: true,
            sum: true,
            skewness: true,
            kurtosis: true,
            save_standardized: false,
            sort: DescriptivesSortOrder::VariableList,
            missing: DescriptivesMissingMode::Variable,
        }
    }

    pub fn spps_default() -> Self {
        Self {
            mean: true,
            se_mean: false,
            std_dev: true,
            variance: false,
            min: true,
            max: true,
            range: false,
            sum: false,
            skewness: false,
            kurtosis: false,
            save_standardized: false,
            sort: DescriptivesSortOrder::VariableList,
            missing: DescriptivesMissingMode::Variable,
        }
    }
}

/// Computes descriptive statistics for a slice of floating point numbers.
/// Any NaN or infinite values are treated as missing.
pub fn compute_descriptives(var_name: &str, values: &[Option<f64>]) -> DescriptivesResult {
    compute_descriptives_weighted(var_name, values, &[])
}

/// Computes weighted descriptive statistics for a slice of floating point numbers and weights.
/// If `weights` is empty, uniform unit weights (`1.0`) are assumed for all observations.
/// Non-positive or non-finite weights cause the observation to be skipped.
pub fn compute_descriptives_weighted(
    var_name: &str,
    values: &[Option<f64>],
    weights: &[f64],
) -> DescriptivesResult {
    let mut valid_pairs: Vec<(f64, f64)> = Vec::with_capacity(values.len());
    let mut n_missing = 0_usize;

    for (i, v) in values.iter().enumerate() {
        let w = if weights.is_empty() {
            1.0
        } else {
            weights.get(i).copied().unwrap_or(0.0)
        };

        if w <= 0.0 || !w.is_finite() {
            continue;
        }

        let w_rounded = w.round() as usize;

        match v {
            Some(x) if x.is_finite() => valid_pairs.push((*x, w)),
            _ => n_missing += w_rounded,
        }
    }

    if valid_pairs.is_empty() {
        return DescriptivesResult {
            variable: var_name.to_string(),
            n_valid: 0,
            n_missing,
            mean: None,
            se_mean: None,
            std_dev: None,
            variance: None,
            min: None,
            max: None,
            range: None,
            sum: None,
            skewness: None,
            se_skewness: None,
            kurtosis: None,
            se_kurtosis: None,
            bootstrap: None,
            listwise_n: None,
            applied_options: None,
        };
    }

    let w_sum: f64 = valid_pairs.iter().map(|&(_, w)| w).sum();
    let n_valid = w_sum.round() as usize;

    let sum: f64 = valid_pairs.iter().map(|&(x, w)| w * x).sum();
    let mean = sum / w_sum;

    let mut min = valid_pairs[0].0;
    let mut max = valid_pairs[0].0;
    for &(x, _) in &valid_pairs[1..] {
        if x < min {
            min = x;
        }
        if x > max {
            max = x;
        }
    }
    let range = max - min;

    let (variance, std_dev, se_mean) = if w_sum >= 2.0 {
        let sum_sq_diff: f64 = valid_pairs
            .iter()
            .map(|&(x, w)| w * (x - mean).powi(2))
            .sum();
        let var = sum_sq_diff / (w_sum - 1.0);
        let sd = var.sqrt();
        let se = sd / w_sum.sqrt();
        (Some(var), Some(sd), Some(se))
    } else {
        (None, None, None)
    };

    // Skewness calculation (SPSS / SciPy bias=False formula)
    let (skewness, se_skewness) = if w_sum >= 3.0 {
        let sd = match std_dev {
            Some(s) if s > 0.0 => s,
            _ => 0.0,
        };

        if sd == 0.0 {
            (Some(0.0), None)
        } else {
            let sum_cubed_diff: f64 = valid_pairs
                .iter()
                .map(|&(x, w)| w * (x - mean).powi(3))
                .sum();
            let factor = w_sum / ((w_sum - 1.0) * (w_sum - 2.0));
            let skew = factor * (sum_cubed_diff / sd.powi(3));

            let se_skew_var =
                (6.0 * w_sum * (w_sum - 1.0)) / ((w_sum - 2.0) * (w_sum + 1.0) * (w_sum + 3.0));
            let se_skew = if se_skew_var > 0.0 {
                Some(se_skew_var.sqrt())
            } else {
                None
            };

            (Some(skew), se_skew)
        }
    } else {
        (None, None)
    };

    // Kurtosis calculation (SPSS / SciPy Fisher excess kurtosis, bias=False)
    let (kurtosis, se_kurtosis) = if w_sum >= 4.0 {
        let sd = match std_dev {
            Some(s) if s > 0.0 => s,
            _ => 0.0,
        };

        if sd == 0.0 {
            (Some(0.0), None)
        } else {
            let m2 = valid_pairs
                .iter()
                .map(|&(x, w)| w * (x - mean).powi(2))
                .sum::<f64>()
                / w_sum;
            let m4 = valid_pairs
                .iter()
                .map(|&(x, w)| w * (x - mean).powi(4))
                .sum::<f64>()
                / w_sum;

            if m2 > 0.0 {
                let term1 = (w_sum + 1.0) * (m4 / (m2 * m2));
                let term2 = 3.0 * (w_sum - 1.0);
                let multiplier = (w_sum - 1.0) / ((w_sum - 2.0) * (w_sum - 3.0));
                let kurt = multiplier * (term1 - term2);

                let se_kurt = se_skewness.and_then(|se_s| {
                    let se_kurt_factor = (w_sum * w_sum - 1.0) / ((w_sum - 3.0) * (w_sum + 5.0));
                    if se_kurt_factor > 0.0 {
                        Some(2.0 * se_s * se_kurt_factor.sqrt())
                    } else {
                        None
                    }
                });

                (Some(kurt), se_kurt)
            } else {
                (Some(0.0), None)
            }
        }
    } else {
        (None, None)
    };

    DescriptivesResult {
        variable: var_name.to_string(),
        n_valid,
        n_missing,
        mean: Some(mean),
        se_mean,
        std_dev,
        variance,
        min: Some(min),
        max: Some(max),
        range: Some(range),
        sum: Some(sum),
        skewness,
        se_skewness,
        kurtosis,
        se_kurtosis,
        bootstrap: None,
        listwise_n: None,
        applied_options: None,
    }
}

/// Applies `DescriptivesOptions` to a computed `DescriptivesResult`, clearing
/// optional statistics that were not requested and recording `applied_options`.
pub fn apply_options_filter(res: &mut DescriptivesResult, options: &DescriptivesOptions) {
    res.applied_options = Some(options.clone());
    if !options.mean {
        res.mean = None;
    }
    if !options.se_mean {
        res.se_mean = None;
    }
    if !options.std_dev {
        res.std_dev = None;
    }
    if !options.variance {
        res.variance = None;
    }
    if !options.min {
        res.min = None;
    }
    if !options.max {
        res.max = None;
    }
    if !options.range {
        res.range = None;
    }
    if !options.sum {
        res.sum = None;
    }
    if !options.skewness {
        res.skewness = None;
        res.se_skewness = None;
    }
    if !options.kurtosis {
        res.kurtosis = None;
        res.se_kurtosis = None;
    }
}

fn cmp_opt_f64(a: Option<f64>, b: Option<f64>, descending: bool) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a.filter(|v| v.is_finite()), b.filter(|v| v.is_finite())) {
        (Some(x), Some(y)) => {
            let ord = x.partial_cmp(&y).unwrap_or(Ordering::Equal);
            if descending {
                ord.reverse()
            } else {
                ord
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Sorts descriptive results according to the requested SPSS display order.
pub fn sort_descriptives_results(results: &mut [DescriptivesResult], order: DescriptivesSortOrder) {
    match order {
        DescriptivesSortOrder::VariableList => {}
        DescriptivesSortOrder::Alphabetic => {
            results.sort_by(|a, b| {
                a.variable
                    .to_lowercase()
                    .cmp(&b.variable.to_lowercase())
                    .then_with(|| a.variable.cmp(&b.variable))
            });
        }
        DescriptivesSortOrder::AlphabeticDesc => {
            results.sort_by(|a, b| {
                b.variable
                    .to_lowercase()
                    .cmp(&a.variable.to_lowercase())
                    .then_with(|| b.variable.cmp(&a.variable))
            });
        }
        DescriptivesSortOrder::AscendingMeans => {
            results.sort_by(|a, b| cmp_opt_f64(a.mean, b.mean, false));
        }
        DescriptivesSortOrder::DescendingMeans => {
            results.sort_by(|a, b| cmp_opt_f64(a.mean, b.mean, true));
        }
    }
}

/// Computes standardized Z-scores `(x - mean) / std_dev` using weighted sample
/// mean and standard deviation (`ddof = 1`).
/// Returns `None` for missing/non-finite entries or for the whole vector when
/// `std_dev` is unavailable or zero.
pub fn compute_standardized_values(values: &[Option<f64>], weights: &[f64]) -> Vec<Option<f64>> {
    let stats = compute_descriptives_weighted("", values, weights);
    let (Some(mean), Some(sd)) = (stats.mean, stats.std_dev) else {
        return vec![None; values.len()];
    };
    if !mean.is_finite() || !sd.is_finite() || sd == 0.0 {
        return vec![None; values.len()];
    }

    values
        .iter()
        .map(|v| match v {
            Some(x) if x.is_finite() => Some((x - mean) / sd),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
#[path = "descriptives_tests.rs"]
mod tests;
