use statcraft_core::{RawValue, ValueLabels};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Cutpoint {
    pub upper_bound: f64,
    pub inclusive: bool, // true: <= bound, false: < bound
}

impl Cutpoint {
    pub fn new(upper_bound: f64, inclusive: bool) -> Self {
        Self {
            upper_bound,
            inclusive,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinningResult {
    pub binned_values: Vec<RawValue>,
    pub value_labels: ValueLabels,
}

pub fn format_bound(val: f64) -> String {
    if val.fract() == 0.0 && val.abs() < 1e15 {
        format!("{:.0}", val)
    } else {
        let s = format!("{:.4}", val);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub fn generate_bin_labels(cutpoints: &[Cutpoint]) -> Vec<String> {
    if cutpoints.is_empty() {
        return vec!["All".to_string()];
    }

    let mut sorted = cutpoints.to_vec();
    sorted.sort_by(|a, b| {
        a.upper_bound
            .partial_cmp(&b.upper_bound)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let k = sorted.len();
    let mut labels = Vec::with_capacity(k + 1);

    // Bin 1: first interval
    let first = &sorted[0];
    if first.inclusive {
        labels.push(format!("<= {}", format_bound(first.upper_bound)));
    } else {
        labels.push(format!("< {}", format_bound(first.upper_bound)));
    }

    // Intermediate intervals: Bin 2 to Bin k
    for i in 1..k {
        let prev = &sorted[i - 1];
        let curr = &sorted[i];
        labels.push(format!(
            "{} - {}",
            format_bound(prev.upper_bound),
            format_bound(curr.upper_bound)
        ));
    }

    // Last interval: Bin k + 1
    let last = &sorted[k - 1];
    if last.inclusive {
        labels.push(format!("> {}", format_bound(last.upper_bound)));
    } else {
        labels.push(format!(">= {}", format_bound(last.upper_bound)));
    }

    labels
}

pub fn compute_visual_binning(values: &[RawValue], cutpoints: &[Cutpoint]) -> BinningResult {
    compute_visual_binning_with_labels(values, cutpoints, None)
}

pub fn compute_visual_binning_with_labels(
    values: &[RawValue],
    cutpoints: &[Cutpoint],
    custom_labels: Option<&[String]>,
) -> BinningResult {
    let mut sorted_cutpoints = cutpoints.to_vec();
    sorted_cutpoints.sort_by(|a, b| {
        a.upper_bound
            .partial_cmp(&b.upper_bound)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let auto_labels = generate_bin_labels(&sorted_cutpoints);
    let k = sorted_cutpoints.len();

    let mut binned_values = Vec::with_capacity(values.len());

    for v in values {
        match v {
            RawValue::Number(num) if !num.is_nan() => {
                let mut assigned_bin = k + 1;
                for (idx, cp) in sorted_cutpoints.iter().enumerate() {
                    let inside = if cp.inclusive {
                        *num <= cp.upper_bound
                    } else {
                        *num < cp.upper_bound
                    };
                    if inside {
                        assigned_bin = idx + 1;
                        break;
                    }
                }
                binned_values.push(RawValue::Number(assigned_bin as f64));
            }
            _ => {
                binned_values.push(RawValue::Null);
            }
        }
    }

    let mut value_labels = ValueLabels::new();
    let num_bins = k + 1;
    for bin_idx in 1..=num_bins {
        let default_label = auto_labels
            .get(bin_idx - 1)
            .cloned()
            .unwrap_or_else(|| format!("Bin {}", bin_idx));

        let label = if let Some(custom) = custom_labels {
            custom
                .get(bin_idx - 1)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or(default_label)
        } else {
            default_label
        };

        value_labels.insert(&bin_idx.to_string(), &label);
    }

    BinningResult {
        binned_values,
        value_labels,
    }
}

pub fn generate_equal_width_cutpoints(values: &[RawValue], num_bins: usize) -> Vec<Cutpoint> {
    if num_bins <= 1 {
        return Vec::new();
    }

    let nums: Vec<f64> = values
        .iter()
        .filter_map(|v| match v {
            RawValue::Number(n) if !n.is_nan() => Some(*n),
            _ => None,
        })
        .collect();

    if nums.is_empty() {
        return Vec::new();
    }

    let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if max <= min {
        return Vec::new();
    }

    let width = (max - min) / (num_bins as f64);
    let mut cutpoints = Vec::with_capacity(num_bins - 1);
    for i in 1..num_bins {
        let upper_bound = min + (i as f64) * width;
        cutpoints.push(Cutpoint::new(upper_bound, true));
    }

    cutpoints
}

pub fn generate_equal_percentile_cutpoints(values: &[RawValue], num_bins: usize) -> Vec<Cutpoint> {
    if num_bins <= 1 {
        return Vec::new();
    }

    let mut nums: Vec<f64> = values
        .iter()
        .filter_map(|v| match v {
            RawValue::Number(n) if !n.is_nan() => Some(*n),
            _ => None,
        })
        .collect();

    if nums.is_empty() {
        return Vec::new();
    }

    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut cutpoints = Vec::with_capacity(num_bins - 1);
    for i in 1..num_bins {
        let p = (i as f64) / (num_bins as f64);
        if let Some(val) = crate::frequencies::compute_percentile(&nums, p) {
            cutpoints.push(Cutpoint::new(val, true));
        }
    }

    cutpoints.dedup_by(|a, b| (a.upper_bound - b.upper_bound).abs() < 1e-9);
    cutpoints
}

pub fn generate_stddev_cutpoints(values: &[RawValue]) -> Vec<Cutpoint> {
    let nums: Vec<f64> = values
        .iter()
        .filter_map(|v| match v {
            RawValue::Number(n) if !n.is_nan() => Some(*n),
            _ => None,
        })
        .collect();

    if nums.len() < 2 {
        return Vec::new();
    }

    let n = nums.len() as f64;
    let sum: f64 = nums.iter().sum();
    let mean = sum / n;
    let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let stddev = variance.sqrt();

    if stddev <= 1e-9 {
        return Vec::new();
    }

    let factors = [-2.0, -1.0, 0.0, 1.0, 2.0];
    let mut cutpoints = Vec::with_capacity(factors.len());
    for f in factors {
        cutpoints.push(Cutpoint::new(mean + f * stddev, true));
    }

    cutpoints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_visual_binning_basic() {
        let values = vec![
            RawValue::Number(10.0),
            RawValue::Number(25.0),
            RawValue::Number(30.0),
            RawValue::Number(60.0),
            RawValue::Null,
        ];

        let cutpoints = vec![Cutpoint::new(25.0, true), Cutpoint::new(50.0, true)];

        let res = compute_visual_binning(&values, &cutpoints);

        assert_eq!(
            res.binned_values,
            vec![
                RawValue::Number(1.0),
                RawValue::Number(1.0),
                RawValue::Number(2.0),
                RawValue::Number(3.0),
                RawValue::Null,
            ]
        );

        assert_eq!(res.value_labels.get_label("1"), Some("<= 25"));
        assert_eq!(res.value_labels.get_label("2"), Some("25 - 50"));
        assert_eq!(res.value_labels.get_label("3"), Some("> 50"));
    }

    #[test]
    fn test_compute_visual_binning_exclusive() {
        let values = vec![RawValue::Number(25.0), RawValue::Number(50.0)];

        let cutpoints = vec![Cutpoint::new(25.0, false), Cutpoint::new(50.0, false)];

        let res = compute_visual_binning(&values, &cutpoints);

        assert_eq!(
            res.binned_values,
            vec![RawValue::Number(2.0), RawValue::Number(3.0),]
        );

        assert_eq!(res.value_labels.get_label("1"), Some("< 25"));
        assert_eq!(res.value_labels.get_label("2"), Some("25 - 50"));
        assert_eq!(res.value_labels.get_label("3"), Some(">= 50"));
    }

    #[test]
    fn test_generate_equal_width() {
        let values: Vec<RawValue> = (0..=100).map(|i| RawValue::Number(i as f64)).collect();
        let cps = generate_equal_width_cutpoints(&values, 4);
        assert_eq!(cps.len(), 3);
        assert!((cps[0].upper_bound - 25.0).abs() < 1e-6);
        assert!((cps[1].upper_bound - 50.0).abs() < 1e-6);
        assert!((cps[2].upper_bound - 75.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_equal_percentiles() {
        let values: Vec<RawValue> = (1..=100).map(|i| RawValue::Number(i as f64)).collect();
        let cps = generate_equal_percentile_cutpoints(&values, 4);
        assert_eq!(cps.len(), 3);
        assert!((cps[0].upper_bound - 25.75).abs() < 1.0);
        assert!((cps[1].upper_bound - 50.5).abs() < 1.0);
        assert!((cps[2].upper_bound - 75.25).abs() < 1.0);
    }

    #[test]
    fn test_generate_stddev() {
        let values = vec![
            RawValue::Number(10.0),
            RawValue::Number(20.0),
            RawValue::Number(30.0),
            RawValue::Number(40.0),
            RawValue::Number(50.0),
        ];
        let cps = generate_stddev_cutpoints(&values);
        assert_eq!(cps.len(), 5);
        // mean is 30.0
        assert!((cps[2].upper_bound - 30.0).abs() < 1e-6);
    }
}
