use super::types::AdpScreeningOptions;
use statcraft_core::RawValue;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ScreeningVerdict {
    pub is_screened_out: bool,
    pub reason: Option<String>,
    pub missing_pct: f64,
    pub max_freq_pct: f64,
    pub unique_pct: f64,
}

/// Evaluates a candidate input column against ADP screening rules:
/// 1. Excessive missing values (> max_missing_pct)
/// 2. Quasi-constant values (mode frequency > max_category_pct)
/// 3. Identifier-like high cardinality (> max_unique_pct with N >= 20)
pub fn evaluate_screening(
    values: &[RawValue],
    weights: Option<&[f64]>,
    options: &AdpScreeningOptions,
) -> ScreeningVerdict {
    let n = values.len();
    if n == 0 {
        return ScreeningVerdict {
            is_screened_out: true,
            reason: Some("Field contains 0 cases".to_string()),
            missing_pct: 100.0,
            max_freq_pct: 0.0,
            unique_pct: 0.0,
        };
    }

    let mut total_weight = 0.0;
    let mut missing_weight = 0.0;
    let mut valid_weight = 0.0;

    // Map serialized key -> (frequency count, weighted count)
    let mut freq_map: HashMap<String, (usize, f64)> = HashMap::new();
    let mut valid_count = 0usize;

    for (i, val) in values.iter().enumerate() {
        let w = match weights {
            Some(w_slice) if i < w_slice.len() => {
                let weight_val = w_slice[i];
                if weight_val.is_finite() && weight_val > 0.0 {
                    weight_val
                } else {
                    0.0
                }
            }
            _ => 1.0,
        };

        total_weight += w;

        match val {
            RawValue::Null => {
                missing_weight += w;
            }
            RawValue::Number(num) if num.is_nan() => {
                missing_weight += w;
            }
            RawValue::Number(num) => {
                valid_count += 1;
                valid_weight += w;
                let key = if num.fract() == 0.0 {
                    format!("{:.0}", num)
                } else {
                    format!("{:.8}", num)
                };
                let entry = freq_map.entry(key).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += w;
            }
            RawValue::Text(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    missing_weight += w;
                } else {
                    valid_count += 1;
                    valid_weight += w;
                    let entry = freq_map.entry(trimmed.to_string()).or_insert((0, 0.0));
                    entry.0 += 1;
                    entry.1 += w;
                }
            }
        }
    }

    let missing_pct = if total_weight > 0.0 {
        (missing_weight / total_weight) * 100.0
    } else {
        100.0
    };

    if valid_count == 0 || valid_weight <= 0.0 {
        return ScreeningVerdict {
            is_screened_out: true,
            reason: Some(format!(
                "Too many missing values ({:.1}% > {:.1}%)",
                missing_pct, options.max_missing_pct
            )),
            missing_pct,
            max_freq_pct: 0.0,
            unique_pct: 0.0,
        };
    }

    if missing_pct > options.max_missing_pct {
        return ScreeningVerdict {
            is_screened_out: true,
            reason: Some(format!(
                "Too many missing values ({:.1}% > {:.1}%)",
                missing_pct, options.max_missing_pct
            )),
            missing_pct,
            max_freq_pct: 0.0,
            unique_pct: 0.0,
        };
    }

    let unique_count = freq_map.len();
    let unique_pct = (unique_count as f64 / valid_count as f64) * 100.0;

    let max_weighted_freq = freq_map
        .values()
        .map(|(_, w)| *w)
        .fold(0.0_f64, |a, b| a.max(b));
    let max_freq_pct = (max_weighted_freq / valid_weight) * 100.0;

    // Rule 2: Constant or quasi-constant (e.g. mode > 95%)
    if max_freq_pct > options.max_category_pct {
        return ScreeningVerdict {
            is_screened_out: true,
            reason: Some(format!(
                "Constant or quasi-constant (predominant value represents {:.1}% of cases)",
                max_freq_pct
            )),
            missing_pct,
            max_freq_pct,
            unique_pct,
        };
    }

    // Rule 3: Identifier screening (high cardinality unique values, only if valid_count >= 20)
    if valid_count >= 20 && unique_pct > options.max_unique_pct {
        return ScreeningVerdict {
            is_screened_out: true,
            reason: Some(format!(
                "High-cardinality identifier ({:.1}% unique values)",
                unique_pct
            )),
            missing_pct,
            max_freq_pct,
            unique_pct,
        };
    }

    ScreeningVerdict {
        is_screened_out: false,
        reason: None,
        missing_pct,
        max_freq_pct,
        unique_pct,
    }
}
