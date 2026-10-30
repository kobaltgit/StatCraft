use serde::{Deserialize, Serialize};
use statcraft_core::RawValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueCriterion {
    ExactNumber(f64),
    ExactString(String),
    Range { low: f64, high: f64 },
    LowestThru(f64),
    ThruHighest(f64),
    SystemMissing,
}

impl ValueCriterion {
    pub fn matches(&self, val: &RawValue) -> bool {
        match (self, val) {
            (ValueCriterion::SystemMissing, RawValue::Null) => true,
            (ValueCriterion::SystemMissing, RawValue::Number(n)) => n.is_nan(),
            (ValueCriterion::ExactNumber(target), RawValue::Number(n)) => {
                !n.is_nan() && (n - target).abs() < 1e-9
            }
            (ValueCriterion::ExactNumber(target), RawValue::Text(s)) => s
                .trim()
                .parse::<f64>()
                .map(|n| (n - target).abs() < 1e-9)
                .unwrap_or(false),
            (ValueCriterion::ExactString(target), RawValue::Text(s)) => s == target,
            (ValueCriterion::ExactString(target), RawValue::Number(n)) => {
                if let Ok(num) = target.trim().parse::<f64>() {
                    !n.is_nan() && (n - num).abs() < 1e-9
                } else {
                    false
                }
            }
            (ValueCriterion::Range { low, high }, RawValue::Number(n)) => {
                let min = low.min(*high);
                let max = low.max(*high);
                !n.is_nan() && *n >= min && *n <= max
            }
            (ValueCriterion::Range { low, high }, RawValue::Text(s)) => {
                let min = low.min(*high);
                let max = low.max(*high);
                s.trim()
                    .parse::<f64>()
                    .map(|n| !n.is_nan() && n >= min && n <= max)
                    .unwrap_or(false)
            }
            (ValueCriterion::LowestThru(high), RawValue::Number(n)) => !n.is_nan() && *n <= *high,
            (ValueCriterion::LowestThru(high), RawValue::Text(s)) => s
                .trim()
                .parse::<f64>()
                .map(|n| !n.is_nan() && n <= *high)
                .unwrap_or(false),
            (ValueCriterion::ThruHighest(low), RawValue::Number(n)) => !n.is_nan() && *n >= *low,
            (ValueCriterion::ThruHighest(low), RawValue::Text(s)) => s
                .trim()
                .parse::<f64>()
                .map(|n| !n.is_nan() && n >= *low)
                .unwrap_or(false),
            _ => false,
        }
    }

    pub fn to_sps(&self) -> String {
        match self {
            ValueCriterion::ExactNumber(n) => format_num(*n),
            ValueCriterion::ExactString(s) => format!("'{}'", s),
            ValueCriterion::Range { low, high } => {
                format!("{} THRU {}", format_num(*low), format_num(*high))
            }
            ValueCriterion::LowestThru(high) => format!("LOWEST THRU {}", format_num(*high)),
            ValueCriterion::ThruHighest(low) => format!("{} THRU HIGHEST", format_num(*low)),
            ValueCriterion::SystemMissing => "SYSMIS".to_string(),
        }
    }

    pub fn display_label(&self) -> String {
        match self {
            ValueCriterion::ExactNumber(n) => format_num(*n),
            ValueCriterion::ExactString(s) => format!("\"{}\"", s),
            ValueCriterion::Range { low, high } => {
                format!("{} through {}", format_num(*low), format_num(*high))
            }
            ValueCriterion::LowestThru(high) => format!("LOWEST through {}", format_num(*high)),
            ValueCriterion::ThruHighest(low) => format!("{} through HIGHEST", format_num(*low)),
            ValueCriterion::SystemMissing => "SYSMIS".to_string(),
        }
    }
}

fn format_num(val: f64) -> String {
    if val.fract() == 0.0 && val.abs() < 1e12 {
        format!("{:.0}", val)
    } else {
        format!("{}", val)
    }
}

