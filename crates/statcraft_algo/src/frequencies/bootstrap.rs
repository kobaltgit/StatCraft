use super::options::FrequenciesStatisticsOptions;
use super::stats::compute_frequencies_statistics;
use super::table::FrequencyInputItem;
use crate::bootstrap::{
    run_bootstrap_scalar, BootstrapError, BootstrapOptions, BootstrapSamplingMethod,
    BootstrapSummary, SamplingPlan,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Bootstrap summaries for active descriptive statistics in `FREQUENCIES`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FrequenciesBootstrapStats {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub se_mean: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub std_dev: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variance: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skewness: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kurtosis: Option<BootstrapSummary>,
}

/// Complete bootstrap results attached to a `FrequencyResult`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrequenciesBootstrapResult {
    pub options: BootstrapOptions,
    pub stats: FrequenciesBootstrapStats,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub category_percents: BTreeMap<String, BootstrapSummary>,
}

/// Computes bootstrap summaries for requested `FREQUENCIES` statistics and category percents.
pub fn compute_frequencies_bootstrap(
    items: &[FrequencyInputItem],
    strata_keys: Option<&[String]>,
    stats_options: Option<&FrequenciesStatisticsOptions>,
    boot_options: &BootstrapOptions,
) -> Result<FrequenciesBootstrapResult, BootstrapError> {
    boot_options.validate()?;
    if items.is_empty() {
        return Err(BootstrapError::EmptyDataset);
    }

    let plan = match (&boot_options.sampling_method, strata_keys) {
        (BootstrapSamplingMethod::Stratified { .. }, Some(keys)) => {
            if keys.len() != items.len() {
                return Err(BootstrapError::StrataLengthMismatch {
                    expected: items.len(),
                    actual: keys.len(),
                });
            }
            SamplingPlan::stratified(keys)?
        }
        _ => SamplingPlan::simple(items.len())?,
    };

    let mut boot_stats = FrequenciesBootstrapStats::default();
    if let Some(opts) = stats_options {
        let eval_stat = |indices: &[usize],
                         single_opts: &FrequenciesStatisticsOptions,
                         extract: fn(&super::stats::FrequenciesStatisticsResult) -> Option<f64>|
         -> Option<f64> {
            let mut pairs = Vec::with_capacity(indices.len());
            for &idx in indices {
                let it = &items[idx];
                if !it.is_missing && it.weight.is_finite() && it.weight > 0.0 {
                    if let Some(x) = it.numeric_value {
                        if x.is_finite() {
                            pairs.push((x, it.weight));
                        }
                    }
                }
            }
            if pairs.is_empty() {
                return None;
            }
            let res = compute_frequencies_statistics(&pairs, single_opts);
            extract(&res)
        };

        if opts.mean {
            let so = FrequenciesStatisticsOptions {
                mean: true,
                group_midpoints: opts.group_midpoints,
                ..Default::default()
            };
            boot_stats.mean =
                run_bootstrap_scalar(&plan, boot_options, |idx| eval_stat(idx, &so, |r| r.mean))
                    .ok();
        }
        if opts.se_mean {
            let so = FrequenciesStatisticsOptions {
                se_mean: true,
                group_midpoints: opts.group_midpoints,
                ..Default::default()
            };
            boot_stats.se_mean = run_bootstrap_scalar(&plan, boot_options, |idx| {
                eval_stat(idx, &so, |r| r.se_mean)
            })
            .ok();
        }
        if opts.median {
            let so = FrequenciesStatisticsOptions {
                median: true,
                group_midpoints: opts.group_midpoints,
                ..Default::default()
            };
            boot_stats.median =
                run_bootstrap_scalar(&plan, boot_options, |idx| eval_stat(idx, &so, |r| r.median))
                    .ok();
        }
        if opts.std_dev {
            let so = FrequenciesStatisticsOptions {
                std_dev: true,
                group_midpoints: opts.group_midpoints,
                ..Default::default()
            };
            boot_stats.std_dev = run_bootstrap_scalar(&plan, boot_options, |idx| {
                eval_stat(idx, &so, |r| r.std_dev)
            })
            .ok();
        }
        if opts.variance {
            let so = FrequenciesStatisticsOptions {
                variance: true,
                group_midpoints: opts.group_midpoints,
                ..Default::default()
            };
            boot_stats.variance = run_bootstrap_scalar(&plan, boot_options, |idx| {
                eval_stat(idx, &so, |r| r.variance)
            })
            .ok();
        }
        if opts.skewness {
            let so = FrequenciesStatisticsOptions {
                skewness: true,
                group_midpoints: opts.group_midpoints,
                ..Default::default()
            };
            boot_stats.skewness = run_bootstrap_scalar(&plan, boot_options, |idx| {
                eval_stat(idx, &so, |r| r.skewness)
            })
            .ok();
        }
        if opts.kurtosis {
            let so = FrequenciesStatisticsOptions {
                kurtosis: true,
                group_midpoints: opts.group_midpoints,
                ..Default::default()
            };
            boot_stats.kurtosis = run_bootstrap_scalar(&plan, boot_options, |idx| {
                eval_stat(idx, &so, |r| r.kurtosis)
            })
            .ok();
        }
    }

    let unique_cats: BTreeSet<String> = items
        .iter()
        .filter(|it| !it.is_missing)
        .map(|it| it.value.clone())
        .collect();

    let mut category_percents = BTreeMap::new();
    for cat_val in unique_cats {
        if let Ok(summary) = run_bootstrap_scalar(&plan, boot_options, |indices| {
            let mut total_w = 0.0_f64;
            let mut cat_w = 0.0_f64;
            for &idx in indices {
                let it = &items[idx];
                if it.weight.is_finite() && it.weight > 0.0 {
                    total_w += it.weight;
                    if !it.is_missing && it.value == cat_val {
                        cat_w += it.weight;
                    }
                }
            }
            if total_w > 0.0 {
                Some((cat_w / total_w) * 100.0)
            } else {
                None
            }
        }) {
            category_percents.insert(cat_val, summary);
        }
    }

    Ok(FrequenciesBootstrapResult {
        options: boot_options.clone(),
        stats: boot_stats,
        category_percents,
    })
}
