use serde::{Deserialize, Serialize};
use statcraft_core::RawValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShiftDirection {
    Lag,
    Lead,
}

pub fn compute_shift_raw(
    values: &[RawValue],
    direction: ShiftDirection,
    step: usize,
) -> Vec<RawValue> {
    let n = values.len();
    if step == 0 || n == 0 {
        return values.to_vec();
    }
    if step >= n {
        return vec![RawValue::Null; n];
    }

    let mut result = Vec::with_capacity(n);
    match direction {
        ShiftDirection::Lag => {
            for _ in 0..step {
                result.push(RawValue::Null);
            }
            for item in &values[..n - step] {
                result.push(item.clone());
            }
        }
        ShiftDirection::Lead => {
            for item in &values[step..] {
                result.push(item.clone());
            }
            for _ in 0..step {
                result.push(RawValue::Null);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_shift_lag() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Number(20.0),
            RawValue::Number(30.0),
            RawValue::Number(40.0),
        ];

        let lag1 = compute_shift_raw(&vals, ShiftDirection::Lag, 1);
        assert_eq!(
            lag1,
            vec![
                RawValue::Null,
                RawValue::Number(10.0),
                RawValue::Number(20.0),
                RawValue::Number(30.0),
            ]
        );

        let lag2 = compute_shift_raw(&vals, ShiftDirection::Lag, 2);
        assert_eq!(
            lag2,
            vec![
                RawValue::Null,
                RawValue::Null,
                RawValue::Number(10.0),
                RawValue::Number(20.0),
            ]
        );
    }

    #[test]
    fn test_compute_shift_lead() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Number(20.0),
            RawValue::Number(30.0),
            RawValue::Number(40.0),
        ];

        let lead1 = compute_shift_raw(&vals, ShiftDirection::Lead, 1);
        assert_eq!(
            lead1,
            vec![
                RawValue::Number(20.0),
                RawValue::Number(30.0),
                RawValue::Number(40.0),
                RawValue::Null,
            ]
        );

        let lead2 = compute_shift_raw(&vals, ShiftDirection::Lead, 2);
        assert_eq!(
            lead2,
            vec![
                RawValue::Number(30.0),
                RawValue::Number(40.0),
                RawValue::Null,
                RawValue::Null,
            ]
        );
    }

    #[test]
    fn test_compute_shift_zero_step() {
        let vals = vec![RawValue::Number(1.0), RawValue::Number(2.0)];
        assert_eq!(compute_shift_raw(&vals, ShiftDirection::Lag, 0), vals);
        assert_eq!(compute_shift_raw(&vals, ShiftDirection::Lead, 0), vals);
    }

    #[test]
    fn test_compute_shift_step_ge_len() {
        let vals = vec![RawValue::Number(1.0), RawValue::Number(2.0)];
        let res_lag = compute_shift_raw(&vals, ShiftDirection::Lag, 2);
        assert_eq!(res_lag, vec![RawValue::Null, RawValue::Null]);

        let res_lead = compute_shift_raw(&vals, ShiftDirection::Lead, 5);
        assert_eq!(res_lead, vec![RawValue::Null, RawValue::Null]);
    }

    #[test]
    fn test_compute_shift_empty() {
        let vals: Vec<RawValue> = vec![];
        let res = compute_shift_raw(&vals, ShiftDirection::Lag, 3);
        assert!(res.is_empty());
    }

    #[test]
    fn test_compute_shift_with_nulls_and_strings() {
        let vals = vec![
            RawValue::Text("first".to_string()),
            RawValue::Null,
            RawValue::Text("third".to_string()),
        ];
        let res_lag = compute_shift_raw(&vals, ShiftDirection::Lag, 1);
        assert_eq!(
            res_lag,
            vec![
                RawValue::Null,
                RawValue::Text("first".to_string()),
                RawValue::Null,
            ]
        );

        let res_lead = compute_shift_raw(&vals, ShiftDirection::Lead, 1);
        assert_eq!(
            res_lead,
            vec![
                RawValue::Null,
                RawValue::Text("third".to_string()),
                RawValue::Null,
            ]
        );
    }
}
