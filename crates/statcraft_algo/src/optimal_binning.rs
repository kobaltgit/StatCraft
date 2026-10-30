use statcraft_core::{RawValue, ValueLabels};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Clone, PartialEq)]
pub struct OptimalBinningResult {
    pub cutpoints: Vec<f64>,
    pub binned_values: Vec<RawValue>,
    pub value_labels: ValueLabels,
}

#[derive(Clone, Copy)]
struct Point<'a> {
    x: f64,
    y: &'a str,
}

#[derive(Clone)]
struct SplitCandidate {
    gain: f64,
    cutpoint: f64,
    start: usize,
    split_pos: usize, // relative index inside slice
    end: usize,
}

impl PartialEq for SplitCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.gain == other.gain && self.cutpoint == other.cutpoint
    }
}

impl Eq for SplitCandidate {}

impl Ord for SplitCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.gain
            .partial_cmp(&other.gain)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for SplitCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn calculate_entropy(counts: &HashMap<&str, usize>, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let n = total as f64;
    let mut ent = 0.0;
    for &cnt in counts.values() {
        if cnt > 0 {
            let p = cnt as f64 / n;
            ent -= p * p.log2();
        }
    }
    ent
}

fn find_best_split(slice: &[Point], global_start: usize) -> Option<SplitCandidate> {
    let n = slice.len();
    if n < 2 {
        return None;
    }

    let mut right_counts = HashMap::new();
    for p in slice {
        *right_counts.entry(p.y).or_insert(0) += 1;
    }

    let k = right_counts.len();
    if k <= 1 {
        return None;
    }

    let total_ent = calculate_entropy(&right_counts, n);
    let mut left_counts = HashMap::new();
    let mut best_candidate: Option<SplitCandidate> = None;
    let mut max_gain = -1.0;

    let total_n = n as f64;

    for i in 0..(n - 1) {
        let curr = slice[i];
        *left_counts.entry(curr.y).or_insert(0) += 1;
        if let Some(rc) = right_counts.get_mut(curr.y) {
            *rc -= 1;
            if *rc == 0 {
                right_counts.remove(curr.y);
            }
        }

        let next = slice[i + 1];
        if curr.x < next.x && curr.y != next.y {
            let n1 = (i + 1) as f64;
            let n2 = (n - i - 1) as f64;
            let ent1 = calculate_entropy(&left_counts, i + 1);
            let ent2 = calculate_entropy(&right_counts, n - i - 1);
            let cond_ent = (n1 / total_n) * ent1 + (n2 / total_n) * ent2;
            let gain = total_ent - cond_ent;

            if gain > max_gain {
                let k1 = left_counts.len() as f64;
                let k2 = right_counts.len() as f64;
                let k_f = k as f64;

                let delta = (3.0_f64.powi(k as i32) - 2.0).log2()
                    - (k_f * total_ent - k1 * ent1 - k2 * ent2);
                let threshold = ((total_n - 1.0).log2() + delta) / total_n;

                if gain > threshold {
                    max_gain = gain;
                    best_candidate = Some(SplitCandidate {
                        gain,
                        cutpoint: (curr.x + next.x) / 2.0,
                        start: global_start,
                        split_pos: i + 1,
                        end: global_start + n,
                    });
                }
            }
        }
    }

    best_candidate
}

