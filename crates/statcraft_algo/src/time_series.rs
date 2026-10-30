use serde::{Deserialize, Serialize};
use statcraft_core::RawValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSeriesFunction {
    Diff(usize),
    SeasonalDiff(usize),
    SimpleMovingAverage(usize),
    CumulativeSum,
}

pub fn compute_time_series_transform(
    values: &[RawValue],
    function: TimeSeriesFunction,
) -> Vec<RawValue> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }

    match function {
        TimeSeriesFunction::Diff(order) => {
            if order == 0 {
                return values.to_vec();
            }
            if order >= n {
                return vec![RawValue::Null; n];
            }

            let mut current: Vec<Option<f64>> = values
                .iter()
                .map(|v| match v {
                    RawValue::Number(x) if !x.is_nan() => Some(*x),
                    _ => None,
                })
                .collect();

            for step in 1..=order {
                let mut next = vec![None; n];
                for t in step..n {
                    if let (Some(curr), Some(prev)) = (current[t], current[t - 1]) {
                        let diff = curr - prev;
                        if diff.is_finite() {
                            next[t] = Some(diff);
                        }
                    }
                }
                current = next;
            }

            current
                .into_iter()
                .map(|opt| match opt {
                    Some(val) => RawValue::Number(val),
                    None => RawValue::Null,
                })
                .collect()
        }
        TimeSeriesFunction::SeasonalDiff(period) => {
            if period == 0 {
                return values.to_vec();
            }
            if period >= n {
                return vec![RawValue::Null; n];
            }

            let mut result = Vec::with_capacity(n);
            for _ in 0..period {
                result.push(RawValue::Null);
            }
            for t in period..n {
                match (&values[t], &values[t - period]) {
                    (RawValue::Number(curr), RawValue::Number(prev))
                        if !curr.is_nan() && !prev.is_nan() =>
                    {
                        let diff = curr - prev;
                        if diff.is_finite() {
                            result.push(RawValue::Number(diff));
                        } else {
                            result.push(RawValue::Null);
                        }
                    }
                    _ => result.push(RawValue::Null),
                }
            }
            result
        }
        TimeSeriesFunction::SimpleMovingAverage(span) => {
            if span == 0 || span > n {
                return vec![RawValue::Null; n];
            }

            let mut result = Vec::with_capacity(n);
            let span_f64 = span as f64;

            for _ in 0..(span - 1) {
                result.push(RawValue::Null);
            }

            for t in (span - 1)..n {
                let window = &values[(t + 1 - span)..=t];
                let mut sum = 0.0;
                let mut valid = true;
                for item in window {
                    match item {
                        RawValue::Number(num) if !num.is_nan() => {
                            sum += num;
                        }
                        _ => {
                            valid = false;
                            break;
                        }
                    }
                }
                if valid && sum.is_finite() {
                    result.push(RawValue::Number(sum / span_f64));
                } else {
                    result.push(RawValue::Null);
                }
            }
            result
        }
        TimeSeriesFunction::CumulativeSum => {
            let mut result = Vec::with_capacity(n);
            let mut running_sum = 0.0;
            let mut broken = false;

            for val in values {
                if broken {
                    result.push(RawValue::Null);
                    continue;
                }
                match val {
                    RawValue::Number(num) if !num.is_nan() => {
                        running_sum += num;
                        if running_sum.is_finite() {
                            result.push(RawValue::Number(running_sum));
                        } else {
                            broken = true;
                            result.push(RawValue::Null);
                        }
                    }
                    _ => {
                        broken = true;
                        result.push(RawValue::Null);
                    }
                }
            }
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let empty: Vec<RawValue> = vec![];
        assert_eq!(
            compute_time_series_transform(&empty, TimeSeriesFunction::Diff(1)),
            Vec::<RawValue>::new()
        );
        assert_eq!(
            compute_time_series_transform(&empty, TimeSeriesFunction::SeasonalDiff(2)),
            Vec::<RawValue>::new()
        );
        assert_eq!(
            compute_time_series_transform(&empty, TimeSeriesFunction::SimpleMovingAverage(3)),
            Vec::<RawValue>::new()
        );
        assert_eq!(
            compute_time_series_transform(&empty, TimeSeriesFunction::CumulativeSum),
            Vec::<RawValue>::new()
        );
    }

    #[test]
    fn test_diff_order_1() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Number(15.0),
            RawValue::Number(22.0),
            RawValue::Number(20.0),
            RawValue::Number(28.0),
        ];
        let diff = compute_time_series_transform(&vals, TimeSeriesFunction::Diff(1));
        assert_eq!(
            diff,
            vec![
                RawValue::Null,
                RawValue::Number(5.0),
                RawValue::Number(7.0),
                RawValue::Number(-2.0),
                RawValue::Number(8.0),
            ]
        );
    }

    #[test]
    fn test_diff_order_2() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Number(15.0),
            RawValue::Number(22.0),
            RawValue::Number(20.0),
            RawValue::Number(28.0),
        ];
        // Order 1: [Null, 5.0, 7.0, -2.0, 8.0]
        // Order 2: [Null, Null, 2.0, -9.0, 10.0]
        let diff = compute_time_series_transform(&vals, TimeSeriesFunction::Diff(2));
        assert_eq!(
            diff,
            vec![
                RawValue::Null,
                RawValue::Null,
                RawValue::Number(2.0),
                RawValue::Number(-9.0),
                RawValue::Number(10.0),
            ]
        );
    }

    #[test]
    fn test_diff_order_exceeds_len() {
        let vals = vec![RawValue::Number(1.0), RawValue::Number(2.0)];
        let diff = compute_time_series_transform(&vals, TimeSeriesFunction::Diff(5));
        assert_eq!(diff, vec![RawValue::Null, RawValue::Null]);
    }

    #[test]
    fn test_diff_with_missing() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Null,
            RawValue::Number(20.0),
            RawValue::Number(25.0),
        ];
        let diff = compute_time_series_transform(&vals, TimeSeriesFunction::Diff(1));
        assert_eq!(
            diff,
            vec![
                RawValue::Null,
                RawValue::Null,
                RawValue::Null,
                RawValue::Number(5.0),
            ]
        );
    }

    #[test]
    fn test_seasonal_diff() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Number(15.0),
            RawValue::Number(22.0),
            RawValue::Number(20.0),
            RawValue::Number(28.0),
        ];
        let sdiff = compute_time_series_transform(&vals, TimeSeriesFunction::SeasonalDiff(2));
        assert_eq!(
            sdiff,
            vec![
                RawValue::Null,
                RawValue::Null,
                RawValue::Number(12.0), // 22 - 10
                RawValue::Number(5.0),  // 20 - 15
                RawValue::Number(6.0),  // 28 - 22
            ]
        );
    }

    #[test]
    fn test_sma() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Number(20.0),
            RawValue::Number(30.0),
            RawValue::Number(40.0),
        ];
        let sma = compute_time_series_transform(&vals, TimeSeriesFunction::SimpleMovingAverage(3));
        assert_eq!(
            sma,
            vec![
                RawValue::Null,
                RawValue::Null,
                RawValue::Number(20.0), // (10+20+30)/3
                RawValue::Number(30.0), // (20+30+40)/3
            ]
        );
    }

    #[test]
    fn test_sma_with_missing() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Null,
            RawValue::Number(30.0),
            RawValue::Number(40.0),
        ];
        let sma = compute_time_series_transform(&vals, TimeSeriesFunction::SimpleMovingAverage(2));
        assert_eq!(
            sma,
            vec![
                RawValue::Null,
                RawValue::Null,
                RawValue::Null,
                RawValue::Number(35.0), // (30+40)/2
            ]
        );
    }

    #[test]
    fn test_csum() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Number(20.0),
            RawValue::Number(30.0),
            RawValue::Number(40.0),
        ];
        let csum = compute_time_series_transform(&vals, TimeSeriesFunction::CumulativeSum);
        assert_eq!(
            csum,
            vec![
                RawValue::Number(10.0),
                RawValue::Number(30.0),
                RawValue::Number(60.0),
                RawValue::Number(100.0),
            ]
        );
    }

    #[test]
    fn test_csum_with_missing() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Number(20.0),
            RawValue::Null,
            RawValue::Number(40.0),
        ];
        let csum = compute_time_series_transform(&vals, TimeSeriesFunction::CumulativeSum);
        assert_eq!(
            csum,
            vec![
                RawValue::Number(10.0),
                RawValue::Number(30.0),
                RawValue::Null,
                RawValue::Null,
            ]
        );
    }
}
