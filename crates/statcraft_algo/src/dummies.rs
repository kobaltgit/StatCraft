use serde::{Deserialize, Serialize};
use statcraft_core::{RawValue, ValueLabels};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DummySpec {
    pub root_name: String,
    pub category_value: RawValue,
    pub category_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedDummyColumn {
    pub col_name: String,
    pub label: String,
    pub values: Vec<RawValue>,
}

fn raw_value_matches(a: &RawValue, b: &RawValue) -> bool {
    match (a, b) {
        (RawValue::Number(x), RawValue::Number(y)) => {
            if x.is_nan() || y.is_nan() {
                false
            } else {
                (x - y).abs() < 1e-12 || x.total_cmp(y) == Ordering::Equal
            }
        }
        (RawValue::Text(x), RawValue::Text(y)) => x.trim() == y.trim(),
        _ => false,
    }
}

fn raw_value_cmp(a: &RawValue, b: &RawValue) -> Ordering {
    match (a, b) {
        (RawValue::Number(x), RawValue::Number(y)) => x.total_cmp(y),
        (RawValue::Text(x), RawValue::Text(y)) => x.cmp(y),
        (RawValue::Number(_), RawValue::Text(_)) => Ordering::Less,
        (RawValue::Text(_), RawValue::Number(_)) => Ordering::Greater,
        (RawValue::Null, RawValue::Null) => Ordering::Equal,
        (RawValue::Null, _) => Ordering::Less,
        (_, RawValue::Null) => Ordering::Greater,
    }
}

pub fn format_category_raw(val: &RawValue) -> String {
    match val {
        RawValue::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e12 {
                format!("{:.0}", n)
            } else {
                format!("{}", n)
            }
        }
        RawValue::Text(s) => s.trim().to_string(),
        RawValue::Null => String::new(),
    }
}

fn sanitize_name_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_underscore = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_was_underscore = false;
        } else if !last_was_underscore {
            out.push('_');
            last_was_underscore = true;
        }
    }
    out.trim_matches('_').to_string()
}

