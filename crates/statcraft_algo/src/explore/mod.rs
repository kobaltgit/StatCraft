pub mod descriptives;
pub mod extremes;
pub mod normality_ks;
pub mod normality_sw;

pub use descriptives::*;
pub use extremes::*;
pub use normality_ks::*;
pub use normality_sw::*;

use crate::AlgoError;
use serde::{Deserialize, Serialize};
use statcraft_core::{DataSet, RawValue};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExploreMissingHandling {
    #[default]
    Listwise,
    Pairwise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreOptions {
    pub dependent_vars: Vec<String>,
    pub factor_vars: Vec<String>,
    pub compute_descriptives: bool,
    pub compute_normality: bool,
    pub compute_extremes: bool,
    pub confidence_level: f64,
    pub missing: ExploreMissingHandling,
}

impl Default for ExploreOptions {
    fn default() -> Self {
        Self {
            dependent_vars: Vec::new(),
            factor_vars: Vec::new(),
            compute_descriptives: true,
            compute_normality: true,
            compute_extremes: true,
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
    pub normality: Option<NormalityTestsResult>,
    pub extremes: Option<ExtremeValuesResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreVariableResult {
    pub variable: String,
    pub factor_var: Option<String>,
    pub groups: Vec<ExploreGroupResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreResult {
    pub variables: Vec<ExploreVariableResult>,
}

fn extract_numeric_value(dataset: &DataSet, row: usize, col_idx: usize) -> Option<f64> {
    let raw = dataset.get_cell(row, col_idx)?;
    if let Some(meta) = dataset.dictionary.get(col_idx) {
        if meta.missing.is_missing(raw) {
            return None;
        }
    }
    match raw {
        RawValue::Number(n) if n.is_finite() => Some(*n),
        RawValue::Text(s) => s.trim().parse::<f64>().ok().filter(|x| x.is_finite()),
        _ => None,
    }
}

fn extract_factor_value(dataset: &DataSet, row: usize, col_idx: usize) -> Option<String> {
    let raw = dataset.get_cell(row, col_idx)?;
    if let Some(meta) = dataset.dictionary.get(col_idx) {
        if meta.missing.is_missing(raw) {
            return None;
        }
    }
    match raw {
        RawValue::Text(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed == "." || trimmed == "NA" {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        RawValue::Number(n) if n.is_finite() => Some(format!("{n}")),
        _ => None,
    }
}

fn build_group_result(
    var_name: &str,
    factor_level: Option<String>,
    indexed_vals: &[(usize, f64)],
    n_missing: usize,
    options: &ExploreOptions,
) -> ExploreGroupResult {
    let n_valid = indexed_vals.len();
    let n_total = n_valid + n_missing;
    let (pct_valid, pct_missing, pct_total) = if n_total > 0 {
        (
            (n_valid as f64 / n_total as f64) * 100.0,
            (n_missing as f64 / n_total as f64) * 100.0,
            100.0,
        )
    } else {
        (0.0, 0.0, 0.0)
    };

    let case_processing = ExploreCaseProcessing {
        variable: var_name.to_string(),
        factor_level: factor_level.clone(),
        n_valid,
        pct_valid,
        n_missing,
        pct_missing,
        n_total,
        pct_total,
    };

    let vals: Vec<f64> = indexed_vals.iter().map(|&(_, v)| v).collect();

    let descriptives = if options.compute_descriptives {
        Some(compute_explore_descriptives(
            &vals,
            n_missing,
            options.confidence_level,
        ))
    } else {
        None
    };

    let normality = if options.compute_normality {
        let ks = compute_lilliefors_ks(&vals).ok();
        let sw = compute_shapiro_wilk(&vals).ok();
        Some(NormalityTestsResult {
            variable: var_name.to_string(),
            factor_level: factor_level.clone(),
            kolmogorov_smirnov: ks,
            shapiro_wilk: sw,
        })
    } else {
        None
    };

    let extremes = if options.compute_extremes {
        Some(compute_extreme_values(
            var_name,
            factor_level.clone(),
            indexed_vals,
        ))
    } else {
        None
    };

    ExploreGroupResult {
        factor_level,
        case_processing,
        descriptives,
        normality,
        extremes,
    }
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
            let mut group_cases: BTreeMap<String, Vec<(usize, f64)>> = BTreeMap::new();
            let mut group_missing: BTreeMap<String, usize> = BTreeMap::new();

            for (row_pos, &r) in active_rows.iter().enumerate() {
                let case_num = r + 1; // 1-based case number
                let is_valid_row = if options.missing == ExploreMissingHandling::Listwise {
                    listwise_valid_rows.get(row_pos).copied().unwrap_or(false)
                } else {
                    true
                };

                let factor_opt = extract_factor_value(data, r, f_col);
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
                            if let Some(list) = group_cases.get_mut(&f_level) {
                                list.push((case_num, v));
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

            for (f_level, cases) in group_cases {
                let n_miss = group_missing.get(&f_level).copied().unwrap_or(0);
                groups.push(build_group_result(
                    dep_name,
                    Some(f_level),
                    &cases,
                    n_miss,
                    options,
                ));
            }

            var_results.push(ExploreVariableResult {
                variable: dep_name.clone(),
                factor_var: Some(f_name.clone()),
                groups,
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
                    Some(v) => cases.push((case_num, v)),
                    None => n_miss += 1,
                }
            }

            groups.push(build_group_result(dep_name, None, &cases, n_miss, options));

            var_results.push(ExploreVariableResult {
                variable: dep_name.clone(),
                factor_var: None,
                groups,
            });
        }
    }

    Ok(ExploreResult {
        variables: var_results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_explore_single_var() {
        let mut ds = DataSet::blank(10, 1).unwrap();
        for r in 0..10 {
            ds.table.columns[0].values[r] = RawValue::Number((r + 1) as f64 * 10.0);
        }

        let options = ExploreOptions {
            dependent_vars: vec!["VAR00001".to_string()],
            ..Default::default()
        };

        let res = compute_explore(&ds, &options).unwrap();
        assert_eq!(res.variables.len(), 1);
        let var_res = &res.variables[0];
        assert_eq!(var_res.groups.len(), 1);

        let group = &var_res.groups[0];
        assert_eq!(group.case_processing.n_valid, 10);
        assert_eq!(group.case_processing.n_missing, 0);

        let desc = group.descriptives.as_ref().unwrap();
        assert_eq!(desc.mean, Some(55.0));

        let norm = group.normality.as_ref().unwrap();
        assert!(norm.kolmogorov_smirnov.is_some());
        assert!(norm.shapiro_wilk.is_some());

        let extremes = group.extremes.as_ref().unwrap();
        assert_eq!(extremes.lowest.len(), 5);
        assert_eq!(extremes.highest.len(), 5);
    }

    #[test]
    fn test_compute_explore_with_factor() {
        let mut ds = DataSet::blank(6, 2).unwrap();
        // Col 0: values [10, 20, 30, 40, 50, 60]
        // Col 1: factor ["A", "A", "A", "B", "B", "B"]
        for r in 0..6 {
            ds.table.columns[0].values[r] = RawValue::Number((r + 1) as f64 * 10.0);
            let grp = if r < 3 { "A" } else { "B" };
            ds.table.columns[1].values[r] = RawValue::Text(grp.to_string());
        }

        let options = ExploreOptions {
            dependent_vars: vec!["VAR00001".to_string()],
            factor_vars: vec!["VAR00002".to_string()],
            ..Default::default()
        };

        let res = compute_explore(&ds, &options).unwrap();
        assert_eq!(res.variables.len(), 1);
        let groups = &res.variables[0].groups;
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].factor_level.as_deref(), Some("A"));
        assert_eq!(groups[0].case_processing.n_valid, 3);
        assert_eq!(groups[1].factor_level.as_deref(), Some("B"));
        assert_eq!(groups[1].case_processing.n_valid, 3);
    }
}
