use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MEstimatorsResult {
    pub huber: Option<f64>,
    pub tukey: Option<f64>,
    pub hampel: Option<f64>,
    pub andrews: Option<f64>,
}

fn median_of_sorted(sorted: &[f64]) -> Option<f64> {
    let n = sorted.len();
    if n == 0 {
        return None;
    }
    if n % 2 == 1 {
        Some(sorted[n / 2])
    } else {
        Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
    }
}

/// Computes the 4 canonical IBM SPSS Statistics `EXAMINE` M-estimators:
/// - Huber's M-estimator (c = 1.339)
/// - Tukey's Biweight (c = 4.685)
/// - Hampel's M-estimator (a = 1.7, b = 3.4, c = 8.5)
/// - Andrews' Wave (c = 1.34 * pi)
///
/// Uses unscaled MAD = median(|x_i - median(x)|). Returns `None` for any estimator
/// when the sample is empty, MAD == 0, or the denominator vanishes.
pub fn compute_m_estimators(valid_values: &[f64]) -> MEstimatorsResult {
    if valid_values.is_empty() {
        return MEstimatorsResult {
            huber: None,
            tukey: None,
            hampel: None,
            andrews: None,
        };
    }

    let mut sorted = valid_values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let med = match median_of_sorted(&sorted) {
        Some(m) if m.is_finite() => m,
        _ => {
            return MEstimatorsResult {
                huber: None,
                tukey: None,
                hampel: None,
                andrews: None,
            };
        }
    };

    let mut abs_devs: Vec<f64> = sorted.iter().map(|&x| (x - med).abs()).collect();
    abs_devs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mad = match median_of_sorted(&abs_devs) {
        Some(s) if s.is_finite() && s > 0.0 => s,
        _ => {
            return MEstimatorsResult {
                huber: None,
                tukey: None,
                hampel: None,
                andrews: None,
            };
        }
    };

    let huber = compute_huber(&sorted, med, mad, 1.339);
    let tukey = compute_tukey(&sorted, med, mad, 4.685);
    let hampel = compute_hampel(&sorted, med, mad, 1.7, 3.4, 8.5);
    let andrews = compute_andrews(&sorted, med, mad, 1.34 * std::f64::consts::PI);

    MEstimatorsResult {
        huber,
        tukey,
        hampel,
        andrews,
    }
}

fn compute_huber(values: &[f64], med: f64, mad: f64, c: f64) -> Option<f64> {
    let mut sum_psi = 0.0_f64;
    let mut sum_dpsi = 0.0_f64;

    for &x in values {
        let u = (x - med) / mad;
        if u.abs() <= c {
            sum_psi += u;
            sum_dpsi += 1.0;
        } else if u > c {
            sum_psi += c;
        } else {
            sum_psi -= c;
        }
    }

    if sum_dpsi.abs() < 1e-15 {
        return None;
    }
    let est = med + mad * (sum_psi / sum_dpsi);
    est.is_finite().then_some(est)
}

fn compute_tukey(values: &[f64], med: f64, mad: f64, c: f64) -> Option<f64> {
    let mut sum_psi = 0.0_f64;
    let mut sum_dpsi = 0.0_f64;

    for &x in values {
        let u = (x - med) / mad;
        if u.abs() <= c {
            let r = u / c;
            let r2 = r * r;
            let one_minus_r2 = 1.0 - r2;
            sum_psi += u * one_minus_r2 * one_minus_r2;
            sum_dpsi += one_minus_r2 * (1.0 - 5.0 * r2);
        }
    }

    if sum_dpsi.abs() < 1e-15 {
        return None;
    }
    let est = med + mad * (sum_psi / sum_dpsi);
    est.is_finite().then_some(est)
}

