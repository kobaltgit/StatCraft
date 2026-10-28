use crate::anova_oneway::AlgoError;
use crate::correlations_types::*;
use crate::crosstabs::regularized_incomplete_gamma_q;
use crate::ttest_onesample::student_t_sf_2tailed;

pub use crate::correlations_types::*;

pub fn compute_bivariate_correlations(
    variables: &[VariableInput<'_>],
    options: &BivariateCorrelationsOptions,
) -> Result<BivariateCorrelationsResult, AlgoError> {
    if variables.len() < 2 {
        return Err(AlgoError::InvalidInput(
            "At least two variables are required for bivariate correlations".to_string(),
        ));
    }

    let k = variables.len();
    let var_names: Vec<String> = variables.iter().map(|(n, _)| n.to_string()).collect();
    let num_rows = variables[0].1.len();

    // 1. Filter data based on missing handling
    let (cleaned_vars, listwise_valid_mask) = if options.missing == MissingHandling::Listwise {
        let mut mask = vec![true; num_rows];
        for (_, vals) in variables {
            for (row_idx, val) in vals.iter().enumerate() {
                if val.is_none_or(|v| !v.is_finite()) {
                    mask[row_idx] = false;
                }
            }
        }
        let mut filtered: Vec<Vec<Option<f64>>> = Vec::with_capacity(k);
        for (_, vals) in variables {
            let col = vals
                .iter()
                .enumerate()
                .filter(|(idx, _)| mask[*idx])
                .map(|(_, &v)| v)
                .collect();
            filtered.push(col);
        }
        (Some(filtered), Some(mask))
    } else {
        (None, None)
    };

    // 2. Descriptives (if requested)
    let descriptives = if options.descriptives {
        let mut desc_vec = Vec::with_capacity(k);
        for i in 0..k {
            let data: Vec<f64> = if let Some(ref f) = cleaned_vars {
                f[i].iter().filter_map(|&v| v).collect()
            } else {
                variables[i]
                    .1
                    .iter()
                    .filter_map(|&v| v)
                    .filter(|v| v.is_finite())
                    .collect()
            };
            let n = data.len();
            let (mean, std_dev) = if n > 0 {
                let m = data.iter().sum::<f64>() / (n as f64);
                let sd = if n > 1 {
                    let ss: f64 = data.iter().map(|&x| (x - m).powi(2)).sum();
                    Some((ss / ((n - 1) as f64)).max(0.0).sqrt())
                } else {
                    None
                };
                (Some(m), sd)
            } else {
                (None, None)
            };
            desc_vec.push(VarDescriptives {
                var_name: var_names[i].clone(),
                mean,
                std_dev,
                n,
            });
        }
        Some(desc_vec)
    } else {
        None
    };

    // Helper to get pairwise valid slices
    let get_pair = |i: usize, j: usize| -> (Vec<f64>, Vec<f64>) {
        if let Some(ref f) = cleaned_vars {
            let mut x = Vec::with_capacity(f[i].len());
            let mut y = Vec::with_capacity(f[j].len());
            for (vx, vy) in f[i].iter().zip(f[j].iter()) {
                if let (Some(vx), Some(vy)) = (*vx, *vy) {
                    x.push(vx);
                    y.push(vy);
                }
            }
            (x, y)
        } else {
            let len = variables[i].1.len().min(variables[j].1.len());
            let mut x = Vec::with_capacity(len);
            let mut y = Vec::with_capacity(len);
            for (vx, vy) in variables[i].1.iter().zip(variables[j].1.iter()) {
                if let (Some(vx), Some(vy)) = (*vx, *vy) {
                    if vx.is_finite() && vy.is_finite() {
                        x.push(vx);
                        y.push(vy);
                    }
                }
            }
            (x, y)
        }
    };

    let mut matrices = Vec::new();

    // 3. Pearson Matrix
    if options.pearson {
        let mut cells = vec![vec![CorrelationCell::empty(); k]; k];
        for (i, row) in cells.iter_mut().enumerate().take(k) {
            for (j, cell) in row.iter_mut().enumerate().take(k) {
                *cell = compute_pearson_cell(
                    i == j,
                    &get_pair(i, j),
                    options.significance,
                    options.flag_significant,
                    options.cross_prod_cov,
                );
            }
        }
        matrices.push(CorrelationMatrixResult {
            corr_type: CorrelationType::Pearson,
            variables: var_names.clone(),
            cells,
        });
    }

    // 4. Kendall Tau-b Matrix
    if options.kendall {
        let mut cells = vec![vec![CorrelationCell::empty(); k]; k];
        for (i, row) in cells.iter_mut().enumerate().take(k) {
            for (j, cell) in row.iter_mut().enumerate().take(k) {
                *cell = compute_kendall_cell(
                    i == j,
                    &get_pair(i, j),
                    options.significance,
                    options.flag_significant,
                );
            }
        }
        matrices.push(CorrelationMatrixResult {
            corr_type: CorrelationType::KendallTauB,
            variables: var_names.clone(),
            cells,
        });
    }

    // 5. Spearman Matrix
    if options.spearman {
        let mut cells = vec![vec![CorrelationCell::empty(); k]; k];
        for (i, row) in cells.iter_mut().enumerate().take(k) {
            for (j, cell) in row.iter_mut().enumerate().take(k) {
                *cell = compute_spearman_cell(
                    i == j,
                    &get_pair(i, j),
                    options.significance,
                    options.flag_significant,
                );
            }
        }
        matrices.push(CorrelationMatrixResult {
            corr_type: CorrelationType::Spearman,
            variables: var_names.clone(),
            cells,
        });
    }

    let _ = listwise_valid_mask;

    Ok(BivariateCorrelationsResult {
        descriptives,
        matrices,
        options: options.clone(),
    })
}

fn determine_flag(p_val: Option<f64>, flag_sig: bool) -> SignificanceFlag {
    if !flag_sig {
        return SignificanceFlag::None;
    }
    match p_val {
        Some(p) if p < 0.01 => SignificanceFlag::DoubleStar,
        Some(p) if p < 0.05 => SignificanceFlag::SingleStar,
        _ => SignificanceFlag::None,
    }
}

fn compute_pearson_cell(
    is_diag: bool,
    pair: &(Vec<f64>, Vec<f64>),
    sig_test: SignificanceTest,
    flag_sig: bool,
    cross_cov: bool,
) -> CorrelationCell {
    let (ref x, ref y) = pair;
    let n = x.len();
    if n == 0 {
        return CorrelationCell::empty();
    }
    if is_diag {
        let (ss_cp, cov) = if cross_cov && n > 1 {
            let m = x.iter().sum::<f64>() / (n as f64);
            let ss: f64 = x.iter().map(|&v| (v - m).powi(2)).sum();
            (Some(ss), Some(ss / ((n - 1) as f64)))
        } else {
            (None, None)
        };
        return CorrelationCell {
            r: Some(1.0),
            sig: None,
            n,
            ss_cross_prod: ss_cp,
            covariance: cov,
            flag: SignificanceFlag::None,
        };
    }
    if n < 2 {
        return CorrelationCell {
            r: None,
            sig: None,
            n,
            ss_cross_prod: None,
            covariance: None,
            flag: SignificanceFlag::None,
        };
    }

    let mx = x.iter().sum::<f64>() / (n as f64);
    let my = y.iter().sum::<f64>() / (n as f64);
    let mut ss_x = 0.0;
    let mut ss_y = 0.0;
    let mut ss_xy = 0.0;
    for idx in 0..n {
        let dx = x[idx] - mx;
        let dy = y[idx] - my;
        ss_x += dx * dx;
        ss_y += dy * dy;
        ss_xy += dx * dy;
    }

    let (ss_cp, cov) = if cross_cov {
        (Some(ss_xy), Some(ss_xy / ((n - 1) as f64)))
    } else {
        (None, None)
    };

    if ss_x <= 0.0 || ss_y <= 0.0 {
        return CorrelationCell {
            r: None,
            sig: None,
            n,
            ss_cross_prod: ss_cp,
            covariance: cov,
            flag: SignificanceFlag::None,
        };
    }

    let r = (ss_xy / (ss_x * ss_y).sqrt()).clamp(-1.0, 1.0);
    let sig = if n <= 2 {
        None
    } else {
        let df = (n - 2) as f64;
        let diff = (1.0 - r * r).max(0.0);
        if diff < 1e-15 {
            Some(0.0)
        } else {
            let t = r * (df / diff).sqrt();
            let p2 = student_t_sf_2tailed(t, df);
            Some(if sig_test == SignificanceTest::TwoTailed {
                p2
            } else {
                p2 / 2.0
            })
        }
    };

    let flag = determine_flag(sig, flag_sig);
    CorrelationCell {
        r: Some(r),
        sig,
        n,
        ss_cross_prod: ss_cp,
        covariance: cov,
        flag,
    }
}

fn rank_slice(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    if n == 0 {
        return Vec::new();
    }
    let mut indexed: Vec<(usize, f64)> = data.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && (indexed[j].1 - indexed[i].1).abs() < 1e-15 {
            j += 1;
        }
        let avg_rank = (i + 1 + j) as f64 / 2.0;
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }
    ranks
}

