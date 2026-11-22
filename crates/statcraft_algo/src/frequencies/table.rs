use super::charts::FrequencyChartResult;
use super::options::FrequenciesStatisticsOptions;
use super::percentiles::compute_quartiles;
use super::stats::{compute_frequencies_statistics, FrequenciesStatisticsResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrequencyCategory {
    pub value: String,
    pub label: Option<String>,
    pub count: usize,
    pub percent: f64,
    pub valid_percent: Option<f64>,
    pub cumulative_percent: Option<f64>,
}

impl FrequencyCategory {
    pub fn frequency(&self) -> usize {
        self.count
    }

    pub fn display_label(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FrequencyResult {
    pub variable: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variable_label: Option<String>,
    pub n_valid: usize,
    pub n_missing: usize,
    pub categories: Vec<FrequencyCategory>,
    pub quartiles: Option<(f64, f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistics: Option<FrequenciesStatisticsResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chart: Option<FrequencyChartResult>,
}

impl FrequencyResult {
    pub fn display_title(&self) -> String {
        if let Some(ref lbl) = self.variable_label {
            if !lbl.trim().is_empty() {
                return format!("{} [{}]", lbl.trim(), self.variable);
            }
        }
        self.variable.clone()
    }

    pub fn display_label_or_name(&self) -> &str {
        if let Some(ref lbl) = self.variable_label {
            if !lbl.trim().is_empty() {
                return lbl.as_str();
            }
        }
        &self.variable
    }
}

pub type FrequencyTable = FrequencyResult;

#[derive(Debug, Clone)]
pub struct FrequencyInputItem {
    pub value: String,
    pub label: Option<String>,
    pub is_missing: bool,
    pub numeric_value: Option<f64>,
    pub weight: f64,
}

/// Computes a standard SPSS-style frequency table from string categories.
/// None or empty strings are treated as missing values.
pub fn compute_frequencies(var_name: &str, values: &[Option<String>]) -> FrequencyResult {
    let mut items = Vec::with_capacity(values.len());
    for v in values {
        match v {
            Some(s) if !s.trim().is_empty() => {
                let trimmed = s.trim().to_string();
                let num = trimmed.parse::<f64>().ok();
                items.push(FrequencyInputItem {
                    value: trimmed,
                    label: None,
                    is_missing: false,
                    numeric_value: num,
                    weight: 1.0,
                });
            }
            _ => {
                items.push(FrequencyInputItem {
                    value: String::new(),
                    label: None,
                    is_missing: true,
                    numeric_value: None,
                    weight: 1.0,
                });
            }
        }
    }
    compute_frequencies_detailed(var_name, &items)
}

/// Computes a frequency table with full support for metadata (value labels,
/// user-defined missing values, and numeric quartiles).
pub fn compute_frequencies_detailed(
    var_name: &str,
    items: &[FrequencyInputItem],
) -> FrequencyResult {
    compute_frequencies_full(var_name, items, None)
}

/// Computes a frequency table and optional univariate statistics.
pub fn compute_frequencies_full(
    var_name: &str,
    items: &[FrequencyInputItem],
    stats_options: Option<&FrequenciesStatisticsOptions>,
) -> FrequencyResult {
    let mut counts: BTreeMap<String, f64> = BTreeMap::new();
    let mut labels: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut valid_numbers: Vec<f64> = Vec::new();
    let mut valid_pairs: Vec<(f64, f64)> = Vec::new();
    let mut n_missing = 0_usize;
    let mut sum_valid_weight = 0.0_f64;
    let mut sum_missing_weight = 0.0_f64;
    let mut has_non_numeric = false;

    for item in items {
        if item.weight <= 0.0 || !item.weight.is_finite() {
            continue;
        }

        let w_rounded = item.weight.round() as usize;

        if item.is_missing || item.value.trim().is_empty() {
            n_missing += w_rounded;
            sum_missing_weight += item.weight;
        } else {
            let key = item.value.trim().to_string();
            *counts.entry(key.clone()).or_insert(0.0) += item.weight;
            labels.entry(key).or_insert_with(|| item.label.clone());
            sum_valid_weight += item.weight;

            if let Some(num) = item.numeric_value {
                if num.is_finite() {
                    valid_pairs.push((num, item.weight));
                    for _ in 0..w_rounded.max(1) {
                        valid_numbers.push(num);
                    }
                } else {
                    has_non_numeric = true;
                }
            } else {
                has_non_numeric = true;
            }
        }
    }

    let n_valid = sum_valid_weight.round() as usize;
    let sum_total_weight = sum_valid_weight + sum_missing_weight;

    // Sort categories: numerically if both are parseable as floats, otherwise lexically
    let mut sorted_keys: Vec<String> = counts.keys().cloned().collect();
    sorted_keys.sort_by(|a, b| match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
        _ => a.cmp(b),
    });

    let mut categories = Vec::with_capacity(sorted_keys.len());
    let mut running_cum_percent = 0.0_f64;

    for key in sorted_keys {
        let cat_weight = counts.get(&key).copied().unwrap_or(0.0);
        let count = cat_weight.round() as usize;
        let label = labels.get(&key).cloned().flatten();

        let percent = if sum_total_weight > 0.0 {
            (cat_weight / sum_total_weight) * 100.0
        } else {
            0.0
        };

        let valid_percent = if sum_valid_weight > 0.0 {
            Some((cat_weight / sum_valid_weight) * 100.0)
        } else {
            None
        };

        if let Some(vp) = valid_percent {
            running_cum_percent += vp;
        }

        categories.push(FrequencyCategory {
            value: key,
            label,
            count,
            percent,
            valid_percent,
            cumulative_percent: if valid_percent.is_some() {
                Some(running_cum_percent)
            } else {
                None
            },
        });
    }

    // Compute quartiles if all valid values are numeric
    valid_numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let quartiles = if !valid_numbers.is_empty() && !has_non_numeric {
        compute_quartiles(&valid_numbers)
    } else {
        None
    };

    // Compute statistics if options are provided and any statistic is requested
    let statistics = match stats_options {
        Some(opts) if opts.any_requested() && !has_non_numeric && !valid_pairs.is_empty() => {
            Some(compute_frequencies_statistics(&valid_pairs, opts))
        }
        _ => None,
    };

    FrequencyResult {
        variable: var_name.to_string(),
        variable_label: None,
        n_valid,
        n_missing,
        categories,
        quartiles,
        statistics,
        chart: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_table_calculations() {
        let sample = vec![
            Some("Male".to_string()),
            Some("Female".to_string()),
            Some("Male".to_string()),
            Some("Male".to_string()),
            Some("Female".to_string()),
            None,
        ];

        let table = compute_frequencies("gender", &sample);

        assert_eq!(table.variable, "gender");
        assert_eq!(table.n_valid, 5);
        assert_eq!(table.n_missing, 1);
        assert_eq!(table.categories.len(), 2);

        let female = &table.categories[0];
        assert_eq!(female.value, "Female");
        assert_eq!(female.count, 2);
        assert_eq!(female.frequency(), 2);
        assert!((female.percent - (2.0 / 6.0 * 100.0)).abs() < 1e-6);
        assert!((female.valid_percent.unwrap_or(0.0) - 40.0).abs() < 1e-6);
        assert!((female.cumulative_percent.unwrap_or(0.0) - 40.0).abs() < 1e-6);

        let male = &table.categories[1];
        assert_eq!(male.value, "Male");
        assert_eq!(male.count, 3);
        assert_eq!(male.frequency(), 3);
        assert!((male.percent - (3.0 / 6.0 * 100.0)).abs() < 1e-6);
        assert!((male.valid_percent.unwrap_or(0.0) - 60.0).abs() < 1e-6);
        assert!((male.cumulative_percent.unwrap_or(0.0) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_frequency_empty_sample() {
        let empty: Vec<Option<String>> = vec![];
        let table = compute_frequencies("empty", &empty);
        assert_eq!(table.n_valid, 0);
        assert_eq!(table.n_missing, 0);
        assert!(table.categories.is_empty());
        assert!(table.quartiles.is_none());
        assert!(table.statistics.is_none());
    }

    #[test]
    fn test_frequency_numeric_sorting_and_quartiles() {
        let items = vec![
            FrequencyInputItem {
                value: "50".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(50.0),
                weight: 1.0,
            },
            FrequencyInputItem {
                value: "10".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(10.0),
                weight: 1.0,
            },
            FrequencyInputItem {
                value: "20".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(20.0),
                weight: 1.0,
            },
            FrequencyInputItem {
                value: "40".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(40.0),
                weight: 1.0,
            },
            FrequencyInputItem {
                value: "30".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(30.0),
                weight: 1.0,
            },
            FrequencyInputItem {
                value: "99".to_string(),
                label: Some("Missing".to_string()),
                is_missing: true,
                numeric_value: Some(99.0),
                weight: 1.0,
            },
        ];

        let result = compute_frequencies_detailed("score", &items);

        assert_eq!(result.n_valid, 5);
        assert_eq!(result.n_missing, 1);
        assert_eq!(result.categories.len(), 5);

        assert_eq!(result.categories[0].value, "10");
        assert_eq!(result.categories[1].value, "20");
        assert_eq!(result.categories[2].value, "30");
        assert_eq!(result.categories[3].value, "40");
        assert_eq!(result.categories[4].value, "50");

        let (q1, q2, q3) = result.quartiles.expect("Quartiles should be calculated");
        assert!((q1 - 15.0).abs() < 1e-6);
        assert!((q2 - 30.0).abs() < 1e-6);
        assert!((q3 - 45.0).abs() < 1e-6);
    }

    #[test]
    fn test_frequency_weighted() {
        let items = vec![
            FrequencyInputItem {
                value: "A".to_string(),
                label: None,
                is_missing: false,
                numeric_value: None,
                weight: 10.0,
            },
            FrequencyInputItem {
                value: "B".to_string(),
                label: None,
                is_missing: false,
                numeric_value: None,
                weight: 20.0,
            },
            FrequencyInputItem {
                value: "C".to_string(),
                label: None,
                is_missing: false,
                numeric_value: None,
                weight: 0.0,
            },
            FrequencyInputItem {
                value: "D".to_string(),
                label: None,
                is_missing: false,
                numeric_value: None,
                weight: -5.0,
            },
            FrequencyInputItem {
                value: "".to_string(),
                label: None,
                is_missing: true,
                numeric_value: None,
                weight: 5.0,
            },
        ];

        let result = compute_frequencies_detailed("group", &items);

        assert_eq!(result.n_valid, 30);
        assert_eq!(result.n_missing, 5);
        assert_eq!(result.categories.len(), 2);

        let cat_a = &result.categories[0];
        assert_eq!(cat_a.value, "A");
        assert_eq!(cat_a.count, 10);
        assert!((cat_a.percent - (10.0 / 35.0 * 100.0)).abs() < 1e-6);
        assert!((cat_a.valid_percent.unwrap_or(0.0) - (10.0 / 30.0 * 100.0)).abs() < 1e-6);

        let cat_b = &result.categories[1];
        assert_eq!(cat_b.value, "B");
        assert_eq!(cat_b.count, 20);
        assert!((cat_b.percent - (20.0 / 35.0 * 100.0)).abs() < 1e-6);
        assert!((cat_b.valid_percent.unwrap_or(0.0) - (20.0 / 30.0 * 100.0)).abs() < 1e-6);
    }

    #[test]
    fn test_frequency_result_display_title_and_label() {
        let mut res = FrequencyResult {
            variable: "EDU_HOMEWORK".to_string(),
            variable_label: Some("Time spent on homework".to_string()),
            ..Default::default()
        };
        assert_eq!(res.display_title(), "Time spent on homework [EDU_HOMEWORK]");
        assert_eq!(res.display_label_or_name(), "Time spent on homework");

        res.variable_label = None;
        assert_eq!(res.display_title(), "EDU_HOMEWORK");
        assert_eq!(res.display_label_or_name(), "EDU_HOMEWORK");

        res.variable_label = Some("   ".to_string());
        assert_eq!(res.display_title(), "EDU_HOMEWORK");
        assert_eq!(res.display_label_or_name(), "EDU_HOMEWORK");
    }
}
