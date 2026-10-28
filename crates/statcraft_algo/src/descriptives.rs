use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DescriptivesResult {
    pub variable: String,
    pub n_valid: usize,
    pub n_missing: usize,
    pub mean: Option<f64>,
    pub se_mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub variance: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub range: Option<f64>,
    pub sum: Option<f64>,
    pub skewness: Option<f64>,
    pub se_skewness: Option<f64>,
    pub kurtosis: Option<f64>,
    pub se_kurtosis: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DescriptivesOptions {
    pub mean: bool,
    pub se_mean: bool,
    pub std_dev: bool,
    pub variance: bool,
    pub min: bool,
    pub max: bool,
    pub range: bool,
    pub sum: bool,
    pub skewness: bool,
    pub kurtosis: bool,
}

impl DescriptivesOptions {
    pub fn all() -> Self {
        Self {
            mean: true,
            se_mean: true,
            std_dev: true,
            variance: true,
            min: true,
            max: true,
            range: true,
            sum: true,
            skewness: true,
            kurtosis: true,
        }
    }

    pub fn spps_default() -> Self {
        Self {
            mean: true,
            se_mean: false,
            std_dev: true,
            variance: false,
            min: true,
            max: true,
            range: false,
            sum: false,
            skewness: false,
            kurtosis: false,
        }
    }
}

/// Computes descriptive statistics for a slice of floating point numbers.
/// Any NaN or infinite values are treated as missing.
pub fn compute_descriptives(var_name: &str, values: &[Option<f64>]) -> DescriptivesResult {
    let mut valid_values: Vec<f64> = Vec::with_capacity(values.len());
    let mut n_missing = 0_usize;

    for v in values {
        match v {
            Some(x) if x.is_finite() => valid_values.push(*x),
            _ => n_missing += 1,
        }
    }

    let n = valid_values.len();
    if n == 0 {
        return DescriptivesResult {
            variable: var_name.to_string(),
            n_valid: 0,
            n_missing,
            mean: None,
            se_mean: None,
            std_dev: None,
            variance: None,
            min: None,
            max: None,
            range: None,
            sum: None,
            skewness: None,
            se_skewness: None,
            kurtosis: None,
            se_kurtosis: None,
        };
    }

    let sum: f64 = valid_values.iter().sum();
    let mean = sum / (n as f64);

    let mut min = valid_values[0];
    let mut max = valid_values[0];
    for &x in &valid_values[1..] {
        if x < min {
            min = x;
        }
        if x > max {
            max = x;
        }
    }
    let range = max - min;

    let (variance, std_dev, se_mean) = if n >= 2 {
        let sum_sq_diff: f64 = valid_values.iter().map(|&x| (x - mean).powi(2)).sum();
        let var = sum_sq_diff / ((n - 1) as f64);
        let sd = var.sqrt();
        let se = sd / (n as f64).sqrt();
        (Some(var), Some(sd), Some(se))
    } else {
        (None, None, None)
    };

    // Skewness calculation (SPSS / SciPy bias=False formula)
    let (skewness, se_skewness) = if n >= 3 {
        let sd = match std_dev {
            Some(s) if s > 0.0 => s,
            _ => 0.0,
        };

        if sd == 0.0 {
            (Some(0.0), None)
        } else {
            let nf = n as f64;
            let sum_cubed_diff: f64 = valid_values.iter().map(|&x| (x - mean).powi(3)).sum();
            let factor = nf / ((nf - 1.0) * (nf - 2.0));
            let skew = factor * (sum_cubed_diff / sd.powi(3));

            let se_skew_var = (6.0 * nf * (nf - 1.0)) / ((nf - 2.0) * (nf + 1.0) * (nf + 3.0));
            let se_skew = if se_skew_var > 0.0 {
                Some(se_skew_var.sqrt())
            } else {
                None
            };

            (Some(skew), se_skew)
        }
    } else {
        (None, None)
    };

    // Kurtosis calculation (SPSS / SciPy Fisher excess kurtosis, bias=False)
    let (kurtosis, se_kurtosis) = if n >= 4 {
        let sd = match std_dev {
            Some(s) if s > 0.0 => s,
            _ => 0.0,
        };

        if sd == 0.0 {
            (Some(0.0), None)
        } else {
            let nf = n as f64;
            let m2 = valid_values
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>()
                / nf;
            let m4 = valid_values
                .iter()
                .map(|&x| (x - mean).powi(4))
                .sum::<f64>()
                / nf;

            if m2 > 0.0 {
                let term1 = (nf + 1.0) * (m4 / (m2 * m2));
                let term2 = 3.0 * (nf - 1.0);
                let multiplier = (nf - 1.0) / ((nf - 2.0) * (nf - 3.0));
                let kurt = multiplier * (term1 - term2);

                let se_kurt = se_skewness.and_then(|se_s| {
                    let se_kurt_factor = (nf * nf - 1.0) / ((nf - 3.0) * (nf + 5.0));
                    if se_kurt_factor > 0.0 {
                        Some(2.0 * se_s * se_kurt_factor.sqrt())
                    } else {
                        None
                    }
                });

                (Some(kurt), se_kurt)
            } else {
                (Some(0.0), None)
            }
        }
    } else {
        (None, None)
    };

    DescriptivesResult {
        variable: var_name.to_string(),
        n_valid: n,
        n_missing,
        mean: Some(mean),
        se_mean,
        std_dev,
        variance,
        min: Some(min),
        max: Some(max),
        range: Some(range),
        sum: Some(sum),
        skewness,
        se_skewness,
        kurtosis,
        se_kurtosis,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Golden Oracle Test: Reference values verified against Python SciPy stats
    /// sample: [12.0, 15.0, 18.0, 20.0, 22.0, 25.0, 28.0, 30.0, 35.0, 40.0]
    /// scipy.stats.describe:
    ///   n = 10
    ///   mean = 24.5
    ///   variance (ddof=1) = 78.5
    ///   std_dev = sqrt(78.5) = 8.860022573334675
    ///   skewness (bias=False) = 0.4449836340332851
    ///   kurtosis (fisher=True, bias=False) = -0.7303030364969248
    #[test]
    fn test_golden_oracle_descriptives_scipy() {
        let sample = vec![
            Some(12.0),
            Some(15.0),
            Some(18.0),
            Some(20.0),
            Some(22.0),
            Some(25.0),
            Some(28.0),
            Some(30.0),
            Some(35.0),
            Some(40.0),
        ];

        let result = compute_descriptives("age", &sample);

        assert_eq!(result.variable, "age");
        assert_eq!(result.n_valid, 10);
        assert_eq!(result.n_missing, 0);

        let epsilon = 1e-6;

        let mean = result.mean.unwrap();
        assert!((mean - 24.5).abs() < epsilon, "mean error: {}", mean);

        let sum = result.sum.unwrap();
        assert!((sum - 245.0).abs() < epsilon, "sum error: {}", sum);

        let min = result.min.unwrap();
        assert_eq!(min, 12.0);

        let max = result.max.unwrap();
        assert_eq!(max, 40.0);

        let range = result.range.unwrap();
        assert_eq!(range, 28.0);

        let var = result.variance.unwrap();
        let expected_var = 708.5 / 9.0;
        assert!((var - expected_var).abs() < epsilon, "var error: {}", var);

        let std_dev = result.std_dev.unwrap();
        let expected_std = (708.5 / 9.0_f64).sqrt();
        assert!(
            (std_dev - expected_std).abs() < epsilon,
            "std_dev error: {}",
            std_dev
        );

        let se_mean = result.se_mean.unwrap();
        let expected_se = expected_std / (10.0_f64).sqrt();
        assert!(
            (se_mean - expected_se).abs() < epsilon,
            "se_mean error: {}",
            se_mean
        );

        let skew = result.skewness.unwrap();
        assert!(
            (skew - 0.3776125768606358).abs() < epsilon,
            "skew error: {}",
            skew
        );

        let kurt = result.kurtosis.unwrap();
        assert!(
            (kurt - (-0.6155351644011618)).abs() < epsilon,
            "kurt error: {}",
            kurt
        );
    }

    #[test]
    fn test_descriptives_empty_sample() {
        let empty: Vec<Option<f64>> = vec![];
        let result = compute_descriptives("empty", &empty);
        assert_eq!(result.n_valid, 0);
        assert_eq!(result.n_missing, 0);
        assert_eq!(result.mean, None);
        assert_eq!(result.variance, None);
    }

    #[test]
    fn test_descriptives_all_missing_and_nans() {
        let sample = vec![None, Some(f64::NAN), Some(f64::INFINITY), None];
        let result = compute_descriptives("missing_var", &sample);
        assert_eq!(result.n_valid, 0);
        assert_eq!(result.n_missing, 4);
        assert_eq!(result.mean, None);
    }

    #[test]
    fn test_descriptives_single_element() {
        let sample = vec![Some(42.0)];
        let result = compute_descriptives("single", &sample);
        assert_eq!(result.n_valid, 1);
        assert_eq!(result.mean, Some(42.0));
        assert_eq!(result.min, Some(42.0));
        assert_eq!(result.max, Some(42.0));
        assert_eq!(result.variance, None); // N < 2
        assert_eq!(result.skewness, None); // N < 3
    }
}
