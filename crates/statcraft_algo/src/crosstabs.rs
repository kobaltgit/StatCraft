use crate::anova_oneway::AlgoError;
use crate::crosstabs_types::*;
use crate::ttest_onesample::ln_gamma;
use std::collections::BTreeMap;

pub use crate::crosstabs_types::*;

/// Upper regularized incomplete gamma function Q(a, x) = Gamma(a, x) / Gamma(a).
pub fn regularized_incomplete_gamma_q(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    if a <= 0.0 {
        return 0.0;
    }
    if x < a + 1.0 {
        1.0 - regularized_incomplete_gamma_p(a, x)
    } else {
        const TINY: f64 = 1.0e-30;
        const MAX_ITER: usize = 200;
        const EPS: f64 = 1.0e-15;

        let mut b = x + 1.0 - a;
        let mut c = 1.0 / TINY;
        let mut d = 1.0 / b;
        let mut f = d;

        for i in 1..=MAX_ITER {
            let an = -(i as f64) * ((i as f64) - a);
            b += 2.0;
            d = an * d + b;
            if d.abs() < TINY {
                d = TINY;
            }
            c = b + an / c;
            if c.abs() < TINY {
                c = TINY;
            }
            d = 1.0 / d;
            let del = d * c;
            f *= del;
            if (del - 1.0).abs() < EPS {
                break;
            }
        }
        (-x + a * x.ln() - ln_gamma(a)).exp() * f
    }
}

/// Lower regularized incomplete gamma function P(a, x) = gamma(a, x) / Gamma(a).
pub fn regularized_incomplete_gamma_p(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if a <= 0.0 {
        return 1.0;
    }
    if x < a + 1.0 {
        let mut sum = 1.0 / a;
        let mut term = 1.0 / a;
        let mut n = 1.0;
        let max_iter = 200;
        while n <= max_iter as f64 {
            term *= x / (a + n);
            sum += term;
            if term.abs() < sum.abs() * 1.0e-15 {
                break;
            }
            n += 1.0;
        }
        (-x + a * x.ln() - ln_gamma(a)).exp() * sum
    } else {
        1.0 - regularized_incomplete_gamma_q(a, x)
    }
}

/// Survival function for Chi-Square distribution P(X >= chisq) with df degrees of freedom.
pub fn chi_square_sf(chisq: f64, df: usize) -> f64 {
    if df == 0 || chisq <= 0.0 {
        return 1.0;
    }
    if chisq.is_nan() {
        return 1.0;
    }
    if chisq.is_infinite() {
        return 0.0;
    }
    let a = (df as f64) * 0.5;
    let x = chisq * 0.5;
    regularized_incomplete_gamma_q(a, x).clamp(0.0, 1.0)
}

/// Fisher's Exact Test two-sided p-value for a 2x2 contingency table.
pub fn fisher_exact_2x2(a: usize, b: usize, c: usize, d: usize) -> f64 {
    let r1 = a + b;
    let r2 = c + d;
    let c1 = a + c;
    let c2 = b + d;
    let n = r1 + r2;

    if n == 0 {
        return 1.0;
    }

    let log_hyper = |k: usize| -> f64 {
        let k_i = k as f64;
        let r1_f = r1 as f64;
        let r2_f = r2 as f64;
        let c1_f = c1 as f64;
        let c2_f = c2 as f64;
        let n_f = n as f64;

        ln_gamma(r1_f + 1.0) + ln_gamma(r2_f + 1.0) + ln_gamma(c1_f + 1.0) + ln_gamma(c2_f + 1.0)
            - ln_gamma(n_f + 1.0)
            - ln_gamma(k_i + 1.0)
            - ln_gamma(r1_f - k_i + 1.0)
            - ln_gamma(c1_f - k_i + 1.0)
            - ln_gamma(r2_f - c1_f + k_i + 1.0)
    };

    let p_observed = log_hyper(a).exp();
    let min_k = c1.saturating_sub(r2);
    let max_k = std::cmp::min(r1, c1);

    let mut p_two_sided = 0.0_f64;
    for k in min_k..=max_k {
        let p_k = log_hyper(k).exp();
        if p_k <= p_observed * (1.0 + 1.0e-11) {
            p_two_sided += p_k;
        }
    }

    p_two_sided.clamp(0.0, 1.0)
}

