use super::types::{AdpPartitionOptions, PartitionCounts};
use statcraft_core::{RawValue, ValueLabels};
use std::collections::BTreeMap;

struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        let initial = if seed == 0 { 2026 } else { seed };
        Self(initial)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

pub struct PartitionResult {
    pub values: Vec<RawValue>,
    pub counts: PartitionCounts,
    pub labels: ValueLabels,
}

/// Generates a partition assignment variable (_Partition_) dividing cases into
/// Training (1), Testing (2), and Validation (3) subsets.
pub fn compute_partition(n_cases: usize, options: &AdpPartitionOptions) -> PartitionResult {
    let mut total_pct = options.train_pct + options.test_pct + options.validate_pct;
    if total_pct <= 0.0 {
        total_pct = 100.0;
    }

    let train_thresh = options.train_pct / total_pct;
    let test_thresh = train_thresh + (options.test_pct / total_pct);

    let seed = options.seed.unwrap_or(2026);
    let mut rng = SimpleRng::new(seed);

    let mut values = Vec::with_capacity(n_cases);
    let mut counts = PartitionCounts {
        train_count: 0,
        test_count: 0,
        validate_count: 0,
        total_count: n_cases,
    };

    for _ in 0..n_cases {
        let u = rng.next_f64();
        if u < train_thresh {
            values.push(RawValue::Number(1.0));
            counts.train_count += 1;
        } else if u < test_thresh {
            values.push(RawValue::Number(2.0));
            counts.test_count += 1;
        } else {
            values.push(RawValue::Number(3.0));
            counts.validate_count += 1;
        }
    }

    let mut labels = BTreeMap::new();
    labels.insert(
        "1".to_string(),
        format!("Training ({:.0}%)", (options.train_pct / total_pct) * 100.0),
    );
    labels.insert(
        "2".to_string(),
        format!("Testing ({:.0}%)", (options.test_pct / total_pct) * 100.0),
    );
    if options.validate_pct > 0.0 {
        labels.insert(
            "3".to_string(),
            format!(
                "Validation ({:.0}%)",
                (options.validate_pct / total_pct) * 100.0
            ),
        );
    }

    PartitionResult {
        values,
        counts,
        labels: ValueLabels { mapping: labels },
    }
}
