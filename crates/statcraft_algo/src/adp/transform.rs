use super::types::{AdpBinningOptions, AdpCategoricalOptions, AdpRescaleMethod};
use crate::optimal_binning::compute_optimal_binning;
use statcraft_core::{RawValue, ValueLabels};
use std::collections::HashMap;

/// Rescales numeric values using either Z-Score or Min-Max normalization.
pub fn rescale_numeric(
    values: &[RawValue],
    method: AdpRescaleMethod,
    weights: Option<&[f64]>,
) -> Vec<RawValue> {
    match method {
        AdpRescaleMethod::None => values.to_vec(),
        AdpRescaleMethod::ZScore => {
            let mut sum_w = 0.0;
            let mut sum_wx = 0.0;
            for (i, v) in values.iter().enumerate() {
                if let RawValue::Number(num) = v {
                    if num.is_finite() {
                        let w = weights.and_then(|ws| ws.get(i)).copied().unwrap_or(1.0);
                        if w > 0.0 {
                            sum_w += w;
                            sum_wx += w * *num;
                        }
                    }
                }
            }

            if sum_w <= 0.0 {
                return values.to_vec();
            }

            let mean = sum_wx / sum_w;
            let mut sum_w_diff_sq = 0.0;
            let mut sum_w_sq = 0.0;

            for (i, v) in values.iter().enumerate() {
                if let RawValue::Number(num) = v {
                    if num.is_finite() {
                        let w = weights.and_then(|ws| ws.get(i)).copied().unwrap_or(1.0);
                        if w > 0.0 {
                            sum_w_diff_sq += w * (*num - mean).powi(2);
                            sum_w_sq += w * w;
                        }
                    }
                }
            }

            let denom = sum_w - (sum_w_sq / sum_w);
            let sd = if denom > 0.0 {
                (sum_w_diff_sq / denom).max(0.0).sqrt()
            } else {
                0.0
            };

            if sd <= 1e-12 {
                return values.to_vec();
            }

            values
                .iter()
                .map(|v| match v {
                    RawValue::Number(num) if num.is_finite() => {
                        RawValue::Number((*num - mean) / sd)
                    }
                    other => other.clone(),
                })
                .collect()
        }
        AdpRescaleMethod::MinMax { min, max } => {
            let mut min_val = f64::INFINITY;
            let mut max_val = f64::NEG_INFINITY;

            for v in values {
                if let RawValue::Number(num) = v {
                    if num.is_finite() {
                        if *num < min_val {
                            min_val = *num;
                        }
                        if *num > max_val {
                            max_val = *num;
                        }
                    }
                }
            }

            let range = max_val - min_val;
            if range <= 1e-12 {
                return values.to_vec();
            }

            let target_range = max - min;
            values
                .iter()
                .map(|v| match v {
                    RawValue::Number(num) if num.is_finite() => {
                        let normalized = (*num - min_val) / range;
                        RawValue::Number(min + normalized * target_range)
                    }
                    other => other.clone(),
                })
                .collect()
        }
    }
}

pub struct CategoricalMergeResult {
    pub values: Vec<RawValue>,
    pub value_labels: ValueLabels,
    pub merged_count: usize,
}

