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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrequencyResult {
    pub variable: String,
    pub n_valid: usize,
    pub n_missing: usize,
    pub categories: Vec<FrequencyCategory>,
    pub quartiles: Option<(f64, f64, f64)>,
}

pub type FrequencyTable = FrequencyResult;

#[derive(Debug, Clone)]
pub struct FrequencyInputItem {
    pub value: String,
    pub label: Option<String>,
    pub is_missing: bool,
    pub numeric_value: Option<f64>,
}

/// Computes quartiles (Q1 = 25%, Q2 = 50% / Median, Q3 = 75%)
/// using the standard SPSS HAVERAGE (Weibull / Type 6) method: rank = p * (n + 1).
pub fn compute_quartiles(sorted_numeric_values: &[f64]) -> Option<(f64, f64, f64)> {
    if sorted_numeric_values.is_empty() {
        return None;
    }
    let q1 = compute_percentile(sorted_numeric_values, 0.25)?;
    let q2 = compute_percentile(sorted_numeric_values, 0.50)?;
    let q3 = compute_percentile(sorted_numeric_values, 0.75)?;
    Some((q1, q2, q3))
}

/// Computes a percentile using the SPSS HAVERAGE interpolation method:
/// rank = p * (n + 1).
pub fn compute_percentile(sorted_values: &[f64], p: f64) -> Option<f64> {
    let n = sorted_values.len();
    if n == 0 {
        return None;
    }
    if n == 1 {
        return Some(sorted_values[0]);
    }

    let r = p * (n as f64 + 1.0);
    if r <= 1.0 {
        return Some(sorted_values[0]);
    }
    if r >= n as f64 {
        return Some(sorted_values[n - 1]);
    }

    let k = r.floor() as usize;
    let d = r - r.floor();
    // 1-based rank k maps to index k - 1
    if k == 0 {
        return Some(sorted_values[0]);
    }
    let val_k = sorted_values.get(k - 1).copied()?;
    let val_k_next = sorted_values.get(k).copied()?;
    Some(val_k + d * (val_k_next - val_k))
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
                });
            }
            _ => {
                items.push(FrequencyInputItem {
                    value: String::new(),
                    label: None,
                    is_missing: true,
                    numeric_value: None,
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
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut labels: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut valid_numbers: Vec<f64> = Vec::new();
    let mut n_missing = 0_usize;
    let mut n_valid = 0_usize;

    for item in items {
        if item.is_missing || item.value.trim().is_empty() {
            n_missing += 1;
        } else {
            let key = item.value.trim().to_string();
            *counts.entry(key.clone()).or_insert(0) += 1;
            labels.entry(key).or_insert_with(|| item.label.clone());
            n_valid += 1;

            if let Some(num) = item.numeric_value {
                if num.is_finite() {
                    valid_numbers.push(num);
                }
            }
        }
    }

    let n_total = items.len();
    let n_total_f = n_total as f64;
    let n_valid_f = n_valid as f64;

    // Sort categories: numerically if both are parseable as floats, otherwise lexically
    let mut sorted_keys: Vec<String> = counts.keys().cloned().collect();
    sorted_keys.sort_by(|a, b| match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
        _ => a.cmp(b),
    });

    let mut categories = Vec::with_capacity(sorted_keys.len());
    let mut running_cum_percent = 0.0_f64;

    for key in sorted_keys {
        let count = counts.get(&key).copied().unwrap_or(0);
        let label = labels.get(&key).cloned().flatten();

        let percent = if n_total > 0 {
            (count as f64 / n_total_f) * 100.0
        } else {
            0.0
        };

        let valid_percent = if n_valid > 0 {
            Some((count as f64 / n_valid_f) * 100.0)
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
    let quartiles = if !valid_numbers.is_empty() && valid_numbers.len() == n_valid {
        compute_quartiles(&valid_numbers)
    } else {
        None
    };

    FrequencyResult {
        variable: var_name.to_string(),
        n_valid,
        n_missing,
        categories,
        quartiles,
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
    }

    #[test]
    fn test_frequency_numeric_sorting_and_quartiles() {
        // Sample numbers: 10, 20, 30, 40, 50
        let items = vec![
            FrequencyInputItem {
                value: "50".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(50.0),
            },
            FrequencyInputItem {
                value: "10".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(10.0),
            },
            FrequencyInputItem {
                value: "20".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(20.0),
            },
            FrequencyInputItem {
                value: "40".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(40.0),
            },
            FrequencyInputItem {
                value: "30".to_string(),
                label: None,
                is_missing: false,
                numeric_value: Some(30.0),
            },
            FrequencyInputItem {
                value: "99".to_string(),
                label: Some("Missing".to_string()),
                is_missing: true,
                numeric_value: Some(99.0),
            },
        ];

        let result = compute_frequencies_detailed("score", &items);

        assert_eq!(result.n_valid, 5);
        assert_eq!(result.n_missing, 1);
        assert_eq!(result.categories.len(), 5);

        // Numeric order check: 10, 20, 30, 40, 50
        assert_eq!(result.categories[0].value, "10");
        assert_eq!(result.categories[1].value, "20");
        assert_eq!(result.categories[2].value, "30");
        assert_eq!(result.categories[3].value, "40");
        assert_eq!(result.categories[4].value, "50");

        // Quartiles for [10, 20, 30, 40, 50]:
        // n = 5
        // r(0.25) = 0.25 * 6 = 1.5 -> 10 + 0.5 * (20 - 10) = 15.0
        // r(0.50) = 0.50 * 6 = 3.0 -> 30.0
        // r(0.75) = 0.75 * 6 = 4.5 -> 40 + 0.5 * (50 - 40) = 45.0
        let (q1, q2, q3) = result.quartiles.expect("Quartiles should be calculated");
        assert!((q1 - 15.0).abs() < 1e-6);
        assert!((q2 - 30.0).abs() < 1e-6);
        assert!((q3 - 45.0).abs() < 1e-6);
    }
}
