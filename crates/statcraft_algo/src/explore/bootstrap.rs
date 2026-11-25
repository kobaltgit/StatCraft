use super::descriptives::compute_explore_descriptives;
use super::{extract_factor_value, extract_numeric_value, ExploreMissingHandling, ExploreOptions};
use crate::bootstrap::{
    run_bootstrap_scalar, BootstrapError, BootstrapOptions, BootstrapSummary, SamplingPlan,
};
use serde::{Deserialize, Serialize};
use statcraft_core::DataSet;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreBootstrapGroup {
    pub variable: String,
    pub factor_level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trimmed_mean_5pct: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variance: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub std_dev: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iqr: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skewness: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kurtosis: Option<BootstrapSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreBootstrap {
    pub options: BootstrapOptions,
    pub groups: Vec<ExploreBootstrapGroup>,
}

/// Computes bootstrap summaries for `Explore` descriptive statistics across all
/// dependent variable / factor-level groups.
pub fn compute_explore_bootstrap(
    data: &DataSet,
    options: &ExploreOptions,
    strata_keys: Option<&[String]>,
    boot_options: &BootstrapOptions,
) -> Result<ExploreBootstrap, BootstrapError> {
    let active_rows = data.active_row_indices();
    let n = active_rows.len();
    let plan = SamplingPlan::from_options(n, strata_keys, boot_options)?;

    let factor_col = options
        .factor_vars
        .first()
        .and_then(|fv| data.dictionary.find_index(fv));

    let factor_vals: Option<Vec<Option<String>>> = factor_col.map(|f_idx| {
        active_rows
            .iter()
            .map(|&r| match extract_factor_value(data, r, f_idx) {
                Some(lvl) => Some(lvl),
                None if options.missing == ExploreMissingHandling::Report => {
                    Some("(Missing)".to_string())
                }
                None => None,
            })
            .collect()
    });

    let mut dep_cols = Vec::with_capacity(options.dependent_vars.len());
    for var in &options.dependent_vars {
        if let Some(col_idx) = data.dictionary.find_index(var) {
            let vals: Vec<Option<f64>> = active_rows
                .iter()
                .map(|&r| extract_numeric_value(data, r, col_idx))
                .collect();
            dep_cols.push((var.clone(), vals));
        }
    }

    let listwise_valid: Vec<bool> = if options.missing == ExploreMissingHandling::Listwise {
        (0..n)
            .map(|i| {
                let deps_ok = dep_cols.iter().all(|(_, vals)| vals[i].is_some());
                let fac_ok = factor_vals
                    .as_ref()
                    .map(|fv| fv[i].is_some())
                    .unwrap_or(true);
                deps_ok && fac_ok
            })
            .collect()
    } else {
        vec![true; n]
    };

    let mut groups_out = Vec::new();
    let conf = options.confidence_level;

    for (var_name, dep_vals) in &dep_cols {
        if let Some(ref f_vals) = factor_vals {
            let mut levels: BTreeMap<String, ()> = BTreeMap::new();
            for lvl in f_vals.iter().flatten() {
                levels.insert(lvl.clone(), ());
            }
            for level_name in levels.keys() {
                let eval_stat = |indices: &[usize],
                                 extract: fn(&super::ExploreDescriptives) -> Option<f64>|
                 -> Option<f64> {
                    let mut sub = Vec::new();
                    for &idx in indices {
                        if idx < n
                            && listwise_valid[idx]
                            && f_vals[idx].as_deref() == Some(level_name.as_str())
                        {
                            if let Some(v) = dep_vals[idx] {
                                sub.push(v);
                            }
                        }
                    }
                    if sub.is_empty() {
                        return None;
                    }
                    let d = compute_explore_descriptives(&sub, 0, conf);
                    extract(&d)
                };

                groups_out.push(ExploreBootstrapGroup {
                    variable: var_name.clone(),
                    factor_level: Some(level_name.clone()),
                    mean: run_bootstrap_scalar(&plan, boot_options, |idx| {
                        eval_stat(idx, |d| d.mean)
                    })
                    .ok(),
                    trimmed_mean_5pct: run_bootstrap_scalar(&plan, boot_options, |idx| {
                        eval_stat(idx, |d| d.trimmed_mean_5pct)
                    })
                    .ok(),
                    median: run_bootstrap_scalar(&plan, boot_options, |idx| {
                        eval_stat(idx, |d| d.median)
                    })
                    .ok(),
                    variance: run_bootstrap_scalar(&plan, boot_options, |idx| {
                        eval_stat(idx, |d| d.variance)
                    })
                    .ok(),
                    std_dev: run_bootstrap_scalar(&plan, boot_options, |idx| {
                        eval_stat(idx, |d| d.std_dev)
                    })
                    .ok(),
                    iqr: run_bootstrap_scalar(&plan, boot_options, |idx| eval_stat(idx, |d| d.iqr))
                        .ok(),
                    skewness: run_bootstrap_scalar(&plan, boot_options, |idx| {
                        eval_stat(idx, |d| d.skewness)
                    })
                    .ok(),
                    kurtosis: run_bootstrap_scalar(&plan, boot_options, |idx| {
                        eval_stat(idx, |d| d.kurtosis)
                    })
                    .ok(),
                });
            }
        } else {
            let eval_stat = |indices: &[usize],
                             extract: fn(&super::ExploreDescriptives) -> Option<f64>|
             -> Option<f64> {
                let mut sub = Vec::new();
                for &idx in indices {
                    if idx < n && listwise_valid[idx] {
                        if let Some(v) = dep_vals[idx] {
                            sub.push(v);
                        }
                    }
                }
                if sub.is_empty() {
                    return None;
                }
                let d = compute_explore_descriptives(&sub, 0, conf);
                extract(&d)
            };

            groups_out.push(ExploreBootstrapGroup {
                variable: var_name.clone(),
                factor_level: None,
                mean: run_bootstrap_scalar(&plan, boot_options, |idx| eval_stat(idx, |d| d.mean))
                    .ok(),
                trimmed_mean_5pct: run_bootstrap_scalar(&plan, boot_options, |idx| {
                    eval_stat(idx, |d| d.trimmed_mean_5pct)
                })
                .ok(),
                median: run_bootstrap_scalar(&plan, boot_options, |idx| {
                    eval_stat(idx, |d| d.median)
                })
                .ok(),
                variance: run_bootstrap_scalar(&plan, boot_options, |idx| {
                    eval_stat(idx, |d| d.variance)
                })
                .ok(),
                std_dev: run_bootstrap_scalar(&plan, boot_options, |idx| {
                    eval_stat(idx, |d| d.std_dev)
                })
                .ok(),
                iqr: run_bootstrap_scalar(&plan, boot_options, |idx| eval_stat(idx, |d| d.iqr))
                    .ok(),
                skewness: run_bootstrap_scalar(&plan, boot_options, |idx| {
                    eval_stat(idx, |d| d.skewness)
                })
                .ok(),
                kurtosis: run_bootstrap_scalar(&plan, boot_options, |idx| {
                    eval_stat(idx, |d| d.kurtosis)
                })
                .ok(),
            });
        }
    }

    Ok(ExploreBootstrap {
        options: boot_options.clone(),
        groups: groups_out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::RawValue;

    #[test]
    fn test_explore_bootstrap_runs_deterministically() {
        let mut ds = DataSet::blank(6, 2).unwrap();
        let vals = [35000.0, 45000.0, 55000.0, 65000.0, 75000.0, 85000.0];
        let facs = ["1", "2", "1", "2", "1", "2"];
        for r in 0..6 {
            ds.table.columns[0].values[r] = RawValue::Number(vals[r]);
            ds.table.columns[1].values[r] = RawValue::Text(facs[r].to_string());
        }

        let opts = ExploreOptions {
            dependent_vars: vec!["VAR00001".to_string()],
            factor_vars: vec!["VAR00002".to_string()],
            ..Default::default()
        };
        let boot_opts = BootstrapOptions {
            enabled: true,
            samples: 50,
            seed: Some(42),
            ..Default::default()
        };

        let res = compute_explore_bootstrap(&ds, &opts, None, &boot_opts)
            .expect("Bootstrap should succeed");
        assert_eq!(res.groups.len(), 2);
        assert!(res.groups[0].mean.is_some());
        assert!(res.groups[0].median.is_some());
        assert!(res.groups[1].std_dev.is_some());
    }
}