pub fn compute_count_across_columns(
    rows: usize,
    columns_data: &[&[RawValue]],
    criteria: &[ValueCriterion],
) -> Vec<RawValue> {
    let mut result = Vec::with_capacity(rows);
    for row_idx in 0..rows {
        let mut count = 0;
        for col in columns_data {
            if let Some(val) = col.get(row_idx) {
                if criteria.iter().any(|c| c.matches(val)) {
                    count += 1;
                }
            }
        }
        result.push(RawValue::Number(count as f64));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_criterion_matches_exact_number() {
        let crit = ValueCriterion::ExactNumber(5.0);
        assert!(crit.matches(&RawValue::Number(5.0)));
        assert!(crit.matches(&RawValue::Number(5.0000000001)));
        assert!(!crit.matches(&RawValue::Number(4.9)));
        assert!(crit.matches(&RawValue::Text("5".to_string())));
        assert!(!crit.matches(&RawValue::Null));
    }

    #[test]
    fn test_criterion_matches_exact_string() {
        let crit = ValueCriterion::ExactString("Yes".to_string());
        assert!(crit.matches(&RawValue::Text("Yes".to_string())));
        assert!(!crit.matches(&RawValue::Text("No".to_string())));
        assert!(!crit.matches(&RawValue::Null));
    }

    #[test]
    fn test_criterion_matches_range() {
        let crit = ValueCriterion::Range {
            low: 10.0,
            high: 20.0,
        };
        assert!(crit.matches(&RawValue::Number(10.0)));
        assert!(crit.matches(&RawValue::Number(15.5)));
        assert!(crit.matches(&RawValue::Number(20.0)));
        assert!(!crit.matches(&RawValue::Number(9.99)));
        assert!(!crit.matches(&RawValue::Number(20.01)));
    }

    #[test]
    fn test_criterion_matches_lowest_thru() {
        let crit = ValueCriterion::LowestThru(0.0);
        assert!(crit.matches(&RawValue::Number(-100.0)));
        assert!(crit.matches(&RawValue::Number(0.0)));
        assert!(!crit.matches(&RawValue::Number(0.1)));
    }

    #[test]
    fn test_criterion_matches_thru_highest() {
        let crit = ValueCriterion::ThruHighest(100.0);
        assert!(!crit.matches(&RawValue::Number(99.9)));
        assert!(crit.matches(&RawValue::Number(100.0)));
        assert!(crit.matches(&RawValue::Number(500.0)));
    }

    #[test]
    fn test_criterion_matches_system_missing() {
        let crit = ValueCriterion::SystemMissing;
        assert!(crit.matches(&RawValue::Null));
        assert!(crit.matches(&RawValue::Number(f64::NAN)));
        assert!(!crit.matches(&RawValue::Number(0.0)));
        assert!(!crit.matches(&RawValue::Text("".to_string())));
    }

    #[test]
    fn test_compute_count_across_columns_basic() {
        let col1 = vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(3.0),
            RawValue::Null,
        ];
        let col2 = vec![
            RawValue::Number(2.0),
            RawValue::Number(2.0),
            RawValue::Number(5.0),
            RawValue::Number(1.0),
        ];
        let col3 = vec![
            RawValue::Number(1.0),
            RawValue::Null,
            RawValue::Number(2.0),
            RawValue::Null,
        ];

        let cols: Vec<&[RawValue]> = vec![&col1, &col2, &col3];
        let criteria = vec![
            ValueCriterion::ExactNumber(1.0),
            ValueCriterion::ExactNumber(2.0),
        ];

        let result = compute_count_across_columns(4, &cols, &criteria);
        assert_eq!(
            result,
            vec![
                RawValue::Number(3.0), // row 0: col1=1 (yes), col2=2 (yes), col3=1 (yes) -> 3
                RawValue::Number(2.0), // row 1: col1=2 (yes), col2=2 (yes), col3=Null (no) -> 2
                RawValue::Number(1.0), // row 2: col1=3 (no), col2=5 (no), col3=2 (yes) -> 1
                RawValue::Number(1.0), // row 3: col1=Null (no), col2=1 (yes), col3=Null (no) -> 1
            ]
        );
    }

    #[test]
    fn test_compute_count_with_system_missing() {
        let col1 = vec![RawValue::Null, RawValue::Number(5.0)];
        let col2 = vec![RawValue::Number(5.0), RawValue::Null];

        let cols: Vec<&[RawValue]> = vec![&col1, &col2];
        let criteria = vec![ValueCriterion::SystemMissing];

        let result = compute_count_across_columns(2, &cols, &criteria);
        assert_eq!(result, vec![RawValue::Number(1.0), RawValue::Number(1.0)]);
    }

    #[test]
    fn test_criterion_sps_and_labels() {
        assert_eq!(ValueCriterion::ExactNumber(42.0).to_sps(), "42");
        assert_eq!(ValueCriterion::ExactString("A".to_string()).to_sps(), "'A'");
        assert_eq!(
            ValueCriterion::Range {
                low: 1.0,
                high: 5.0
            }
            .to_sps(),
            "1 THRU 5"
        );
        assert_eq!(ValueCriterion::LowestThru(10.0).to_sps(), "LOWEST THRU 10");
        assert_eq!(
            ValueCriterion::ThruHighest(20.0).to_sps(),
            "20 THRU HIGHEST"
        );
        assert_eq!(ValueCriterion::SystemMissing.to_sps(), "SYSMIS");
    }
}
