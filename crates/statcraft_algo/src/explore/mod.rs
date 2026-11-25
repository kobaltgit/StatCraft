pub mod bootstrap;
pub mod descriptives;
pub mod extremes;
mod helpers;
pub mod m_estimators;
pub mod normality_ks;
pub mod normality_sw;
pub mod percentiles;
pub mod plots_data;
pub mod spread_level;

pub use bootstrap::*;
pub use descriptives::*;
pub use extremes::*;
use helpers::{
    build_group_result, extract_factor_value, extract_label_value, extract_numeric_value,
};
pub use m_estimators::*;
pub use normality_ks::*;
pub use normality_sw::*;
pub use percentiles::*;
pub use plots_data::*;
pub use spread_level::*;

use crate::AlgoError;
use serde::{Deserialize, Serialize};
use statcraft_core::DataSet;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExploreMissingHandling {
    #[default]
    Listwise,
    Pairwise,
    Report,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreOptions {
    pub dependent_vars: Vec<String>,
    pub factor_vars: Vec<String>,
    #[serde(default)]
    pub label_var: Option<String>,
    pub compute_descriptives: bool,
    #[serde(default)]
    pub compute_m_estimators: bool,
    #[serde(default)]
    pub compute_percentiles: bool,
    pub compute_normality: bool,
    pub compute_extremes: bool,
    #[serde(default)]
    pub spread_level_mode: SpreadLevelMode,
    #[serde(default)]
    pub display_mode: ExploreOutputDisplay,
    #[serde(default)]
    pub boxplot_mode: ExploreBoxplotDisplay,
    #[serde(default = "default_true")]
    pub plot_stem_leaf: bool,
    #[serde(default)]
    pub plot_histogram: bool,
    pub confidence_level: f64,
    pub missing: ExploreMissingHandling,
}

fn default_true() -> bool {
    true
}

impl Default for ExploreOptions {
    fn default() -> Self {
        Self {
            dependent_vars: Vec::new(),
            factor_vars: Vec::new(),
            label_var: None,
            compute_descriptives: true,
            compute_m_estimators: false,
            compute_percentiles: false,
            compute_normality: true,
            compute_extremes: true,
            spread_level_mode: SpreadLevelMode::None,
            display_mode: ExploreOutputDisplay::Both,
            boxplot_mode: ExploreBoxplotDisplay::FactorLevelsTogether,
            plot_stem_leaf: true,
            plot_histogram: false,
            confidence_level: 0.95,
            missing: ExploreMissingHandling::Listwise,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreCaseProcessing {
    pub variable: String,
    pub factor_level: Option<String>,
    pub n_valid: usize,
    pub pct_valid: f64,
    pub n_missing: usize,
    pub pct_missing: f64,
    pub n_total: usize,
    pub pct_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalityTestsResult {
    pub variable: String,
    pub factor_level: Option<String>,
    pub kolmogorov_smirnov: Option<NormalityTestRow>,
    pub shapiro_wilk: Option<NormalityTestRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreGroupResult {
    pub factor_level: Option<String>,
    pub case_processing: ExploreCaseProcessing,
    pub descriptives: Option<ExploreDescriptives>,
    #[serde(default)]
    pub m_estimators: Option<MEstimatorsResult>,
    #[serde(default)]
    pub percentiles: Option<ExplorePercentilesResult>,
    pub normality: Option<NormalityTestsResult>,
    pub extremes: Option<ExtremeValuesResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plot_data: Option<ExploreGroupPlotData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreVariableResult {
    pub variable: String,
    pub factor_var: Option<String>,
    pub groups: Vec<ExploreGroupResult>,
    #[serde(default)]
    pub levene_test: Option<ExploreLeveneResult>,
    #[serde(default)]
    pub spread_level: Option<SpreadLevelResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreResult {
    pub variables: Vec<ExploreVariableResult>,
    #[serde(default)]
    pub display_mode: ExploreOutputDisplay,
    #[serde(default)]
    pub boxplot_mode: ExploreBoxplotDisplay,
    #[serde(default)]
    pub plot_stem_leaf: bool,
    #[serde(default)]
    pub plot_histogram: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<ExploreBootstrap>,
}

/// Executes the Explore (EXAMINE) procedure on a DataSet.
pub fn compute_explore(
    data: &DataSet,
    options: &ExploreOptions,
) -> Result<ExploreResult, AlgoError> {
    if options.dependent_vars.is_empty() {
        return Err(AlgoError::InvalidInput(
            "At least one dependent variable must be specified for Explore".to_string(),
        ));
    }

    let mut dep_indices = Vec::with_capacity(options.dependent_vars.len());
    for var in &options.dependent_vars {
        let idx = data.dictionary.find_index(var).ok_or_else(|| {
            AlgoError::InvalidInput(format!(
                "Dependent variable '{var}' not found in dictionary"
            ))
        })?;
        dep_indices.push((var.clone(), idx));
    }

    let factor_info = if let Some(factor_var) = options.factor_vars.first() {
        let idx = data.dictionary.find_index(factor_var).ok_or_else(|| {
            AlgoError::InvalidInput(format!(
                "Factor variable '{factor_var}' not found in dictionary"
            ))
        })?;
        Some((factor_var.clone(), idx))
    } else {
        None
    };

    let label_col = if let Some(ref l_var) = options.label_var {
        let trimmed = l_var.trim();
        if trimmed.is_empty() {
            None
        } else {
            let idx = data.dictionary.find_index(trimmed).ok_or_else(|| {
                AlgoError::InvalidInput(format!(
                    "Label variable '{trimmed}' not found in dictionary"
                ))
            })?;
            Some(idx)
        }
    } else {
        None
    };

    let active_rows = data.active_row_indices();

    // Listwise validity mask if requested
    let listwise_valid_rows: Vec<bool> = if options.missing == ExploreMissingHandling::Listwise {
        active_rows
            .iter()
            .map(|&r| {
                for &(_, col_idx) in &dep_indices {
                    if extract_numeric_value(data, r, col_idx).is_none() {
                        return false;
                    }
                }
                if let Some((_, f_idx)) = factor_info {
                    if extract_factor_value(data, r, f_idx).is_none() {
                        return false;
                    }
                }
                true
            })
            .collect()
    } else {
        Vec::new()
    };

    let mut var_results = Vec::with_capacity(dep_indices.len());

    for (dep_name, dep_col) in &dep_indices {
        let mut groups = Vec::new();

        if let Some((ref f_name, f_col)) = factor_info {
            // Group by factor level
            let mut group_cases: BTreeMap<String, Vec<(usize, f64, Option<String>)>> =
                BTreeMap::new();
            let mut group_missing: BTreeMap<String, usize> = BTreeMap::new();

            for (row_pos, &r) in active_rows.iter().enumerate() {
                let case_num = r + 1; // 1-based case number
                let is_valid_row = if options.missing == ExploreMissingHandling::Listwise {
                    listwise_valid_rows.get(row_pos).copied().unwrap_or(false)
                } else {
                    true
                };

                let factor_opt = match extract_factor_value(data, r, f_col) {
                    Some(lvl) => Some(lvl),
                    None if options.missing == ExploreMissingHandling::Report => {
                        Some("(Missing)".to_string())
                    }
                    None => None,
                };

                if let Some(f_level) = factor_opt {
                    group_cases.entry(f_level.clone()).or_default();
                    group_missing.entry(f_level.clone()).or_default();

                    let val_opt = if is_valid_row {
                        extract_numeric_value(data, r, *dep_col)
                    } else {
                        None
                    };

                    match val_opt {
                        Some(v) => {
                            let label = label_col.and_then(|lc| extract_label_value(data, r, lc));
                            if let Some(list) = group_cases.get_mut(&f_level) {
                                list.push((case_num, v, label));
                            }
                        }
                        None => {
                            if let Some(count) = group_missing.get_mut(&f_level) {
                                *count += 1;
                            }
                        }
                    }
                }
            }

            let mut raw_groups_for_levene: Vec<(String, Vec<f64>)> =
                Vec::with_capacity(group_cases.len());

            for (f_level, cases) in group_cases {
                let n_miss = group_missing.get(&f_level).copied().unwrap_or(0);
                let vals_only: Vec<f64> = cases.iter().map(|&(_, v, _)| v).collect();
                if !vals_only.is_empty() {
                    raw_groups_for_levene.push((f_level.clone(), vals_only));
                }
                groups.push(build_group_result(
                    dep_name,
                    Some(f_level),
                    &cases,
                    n_miss,
                    options,
                ));
            }

            let (levene_test, spread_level) = if options.spread_level_mode != SpreadLevelMode::None
            {
                (
                    compute_explore_levene(dep_name, f_name, &raw_groups_for_levene),
                    compute_spread_level(
                        dep_name,
                        f_name,
                        &raw_groups_for_levene,
                        options.spread_level_mode,
                    ),
                )
            } else {
                (None, None)
            };

            var_results.push(ExploreVariableResult {
                variable: dep_name.clone(),
                factor_var: Some(f_name.clone()),
                groups,
                levene_test,
                spread_level,
            });
        } else {
            // Overall group (no factor)
            let mut cases = Vec::with_capacity(active_rows.len());
            let mut n_miss = 0;

            for (row_pos, &r) in active_rows.iter().enumerate() {
                let case_num = r + 1;
                let is_valid_row = if options.missing == ExploreMissingHandling::Listwise {
                    listwise_valid_rows.get(row_pos).copied().unwrap_or(false)
                } else {
                    true
                };

                let val_opt = if is_valid_row {
                    extract_numeric_value(data, r, *dep_col)
                } else {
                    None
                };

                match val_opt {
                    Some(v) => {
                        let label = label_col.and_then(|lc| extract_label_value(data, r, lc));
                        cases.push((case_num, v, label));
                    }
                    None => n_miss += 1,
                }
            }

            groups.push(build_group_result(dep_name, None, &cases, n_miss, options));

            var_results.push(ExploreVariableResult {
                variable: dep_name.clone(),
                factor_var: None,
                groups,
                levene_test: None,
                spread_level: None,
            });
        }
    }

    Ok(ExploreResult {
        variables: var_results,
        display_mode: options.display_mode,
        boxplot_mode: options.boxplot_mode,
        plot_stem_leaf: options.plot_stem_leaf,
        plot_histogram: options.plot_histogram,
        bootstrap: None,
    })
}

#[cfg(test)]
mod tests;