pub fn generate_dummies_for_column(
    source_name: &str,
    root_name: Option<&str>,
    values: &[RawValue],
    value_labels: Option<&ValueLabels>,
    omit_first: bool,
) -> Vec<GeneratedDummyColumn> {
    let mut categories: Vec<RawValue> = Vec::new();
    for val in values {
        match val {
            RawValue::Null => continue,
            RawValue::Number(n) if n.is_nan() => continue,
            RawValue::Text(s) if s.trim().is_empty() => continue,
            _ => {
                if !categories
                    .iter()
                    .any(|existing| raw_value_matches(existing, val))
                {
                    categories.push(val.clone());
                }
            }
        }
    }

    categories.sort_by(raw_value_cmp);

    let effective_categories: &[RawValue] = if omit_first {
        if categories.is_empty() {
            &[]
        } else {
            &categories[1..]
        }
    } else {
        &categories[..]
    };

    if effective_categories.is_empty() {
        return Vec::new();
    }

    let root = match root_name {
        Some(r) if !r.trim().is_empty() => r.trim(),
        _ => source_name.trim(),
    };

    let mut result = Vec::with_capacity(effective_categories.len());
    let mut used_col_names: Vec<String> = Vec::new();

    for (idx, cat) in effective_categories.iter().enumerate() {
        let cat_raw_str = format_category_raw(cat);
        let cat_label = value_labels
            .and_then(|vl| vl.get_label(&cat_raw_str))
            .map(|s| s.to_string());

        let sanitized = sanitize_name_component(&cat_raw_str);
        let base_col_name = if sanitized.is_empty() {
            format!("{}_{}", root, idx + 1)
        } else {
            format!("{}_{}", root, sanitized)
        };

        let mut final_col_name = base_col_name.clone();
        let mut counter = 1;
        while used_col_names
            .iter()
            .any(|c| c.eq_ignore_ascii_case(&final_col_name))
        {
            final_col_name = format!("{}_{}", base_col_name, counter);
            counter += 1;
        }
        used_col_names.push(final_col_name.clone());

        let col_label = match &cat_label {
            Some(lbl) if !lbl.trim().is_empty() => format!("{} = {}", source_name, lbl.trim()),
            _ => format!("{} = {}", source_name, cat_raw_str),
        };

        let mut col_values = Vec::with_capacity(values.len());
        for row_val in values {
            match row_val {
                RawValue::Null => col_values.push(RawValue::Null),
                RawValue::Number(n) if n.is_nan() => col_values.push(RawValue::Null),
                v => {
                    if raw_value_matches(v, cat) {
                        col_values.push(RawValue::Number(1.0));
                    } else {
                        col_values.push(RawValue::Number(0.0));
                    }
                }
            }
        }

        result.push(GeneratedDummyColumn {
            col_name: final_col_name,
            label: col_label,
            values: col_values,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_dummies_numeric_all() {
        let values = vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(1.0),
            RawValue::Number(3.0),
            RawValue::Null,
        ];
        let mut labels = ValueLabels::new();
        labels.insert("1", "Clerical");
        labels.insert("2", "Custodial");
        labels.insert("3", "Manager");

        let dummies =
            generate_dummies_for_column("jobcat", Some("d_jobcat"), &values, Some(&labels), false);

        assert_eq!(dummies.len(), 3);

        assert_eq!(dummies[0].col_name, "d_jobcat_1");
        assert_eq!(dummies[0].label, "jobcat = Clerical");
        assert_eq!(
            dummies[0].values,
            vec![
                RawValue::Number(1.0),
                RawValue::Number(0.0),
                RawValue::Number(1.0),
                RawValue::Number(0.0),
                RawValue::Null,
            ]
        );

        assert_eq!(dummies[1].col_name, "d_jobcat_2");
        assert_eq!(dummies[1].label, "jobcat = Custodial");
        assert_eq!(
            dummies[1].values,
            vec![
                RawValue::Number(0.0),
                RawValue::Number(1.0),
                RawValue::Number(0.0),
                RawValue::Number(0.0),
                RawValue::Null,
            ]
        );

        assert_eq!(dummies[2].col_name, "d_jobcat_3");
        assert_eq!(dummies[2].label, "jobcat = Manager");
        assert_eq!(
            dummies[2].values,
            vec![
                RawValue::Number(0.0),
                RawValue::Number(0.0),
                RawValue::Number(0.0),
                RawValue::Number(1.0),
                RawValue::Null,
            ]
        );
    }

    #[test]
    fn test_generate_dummies_omit_first() {
        let values = vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(3.0),
        ];

        let dummies = generate_dummies_for_column("cat", None, &values, None, true);

        // First category (1.0) omitted as reference
        assert_eq!(dummies.len(), 2);
        assert_eq!(dummies[0].col_name, "cat_2");
        assert_eq!(dummies[0].label, "cat = 2");
        assert_eq!(
            dummies[0].values,
            vec![
                RawValue::Number(0.0),
                RawValue::Number(1.0),
                RawValue::Number(0.0)
            ]
        );

        assert_eq!(dummies[1].col_name, "cat_3");
        assert_eq!(dummies[1].label, "cat = 3");
        assert_eq!(
            dummies[1].values,
            vec![
                RawValue::Number(0.0),
                RawValue::Number(0.0),
                RawValue::Number(1.0)
            ]
        );
    }

    #[test]
    fn test_generate_dummies_text() {
        let values = vec![
            RawValue::Text("Male".to_string()),
            RawValue::Text("Female".to_string()),
            RawValue::Text("Female".to_string()),
            RawValue::Null,
        ];

        let dummies = generate_dummies_for_column("gender", Some("d_gender"), &values, None, false);

        assert_eq!(dummies.len(), 2);
        // Alphabetical: "Female" comes first, then "Male"
        assert_eq!(dummies[0].col_name, "d_gender_Female");
        assert_eq!(dummies[0].label, "gender = Female");
        assert_eq!(
            dummies[0].values,
            vec![
                RawValue::Number(0.0),
                RawValue::Number(1.0),
                RawValue::Number(1.0),
                RawValue::Null,
            ]
        );

        assert_eq!(dummies[1].col_name, "d_gender_Male");
        assert_eq!(dummies[1].label, "gender = Male");
        assert_eq!(
            dummies[1].values,
            vec![
                RawValue::Number(1.0),
                RawValue::Number(0.0),
                RawValue::Number(0.0),
                RawValue::Null,
            ]
        );
    }

    #[test]
    fn test_generate_dummies_empty() {
        let values: Vec<RawValue> = vec![RawValue::Null, RawValue::Null];
        let dummies = generate_dummies_for_column("empty_var", None, &values, None, false);
        assert!(dummies.is_empty());
    }

    #[test]
    fn test_generate_dummies_single_category_omit_first() {
        let values = vec![RawValue::Number(1.0), RawValue::Number(1.0)];
        let dummies = generate_dummies_for_column("single", None, &values, None, true);
        assert!(dummies.is_empty());
    }
}