fn compute_spearman_cell(
    is_diag: bool,
    pair: &(Vec<f64>, Vec<f64>),
    sig_test: SignificanceTest,
    flag_sig: bool,
) -> CorrelationCell {
    if is_diag {
        return CorrelationCell {
            r: Some(1.0),
            sig: None,
            n: pair.0.len(),
            ss_cross_prod: None,
            covariance: None,
            flag: SignificanceFlag::None,
        };
    }
    let (ref x, ref y) = pair;
    let n = x.len();
    if n < 2 {
        return CorrelationCell {
            r: None,
            sig: None,
            n,
            ss_cross_prod: None,
            covariance: None,
            flag: SignificanceFlag::None,
        };
    }

    let rx = rank_slice(x);
    let ry = rank_slice(y);
    compute_pearson_cell(false, &(rx, ry), sig_test, flag_sig, false)
}

fn compute_kendall_cell(
    is_diag: bool,
    pair: &(Vec<f64>, Vec<f64>),
    sig_test: SignificanceTest,
    flag_sig: bool,
) -> CorrelationCell {
    if is_diag {
        return CorrelationCell {
            r: Some(1.0),
            sig: None,
            n: pair.0.len(),
            ss_cross_prod: None,
            covariance: None,
            flag: SignificanceFlag::None,
        };
    }
    let (ref x, ref y) = pair;
    let n = x.len();
    if n < 2 {
        return CorrelationCell {
            r: None,
            sig: None,
            n,
            ss_cross_prod: None,
            covariance: None,
            flag: SignificanceFlag::None,
        };
    }

    let mut concordant: i64 = 0;
    let mut discordant: i64 = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = x[i] - x[j];
            let dy = y[i] - y[j];
            let sx = if dx > 1e-15 {
                1
            } else if dx < -1e-15 {
                -1
            } else {
                0
            };
            let sy = if dy > 1e-15 {
                1
            } else if dy < -1e-15 {
                -1
            } else {
                0
            };
            let prod = sx * sy;
            if prod > 0 {
                concordant += 1;
            } else if prod < 0 {
                discordant += 1;
            }
        }
    }

    // Tie counts on X and Y
    let tie_stats = |data: &[f64]| -> (f64, f64, f64) {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut t_sum = 0.0;
        let mut vt = 0.0;
        let mut u3 = 0.0;
        let mut i = 0;
        while i < sorted.len() {
            let mut j = i + 1;
            while j < sorted.len() && (sorted[j] - sorted[i]).abs() < 1e-15 {
                j += 1;
            }
            let t = (j - i) as f64;
            if t > 1.0 {
                t_sum += t * (t - 1.0) / 2.0;
                vt += t * (t - 1.0) * (2.0 * t + 5.0);
                u3 += t * (t - 1.0) * (t - 2.0);
            }
            i = j;
        }
        (t_sum, vt, u3)
    };

    let (tx, vtx, u3x) = tie_stats(x);
    let (ty, vty, u3y) = tie_stats(y);

    let n_f = n as f64;
    let n0 = n_f * (n_f - 1.0) / 2.0;
    let denom = ((n0 - tx) * (n0 - ty)).max(0.0).sqrt();

    if denom <= 0.0 {
        return CorrelationCell {
            r: None,
            sig: None,
            n,
            ss_cross_prod: None,
            covariance: None,
            flag: SignificanceFlag::None,
        };
    }

    let s = (concordant - discordant) as f64;
    let tau_b = (s / denom).clamp(-1.0, 1.0);

    let sig = if n <= 2 {
        None
    } else {
        let v0 = n_f * (n_f - 1.0) * (2.0 * n_f + 5.0) / 18.0;
        let v1 = if n0 > 0.0 { (tx * ty) / n0 } else { 0.0 };
        let v2 = if n >= 3 {
            (u3x * u3y) / (9.0 * n_f * (n_f - 1.0) * (n_f - 2.0))
        } else {
            0.0
        };
        let var_s = (v0 - (vtx + vty) / 18.0) + v1 + v2;
        if var_s > 0.0 {
            let z = s / var_s.sqrt();
            let p2 = regularized_incomplete_gamma_q(0.5, (z * z) / 2.0);
            Some(if sig_test == SignificanceTest::TwoTailed {
                p2
            } else {
                p2 / 2.0
            })
        } else {
            Some(1.0)
        }
    };

    let flag = determine_flag(sig, flag_sig);
    CorrelationCell {
        r: Some(tau_b),
        sig,
        n,
        ss_cross_prod: None,
        covariance: None,
        flag,
    }
}
