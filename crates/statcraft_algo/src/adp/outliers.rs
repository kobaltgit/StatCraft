use super::types::{AdpOutlierOptions, OutlierMethod, OutlierTreatment};
use statcraft_core::RawValue;

#[derive(Debug, Clone, PartialEq)]
pub struct OutlierResult {
    pub treated_values: Vec<RawValue>,
    pub outlier_count: usize,
    pub lower_bound: Option<f64>,
    pub upper_bound: Option<f64>,
}

/// Detects and treats outliers in a numeric column according to ADP options.
pub fn treat_outliers(
    values: &[RawValue],
    weights: Option<&[f64]>,
    options: &AdpOutlierOptions,
) -> OutlierResult {
    if options.method == OutlierMethod::None {
        return OutlierResult {
            treated_values: values.to_vec(),
            outlier_count: 0,
            lower_bound: None,
            upper_bound: None,
        };
    }

    // Collect valid numeric values
    let mut valid_data: Vec<(usize, f64, f64)> = Vec::new(); // (original_idx, value, weight)

    for (i, v) in values.iter().enumerate() {
        if let RawValue::Number(num) = v {
            if num.is_finite() {
                let w = match weights {
                    Some(ws) if i < ws.len() => {
                        let weight_val = ws[i];
                        if weight_val.is_finite() && weight_val > 0.0 {
                            weight_val
                        } else {
                            0.0
                        }
                    }
                    _ => 1.0,
                };
                if w > 0.0 {
                    valid_data.push((i, *num, w));
                }
            }
        }
    }

    if valid_data.len() < 3 {
        return OutlierResult {
            treated_values: values.to_vec(),
            outlier_count: 0,
            lower_bound: None,
            upper_bound: None,
        };
    }

    let (lower_bound, upper_bound) = match options.method {
        OutlierMethod::None => {
            return OutlierResult {
                treated_values: values.to_vec(),
                outlier_count: 0,
                lower_bound: None,
                upper_bound: None,
            }
        }
        OutlierMethod::ZScore { threshold } => {
            let mut sum_w = 0.0;
            let mut sum_wx = 0.0;
            for (_, x, w) in &valid_data {
                sum_w += *w;
                sum_wx += *w * *x;
            }
            if sum_w <= 0.0 {
                return OutlierResult {
                    treated_values: values.to_vec(),
                    outlier_count: 0,
                    lower_bound: None,
                    upper_bound: None,
                };
            }
            let mean = sum_wx / sum_w;

            let mut sum_w_diff_sq = 0.0;
            let mut sum_w_sq = 0.0;
            for (_, x, w) in &valid_data {
                sum_w_diff_sq += *w * (*x - mean).powi(2);
                sum_w_sq += *w * *w;
            }

            let denom = sum_w - (sum_w_sq / sum_w);
            let variance = if denom > 0.0 {
                sum_w_diff_sq / denom
            } else {
                0.0
            };
            let sd = variance.max(0.0).sqrt();

            if sd <= 1e-12 {
                return OutlierResult {
                    treated_values: values.to_vec(),
                    outlier_count: 0,
                    lower_bound: Some(mean),
                    upper_bound: Some(mean),
                };
            }

            (mean - threshold * sd, mean + threshold * sd)
        }
        OutlierMethod::Iqr { multiplier } => {
            let mut nums: Vec<f64> = valid_data.iter().map(|(_, x, _)| *x).collect();
            nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let q1 = compute_percentile_sorted(&nums, 0.25);
            let q3 = compute_percentile_sorted(&nums, 0.75);
            let iqr = (q3 - q1).max(0.0);

            (q1 - multiplier * iqr, q3 + multiplier * iqr)
        }
    };

    let mut treated = values.to_vec();
    let mut outlier_count = 0usize;

    for (i, v) in values.iter().enumerate() {
        if let RawValue::Number(num) = v {
            if num.is_finite() {
                if *num < lower_bound {
                    outlier_count += 1;
                    treated[i] = match options.treatment {
                        OutlierTreatment::Winsorize => RawValue::Number(lower_bound),
                        OutlierTreatment::SetSysmis => RawValue::Null,
                    };
                } else if *num > upper_bound {
                    outlier_count += 1;
                    treated[i] = match options.treatment {
                        OutlierTreatment::Winsorize => RawValue::Number(upper_bound),
                        OutlierTreatment::SetSysmis => RawValue::Null,
                    };
                }
            }
        }
    }

    OutlierResult {
        treated_values: treated,
        outlier_count,
        lower_bound: Some(lower_bound),
        upper_bound: Some(upper_bound),
    }
}

fn compute_percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return sorted[0];
    }

    let pos = p * (n - 1) as f64;
    let idx = pos.floor() as usize;
    let frac = pos - pos.floor();

    if idx + 1 < n {
        sorted[idx] + frac * (sorted[idx + 1] - sorted[idx])
    } else {
        sorted[idx.min(n - 1)]
    }
}