/// Merges sparse categories with relative frequency < threshold into a combined category.
pub fn merge_sparse_categories(
    values: &[RawValue],
    options: &AdpCategoricalOptions,
    weights: Option<&[f64]>,
) -> CategoricalMergeResult {
    if !options.merge_sparse {
        return CategoricalMergeResult {
            values: values.to_vec(),
            value_labels: ValueLabels::new(),
            merged_count: 0,
        };
    }

    let mut cat_weights: HashMap<String, f64> = HashMap::new();
    let mut total_valid_weight = 0.0;

    for (i, v) in values.iter().enumerate() {
        let w = weights.and_then(|ws| ws.get(i)).copied().unwrap_or(1.0);
        if w <= 0.0 {
            continue;
        }

        match v {
            RawValue::Text(s) => {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    *cat_weights.entry(trimmed.to_string()).or_insert(0.0) += w;
                    total_valid_weight += w;
                }
            }
            RawValue::Number(num) if num.is_finite() => {
                let s = if num.fract() == 0.0 {
                    format!("{:.0}", num)
                } else {
                    format!("{:.4}", num)
                };
                *cat_weights.entry(s).or_insert(0.0) += w;
                total_valid_weight += w;
            }
            _ => {}
        }
    }

    if total_valid_weight <= 0.0 || cat_weights.len() <= 2 {
        return CategoricalMergeResult {
            values: values.to_vec(),
            value_labels: ValueLabels::new(),
            merged_count: 0,
        };
    }

    let mut sparse_categories = Vec::new();
    let mut frequent_categories = Vec::new();

    for (cat, w) in &cat_weights {
        let pct = (*w / total_valid_weight) * 100.0;
        if pct < options.sparse_threshold_pct {
            sparse_categories.push(cat.clone());
        } else {
            frequent_categories.push(cat.clone());
        }
    }

    if sparse_categories.is_empty() {
        return CategoricalMergeResult {
            values: values.to_vec(),
            value_labels: ValueLabels::new(),
            merged_count: 0,
        };
    }

    frequent_categories.sort();
    let mut category_to_code: HashMap<String, i64> = HashMap::new();
    let mut value_labels = ValueLabels::new();

    let mut current_code = 1i64;
    for cat in &frequent_categories {
        category_to_code.insert(cat.clone(), current_code);
        let code_str = current_code.to_string();
        value_labels.insert(&code_str, cat);
        current_code += 1;
    }

    let other_code = current_code;
    for cat in &sparse_categories {
        category_to_code.insert(cat.clone(), other_code);
    }
    let other_code_str = other_code.to_string();
    value_labels.insert(&other_code_str, "Other (Merged)");

    let mut merged_count = 0usize;
    let recoded_values: Vec<RawValue> = values
        .iter()
        .map(|v| {
            let key = match v {
                RawValue::Text(s) => {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                }
                RawValue::Number(num) if num.is_finite() => {
                    if num.fract() == 0.0 {
                        Some(format!("{:.0}", num))
                    } else {
                        Some(format!("{:.4}", num))
                    }
                }
                _ => None,
            };

            if let Some(k) = key {
                if let Some(&code) = category_to_code.get(&k) {
                    if code == other_code {
                        merged_count += 1;
                    }
                    RawValue::Number(code as f64)
                } else {
                    RawValue::Null
                }
            } else {
                RawValue::Null
            }
        })
        .collect();

    CategoricalMergeResult {
        values: recoded_values,
        value_labels,
        merged_count,
    }
}

/// Applies supervised MDLP binning to a continuous predictor with respect to a target.
pub fn apply_supervised_binning(
    feature: &[RawValue],
    target: &[RawValue],
    options: &AdpBinningOptions,
) -> Result<(Vec<RawValue>, ValueLabels), String> {
    let result = compute_optimal_binning(feature, target, options.max_bins)?;
    Ok((result.binned_values, result.value_labels))
}

/// Generates pairwise cross-product interactions for numeric variables: P_var1_X_var2 = var1 * var2.
pub fn compute_interaction_product(
    var1_values: &[RawValue],
    var2_values: &[RawValue],
) -> Vec<RawValue> {
    let n = var1_values.len().min(var2_values.len());
    let mut out = Vec::with_capacity(n);

    for i in 0..n {
        match (&var1_values[i], &var2_values[i]) {
            (RawValue::Number(a), RawValue::Number(b)) if a.is_finite() && b.is_finite() => {
                out.push(RawValue::Number(*a * *b));
            }
            _ => {
                out.push(RawValue::Null);
            }
        }
    }

    out
}