pub fn compute_optimal_binning(
    feature: &[RawValue],
    target: &[RawValue],
    max_bins: usize,
) -> Result<OptimalBinningResult, String> {
    if feature.len() != target.len() {
        return Err("Feature and target lengths must match".to_string());
    }

    let max_bins = max_bins.max(2);
    let max_cuts = max_bins - 1;

    let mut target_strings = Vec::with_capacity(target.len());
    for t in target {
        match t {
            RawValue::Number(n) if !n.is_nan() => {
                if n.fract() == 0.0 {
                    target_strings.push(Some(format!("{:.0}", n)));
                } else {
                    target_strings.push(Some(format!("{:.4}", n)));
                }
            }
            RawValue::Text(s) => target_strings.push(Some(s.clone())),
            _ => target_strings.push(None),
        }
    }

    let mut points: Vec<Point> = Vec::new();
    for (i, f) in feature.iter().enumerate() {
        if let RawValue::Number(num) = f {
            if !num.is_nan() {
                if let Some(Some(ref y_str)) = target_strings.get(i) {
                    points.push(Point {
                        x: *num,
                        y: y_str.as_str(),
                    });
                }
            }
        }
    }

    points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(Ordering::Equal));

    let mut cutpoints = Vec::new();

    if points.len() >= 2 && max_cuts > 0 {
        let mut heap = BinaryHeap::new();
        if let Some(cand) = find_best_split(&points, 0) {
            heap.push(cand);
        }

        while let Some(top) = heap.pop() {
            cutpoints.push(top.cutpoint);
            if cutpoints.len() >= max_cuts {
                break;
            }

            let left_slice = &points[top.start..(top.start + top.split_pos)];
            if let Some(cand) = find_best_split(left_slice, top.start) {
                heap.push(cand);
            }

            let right_start = top.start + top.split_pos;
            let right_slice = &points[right_start..top.end];
            if let Some(cand) = find_best_split(right_slice, right_start) {
                heap.push(cand);
            }
        }
    }

    cutpoints.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    cutpoints.dedup();

    let k = cutpoints.len();
    let num_bins = k + 1;
    let mut value_labels = ValueLabels::new();

    if k == 0 {
        value_labels.insert("1", "All");
    } else {
        value_labels.insert(
            "1",
            &format!("<= {}", crate::binning::format_bound(cutpoints[0])),
        );
        for i in 1..k {
            value_labels.insert(
                &(i + 1).to_string(),
                &format!(
                    "{} - {}",
                    crate::binning::format_bound(cutpoints[i - 1]),
                    crate::binning::format_bound(cutpoints[i])
                ),
            );
        }
        value_labels.insert(
            &num_bins.to_string(),
            &format!("> {}", crate::binning::format_bound(cutpoints[k - 1])),
        );
    }

    let mut binned_values = Vec::with_capacity(feature.len());
    for (i, v) in feature.iter().enumerate() {
        let has_target = target_strings.get(i).and_then(|opt| opt.as_ref()).is_some();
        match v {
            RawValue::Number(num) if !num.is_nan() && has_target => {
                let mut assigned = num_bins;
                for (idx, &cp) in cutpoints.iter().enumerate() {
                    if *num <= cp {
                        assigned = idx + 1;
                        break;
                    }
                }
                binned_values.push(RawValue::Number(assigned as f64));
            }
            _ => {
                binned_values.push(RawValue::Null);
            }
        }
    }

    Ok(OptimalBinningResult {
        cutpoints,
        binned_values,
        value_labels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_two_class_split() {
        let feature = vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(3.0),
            RawValue::Number(4.0),
            RawValue::Number(10.0),
            RawValue::Number(11.0),
            RawValue::Number(12.0),
            RawValue::Number(13.0),
        ];
        let target = vec![
            RawValue::Number(0.0),
            RawValue::Number(0.0),
            RawValue::Number(0.0),
            RawValue::Number(0.0),
            RawValue::Number(1.0),
            RawValue::Number(1.0),
            RawValue::Number(1.0),
            RawValue::Number(1.0),
        ];

        let res = compute_optimal_binning(&feature, &target, 5).unwrap();
        assert_eq!(res.cutpoints.len(), 1);
        assert!((res.cutpoints[0] - 7.0).abs() < 1e-6);
        assert_eq!(res.binned_values[0], RawValue::Number(1.0));
        assert_eq!(res.binned_values[3], RawValue::Number(1.0));
        assert_eq!(res.binned_values[4], RawValue::Number(2.0));
        assert_eq!(res.binned_values[7], RawValue::Number(2.0));
        assert_eq!(res.value_labels.get_label("1"), Some("<= 7"));
        assert_eq!(res.value_labels.get_label("2"), Some("> 7"));
    }

    #[test]
    fn test_constant_feature() {
        let feature = vec![
            RawValue::Number(5.0),
            RawValue::Number(5.0),
            RawValue::Number(5.0),
            RawValue::Number(5.0),
        ];
        let target = vec![
            RawValue::Number(0.0),
            RawValue::Number(1.0),
            RawValue::Number(0.0),
            RawValue::Number(1.0),
        ];

        let res = compute_optimal_binning(&feature, &target, 5).unwrap();
        assert!(res.cutpoints.is_empty());
        assert_eq!(res.binned_values[0], RawValue::Number(1.0));
        assert_eq!(res.value_labels.get_label("1"), Some("All"));
    }

    #[test]
    fn test_single_class_target() {
        let feature = vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(3.0),
            RawValue::Number(4.0),
        ];
        let target = vec![
            RawValue::Text("A".into()),
            RawValue::Text("A".into()),
            RawValue::Text("A".into()),
            RawValue::Text("A".into()),
        ];

        let res = compute_optimal_binning(&feature, &target, 5).unwrap();
        assert!(res.cutpoints.is_empty());
        assert_eq!(res.binned_values[0], RawValue::Number(1.0));
    }

    #[test]
    fn test_null_handling() {
        let feature = vec![
            RawValue::Number(1.0),
            RawValue::Null,
            RawValue::Number(10.0),
            RawValue::Number(11.0),
        ];
        let target = vec![
            RawValue::Number(0.0),
            RawValue::Number(0.0),
            RawValue::Null,
            RawValue::Number(1.0),
        ];

        let res = compute_optimal_binning(&feature, &target, 5).unwrap();
        assert_eq!(res.binned_values[1], RawValue::Null);
        assert_eq!(res.binned_values[2], RawValue::Null);
    }
}
