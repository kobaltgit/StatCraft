use super::normality_sw::{normal_cdf, NormalityTestRow};
use crate::AlgoError;

/// Computes the Kolmogorov-Smirnov test statistic with Lilliefors significance correction
/// for testing composite normality with unknown mean and variance.
/// Implements the Dallal & Wilkinson (1986) analytic approximation per IBM SPSS 24 Algorithms Guide.
pub fn compute_lilliefors_ks(values: &[f64]) -> Result<NormalityTestRow, AlgoError> {
    let n = values.len();
    if n < 3 {
        return Err(AlgoError::InsufficientData(format!(
            "Kolmogorov-Smirnov Lilliefors test requires at least 3 cases, got {n}"
        )));
    }

    let n_f = n as f64;
    let mean = values.iter().sum::<f64>() / n_f;
    let sum_sq_diff: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum();
    let variance = sum_sq_diff / (n_f - 1.0);

    if variance <= 1e-15 || !variance.is_finite() {
        return Err(AlgoError::InvalidInput(
            "Kolmogorov-Smirnov test cannot be computed for constant data (variance is zero)"
                .to_string(),
        ));
    }

    let std_dev = variance.sqrt();

    // Standardize values
    let mut z_scores: Vec<f64> = values.iter().map(|&x| (x - mean) / std_dev).collect();
    z_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Compute maximum absolute deviation D
    let mut max_d = 0.0_f64;
    for (idx, &z) in z_scores.iter().enumerate() {
        let f_z = normal_cdf(z);
        let s_curr = (idx + 1) as f64 / n_f;
        let s_prev = idx as f64 / n_f;

        let d1 = (s_curr - f_z).abs();
        let d2 = (s_prev - f_z).abs();

        if d1 > max_d {
            max_d = d1;
        }
        if d2 > max_d {
            max_d = d2;
        }
    }

    // Dallal & Wilkinson (1986) approximation formula per IBM SPSS 24 Algorithms Guide
    let term = -7.01256 * (max_d * max_d) * (n_f + 2.78019)
        + 2.99587 * max_d * (n_f + 2.78019).sqrt()
        - 0.122119
        + 0.974598 / n_f.sqrt()
        + 1.67997 / n_f;
    // Standard SPSS bounds: .200* is the upper bound for display, .001 is lower bound
    let p = term.exp().clamp(0.001, 0.20);

    Ok(NormalityTestRow {
        statistic: max_d,
        df: n,
        sig: p,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lilliefors_ks_normal_data() {
        let data = vec![
            10.2, 10.5, 9.8, 10.1, 10.0, 9.9, 10.3, 10.0, 9.7, 10.4, 10.1, 9.9,
        ];
        let res = compute_lilliefors_ks(&data).unwrap();
        assert_eq!(res.df, 12);
        assert!(res.statistic < 0.3);
        assert!(res.sig > 0.10);
    }

    #[test]
    fn test_lilliefors_ks_skewed_data() {
        let data = vec![1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 20.0, 100.0];
        let res = compute_lilliefors_ks(&data).unwrap();
        assert_eq!(res.df, 10);
        assert!(res.statistic > 0.25);
        assert!(res.sig <= 0.05);
    }

    #[test]
    fn test_lilliefors_ks_constant() {
        let data = vec![2.0, 2.0, 2.0, 2.0];
        assert!(compute_lilliefors_ks(&data).is_err());
    }

    #[test]
    fn test_lilliefors_ks_insufficient() {
        let data = vec![1.0, 2.0];
        assert!(compute_lilliefors_ks(&data).is_err());
    }
}