pub fn compute_crosstabs(
    row_var: &str,
    col_var: &str,
    items: &[RawCrosstabItem],
    options: &CrosstabOptions,
) -> Result<CrosstabResult, AlgoError> {
    if items.is_empty() {
        return Err(AlgoError::InsufficientData(format!(
            "No data provided for crosstabulation of '{}' and '{}'",
            row_var, col_var
        )));
    }

    let mut n_valid = 0usize;
    let mut n_missing = 0usize;

    let mut row_cat_map: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut col_cat_map: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut valid_items: Vec<(String, String)> = Vec::new();

    for it in items {
        if it.is_missing
            || it.row_value.is_empty()
            || it.col_value.is_empty()
            || it.row_value == "."
            || it.col_value == "."
            || it.row_value == "NA"
            || it.col_value == "NA"
        {
            n_missing += 1;
            continue;
        }

        n_valid += 1;
        row_cat_map
            .entry(it.row_value.clone())
            .or_insert_with(|| it.row_label.clone());
        col_cat_map
            .entry(it.col_value.clone())
            .or_insert_with(|| it.col_label.clone());
        valid_items.push((it.row_value.clone(), it.col_value.clone()));
    }

    if n_valid == 0 {
        return Err(AlgoError::InsufficientData(format!(
            "No valid cases found for '{}' and '{}'",
            row_var, col_var
        )));
    }

    let row_cats: Vec<CrosstabCategory> = row_cat_map
        .into_iter()
        .map(|(value, label)| CrosstabCategory { value, label })
        .collect();
    let col_cats: Vec<CrosstabCategory> = col_cat_map
        .into_iter()
        .map(|(value, label)| CrosstabCategory { value, label })
        .collect();

    let num_rows = row_cats.len();
    let num_cols = col_cats.len();

    let row_index: BTreeMap<String, usize> = row_cats
        .iter()
        .enumerate()
        .map(|(idx, cat)| (cat.value.clone(), idx))
        .collect();
    let col_index: BTreeMap<String, usize> = col_cats
        .iter()
        .enumerate()
        .map(|(idx, cat)| (cat.value.clone(), idx))
        .collect();

    let mut obs_matrix = vec![vec![0usize; num_cols]; num_rows];
    for (r_val, c_val) in valid_items {
        if let (Some(&r_i), Some(&c_j)) = (row_index.get(&r_val), col_index.get(&c_val)) {
            obs_matrix[r_i][c_j] += 1;
        }
    }

    let mut row_totals = vec![0usize; num_rows];
    let mut col_totals = vec![0usize; num_cols];
    let mut grand_total = 0usize;

    for i in 0..num_rows {
        for j in 0..num_cols {
            let o = obs_matrix[i][j];
            row_totals[i] += o;
            col_totals[j] += o;
            grand_total += o;
        }
    }

    let grand_f = grand_total as f64;
    let mut table_rows: Vec<CrosstabRow> = Vec::with_capacity(num_rows);
    let mut pearson_chisq = 0.0_f64;
    let mut likelihood_ratio = 0.0_f64;

    for (i, row_cat) in row_cats.iter().enumerate() {
        let r_tot = row_totals[i];
        let r_tot_f = r_tot as f64;
        let mut cells: Vec<CrosstabCell> = Vec::with_capacity(num_cols);

        for (j, _) in col_cats.iter().enumerate() {
            let obs = obs_matrix[i][j];
            let c_tot = col_totals[j];
            let c_tot_f = c_tot as f64;

            let expected = if grand_f > 0.0 {
                (r_tot_f * c_tot_f) / grand_f
            } else {
                0.0
            };

            let row_pct = if r_tot_f > 0.0 {
                (obs as f64 / r_tot_f) * 100.0
            } else {
                0.0
            };

            let col_pct = if c_tot_f > 0.0 {
                (obs as f64 / c_tot_f) * 100.0
            } else {
                0.0
            };

            let total_pct = if grand_f > 0.0 {
                (obs as f64 / grand_f) * 100.0
            } else {
                0.0
            };

            let residual = (obs as f64) - expected;

            if expected > 0.0 {
                pearson_chisq += residual.powi(2) / expected;
                if obs > 0 {
                    likelihood_ratio += (obs as f64) * ((obs as f64) / expected).ln();
                }
            }

            cells.push(CrosstabCell {
                observed: obs,
                expected: if options.expected {
                    Some(expected)
                } else {
                    None
                },
                row_pct: if options.row_pct { Some(row_pct) } else { None },
                col_pct: if options.col_pct { Some(col_pct) } else { None },
                total_pct: if options.total_pct {
                    Some(total_pct)
                } else {
                    None
                },
                residual: if options.residuals {
                    Some(residual)
                } else {
                    None
                },
            });
        }

        let row_total_pct = if grand_f > 0.0 {
            Some((r_tot_f / grand_f) * 100.0)
        } else {
            None
        };

        table_rows.push(CrosstabRow {
            category: row_cat.clone(),
            cells,
            row_total: r_tot,
            row_total_pct,
        });
    }

    likelihood_ratio *= 2.0;

    let col_total_pcts = if grand_f > 0.0 && options.total_pct {
        Some(
            col_totals
                .iter()
                .map(|&c| (c as f64 / grand_f) * 100.0)
                .collect(),
        )
    } else {
        None
    };

    let mut chi_square_tests = Vec::new();
    let mut association_measures = Vec::new();

    let df = if num_rows > 1 && num_cols > 1 {
        (num_rows - 1) * (num_cols - 1)
    } else {
        0
    };

    if options.chisq && df > 0 {
        let pearson_sig = chi_square_sf(pearson_chisq, df);
        chi_square_tests.push(ChiSquareTestRow {
            name: "Pearson Chi-Square".to_string(),
            value: pearson_chisq,
            df: Some(df),
            asymp_sig: Some(pearson_sig),
            exact_sig: None,
        });

        if num_rows == 2 && num_cols == 2 {
            let a = obs_matrix[0][0];
            let b = obs_matrix[0][1];
            let c = obs_matrix[1][0];
            let d = obs_matrix[1][1];

            let diff = ((a * d) as f64 - (b * c) as f64).abs();
            let num = grand_f * (diff - 0.5 * grand_f).max(0.0).powi(2);
            let denom = row_totals[0] as f64
                * row_totals[1] as f64
                * col_totals[0] as f64
                * col_totals[1] as f64;

            let yates_chisq = if denom > 0.0 { num / denom } else { 0.0 };
            let yates_sig = chi_square_sf(yates_chisq, 1);

            chi_square_tests.push(ChiSquareTestRow {
                name: "Continuity Correction".to_string(),
                value: yates_chisq,
                df: Some(1),
                asymp_sig: Some(yates_sig),
                exact_sig: None,
            });

            let lr_sig = chi_square_sf(likelihood_ratio, df);
            chi_square_tests.push(ChiSquareTestRow {
                name: "Likelihood Ratio".to_string(),
                value: likelihood_ratio,
                df: Some(df),
                asymp_sig: Some(lr_sig),
                exact_sig: None,
            });

            let fisher_sig = fisher_exact_2x2(a, b, c, d);
            chi_square_tests.push(ChiSquareTestRow {
                name: "Fisher's Exact Test".to_string(),
                value: 0.0,
                df: None,
                asymp_sig: None,
                exact_sig: Some(fisher_sig),
            });
        } else {
            let lr_sig = chi_square_sf(likelihood_ratio, df);
            chi_square_tests.push(ChiSquareTestRow {
                name: "Likelihood Ratio".to_string(),
                value: likelihood_ratio,
                df: Some(df),
                asymp_sig: Some(lr_sig),
                exact_sig: None,
            });
        }

        chi_square_tests.push(ChiSquareTestRow {
            name: "N of Valid Cases".to_string(),
            value: grand_total as f64,
            df: None,
            asymp_sig: None,
            exact_sig: None,
        });
    }

    if options.phi_cramer && grand_f > 0.0 && df > 0 {
        let phi = (pearson_chisq / grand_f).sqrt();
        let k_min = std::cmp::min(num_rows - 1, num_cols - 1);
        let cramer_v = if k_min > 0 {
            (pearson_chisq / (grand_f * k_min as f64)).sqrt()
        } else {
            0.0
        };

        let approx_sig = chi_square_sf(pearson_chisq, df);

        association_measures.push(AssociationMeasure {
            name: "Phi".to_string(),
            value: phi,
            approx_sig: Some(approx_sig),
        });
        association_measures.push(AssociationMeasure {
            name: "Cramer's V".to_string(),
            value: cramer_v,
            approx_sig: Some(approx_sig),
        });
    }

    Ok(CrosstabResult {
        row_var: row_var.to_string(),
        col_var: col_var.to_string(),
        n_valid,
        n_missing,
        col_categories: col_cats,
        rows: table_rows,
        col_totals,
        col_total_pcts,
        grand_total,
        chi_square_tests,
        association_measures,
        options: options.clone(),
    })
}
