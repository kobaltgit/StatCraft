use crate::logistic::math::chisq_p_value;
use crate::logistic::types::{
    ClassificationTable, ClassificationTableRow, HosmerLemeshowGroup, HosmerLemeshowResult,
};

/// Computes the 2x2 Classification Table for binary logistic regression.
pub fn compute_classification_table(y: &[f64], probs: &[f64], cutoff: f64) -> ClassificationTable {
    let mut n00 = 0usize;
    let mut n01 = 0usize;
    let mut n10 = 0usize;
    let mut n11 = 0usize;

    for (actual, &prob) in y.iter().zip(probs.iter()) {
        let is_actual_pos = (actual - 1.0).abs() < 1e-6;
        let is_pred_pos = prob >= cutoff;

        match (is_actual_pos, is_pred_pos) {
            (false, false) => n00 += 1,
            (false, true) => n01 += 1,
            (true, false) => n10 += 1,
            (true, true) => n11 += 1,
        }
    }

    let pct0 = if (n00 + n01) > 0 {
        100.0 * (n00 as f64) / ((n00 + n01) as f64)
    } else {
        0.0
    };

    let pct1 = if (n10 + n11) > 0 {
        100.0 * (n11 as f64) / ((n10 + n11) as f64)
    } else {
        0.0
    };

    let total = y.len();
    let overall = if total > 0 {
        100.0 * ((n00 + n11) as f64) / (total as f64)
    } else {
        0.0
    };

    let rows = vec![
        ClassificationTableRow {
            observed_label: "0".to_string(),
            pred_zero: n00,
            pred_one: n01,
            percent_correct: pct0,
        },
        ClassificationTableRow {
            observed_label: "1".to_string(),
            pred_zero: n10,
            pred_one: n11,
            percent_correct: pct1,
        },
    ];

    ClassificationTable {
        rows,
        overall_percent_correct: overall,
        cutoff,
    }
}

/// Computes Hosmer-Lemeshow goodness-of-fit test using quantile groups.
pub fn compute_hosmer_lemeshow(y: &[f64], probs: &[f64]) -> Option<HosmerLemeshowResult> {
    let n = y.len();
    if n < 10 {
        return None;
    }

    // Sort observations by predicted probability
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| {
        probs[a]
            .partial_cmp(&probs[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let g = 10.min(n / 3).max(3);
    let df = g.saturating_sub(2);
    if df == 0 {
        return None;
    }

    let chunk_size = n / g;
    let remainder = n % g;

    let mut start = 0;
    let mut contingency_table = Vec::with_capacity(g);
    let mut chi_square = 0.0;

    for grp_idx in 0..g {
        let size = chunk_size + if grp_idx < remainder { 1 } else { 0 };
        let end = start + size;
        let grp_indices = &indices[start..end];
        start = end;

        let total = grp_indices.len();
        if total == 0 {
            continue;
        }

        let mut obs_one = 0usize;
        let mut sum_pi = 0.0;
        for &idx in grp_indices {
            if (y[idx] - 1.0).abs() < 1e-6 {
                obs_one += 1;
            }
            sum_pi += probs[idx];
        }

        let obs_zero = total - obs_one;
        let exp_one = sum_pi;
        let exp_zero = (total as f64) - exp_one;

        if exp_one > 1e-12 && exp_zero > 1e-12 {
            let term_one = (obs_one as f64 - exp_one).powi(2) / exp_one;
            let term_zero = (obs_zero as f64 - exp_zero).powi(2) / exp_zero;
            chi_square += term_one + term_zero;
        }

        contingency_table.push(HosmerLemeshowGroup {
            group: grp_idx + 1,
            total,
            observed_one: obs_one,
            expected_one: exp_one,
            observed_zero: obs_zero,
            expected_zero: exp_zero,
        });
    }

    let sig = chisq_p_value(chi_square, df);

    Some(HosmerLemeshowResult {
        chi_square,
        df,
        sig,
        contingency_table,
    })
}
