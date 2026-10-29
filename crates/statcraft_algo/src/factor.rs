use crate::anova_oneway::AlgoError;
use crate::crosstabs::chi_square_sf;
use crate::factor_types::{
    CommunalityRow, ComponentMatrixRow, FactorAnalysisResult, FactorExtractionCriteria,
    FactorOptions, KmoBartlettResult, VarianceExplainedRow,
};
use crate::regression::invert_matrix;

/// Solves symmetric eigenvalue problem using cyclic Jacobi rotations.
/// Returns (eigenvalues in descending order, eigenvectors as column vectors V[row][col]).
#[allow(clippy::needless_range_loop)]
pub fn jacobi_eigen_symmetric(matrix: &[Vec<f64>]) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = matrix.len();
    if n == 0 {
        return (Vec::new(), Vec::new());
    }

    let mut a = matrix.to_vec();
    let mut v = vec![vec![0.0; n]; n];
    for i in 0..n {
        v[i][i] = 1.0;
    }

    const MAX_SWEEPS: usize = 100;
    for _ in 0..MAX_SWEEPS {
        // Calculate sum of off-diagonal elements
        let mut off_diag_sum = 0.0;
        for i in 0..n {
            for j in (i + 1)..n {
                off_diag_sum += a[i][j].abs();
            }
        }

        if off_diag_sum < 1e-15 {
            break;
        }

        for p in 0..n {
            for q in (p + 1)..n {
                let apq = a[p][q];
                if apq.abs() < 1e-16 {
                    continue;
                }

                let app = a[p][p];
                let aqq = a[q][q];
                let h = aqq - app;

                let t = if h.abs() < 1e-16 {
                    if apq >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    }
                } else {
                    let theta = 0.5 * h / apq;
                    let sign = if theta >= 0.0 { 1.0 } else { -1.0 };
                    sign / (theta.abs() + (1.0 + theta * theta).sqrt())
                };

                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = t * c;
                let tau = s / (1.0 + c);

                a[p][p] -= t * apq;
                a[q][q] += t * apq;
                a[p][q] = 0.0;
                a[q][p] = 0.0;

                for r in 0..n {
                    if r != p && r != q {
                        let arp = a[r][p];
                        let arq = a[r][q];
                        let new_arp = arp - s * (arq + tau * arp);
                        let new_arq = arq + s * (arp - tau * arq);
                        a[r][p] = new_arp;
                        a[p][r] = new_arp;
                        a[r][q] = new_arq;
                        a[q][r] = new_arq;
                    }
                }

                for r in 0..n {
                    let vrp = v[r][p];
                    let vrq = v[r][q];
                    v[r][p] = vrp - s * (vrq + tau * vrp);
                    v[r][q] = vrq + s * (vrp - tau * vrq);
                }
            }
        }
    }

    let mut eigen: Vec<(f64, Vec<f64>)> = (0..n)
        .map(|col| {
            let val = a[col][col].max(0.0);
            let vec: Vec<f64> = (0..n).map(|row| v[row][col]).collect();
            (val, vec)
        })
        .collect();

    // Sort descending by eigenvalue
    eigen.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut eigenvalues = Vec::with_capacity(n);
    let mut eigenvectors = vec![vec![0.0; n]; n];

    for (col_idx, (val, col_vec)) in eigen.into_iter().enumerate() {
        eigenvalues.push(val);
        // SPSS sign convention: make column sum positive (or dominant element positive)
        let col_sum: f64 = col_vec.iter().sum();
        let sign = if col_sum < 0.0 {
            -1.0
        } else if col_sum == 0.0 {
            let max_val = col_vec.iter().max_by(|a, b| {
                a.abs()
                    .partial_cmp(&b.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if max_val.copied().unwrap_or(0.0) < 0.0 {
                -1.0
            } else {
                1.0
            }
        } else {
            1.0
        };

        for row_idx in 0..n {
            eigenvectors[row_idx][col_idx] = col_vec[row_idx] * sign;
        }
    }

    (eigenvalues, eigenvectors)
}

/// Computes Principal Component Analysis (PCA) and Factor Analysis.
#[allow(clippy::needless_range_loop)]
pub fn compute_factor_analysis(
    items: &[(&str, &[Option<f64>])],
    weights: &[f64],
    options: &FactorOptions,
) -> Result<FactorAnalysisResult, AlgoError> {
    let p = items.len();
    if p < 2 {
        return Err(AlgoError::InvalidInput(
            "Factor analysis requires at least 2 variables".to_string(),
        ));
    }

    let n_total = items[0].1.len();
    for &(name, series) in items {
        if series.len() != n_total {
            return Err(AlgoError::InvalidInput(format!(
                "Series '{}' length mismatch with first variable",
                name
            )));
        }
    }

    // Listwise missing value deletion & weighted rows
    let mut valid_rows = Vec::new();
    let mut valid_weights = Vec::new();

    for row_idx in 0..n_total {
        let mut row_valid = true;
        for &(_, series) in items {
            match series[row_idx] {
                Some(val) if !val.is_nan() => {}
                _ => {
                    row_valid = false;
                    break;
                }
            }
        }

        if row_valid {
            let w = if weights.is_empty() {
                1.0
            } else {
                weights.get(row_idx).copied().unwrap_or(1.0)
            };
            if w > 0.0 && !w.is_nan() {
                valid_rows.push(row_idx);
                valid_weights.push(w);
            }
        }
    }

    let n_valid = valid_rows.len();
    if n_valid < 2 {
        return Err(AlgoError::InsufficientData(
            "Factor analysis requires at least 2 valid cases".to_string(),
        ));
    }

    let total_w: f64 = valid_weights.iter().sum();

    // Calculate means and standard deviations
    let mut means = vec![0.0; p];
    for (var_idx, &(_, series)) in items.iter().enumerate() {
        let mut sum = 0.0;
        for (idx, &r) in valid_rows.iter().enumerate() {
            sum += valid_weights[idx] * series[r].unwrap_or(0.0);
        }
        means[var_idx] = sum / total_w;
    }

    let mut variances = vec![0.0; p];
    let denom = if total_w > 1.0 { total_w - 1.0 } else { 1.0 };
    for (var_idx, &(_, series)) in items.iter().enumerate() {
        let m = means[var_idx];
        let mut ss = 0.0;
        for (idx, &r) in valid_rows.iter().enumerate() {
            let diff = series[r].unwrap_or(0.0) - m;
            ss += valid_weights[idx] * diff * diff;
        }
        variances[var_idx] = ss / denom;
        if variances[var_idx] <= 1e-15 {
            return Err(AlgoError::ComputationError(format!(
                "Variable '{}' has zero variance",
                items[var_idx].0
            )));
        }
    }

    let std_devs: Vec<f64> = variances.iter().map(|v| v.sqrt()).collect();

    // Compute correlation matrix R (p x p)
    let mut r_matrix = vec![vec![0.0; p]; p];
    for i in 0..p {
        r_matrix[i][i] = 1.0;
        for j in (i + 1)..p {
            let mi = means[i];
            let mj = means[j];
            let mut cross_sum = 0.0;
            for (idx, &r) in valid_rows.iter().enumerate() {
                let di = items[i].1[r].unwrap_or(0.0) - mi;
                let dj = items[j].1[r].unwrap_or(0.0) - mj;
                cross_sum += valid_weights[idx] * di * dj;
            }
            let cov = cross_sum / denom;
            let corr = (cov / (std_devs[i] * std_devs[j])).clamp(-1.0, 1.0);
            r_matrix[i][j] = corr;
            r_matrix[j][i] = corr;
        }
    }

    // Jacobi decomposition of R
    let (eigenvalues, eigenvectors) = jacobi_eigen_symmetric(&r_matrix);

    // KMO and Bartlett's Test of Sphericity
    let kmo_bartlett = if options.kmo_bartlett {
        // Determinant = product of eigenvalues
        let mut ln_det = 0.0;
        for &val in &eigenvalues {
            if val > 1e-15 {
                ln_det += val.ln();
            } else {
                ln_det += (1e-15_f64).ln();
            }
        }

        let p_f = p as f64;
        let n_f = n_valid as f64;
        let bartlett_df = p * (p - 1) / 2;
        let mult = n_f - 1.0 - (2.0 * p_f + 5.0) / 6.0;
        let approx_chi_sq = (-mult * ln_det).max(0.0);
        let bartlett_sig = chi_square_sf(approx_chi_sq, bartlett_df);

        // KMO calculation via inverse of correlation matrix
        let kmo_val = match invert_matrix(&r_matrix) {
            Ok(inv_r) => {
                let mut sum_r2 = 0.0;
                let mut sum_a2 = 0.0;
                for i in 0..p {
                    for j in 0..p {
                        if i != j {
                            let rij = r_matrix[i][j];
                            sum_r2 += rij * rij;

                            let dii = inv_r[i][i].abs().max(1e-15);
                            let djj = inv_r[j][j].abs().max(1e-15);
                            let aij = -inv_r[i][j] / (dii * djj).sqrt();
                            sum_a2 += aij * aij;
                        }
                    }
                }
                if (sum_r2 + sum_a2) > 1e-15 {
                    sum_r2 / (sum_r2 + sum_a2)
                } else {
                    0.0
                }
            }
            Err(_) => 0.0,
        };

        Some(KmoBartlettResult {
            kmo: kmo_val,
            bartlett_approx_chi_square: approx_chi_sq,
            bartlett_df,
            bartlett_sig,
        })
    } else {
        None
    };

    // Determine number of components to extract
    let m = match options.criteria {
        FactorExtractionCriteria::Mineigen(cutoff) => {
            let count = eigenvalues.iter().filter(|&&e| e >= cutoff).count();
            count.max(1).min(p)
        }
        FactorExtractionCriteria::Factors(k) => k.max(1).min(p),
    };

    // Factor Loadings: l[var][comp] = sqrt(lambda_comp) * eigenvector[var][comp]
    let mut loadings = vec![vec![0.0; m]; p];
    for var_idx in 0..p {
        for comp_idx in 0..m {
            let lambda_sqrt = eigenvalues[comp_idx].sqrt();
            loadings[var_idx][comp_idx] = lambda_sqrt * eigenvectors[var_idx][comp_idx];
        }
    }

    // Communalities
    let communalities: Vec<CommunalityRow> = (0..p)
        .map(|var_idx| {
            let extraction: f64 = (0..m)
                .map(|comp_idx| {
                    let l = loadings[var_idx][comp_idx];
                    l * l
                })
                .sum();
            CommunalityRow {
                variable: items[var_idx].0.to_string(),
                initial: 1.0,
                extraction: extraction.min(1.0),
            }
        })
        .collect();

    // Total Variance Explained
    let p_total = p as f64;
    let mut total_variance_explained = Vec::with_capacity(p);
    let mut init_cum = 0.0;
    let mut ext_cum = 0.0;

    for (k, &val) in eigenvalues.iter().enumerate() {
        let pct = (val / p_total) * 100.0;
        init_cum += pct;

        let (ext_tot, ext_pct, ext_cum_val) = if k < m {
            ext_cum += pct;
            (Some(val), Some(pct), Some(ext_cum))
        } else {
            (None, None, None)
        };

        total_variance_explained.push(VarianceExplainedRow {
            component: k + 1,
            initial_total: val,
            initial_pct_variance: pct,
            initial_cumulative_pct: init_cum,
            extraction_total: ext_tot,
            extraction_pct_variance: ext_pct,
            extraction_cumulative_pct: ext_cum_val,
        });
    }

    // Component Matrix
    let component_matrix: Vec<ComponentMatrixRow> = (0..p)
        .map(|var_idx| ComponentMatrixRow {
            variable: items[var_idx].0.to_string(),
            loadings: loadings[var_idx].clone(),
        })
        .collect();

    let variables: Vec<String> = items.iter().map(|&(name, _)| name.to_string()).collect();

    Ok(FactorAnalysisResult {
        variables,
        n_cases: n_valid,
        n_total,
        kmo_bartlett,
        communalities,
        total_variance_explained,
        component_matrix,
        n_components_extracted: m,
    })
}