fn compute_hampel(values: &[f64], med: f64, mad: f64, a: f64, b: f64, c: f64) -> Option<f64> {
    let mut sum_psi = 0.0_f64;
    let mut sum_dpsi = 0.0_f64;
    let cb = c - b;
    if cb <= 0.0 {
        return None;
    }

    for &x in values {
        let u = (x - med) / mad;
        let abs_u = u.abs();
        let sign = if u >= 0.0 { 1.0 } else { -1.0 };

        if abs_u <= a {
            sum_psi += u;
            sum_dpsi += 1.0;
        } else if abs_u <= b {
            sum_psi += a * sign;
        } else if abs_u <= c {
            sum_psi += a * sign * ((c - abs_u) / cb);
            sum_dpsi -= a / cb;
        }
    }

    if sum_dpsi.abs() < 1e-15 {
        return None;
    }
    let est = med + mad * (sum_psi / sum_dpsi);
    est.is_finite().then_some(est)
}

fn compute_andrews(values: &[f64], med: f64, mad: f64, c: f64) -> Option<f64> {
    let mut sum_psi = 0.0_f64;
    let mut sum_dpsi = 0.0_f64;

    for &x in values {
        let u = (x - med) / mad;
        if u.abs() <= c {
            let angle = u / c;
            sum_psi += c * angle.sin();
            sum_dpsi += angle.cos();
        }
    }

    if sum_dpsi.abs() < 1e-15 {
        return None;
    }
    let est = med + mad * (sum_psi / sum_dpsi);
    est.is_finite().then_some(est)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_m_estimators_golden_oracle() {
        // Sample with an outlier: [10, 12, 14, 15, 16, 18, 20, 22, 100]
        // Median = 16.0
        // |x - 16| = [6, 4, 2, 1, 0, 2, 4, 6, 84] -> sorted [0, 1, 2, 2, 4, 4, 6, 6, 84] -> MAD = 4.0
        let data = vec![10.0, 12.0, 14.0, 15.0, 16.0, 18.0, 20.0, 22.0, 100.0];
        let res = compute_m_estimators(&data);

        // Verified against Python / SPSS one-step formula:
        // Huber (c=1.339): u = [-1.5, -1.0, -0.5, -0.25, 0, 0.5, 1.0, 1.5, 21.0]
        // |u| <= 1.339: [-1.0, -0.5, -0.25, 0, 0.5, 1.0] (count=6, sum=-0.25)
        // u < -1.339: 1 (-1.339), u > 1.339: 2 (+2.678) -> sum_psi = -0.25 + 1.339 = 1.089
        // Huber = 16.0 + 4.0 * (1.089 / 6.0) = 16.726
        assert!((res.huber.unwrap() - 16.726).abs() < 1e-6);

        // Tukey, Hampel, Andrews reject u=21.0 (x=100.0) completely:
        // Hampel: all remaining 8 points have |u| <= 1.7, sum_psi = -0.25, sum_dpsi = 8.0 -> 16 - 4*(0.25/8) = 15.875
        let tukey = res.tukey.unwrap();
        let hampel = res.hampel.unwrap();
        let andrews = res.andrews.unwrap();
        assert!((tukey - 15.839539).abs() < 1e-5, "tukey={tukey}");
        assert!((hampel - 15.875).abs() < 1e-6, "hampel={hampel}");
        assert!(andrews > 15.8 && andrews < 15.95, "andrews={andrews}");
    }

    #[test]
    fn test_m_estimators_zero_mad() {
        let data = vec![5.0, 5.0, 5.0, 5.0, 10.0];
        let res = compute_m_estimators(&data);
        assert!(res.huber.is_none());
        assert!(res.tukey.is_none());
        assert!(res.hampel.is_none());
        assert!(res.andrews.is_none());
    }

    #[test]
    fn test_m_estimators_empty() {
        let res = compute_m_estimators(&[]);
        assert!(res.huber.is_none());
        assert!(res.tukey.is_none());
        assert!(res.hampel.is_none());
        assert!(res.andrews.is_none());
    }
}
