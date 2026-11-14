//! Codebook core procedure (CODEBOOK / stat.reports.codebook).
//! Generates dictionary metadata, measurement properties, missing specs,
//! categorical frequency tables, and scale descriptive statistics per IBM SPSS.

use crate::anova_oneway::AlgoError;
use crate::frequencies::{compute_percentile, compute_quartiles};
use serde::{Deserialize, Serialize};
use statcraft_core::variable::{Measure, VariableMetadata};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebookOptions {
    pub max_categories: usize,
    pub include_counts: bool,
    pub include_stats: bool,
}

impl Default for CodebookOptions {
    fn default() -> Self {
        Self {
            max_categories: 200,
            include_counts: true,
            include_stats: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodebookFrequencyRow {
    pub value: String,
    pub label: Option<String>,
    pub count: usize,
    pub percent: f64,
    pub valid_percent: Option<f64>,
    pub cumulative_percent: Option<f64>,
    pub is_missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodebookScaleStats {
    pub valid_count: usize,
    pub missing_count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub q25: f64,
    pub q75: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodebookVariableCard {
    pub name: String,
    pub position: usize,
    pub label: Option<String>,
    pub data_type: String,
    pub measure: String,
    pub missing_description: String,
    pub frequencies: Option<Vec<CodebookFrequencyRow>>,
    pub scale_stats: Option<CodebookScaleStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodebookResult {
    pub dataset_name: Option<String>,
    pub total_cases: usize,
    pub total_variables: usize,
    pub cards: Vec<CodebookVariableCard>,
}

pub struct RawColumnData<'a> {
    pub values: &'a [Option<String>],
    pub numeric_values: &'a [Option<f64>],
}

/// Compute Codebook for a list of variables given raw column data.
pub fn compute_codebook(
    variables: &[&VariableMetadata],
    columns: &[RawColumnData],
    options: &CodebookOptions,
) -> Result<CodebookResult, AlgoError> {
    if variables.len() != columns.len() {
        return Err(AlgoError::InvalidInput(
            "Variables count does not match columns count".to_string(),
        ));
    }

    let total_cases = if let Some(first) = columns.first() {
        first.values.len()
    } else {
        0
    };

    let mut cards = Vec::with_capacity(variables.len());

    for (pos, (var, col)) in variables.iter().zip(columns.iter()).enumerate() {
        let is_scale = matches!(var.measure, Measure::Scale);

        let data_type_str = format!("{:?}", var.var_type);
        let measure_str = match var.measure {
            Measure::Scale => "Scale".to_string(),
            Measure::Ordinal => "Ordinal".to_string(),
            Measure::Nominal => "Nominal".to_string(),
        };

        let missing_desc = format!("{:?}", var.missing);

        let mut frequencies = None;
        let mut scale_stats = None;

        if is_scale && options.include_stats {
            let mut valid_nums = Vec::new();
            let mut missing_count = 0usize;

            for val_opt in col.numeric_values {
                match val_opt {
                    Some(v) if !v.is_nan() => valid_nums.push(*v),
                    _ => missing_count += 1,
                }
            }

            let valid_count = valid_nums.len();
            if valid_count > 0 {
                valid_nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let sum: f64 = valid_nums.iter().sum();
                let mean = sum / (valid_count as f64);

                let variance = if valid_count > 1 {
                    valid_nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                        / ((valid_count - 1) as f64)
                } else {
                    0.0
                };
                let std_dev = variance.sqrt();

                let (q25, median, q75) = compute_quartiles(&valid_nums).unwrap_or_else(|| {
                    let m = compute_percentile(&valid_nums, 0.5).unwrap_or(mean);
                    (m, m, m)
                });

                let min = valid_nums.first().copied().unwrap_or(0.0);
                let max = valid_nums.last().copied().unwrap_or(0.0);

                scale_stats = Some(CodebookScaleStats {
                    valid_count,
                    missing_count,
                    mean,
                    std_dev,
                    median,
                    min,
                    max,
                    q25,
                    q75,
                });
            } else {
                scale_stats = Some(CodebookScaleStats {
                    valid_count: 0,
                    missing_count,
                    mean: 0.0,
                    std_dev: 0.0,
                    median: 0.0,
                    min: 0.0,
                    max: 0.0,
                    q25: 0.0,
                    q75: 0.0,
                });
            }
        }

        if (!is_scale || !options.include_stats) && options.include_counts {
            // Categorical frequencies
            let mut counts: BTreeMap<String, (usize, bool)> = BTreeMap::new();
            let mut valid_total = 0usize;
            let total_rows = col.values.len();

            for val_opt in col.values {
                match val_opt {
                    Some(s) if !s.trim().is_empty() => {
                        let entry = counts.entry(s.clone()).or_insert((0, false));
                        entry.0 += 1;
                        valid_total += 1;
                    }
                    _ => {
                        let entry = counts.entry("(Missing)".to_string()).or_insert((0, true));
                        entry.0 += 1;
                    }
                }
            }

            if counts.len() <= options.max_categories {
                let mut rows = Vec::new();
                let mut cum_valid = 0usize;

                for (val, (cnt, is_missing)) in counts {
                    let pct = if total_rows > 0 {
                        (cnt as f64 / total_rows as f64) * 100.0
                    } else {
                        0.0
                    };

                    let (val_pct, cum_pct) = if !is_missing && valid_total > 0 {
                        cum_valid += cnt;
                        let vp = (cnt as f64 / valid_total as f64) * 100.0;
                        let cp = (cum_valid as f64 / valid_total as f64) * 100.0;
                        (Some(vp), Some(cp))
                    } else {
                        (None, None)
                    };

                    let label = var.values.get_label(&val).map(|s| s.to_string());

                    rows.push(CodebookFrequencyRow {
                        value: val,
                        label,
                        count: cnt,
                        percent: pct,
                        valid_percent: val_pct,
                        cumulative_percent: cum_pct,
                        is_missing,
                    });
                }
                frequencies = Some(rows);
            }
        }

        let label_opt = if var.label.trim().is_empty() {
            None
        } else {
            Some(var.label.clone())
        };

        cards.push(CodebookVariableCard {
            name: var.name.clone(),
            position: pos + 1,
            label: label_opt,
            data_type: data_type_str,
            measure: measure_str,
            missing_description: missing_desc,
            frequencies,
            scale_stats,
        });
    }

    Ok(CodebookResult {
        dataset_name: None,
        total_cases,
        total_variables: variables.len(),
        cards,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::variable::DataType;

    #[test]
    fn test_codebook_scale_and_nominal() {
        let mut var_age =
            VariableMetadata::new("age", DataType::Numeric { decimals: 0 }, Measure::Scale)
                .expect("metadata valid");
        var_age.label = "Age in years".to_string();

        let mut var_gender =
            VariableMetadata::new("gender", DataType::String { max_len: 1 }, Measure::Nominal)
                .expect("metadata valid");
        var_gender.label = "Respondent Gender".to_string();

        let age_str = vec![
            Some("20".to_string()),
            Some("30".to_string()),
            Some("40".to_string()),
        ];
        let age_num = vec![Some(20.0), Some(30.0), Some(40.0)];
        let gender_str = vec![
            Some("F".to_string()),
            Some("M".to_string()),
            Some("F".to_string()),
        ];
        let gender_num = vec![None, None, None];

        let cols = vec![
            RawColumnData {
                values: &age_str,
                numeric_values: &age_num,
            },
            RawColumnData {
                values: &gender_str,
                numeric_values: &gender_num,
            },
        ];

        let res = compute_codebook(&[&var_age, &var_gender], &cols, &CodebookOptions::default())
            .expect("codebook failed");

        assert_eq!(res.total_cases, 3);
        assert_eq!(res.cards.len(), 2);

        let age_card = &res.cards[0];
        assert_eq!(age_card.name, "age");
        let stats = age_card.scale_stats.as_ref().unwrap();
        assert_eq!(stats.valid_count, 3);
        assert!((stats.mean - 30.0).abs() < 1e-9);

        let gender_card = &res.cards[1];
        assert_eq!(gender_card.name, "gender");
        let freq = gender_card.frequencies.as_ref().unwrap();
        assert_eq!(freq.len(), 2);
    }
}
