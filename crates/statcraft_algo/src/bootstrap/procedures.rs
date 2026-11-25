use super::{
    run_bootstrap_scalar, BootstrapError, BootstrapOptions, BootstrapSummary, SamplingPlan,
};
use crate::anova_oneway::{compute_oneway_anova, OneWayAnovaOptions, OneWayAnovaResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OneWayAnovaBootstrapGroup {
    pub group_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub std_dev: Option<BootstrapSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OneWayAnovaBootstrapPair {
    pub group1: String,
    pub group2: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean_diff: Option<BootstrapSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OneWayAnovaBootstrap {
    pub options: BootstrapOptions,
    pub groups: Vec<OneWayAnovaBootstrapGroup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_mean: Option<BootstrapSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pairwise_diffs: Vec<OneWayAnovaBootstrapPair>,
}
use crate::correlations::{compute_bivariate_correlations, VariableInput};
use crate::correlations_types::{
    BivariateCorrelationsOptions, CorrelationBootstrapPair, CorrelationType, CorrelationsBootstrap,
};
use crate::descriptives::{
    compute_descriptives_weighted, DescriptivesBootstrap, DescriptivesOptions, DescriptivesResult,
};
use crate::ttest_indep::{
    compute_indep_ttest, IndepTTestBootstrap, IndepTTestOptions, IndepTTestResult,
};
use crate::ttest_paired::{compute_single_pair, PairedTTestBootstrap, PairedTTestResult};

pub fn compute_descriptives_bootstrap(
    var_name: &str,
    values: &[Option<f64>],
    weights: &[f64],
    strata_keys: Option<&[String]>,
    options: Option<&DescriptivesOptions>,
    boot_options: &BootstrapOptions,
) -> Result<DescriptivesBootstrap, BootstrapError> {
    let plan = SamplingPlan::from_options(values.len(), strata_keys, boot_options)?;
    let opts = options
        .cloned()
        .unwrap_or_else(DescriptivesOptions::spps_default);

    let eval =
        |indices: &[usize], extract: fn(&DescriptivesResult) -> Option<f64>| -> Option<f64> {
            let mut sub_vals = Vec::with_capacity(indices.len());
            let mut sub_w = Vec::with_capacity(indices.len());
            for &i in indices {
                if let Some(v) = values.get(i) {
                    sub_vals.push(*v);
                    if !weights.is_empty() {
                        sub_w.push(weights.get(i).copied().unwrap_or(1.0));
                    }
                }
            }
            let res = compute_descriptives_weighted(var_name, &sub_vals, &sub_w);
            extract(&res)
        };

    let mean = if opts.mean {
        run_bootstrap_scalar(&plan, boot_options, |idx| eval(idx, |r| r.mean)).ok()
    } else {
        None
    };
    let std_dev = if opts.std_dev {
        run_bootstrap_scalar(&plan, boot_options, |idx| eval(idx, |r| r.std_dev)).ok()
    } else {
        None
    };
    let variance = if opts.variance {
        run_bootstrap_scalar(&plan, boot_options, |idx| eval(idx, |r| r.variance)).ok()
    } else {
        None
    };
    let skewness = if opts.skewness {
        run_bootstrap_scalar(&plan, boot_options, |idx| eval(idx, |r| r.skewness)).ok()
    } else {
        None
    };
    let kurtosis = if opts.kurtosis {
        run_bootstrap_scalar(&plan, boot_options, |idx| eval(idx, |r| r.kurtosis)).ok()
    } else {
        None
    };

    Ok(DescriptivesBootstrap {
        options: boot_options.clone(),
        mean,
        std_dev,
        variance,
        skewness,
        kurtosis,
    })
}

pub fn compute_indep_ttest_bootstrap(
    var_name: &str,
    grouping_name: &str,
    var_values: &[Option<f64>],
    group_values: &[Option<String>],
    options: &IndepTTestOptions,
    strata_keys: Option<&[String]>,
    boot_options: &BootstrapOptions,
) -> Result<IndepTTestBootstrap, BootstrapError> {
    let n = var_values.len().min(group_values.len());
    let plan = SamplingPlan::from_options(n, strata_keys, boot_options)?;

    let eval = |indices: &[usize], extract: fn(&IndepTTestResult) -> Option<f64>| -> Option<f64> {
        let mut sub_vals = Vec::with_capacity(indices.len());
        let mut sub_grps = Vec::with_capacity(indices.len());
        for &i in indices {
            if i < n {
                sub_vals.push(var_values[i]);
                sub_grps.push(group_values[i].clone());
            }
        }
        let res = compute_indep_ttest(var_name, grouping_name, &sub_vals, &sub_grps, options);
        extract(&res)
    };

    let group1_mean = run_bootstrap_scalar(&plan, boot_options, |idx| {
        eval(idx, |r| r.group1_stats.mean)
    })
    .ok();
    let group2_mean = run_bootstrap_scalar(&plan, boot_options, |idx| {
        eval(idx, |r| r.group2_stats.mean)
    })
    .ok();
    let mean_diff = run_bootstrap_scalar(&plan, boot_options, |idx| {
        eval(idx, |r| r.equal_variances_assumed.mean_diff)
    })
    .ok();

    Ok(IndepTTestBootstrap {
        options: boot_options.clone(),
        group1_mean,
        group2_mean,
        mean_diff,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn compute_paired_ttest_bootstrap(
    pair_num: usize,
    var1_name: &str,
    var1_vals: &[Option<f64>],
    var2_name: &str,
    var2_vals: &[Option<f64>],
    confidence_level: f64,
    strata_keys: Option<&[String]>,
    boot_options: &BootstrapOptions,
) -> Result<PairedTTestBootstrap, BootstrapError> {
    let n = var1_vals.len().min(var2_vals.len());
    let plan = SamplingPlan::from_options(n, strata_keys, boot_options)?;

    let eval = |indices: &[usize], extract: fn(&PairedTTestResult) -> Option<f64>| -> Option<f64> {
        let mut valid_x = Vec::with_capacity(indices.len());
        let mut valid_y = Vec::with_capacity(indices.len());
        for &i in indices {
            if i < n {
                if let (Some(x), Some(y)) = (var1_vals[i], var2_vals[i]) {
                    if x.is_finite() && y.is_finite() {
                        valid_x.push(x);
                        valid_y.push(y);
                    }
                }
            }
        }
        let res = compute_single_pair(
            pair_num,
            var1_name,
            &valid_x,
            var2_name,
            &valid_y,
            confidence_level,
        );
        extract(&res)
    };

    let var1_mean =
        run_bootstrap_scalar(&plan, boot_options, |idx| eval(idx, |r| r.var1_stats.mean)).ok();
    let var2_mean =
        run_bootstrap_scalar(&plan, boot_options, |idx| eval(idx, |r| r.var2_stats.mean)).ok();
    let mean_diff = run_bootstrap_scalar(&plan, boot_options, |idx| {
        eval(idx, |r| r.differences.mean_diff)
    })
    .ok();
    let correlation = run_bootstrap_scalar(&plan, boot_options, |idx| {
        eval(idx, |r| r.correlation.correlation)
    })
    .ok();

    Ok(PairedTTestBootstrap {
        options: boot_options.clone(),
        var1_mean,
        var2_mean,
        mean_diff,
        correlation,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn compute_oneway_anova_bootstrap(
    dep_name: &str,
    factor_name: &str,
    dep_values: &[Option<f64>],
    factor_values: &[Option<String>],
    options: &OneWayAnovaOptions,
    base_res: &OneWayAnovaResult,
    strata_keys: Option<&[String]>,
    boot_options: &BootstrapOptions,
) -> Result<OneWayAnovaBootstrap, BootstrapError> {
    let n = dep_values.len().min(factor_values.len());
    let plan = SamplingPlan::from_options(n, strata_keys, boot_options)?;

    let eval_anova = |indices: &[usize]| -> Option<OneWayAnovaResult> {
        let mut sub_dep = Vec::with_capacity(indices.len());
        let mut sub_fac = Vec::with_capacity(indices.len());
        for &i in indices {
            if i < n {
                sub_dep.push(dep_values[i]);
                sub_fac.push(factor_values[i].clone());
            }
        }
        compute_oneway_anova(dep_name, factor_name, &sub_dep, &sub_fac, options).ok()
    };

    let mut groups = Vec::with_capacity(base_res.descriptives.len());
    for g in &base_res.descriptives {
        let lbl = g.group_label.as_str();
        let mean = run_bootstrap_scalar(&plan, boot_options, |idx| {
            eval_anova(idx)?
                .descriptives
                .into_iter()
                .find(|d| d.group_label == lbl)
                .map(|d| d.mean)
        })
        .ok();
        let std_dev = run_bootstrap_scalar(&plan, boot_options, |idx| {
            eval_anova(idx)?
                .descriptives
                .into_iter()
                .find(|d| d.group_label == lbl)
                .map(|d| d.std_dev)
        })
        .ok();
        groups.push(OneWayAnovaBootstrapGroup {
            group_label: g.group_label.clone(),
            mean,
            std_dev,
        });
    }

    let total_mean = run_bootstrap_scalar(&plan, boot_options, |idx| {
        eval_anova(idx)?.total_descriptives.map(|t| t.mean)
    })
    .ok();

    let mut pairwise_diffs = Vec::new();
    for i in 0..base_res.descriptives.len() {
        for j in (i + 1)..base_res.descriptives.len() {
            let g1 = base_res.descriptives[i].group_label.as_str();
            let g2 = base_res.descriptives[j].group_label.as_str();
            let mean_diff = run_bootstrap_scalar(&plan, boot_options, |idx| {
                let res = eval_anova(idx)?;
                let m1 = res
                    .descriptives
                    .iter()
                    .find(|d| d.group_label == g1)
                    .map(|d| d.mean)?;
                let m2 = res
                    .descriptives
                    .iter()
                    .find(|d| d.group_label == g2)
                    .map(|d| d.mean)?;
                Some(m1 - m2)
            })
            .ok();
            pairwise_diffs.push(OneWayAnovaBootstrapPair {
                group1: g1.to_string(),
                group2: g2.to_string(),
                mean_diff,
            });
        }
    }

    Ok(OneWayAnovaBootstrap {
        options: boot_options.clone(),
        groups,
        total_mean,
        pairwise_diffs,
    })
}

pub fn compute_bivariate_correlations_bootstrap(
    variables: &[VariableInput<'_>],
    options: &BivariateCorrelationsOptions,
    strata_keys: Option<&[String]>,
    boot_options: &BootstrapOptions,
) -> Result<CorrelationsBootstrap, BootstrapError> {
    let num_rows = variables.first().map(|(_, v)| v.len()).unwrap_or(0);
    let plan = SamplingPlan::from_options(num_rows, strata_keys, boot_options)?;
    let k = variables.len();

    let mut pairs = Vec::new();
    let types = [
        (options.pearson, CorrelationType::Pearson),
        (options.spearman, CorrelationType::Spearman),
        (options.kendall, CorrelationType::KendallTauB),
    ];

    for (enabled, ctype) in types {
        if !enabled {
            continue;
        }
        let single_opts = BivariateCorrelationsOptions {
            pearson: ctype == CorrelationType::Pearson,
            spearman: ctype == CorrelationType::Spearman,
            kendall: ctype == CorrelationType::KendallTauB,
            significance: options.significance,
            flag_significant: false,
            descriptives: false,
            cross_prod_cov: false,
            missing: options.missing,
        };

        for i in 0..k {
            for j in (i + 1)..k {
                let var1 = variables[i].0;
                let var2 = variables[j].0;
                let col1 = variables[i].1;
                let col2 = variables[j].1;

                let r_summary = run_bootstrap_scalar(&plan, boot_options, |indices| {
                    let mut sub1 = Vec::with_capacity(indices.len());
                    let mut sub2 = Vec::with_capacity(indices.len());
                    for &idx in indices {
                        if idx < col1.len() && idx < col2.len() {
                            sub1.push(col1[idx]);
                            sub2.push(col2[idx]);
                        }
                    }
                    let pair_input = [(var1, sub1.as_slice()), (var2, sub2.as_slice())];
                    let res = compute_bivariate_correlations(&pair_input, &single_opts).ok()?;
                    res.matrices.first()?.cells.first()?.get(1)?.r
                })
                .ok();

                pairs.push(CorrelationBootstrapPair {
                    corr_type: ctype,
                    var1: var1.to_string(),
                    var2: var2.to_string(),
                    r: r_summary,
                });
            }
        }
    }

    Ok(CorrelationsBootstrap {
        options: boot_options.clone(),
        pairs,
    })
}
