//! Case Summaries procedure (SUMMARIZE / stat.reports.summarize).
//! Generates row-by-row listings of observations with optional multi-level
//! factor grouping, case limiting, and group/overall summary aggregations.

use crate::anova_oneway::AlgoError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeOptions {
    pub limit_cases: Option<usize>,
    pub display_cases: bool,
    pub compute_mean: bool,
    pub compute_stddev: bool,
    pub compute_sum: bool,
    pub compute_count: bool,
}

impl Default for SummarizeOptions {
    fn default() -> Self {
        Self {
            limit_cases: Some(100),
            display_cases: true,
            compute_mean: true,
            compute_stddev: true,
            compute_sum: true,
            compute_count: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummarizeCaseRow {
    pub case_index: usize,
    pub group_keys: Vec<String>,
    pub values: Vec<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummarizeVariableStats {
    pub variable: String,
    pub count: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub sum: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummarizeGroupBlock {
    pub group_keys: Vec<String>,
    pub cases: Vec<SummarizeCaseRow>,
    pub stats: Vec<SummarizeVariableStats>,
    pub total_cases_in_group: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummarizeResult {
    pub variable_names: Vec<String>,
    pub group_names: Vec<String>,
    pub groups: Vec<SummarizeGroupBlock>,
    pub total_stats: Vec<SummarizeVariableStats>,
    pub total_cases: usize,
    pub limited: bool,
}

/// Compute case summaries for table variables and grouping variables.
pub fn compute_summarize(
    var_names: &[String],
    var_values: &[Vec<Option<String>>],
    var_numeric: &[Vec<Option<f64>>],
    group_names: &[String],
    group_values: &[Vec<Option<String>>],
    options: &SummarizeOptions,
) -> Result<SummarizeResult, AlgoError> {
    let p = var_names.len();
    if p == 0 {
        return Err(AlgoError::InvalidInput(
            "No variables specified".to_string(),
        ));
    }
    if var_values.len() != p || var_numeric.len() != p {
        return Err(AlgoError::InvalidInput(
            "Mismatch between variable names and column matrices".to_string(),
        ));
    }

    let n = if let Some(first) = var_values.first() {
        first.len()
    } else {
        0
    };

    let g = group_names.len();
    if group_values.len() != g {
        return Err(AlgoError::InvalidInput(
            "Mismatch between group names and group columns".to_string(),
        ));
    }

    // Partition rows into groups
    let mut group_map: BTreeMap<Vec<String>, Vec<usize>> = BTreeMap::new();

    for i in 0..n {
        let mut key = Vec::with_capacity(g);
        for col in group_values {
            let s = col
                .get(i)
                .and_then(|x| x.clone())
                .unwrap_or_else(|| "(Missing)".to_string());
            key.push(s);
        }
        group_map.entry(key).or_default().push(i);
    }

    let mut blocks = Vec::new();
    let mut total_cases_shown = 0usize;
    let mut limited = false;

    for (grp_key, row_indices) in group_map {
        let total_in_group = row_indices.len();
        let mut cases = Vec::new();

        if options.display_cases {
            for &idx in &row_indices {
                if let Some(limit) = options.limit_cases {
                    if total_cases_shown >= limit {
                        limited = true;
                        break;
                    }
                }
                let mut row_vals = Vec::with_capacity(p);
                for col in var_values {
                    row_vals.push(col.get(idx).cloned().flatten());
                }
                cases.push(SummarizeCaseRow {
                    case_index: idx + 1,
                    group_keys: grp_key.clone(),
                    values: row_vals,
                });
                total_cases_shown += 1;
            }
        }

        // Stats for this group
        let mut grp_stats = Vec::with_capacity(p);
        for j in 0..p {
            let mut valid_nums = Vec::new();
            for &idx in &row_indices {
                if let Some(Some(num)) = var_numeric[j].get(idx) {
                    if !num.is_nan() {
                        valid_nums.push(*num);
                    }
                }
            }
            let count = valid_nums.len();
            let sum: Option<f64> = if count > 0 && options.compute_sum {
                Some(valid_nums.iter().sum())
            } else {
                None
            };
            let mean: Option<f64> = if count > 0 && options.compute_mean {
                Some(valid_nums.iter().sum::<f64>() / count as f64)
            } else {
                None
            };
            let std_dev: Option<f64> = if count > 1 && options.compute_stddev {
                let m = mean.unwrap_or(0.0);
                let var =
                    valid_nums.iter().map(|x| (x - m).powi(2)).sum::<f64>() / ((count - 1) as f64);
                Some(var.sqrt())
            } else {
                None
            };

            grp_stats.push(SummarizeVariableStats {
                variable: var_names[j].clone(),
                count,
                mean,
                std_dev,
                sum,
            });
        }

        blocks.push(SummarizeGroupBlock {
            group_keys: grp_key,
            cases,
            stats: grp_stats,
            total_cases_in_group: total_in_group,
        });
    }

    // Total stats across entire dataset
    let mut total_stats = Vec::with_capacity(p);
    for j in 0..p {
        let mut valid_nums = Vec::new();
        for idx in 0..n {
            if let Some(Some(num)) = var_numeric[j].get(idx) {
                if !num.is_nan() {
                    valid_nums.push(*num);
                }
            }
        }
        let count = valid_nums.len();
        let sum: Option<f64> = if count > 0 && options.compute_sum {
            Some(valid_nums.iter().sum())
        } else {
            None
        };
        let mean: Option<f64> = if count > 0 && options.compute_mean {
            Some(valid_nums.iter().sum::<f64>() / count as f64)
        } else {
            None
        };
        let std_dev: Option<f64> = if count > 1 && options.compute_stddev {
            let m = mean.unwrap_or(0.0);
            let var =
                valid_nums.iter().map(|x| (x - m).powi(2)).sum::<f64>() / ((count - 1) as f64);
            Some(var.sqrt())
        } else {
            None
        };

        total_stats.push(SummarizeVariableStats {
            variable: var_names[j].clone(),
            count,
            mean,
            std_dev,
            sum,
        });
    }

    Ok(SummarizeResult {
        variable_names: var_names.to_vec(),
        group_names: group_names.to_vec(),
        groups: blocks,
        total_stats,
        total_cases: n,
        limited,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_basic() {
        let var_names = vec!["salary".to_string()];
        let var_values = vec![vec![
            Some("1000".to_string()),
            Some("2000".to_string()),
            Some("3000".to_string()),
        ]];
        let var_numeric = vec![vec![Some(1000.0), Some(2000.0), Some(3000.0)]];
        let group_names = vec!["gender".to_string()];
        let group_values = vec![vec![
            Some("M".to_string()),
            Some("F".to_string()),
            Some("M".to_string()),
        ]];

        let res = compute_summarize(
            &var_names,
            &var_values,
            &var_numeric,
            &group_names,
            &group_values,
            &SummarizeOptions::default(),
        )
        .expect("summarize failed");

        assert_eq!(res.total_cases, 3);
        assert_eq!(res.groups.len(), 2);
        assert_eq!(res.total_stats[0].count, 3);
        assert!((res.total_stats[0].mean.unwrap() - 2000.0).abs() < 1e-9);
    }
}
