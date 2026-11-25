pub mod exact;
pub mod mantel_haenszel;
pub mod measures;
pub mod ordinal_corr;
pub mod types;
pub mod ztest;

#[cfg(test)]
mod tests;

pub use exact::{exact_rxc, fisher_exact_2x2, monte_carlo_rxc};
pub use mantel_haenszel::compute_cmh;
pub use types::*;

use crate::anova_oneway::AlgoError;
use crate::ttest_onesample::ln_gamma;
use measures::compute_all_measures;
use ordinal_corr::weighted_pearson;
use std::collections::BTreeMap;
use ztest::compute_column_proportion_groups;

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

fn parse_category_scores(cats: &[CrosstabCategory]) -> Vec<f64> {
    let mut parsed = Vec::with_capacity(cats.len());
    for c in cats {
        if let Ok(v) = c.value.trim().parse::<f64>() {
            if v.is_finite() {
                parsed.push(v);
                continue;
            }
        }
        return (1..=cats.len()).map(|i| i as f64).collect();
    }
    parsed
}

fn apply_case_weight(w: f64, mode: NonIntegerWeights) -> f64 {
    match mode {
        NonIntegerWeights::RoundCase => w.round(),
        NonIntegerWeights::TruncateCase => w.floor(),
        _ => w,
    }
}

