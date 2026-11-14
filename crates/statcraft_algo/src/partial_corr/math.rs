use crate::anova_oneway::AlgoError;
use crate::correlations_types::SignificanceTest;
use crate::ttest_onesample::student_t_sf_2tailed;

/// Calculates Pearson correlation coefficient for two non-empty numeric slices.
pub fn calc_pearson_slice(x: &[f64], y: &[f64]) -> Result<f64, AlgoError> {
    let n = x.len();
    if n < 2 {
        return Err(AlgoError::InsufficientData("Sample size < 2".to_string()));
    }
    let mean_x = x.iter().sum::<f64>() / (n as f64);
    let mean_y = y.iter().sum::<f64>() / (n as f64);

    let mut ss_x = 0.0;
    let mut ss_y = 0.0;
    let mut ss_xy = 0.0;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        ss_x += dx * dx;
        ss_y += dy * dy;
        ss_xy += dx * dy;
    }

    let denom = (ss_x * ss_y).sqrt();
    if denom <= 1e-15 || !denom.is_finite() {
        return Err(AlgoError::InvalidInput(
            "Constant variable with zero variance detected".to_string(),
        ));
    }

    Ok((ss_xy / denom).clamp(-1.0, 1.0))
}

/// Multiplies matrix A (n x m) and matrix B (m x p) returning (n x p).
#[allow(clippy::needless_range_loop)]
pub fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let m = if n > 0 { a[0].len() } else { 0 };
    let p = if !b.is_empty() { b[0].len() } else { 0 };

    let mut res = vec![vec![0.0; p]; n];
    for i in 0..n {
        for j in 0..p {
            let mut sum = 0.0;
            for k in 0..m {
                sum += a[i][k] * b[k][j];
            }
            res[i][j] = sum;
        }
    }
    res
}

/// Computes p-value and degrees of freedom for a partial correlation r.
pub fn calculate_sig(r: f64, df: usize, sig_test: SignificanceTest) -> (Option<f64>, usize) {
    if df == 0 {
        return (None, 0);
    }
    let r2 = r * r;
    if (1.0 - r2).abs() <= 1e-15 {
        return (Some(0.0), df);
    }
    let t = r * ((df as f64) / (1.0 - r2)).sqrt();
    let p2 = student_t_sf_2tailed(t.abs(), df as f64);
    let p = match sig_test {
        SignificanceTest::TwoTailed => p2,
        SignificanceTest::OneTailed => p2 / 2.0,
    };
    (Some(p.clamp(0.0, 1.0)), df)
}
