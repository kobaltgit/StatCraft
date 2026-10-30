use statcraft_core::{RawValue, ValueLabels};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AutoRecodeOrder {
    #[default]
    LowestValue,
    HighestValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutoRecodeResult {
    pub recoded_values: Vec<RawValue>,
    pub value_labels: ValueLabels,
}

fn is_missing_raw(val: &RawValue) -> bool {
    match val {
        RawValue::Null => true,
        RawValue::Number(n) => n.is_nan(),
        RawValue::Text(s) => s.is_empty(),
    }
}

fn format_raw_value(val: &RawValue) -> String {
    match val {
        RawValue::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{:.0}", n)
            } else {
                format!("{}", n)
            }
        }
        RawValue::Text(s) => s.clone(),
        RawValue::Null => String::new(),
    }
}

pub fn compute_auto_recode(values: &[RawValue], order: AutoRecodeOrder) -> AutoRecodeResult {
    let mut unique_values: Vec<RawValue> = values
        .iter()
        .filter(|v| !is_missing_raw(v))
        .cloned()
        .collect();

    match order {
        AutoRecodeOrder::LowestValue => {
            unique_values.sort_by(|a, b| crate::sort_cases::compare_raw_values(Some(a), Some(b)));
        }
        AutoRecodeOrder::HighestValue => {
            unique_values.sort_by(|a, b| crate::sort_cases::compare_raw_values(Some(b), Some(a)));
        }
    }

    unique_values.dedup_by(|a, b| {
        crate::sort_cases::compare_raw_values(Some(a), Some(b)) == Ordering::Equal
    });

    let mut value_labels = ValueLabels::new();
    for (idx, val) in unique_values.iter().enumerate() {
        let code = (idx + 1).to_string();
        let label = format_raw_value(val);
        value_labels.insert(&code, &label);
    }

    let mut recoded_values = Vec::with_capacity(values.len());
    for val in values {
        if is_missing_raw(val) {
            recoded_values.push(RawValue::Null);
        } else {
            let found_idx = unique_values.iter().position(|u| {
                crate::sort_cases::compare_raw_values(Some(val), Some(u)) == Ordering::Equal
            });

            match found_idx {
                Some(idx) => {
                    recoded_values.push(RawValue::Number((idx + 1) as f64));
                }
                None => {
                    recoded_values.push(RawValue::Null);
                }
            }
        }
    }

    AutoRecodeResult {
        recoded_values,
        value_labels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_recode_strings_lowest() {
        let input = vec![
            RawValue::Text("Banana".to_string()),
            RawValue::Text("Apple".to_string()),
            RawValue::Null,
            RawValue::Text("Cherry".to_string()),
            RawValue::Text("Apple".to_string()),
        ];

        let res = compute_auto_recode(&input, AutoRecodeOrder::LowestValue);
        assert_eq!(
            res.recoded_values,
            vec![
                RawValue::Number(2.0),
                RawValue::Number(1.0),
                RawValue::Null,
                RawValue::Number(3.0),
                RawValue::Number(1.0),
            ]
        );

        assert_eq!(res.value_labels.get_label("1"), Some("Apple"));
        assert_eq!(res.value_labels.get_label("2"), Some("Banana"));
        assert_eq!(res.value_labels.get_label("3"), Some("Cherry"));
    }

    #[test]
    fn test_auto_recode_strings_highest() {
        let input = vec![
            RawValue::Text("Banana".to_string()),
            RawValue::Text("Apple".to_string()),
            RawValue::Null,
            RawValue::Text("Cherry".to_string()),
            RawValue::Text("Apple".to_string()),
        ];

        let res = compute_auto_recode(&input, AutoRecodeOrder::HighestValue);
        assert_eq!(
            res.recoded_values,
            vec![
                RawValue::Number(2.0),
                RawValue::Number(3.0),
                RawValue::Null,
                RawValue::Number(1.0),
                RawValue::Number(3.0),
            ]
        );

        assert_eq!(res.value_labels.get_label("1"), Some("Cherry"));
        assert_eq!(res.value_labels.get_label("2"), Some("Banana"));
        assert_eq!(res.value_labels.get_label("3"), Some("Apple"));
    }

    #[test]
    fn test_auto_recode_numbers() {
        let input = vec![
            RawValue::Number(100.0),
            RawValue::Number(20.0),
            RawValue::Number(50.0),
            RawValue::Number(20.0),
            RawValue::Null,
        ];

        let res = compute_auto_recode(&input, AutoRecodeOrder::LowestValue);
        assert_eq!(
            res.recoded_values,
            vec![
                RawValue::Number(3.0),
                RawValue::Number(1.0),
                RawValue::Number(2.0),
                RawValue::Number(1.0),
                RawValue::Null,
            ]
        );

        assert_eq!(res.value_labels.get_label("1"), Some("20"));
        assert_eq!(res.value_labels.get_label("2"), Some("50"));
        assert_eq!(res.value_labels.get_label("3"), Some("100"));
    }

    #[test]
    fn test_auto_recode_empty_and_all_missing() {
        let res_empty = compute_auto_recode(&[], AutoRecodeOrder::LowestValue);
        assert!(res_empty.recoded_values.is_empty());
        assert!(res_empty.value_labels.mapping.is_empty());

        let res_missing = compute_auto_recode(
            &[RawValue::Null, RawValue::Text("".to_string())],
            AutoRecodeOrder::LowestValue,
        );
        assert_eq!(
            res_missing.recoded_values,
            vec![RawValue::Null, RawValue::Null]
        );
        assert!(res_missing.value_labels.mapping.is_empty());
    }
}
