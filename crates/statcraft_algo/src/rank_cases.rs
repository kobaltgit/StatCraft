use serde::{Deserialize, Serialize};
use statcraft_core::RawValue;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TiesHandling {
    #[default]
    Mean, // Average rank of tied observations (default in SPSS)
    Low,  // Smallest rank of tied observations
    High, // Largest rank of tied observations
    Seq,  // Sequential rank without ties
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RankDirection {
    #[default]
    Ascending, // Smallest value gets rank 1
    Descending, // Largest value gets rank 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RankType {
    #[default]
    SimpleRank, // Standard ranks (1..N)
    FractionalRank,      // r / N
    FractionalPercent,   // (r / N) * 100
    Ntiles(usize),       // Quantiles (e.g. 4 for quartiles)
    Percentiles,         // Percentile ranks
    NormalBlom,          // (r - 3/8) / (w + 1/4) -> Phi^-1
    NormalTukey,         // (r - 1/3) / (w + 1/3) -> Phi^-1
    NormalRankit,        // (r - 1/2) / w -> Phi^-1
    NormalVanderWaerden, // r / (w + 1) -> Phi^-1
}

/// Computes the inverse of the standard normal cumulative distribution function (quantile function / probit)
/// using Peter J. Acklam's rational approximation.
#[allow(clippy::excessive_precision)]
pub fn inv_normal_cdf(p: f64) -> f64 {
    // Acklam's approximation coefficients
    const A: [f64; 6] = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];

    let p_clamped = p.clamp(1e-15, 1.0 - 1e-15);

    if p_clamped < 0.02425 {
        // Lower tail
        let q = (-2.0 * p_clamped.ln()).sqrt();
        let num = ((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5];
        let den = (((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0;
        num / den
    } else if p_clamped <= 0.97575 {
        // Central region
        let q = p_clamped - 0.5;
        let r = q * q;
        let num = ((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5];
        let den = ((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0;
        q * num / den
    } else {
        // Upper tail
        let q = (-2.0 * (1.0 - p_clamped).ln()).sqrt();
        let num = ((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5];
        let den = (((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0;
        -(num / den)
    }
}

fn extract_f64(val: &RawValue) -> Option<f64> {
    match val {
        RawValue::Number(n) => {
            if n.is_nan() {
                None
            } else {
                Some(*n)
            }
        }
        RawValue::Text(s) => s.trim().parse::<f64>().ok().filter(|n| !n.is_nan()),
        RawValue::Null => None,
    }
}

struct ValidItem {
    orig_idx: usize,
    val: f64,
}

pub fn compute_ranks(
    values: &[RawValue],
    direction: RankDirection,
    ties: TiesHandling,
    rank_type: RankType,
) -> Vec<RawValue> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }

    let mut valid_items: Vec<ValidItem> = Vec::with_capacity(n);
    for (idx, v) in values.iter().enumerate() {
        if let Some(f) = extract_f64(v) {
            valid_items.push(ValidItem {
                orig_idx: idx,
                val: f,
            });
        }
    }

    let w = valid_items.len() as f64;
    if valid_items.is_empty() {
        return vec![RawValue::Null; n];
    }

    // Sort valid items according to RankDirection.
    // Preserve stable original order when values are equal.
    valid_items.sort_by(|a, b| {
        let cmp = match direction {
            RankDirection::Ascending => a.val.partial_cmp(&b.val).unwrap_or(Ordering::Equal),
            RankDirection::Descending => b.val.partial_cmp(&a.val).unwrap_or(Ordering::Equal),
        };
        if cmp == Ordering::Equal {
            a.orig_idx.cmp(&b.orig_idx)
        } else {
            cmp
        }
    });

    let mut result = vec![RawValue::Null; n];
    let mut i = 0;
    let valid_count = valid_items.len();

    while i < valid_count {
        let mut j = i + 1;
        while j < valid_count && (valid_items[j].val - valid_items[i].val).abs() < 1e-12 {
            j += 1;
        }

        // The tied group spans 0-based indices [i..j), corresponding to 1-based ranks (i+1)..=j
        let tie_start_rank = (i + 1) as f64;
        let tie_end_rank = j as f64;
        let mean_rank = (tie_start_rank + tie_end_rank) / 2.0;

        for k in i..j {
            let base_rank = match ties {
                TiesHandling::Mean => mean_rank,
                TiesHandling::Low => tie_start_rank,
                TiesHandling::High => tie_end_rank,
                TiesHandling::Seq => (k + 1) as f64,
            };

            let final_val = match rank_type {
                RankType::SimpleRank => base_rank,
                RankType::FractionalRank => base_rank / w,
                RankType::FractionalPercent | RankType::Percentiles => (base_rank / w) * 100.0,
                RankType::Ntiles(groups) => {
                    let g = (groups.max(1)) as f64;
                    ((base_rank * g) / w).ceil().clamp(1.0, g)
                }
                RankType::NormalBlom => {
                    let p = (base_rank - 0.375) / (w + 0.25);
                    inv_normal_cdf(p)
                }
                RankType::NormalTukey => {
                    let p = (base_rank - (1.0 / 3.0)) / (w + (1.0 / 3.0));
                    inv_normal_cdf(p)
                }
                RankType::NormalRankit => {
                    let p = (base_rank - 0.5) / w;
                    inv_normal_cdf(p)
                }
                RankType::NormalVanderWaerden => {
                    let p = base_rank / (w + 1.0);
                    inv_normal_cdf(p)
                }
            };

            if let Some(item) = valid_items.get(k) {
                if let Some(slot) = result.get_mut(item.orig_idx) {
                    *slot = RawValue::Number(final_val);
                }
            }
        }

        i = j;
    }

    result
}