fn apply_cell_weight(w: f64, mode: NonIntegerWeights) -> f64 {
    match mode {
        NonIntegerWeights::RoundCell => w.round(),
        NonIntegerWeights::TruncateCell => w.floor(),
        NonIntegerWeights::RoundCase => w.round(),
        NonIntegerWeights::TruncateCase => w.floor(),
        NonIntegerWeights::NoAdjust => w,
    }
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

    let mut n_valid_f = 0.0_f64;
    let mut n_missing_f = 0.0_f64;

    let mut row_cat_map: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut col_cat_map: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut valid_items: Vec<(&str, &str, f64)> = Vec::new();

    for it in items {
        if it.weight <= 0.0 || it.weight.is_nan() {
            continue;
        }
        let w = apply_case_weight(it.weight, options.noninteger_weights);
        if w <= 0.0 {
            continue;
        }

        if it.is_missing
            || it.row_value.is_empty()
            || it.col_value.is_empty()
            || it.row_value == "."
            || it.col_value == "."
            || it.row_value == "NA"
            || it.col_value == "NA"
        {
            n_missing_f += w;
            continue;
        }

        n_valid_f += w;
        row_cat_map
            .entry(it.row_value.clone())
            .or_insert_with(|| it.row_label.clone());
        col_cat_map
            .entry(it.col_value.clone())
            .or_insert_with(|| it.col_label.clone());
        valid_items.push((&it.row_value, &it.col_value, w));
    }

    if n_valid_f <= 0.0 {
        return Err(AlgoError::InsufficientData(format!(
            "No valid cases found for '{}' and '{}'",
            row_var, col_var
        )));
    }

    let mut row_cats: Vec<CrosstabCategory> = row_cat_map
        .into_iter()
        .map(|(value, label)| CrosstabCategory { value, label })
        .collect();
    if options.row_order == RowOrder::Descending {
        row_cats.reverse();
    }
    let col_cats: Vec<CrosstabCategory> = col_cat_map
        .into_iter()
        .map(|(value, label)| CrosstabCategory { value, label })
        .collect();

    let num_rows = row_cats.len();
    let num_cols = col_cats.len();

    let row_index: BTreeMap<&str, usize> = row_cats
        .iter()
        .enumerate()
        .map(|(idx, cat)| (cat.value.as_str(), idx))
        .collect();
    let col_index: BTreeMap<&str, usize> = col_cats
        .iter()
        .enumerate()
        .map(|(idx, cat)| (cat.value.as_str(), idx))
        .collect();

    let mut obs_f64 = vec![vec![0.0_f64; num_cols]; num_rows];
    for (r_val, c_val, w) in valid_items {
        if let (Some(&r_i), Some(&c_j)) = (row_index.get(r_val), col_index.get(c_val)) {
            obs_f64[r_i][c_j] += w;
        }
    }

    for row in &mut obs_f64 {
        for cell in row.iter_mut() {
            *cell = apply_cell_weight(*cell, options.noninteger_weights);
        }
    }

    let mut obs_usize = vec![vec![0usize; num_cols]; num_rows];
    let mut row_totals_f = vec![0.0_f64; num_rows];
    let mut col_totals_f = vec![0.0_f64; num_cols];
    let mut grand_f = 0.0_f64;

    for i in 0..num_rows {
        for j in 0..num_cols {
            let o = obs_f64[i][j];
            obs_usize[i][j] = o.round() as usize;
            row_totals_f[i] += o;
            col_totals_f[j] += o;
            grand_f += o;
        }
    }

    let row_totals_u: Vec<usize> = row_totals_f.iter().map(|v| v.round() as usize).collect();
    let col_totals_u: Vec<usize> = col_totals_f.iter().map(|v| v.round() as usize).collect();
    let grand_u = grand_f.round() as usize;

    let z_groups = if options.compare_col_proportions {
        compute_column_proportion_groups(&obs_f64, &col_totals_f, options.bonferroni_adjust)
    } else {
        vec![vec![None; num_cols]; num_rows]
    };

    let want_unstd = options.residuals || options.resid_unstandardized;
    let mut table_rows = Vec::with_capacity(num_rows);
    let mut pearson_chisq = 0.0_f64;
    let mut likelihood_ratio = 0.0_f64;

    for (i, row_cat) in row_cats.iter().enumerate() {
        let r_tot_f = row_totals_f[i];
        let mut cells = Vec::with_capacity(num_cols);

        for j in 0..num_cols {
            let obs = obs_f64[i][j];
            let obs_u = obs_usize[i][j];
            let c_tot_f = col_totals_f[j];
            let expected = if grand_f > 0.0 {
                (r_tot_f * c_tot_f) / grand_f
            } else {
                0.0
            };

            let resid = obs - expected;
            if expected > 0.0 {
                pearson_chisq += resid.powi(2) / expected;
                if obs > 0.0 {
                    likelihood_ratio += obs * (obs / expected).ln();
                }
            }

            let std_res = if expected > 0.0 {
                resid / expected.sqrt()
            } else {
                0.0
            };
            let adj_denom = if grand_f > 0.0 {
                (expected * (1.0 - r_tot_f / grand_f) * (1.0 - c_tot_f / grand_f)).max(0.0)
            } else {
                0.0
            };
            let adj_res = if adj_denom > 0.0 {
                resid / adj_denom.sqrt()
            } else {
                0.0
            };

            let suppressed = options.hide_small_counts && obs_u < options.hide_small_threshold;
            cells.push(CrosstabCell {
                observed: obs_u,
                observed_f64: obs,
                suppressed,
                expected: options.expected.then_some(expected),
                row_pct: options.row_pct.then_some(if r_tot_f > 0.0 {
                    obs / r_tot_f * 100.0
                } else {
                    0.0
                }),
                col_pct: options.col_pct.then_some(if c_tot_f > 0.0 {
                    obs / c_tot_f * 100.0
                } else {
                    0.0
                }),
                total_pct: options.total_pct.then_some(if grand_f > 0.0 {
                    obs / grand_f * 100.0
                } else {
                    0.0
                }),
                residual: want_unstd.then_some(resid),
                std_residual: options.resid_standardized.then_some(std_res),
                adj_residual: options.resid_adjusted.then_some(adj_res),
                col_prop_group: z_groups[i][j].clone(),
            });
        }

        table_rows.push(CrosstabRow {
            category: row_cat.clone(),
            cells,
            row_total: row_totals_u[i],
            row_total_pct: (grand_f > 0.0).then_some(r_tot_f / grand_f * 100.0),
        });
    }

    likelihood_ratio *= 2.0;
    let col_total_pcts = (grand_f > 0.0 && options.total_pct)
        .then(|| col_totals_f.iter().map(|&c| c / grand_f * 100.0).collect());

    let row_scores = parse_category_scores(&row_cats);
    let col_scores = parse_category_scores(&col_cats);

    let df = if num_rows > 1 && num_cols > 1 {
        (num_rows - 1) * (num_cols - 1)
    } else {
        0
    };

    let mut chi_square_tests = Vec::new();
    if options.chisq && df > 0 {
        let pearson_sig = chi_square_sf(pearson_chisq, df);
        let (exact_sig, mc_sig, mc_lo, mc_hi) = match options.exact_mode {
            ExactMode::AsymptoticOnly => (None, None, None, None),
            ExactMode::MonteCarlo {
                confidence,
                samples,
            } => {
                let mc = monte_carlo_rxc(
                    &obs_usize,
                    &row_totals_u,
                    &col_totals_u,
                    confidence,
                    samples,
                );
                (None, Some(mc.p_value), Some(mc.ci_lower), Some(mc.ci_upper))
            }
            ExactMode::Exact { time_limit_min } => {
                let ex = exact_rxc(&obs_usize, &row_totals_u, &col_totals_u, time_limit_min);
                (ex, None, None, None)
            }
        };

        chi_square_tests.push(ChiSquareTestRow {
            name: "Pearson Chi-Square".to_string(),
            value: pearson_chisq,
            df: Some(df),
            asymp_sig: Some(pearson_sig),
            exact_sig,
            monte_carlo_sig: mc_sig,
            mc_ci_lower: mc_lo,
            mc_ci_upper: mc_hi,
        });

        if num_rows == 2 && num_cols == 2 {
            let (a, b, c, d) = (
                obs_usize[0][0],
                obs_usize[0][1],
                obs_usize[1][0],
                obs_usize[1][1],
            );
            let diff = ((obs_f64[0][0] * obs_f64[1][1]) - (obs_f64[0][1] * obs_f64[1][0])).abs();
            let num = grand_f * (diff - 0.5 * grand_f).max(0.0).powi(2);
            let denom = row_totals_f[0] * row_totals_f[1] * col_totals_f[0] * col_totals_f[1];
            let yates = if denom > 0.0 { num / denom } else { 0.0 };
            chi_square_tests.push(ChiSquareTestRow {
                name: "Continuity Correction".to_string(),
                value: yates,
                df: Some(1),
                asymp_sig: Some(chi_square_sf(yates, 1)),
                ..Default::default()
            });
            chi_square_tests.push(ChiSquareTestRow {
                name: "Likelihood Ratio".to_string(),
                value: likelihood_ratio,
                df: Some(df),
                asymp_sig: Some(chi_square_sf(likelihood_ratio, df)),
                ..Default::default()
            });
            chi_square_tests.push(ChiSquareTestRow {
                name: "Fisher's Exact Test".to_string(),
                value: 0.0,
                exact_sig: Some(fisher_exact_2x2(a, b, c, d)),
                ..Default::default()
            });
        } else {
            chi_square_tests.push(ChiSquareTestRow {
                name: "Likelihood Ratio".to_string(),
                value: likelihood_ratio,
                df: Some(df),
                asymp_sig: Some(chi_square_sf(likelihood_ratio, df)),
                ..Default::default()
            });
        }

        if grand_f > 1.0 {
            let r_lin = weighted_pearson(&obs_f64, grand_f, &row_scores, &col_scores);
            let lbl = (grand_f - 1.0) * r_lin * r_lin;
            chi_square_tests.push(ChiSquareTestRow {
                name: "Linear-by-Linear Association".to_string(),
                value: lbl,
                df: Some(1),
                asymp_sig: Some(chi_square_sf(lbl, 1)),
                ..Default::default()
            });
        }

        chi_square_tests.push(ChiSquareTestRow {
            name: "N of Valid Cases".to_string(),
            value: grand_f,
            ..Default::default()
        });
    }

    let meas = compute_all_measures(
        &obs_f64,
        &row_totals_f,
        &col_totals_f,
        grand_f,
        &row_scores,
        &col_scores,
        pearson_chisq,
        df,
        options,
        row_var,
        col_var,
    );

    let mut mcnemar_sig = None;
    if let Some(mc_row) = meas.mcnemar_row {
        mcnemar_sig = mc_row.exact_sig.or(mc_row.asymp_sig);
        chi_square_tests.push(mc_row);
    }

    Ok(CrosstabResult {
        row_var: row_var.to_string(),
        col_var: col_var.to_string(),
        layer_keys: Vec::new(),
        n_valid: n_valid_f.round() as usize,
        n_missing: n_missing_f.round() as usize,
        col_categories: col_cats,
        rows: table_rows,
        col_totals: col_totals_u,
        col_total_pcts,
        grand_total: grand_u,
        chi_square_tests,
        association_measures: meas.association,
        directional_measures: meas.directional,
        risk_estimates: meas.risk,
        mcnemar_sig,
        cmh_result: None,
        options: options.clone(),
    })
}
