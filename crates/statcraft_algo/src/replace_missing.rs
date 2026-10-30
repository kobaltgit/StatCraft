use serde::{Deserialize, Serialize};
use statcraft_core::RawValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImputationMethod {
    SeriesMean,
    MeanOfNearby(usize),
    MedianOfNearby(usize),
    LinearInterpolation,
    LinearTrendAtPoint,
}

fn extract_valid_number(val: &RawValue) -> Option<f64> {
    match val {
        RawValue::Number(n) if !n.is_nan() && !n.is_infinite() => Some(*n),
        _ => None,
    }
}

pub fn impute_missing_values(values: &[RawValue], method: ImputationMethod) -> Vec<RawValue> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }

    match method {
        ImputationMethod::SeriesMean => {
            let valid_vals: Vec<f64> = values.iter().filter_map(extract_valid_number).collect();
            let mean = if valid_vals.is_empty() {
                None
            } else {
                let sum: f64 = valid_vals.iter().sum();
                Some(sum / (valid_vals.len() as f64))
            };

            values
                .iter()
                .map(|val| {
                    if extract_valid_number(val).is_some() {
                        val.clone()
                    } else if let Some(m) = mean {
                        RawValue::Number(m)
                    } else {
                        RawValue::Null
                    }
                })
                .collect()
        }
        ImputationMethod::MeanOfNearby(span) => {
            let mut result = Vec::with_capacity(n);
            for i in 0..n {
                if let Some(num) = extract_valid_number(&values[i]) {
                    result.push(RawValue::Number(num));
                    continue;
                }

                if span == 0 {
                    result.push(RawValue::Null);
                    continue;
                }

                let start = i.saturating_sub(span);
                let end = (i + span).min(n - 1);

                let mut sum = 0.0;
                let mut count = 0usize;

                for (j, val) in values.iter().enumerate().take(end + 1).skip(start) {
                    if j != i {
                        if let Some(num) = extract_valid_number(val) {
                            sum += num;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    result.push(RawValue::Number(sum / (count as f64)));
                } else {
                    result.push(RawValue::Null);
                }
            }
            result
        }
        ImputationMethod::MedianOfNearby(span) => {
            let mut result = Vec::with_capacity(n);
            for i in 0..n {
                if let Some(num) = extract_valid_number(&values[i]) {
                    result.push(RawValue::Number(num));
                    continue;
                }

                if span == 0 {
                    result.push(RawValue::Null);
                    continue;
                }

                let start = i.saturating_sub(span);
                let end = (i + span).min(n - 1);

                let mut nearby = Vec::new();
                for (j, val) in values.iter().enumerate().take(end + 1).skip(start) {
                    if j != i {
                        if let Some(num) = extract_valid_number(val) {
                            nearby.push(num);
                        }
                    }
                }

                if nearby.is_empty() {
                    result.push(RawValue::Null);
                } else {
                    nearby.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let len = nearby.len();
                    let median = if len % 2 == 1 {
                        nearby[len / 2]
                    } else {
                        (nearby[len / 2 - 1] + nearby[len / 2]) / 2.0
                    };
                    result.push(RawValue::Number(median));
                }
            }
            result
        }
        ImputationMethod::LinearInterpolation => {
            let mut result = Vec::with_capacity(n);
            for i in 0..n {
                if let Some(num) = extract_valid_number(&values[i]) {
                    result.push(RawValue::Number(num));
                    continue;
                }

                let mut prev = None;
                for j in (0..i).rev() {
                    if let Some(num) = extract_valid_number(&values[j]) {
                        prev = Some((j, num));
                        break;
                    }
                }

                let mut next = None;
                for (j, val) in values.iter().enumerate().take(n).skip(i + 1) {
                    if let Some(num) = extract_valid_number(val) {
                        next = Some((j, num));
                        break;
                    }
                }

                match (prev, next) {
                    (Some((i0, y0)), Some((i1, y1))) => {
                        let factor = (i - i0) as f64 / (i1 - i0) as f64;
                        let val = y0 + factor * (y1 - y0);
                        result.push(RawValue::Number(val));
                    }
                    (Some((_i0, y0)), None) => {
                        result.push(RawValue::Number(y0));
                    }
                    (None, Some((_i1, y1))) => {
                        result.push(RawValue::Number(y1));
                    }
                    (None, None) => {
                        result.push(RawValue::Null);
                    }
                }
            }
            result
        }
        ImputationMethod::LinearTrendAtPoint => {
            let valid_points: Vec<(f64, f64)> = values
                .iter()
                .enumerate()
                .filter_map(|(i, val)| extract_valid_number(val).map(|y| ((i + 1) as f64, y)))
                .collect();

            let m = valid_points.len();
            if m == 0 {
                return vec![RawValue::Null; n];
            }

            if m == 1 {
                let single_val = valid_points[0].1;
                return values
                    .iter()
                    .map(|val| {
                        if extract_valid_number(val).is_some() {
                            val.clone()
                        } else {
                            RawValue::Number(single_val)
                        }
                    })
                    .collect();
            }

            let sum_x: f64 = valid_points.iter().map(|(x, _)| *x).sum();
            let sum_y: f64 = valid_points.iter().map(|(_, y)| *y).sum();
            let mean_x = sum_x / (m as f64);
            let mean_y = sum_y / (m as f64);

            let mut s_xx = 0.0;
            let mut s_xy = 0.0;
            for (x, y) in &valid_points {
                let dx = x - mean_x;
                let dy = y - mean_y;
                s_xx += dx * dx;
                s_xy += dx * dy;
            }

            let (slope, intercept) = if s_xx.abs() < 1e-12 {
                (0.0, mean_y)
            } else {
                let b = s_xy / s_xx;
                let a = mean_y - b * mean_x;
                (b, a)
            };

            values
                .iter()
                .enumerate()
                .map(|(i, val)| {
                    if extract_valid_number(val).is_some() {
                        val.clone()
                    } else {
                        let x = (i + 1) as f64;
                        let y = intercept + slope * x;
                        if y.is_nan() || y.is_infinite() {
                            RawValue::Null
                        } else {
                            RawValue::Number(y)
                        }
                    }
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_mean_imputation() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Null,
            RawValue::Number(20.0),
            RawValue::Number(30.0),
        ];

        let imputed = impute_missing_values(&vals, ImputationMethod::SeriesMean);
        assert_eq!(imputed[0], RawValue::Number(10.0));
        assert_eq!(imputed[1], RawValue::Number(20.0));
        assert_eq!(imputed[2], RawValue::Number(20.0));
        assert_eq!(imputed[3], RawValue::Number(30.0));
    }

    #[test]
    fn test_mean_nearby_imputation() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Null,
            RawValue::Number(30.0),
            RawValue::Number(50.0),
        ];

        let imputed = impute_missing_values(&vals, ImputationMethod::MeanOfNearby(1));
        assert_eq!(imputed[0], RawValue::Number(10.0));
        assert_eq!(imputed[1], RawValue::Number(20.0)); // (10 + 30) / 2
        assert_eq!(imputed[2], RawValue::Number(30.0));
        assert_eq!(imputed[3], RawValue::Number(50.0));
    }

    #[test]
    fn test_median_nearby_imputation() {
        let vals = vec![
            RawValue::Number(10.0),
            RawValue::Number(20.0),
            RawValue::Null,
            RawValue::Number(100.0),
            RawValue::Number(120.0),
        ];

        let imputed = impute_missing_values(&vals, ImputationMethod::MedianOfNearby(2));
        assert_eq!(imputed[2], RawValue::Number(60.0)); // (20 + 100) / 2
    }

    #[test]
    fn test_linear_interpolation() {
        let vals = vec![
            RawValue::Null,
            RawValue::Number(10.0),
            RawValue::Null,
            RawValue::Null,
            RawValue::Number(40.0),
            RawValue::Null,
        ];

        let imputed = impute_missing_values(&vals, ImputationMethod::LinearInterpolation);
        assert_eq!(imputed[0], RawValue::Number(10.0)); // Head edge case -> nearest 10
        assert_eq!(imputed[1], RawValue::Number(10.0));
        assert_eq!(imputed[2], RawValue::Number(20.0)); // 10 + 1 * (40 - 10) / 3 = 20
        assert_eq!(imputed[3], RawValue::Number(30.0)); // 10 + 2 * (40 - 10) / 3 = 30
        assert_eq!(imputed[4], RawValue::Number(40.0));
        assert_eq!(imputed[5], RawValue::Number(40.0)); // Tail edge case -> nearest 40
    }

    #[test]
    fn test_linear_trend() {
        let vals = vec![
            RawValue::Number(2.0),  // x=1, y=2
            RawValue::Null,         // x=2, y=4
            RawValue::Number(6.0),  // x=3, y=6
            RawValue::Null,         // x=4, y=8
            RawValue::Number(10.0), // x=5, y=10
        ];

        let imputed = impute_missing_values(&vals, ImputationMethod::LinearTrendAtPoint);
        assert_eq!(imputed[1], RawValue::Number(4.0));
        assert_eq!(imputed[3], RawValue::Number(8.0));
    }

    #[test]
    fn test_empty_and_all_missing() {
        let empty: Vec<RawValue> = vec![];
        let res_empty = impute_missing_values(&empty, ImputationMethod::SeriesMean);
        assert!(res_empty.is_empty());

        let all_null = vec![RawValue::Null, RawValue::Null];
        let res_mean = impute_missing_values(&all_null, ImputationMethod::SeriesMean);
        assert_eq!(res_mean, vec![RawValue::Null, RawValue::Null]);

        let res_linear = impute_missing_values(&all_null, ImputationMethod::LinearInterpolation);
        assert_eq!(res_linear, vec![RawValue::Null, RawValue::Null]);

        let res_trend = impute_missing_values(&all_null, ImputationMethod::LinearTrendAtPoint);
        assert_eq!(res_trend, vec![RawValue::Null, RawValue::Null]);
    }
}
