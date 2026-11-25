use super::chi_square_sf;
use super::ordinal_corr::{compute_correlations, compute_eta, compute_ordinal_measures};
use super::types::{
    AssociationMeasure, ChiSquareTestRow, CrosstabOptions, DirectionalMeasure, RiskEstimate,
};
use crate::bootstrap::normal_cdf;
use crate::ttest_onesample::ln_gamma;

pub struct MeasuresOutput {
    pub association: Vec<AssociationMeasure>,
    pub directional: Vec<DirectionalMeasure>,
    pub risk: Vec<RiskEstimate>,
    pub mcnemar_row: Option<ChiSquareTestRow>,
}

#[allow(clippy::too_many_arguments)]
pub fn compute_all_measures(
    obs: &[Vec<f64>],
    row_totals: &[f64],
    col_totals: &[f64],
    grand_total: f64,
    row_scores: &[f64],
    col_scores: &[f64],
    pearson_chisq: f64,
    df: usize,
    options: &CrosstabOptions,
    row_var: &str,
    col_var: &str,
) -> MeasuresOutput {
    let mut association = Vec::new();
    let mut directional = Vec::new();
    let mut risk = Vec::new();
    let mut mcnemar_row = None;

    let num_rows = obs.len();
    let num_cols = if num_rows > 0 { obs[0].len() } else { 0 };
    if grand_total <= 0.0 || num_rows == 0 || num_cols == 0 {
        return MeasuresOutput {
            association,
            directional,
            risk,
            mcnemar_row,
        };
    }

    if options.phi_cramer && df > 0 {
        let phi = (pearson_chisq / grand_total).sqrt();
        let k_min = (num_rows - 1).min(num_cols - 1);
        let cramer_v = if k_min > 0 {
            (pearson_chisq / (grand_total * k_min as f64)).sqrt()
        } else {
            0.0
        };
        let sig = Some(chi_square_sf(pearson_chisq, df));
        association.push(AssociationMeasure {
            name: "Phi".to_string(),
            category: "Nominal by Nominal".to_string(),
            value: phi,
            asymp_std_error: None,
            approx_t: None,
            approx_sig: sig,
        });
        association.push(AssociationMeasure {
            name: "Cramer's V".to_string(),
            category: "Nominal by Nominal".to_string(),
            value: cramer_v,
            asymp_std_error: None,
            approx_t: None,
            approx_sig: sig,
        });
    }

    if options.contingency_coef && df > 0 {
        let c = (pearson_chisq / (pearson_chisq + grand_total)).sqrt();
        association.push(AssociationMeasure {
            name: "Contingency Coefficient".to_string(),
            category: "Nominal by Nominal".to_string(),
            value: c,
            asymp_std_error: None,
            approx_t: None,
            approx_sig: Some(chi_square_sf(pearson_chisq, df)),
        });
    }

    if options.lambda {
        compute_lambda(
            obs,
            row_totals,
            col_totals,
            grand_total,
            row_var,
            col_var,
            &mut directional,
        );
    }

    if options.uncertainty_coef && df > 0 {
        compute_uncertainty(
            obs,
            row_totals,
            col_totals,
            grand_total,
            df,
            row_var,
            col_var,
            &mut directional,
        );
    }

    if options.gamma || options.somers_d || options.kendall_tau_b || options.kendall_tau_c {
        compute_ordinal_measures(
            obs,
            row_totals,
            col_totals,
            grand_total,
            options,
            row_var,
            col_var,
            &mut association,
            &mut directional,
        );
    }

    if options.eta {
        compute_eta(
            obs,
            row_totals,
            col_totals,
            grand_total,
            row_scores,
            col_scores,
            row_var,
            col_var,
            &mut directional,
        );
    }

    if options.correlations && grand_total > 2.0 {
        compute_correlations(obs, grand_total, row_scores, col_scores, &mut association);
    }

    if options.kappa && num_rows == num_cols && num_rows > 1 {
        compute_kappa(obs, row_totals, col_totals, grand_total, &mut association);
    }

    if options.risk && num_rows == 2 && num_cols == 2 {
        compute_risk(obs, row_var, col_var, &mut risk);
    }

    if options.mcnemar && num_rows == num_cols && num_rows >= 2 {
        mcnemar_row = Some(compute_mcnemar(obs));
    }

    MeasuresOutput {
        association,
        directional,
        risk,
        mcnemar_row,
    }
}

fn compute_lambda(
    obs: &[Vec<f64>],
    row_totals: &[f64],
    col_totals: &[f64],
    n: f64,
    row_var: &str,
    col_var: &str,
    out: &mut Vec<DirectionalMeasure>,
) {
    let nr = obs.len();
    let nc = obs[0].len();
    let (r_max_idx, rm) =
        row_totals
            .iter()
            .enumerate()
            .fold(
                (0usize, 0.0_f64),
                |(bi, bv), (i, &v)| if v > bv { (i, v) } else { (bi, bv) },
            );
    let (c_max_idx, cm) =
        col_totals
            .iter()
            .enumerate()
            .fold(
                (0usize, 0.0_f64),
                |(bi, bv), (j, &v)| if v > bv { (j, v) } else { (bi, bv) },
            );

    let mut sum_col_max = 0.0;
    let mut sum_col_max_in_rmax = 0.0;
    for j in 0..nc {
        let mut best_i = 0;
        let mut best_v = -1.0;
        for i in 0..nr {
            if obs[i][j] > best_v {
                best_v = obs[i][j];
                best_i = i;
            }
        }
        sum_col_max += best_v.max(0.0);
        if best_i == r_max_idx {
            sum_col_max_in_rmax += best_v.max(0.0);
        }
    }

    let mut sum_row_max = 0.0;
    let mut sum_row_max_in_cmax = 0.0;
    for i in 0..nr {
        let mut best_j = 0;
        let mut best_v = -1.0;
        for j in 0..nc {
            if obs[i][j] > best_v {
                best_v = obs[i][j];
                best_j = j;
            }
        }
        sum_row_max += best_v.max(0.0);
        if best_j == c_max_idx {
            sum_row_max_in_cmax += best_v.max(0.0);
        }
    }

    let denom_sym = 2.0 * n - rm - cm;
    let lam_sym = if denom_sym > 0.0 {
        (sum_col_max + sum_row_max - rm - cm) / denom_sym
    } else {
        0.0
    };
    out.push(DirectionalMeasure {
        name: "Lambda".to_string(),
        direction: "Symmetric".to_string(),
        value: lam_sym,
        asymp_std_error: None,
        approx_t: None,
        approx_sig: None,
    });

    let denom_r = n - rm;
    let lam_r = if denom_r > 0.0 {
        (sum_col_max - rm) / denom_r
    } else {
        0.0
    };
    let ase_r = if denom_r > 0.0 {
        let num = (n - sum_col_max) * (sum_col_max + rm - 2.0 * sum_col_max_in_rmax);
        Some((num.max(0.0) / denom_r.powi(3)).sqrt())
    } else {
        None
    };
    out.push(DirectionalMeasure {
        name: "Lambda".to_string(),
        direction: format!("{} Dependent", row_var),
        value: lam_r,
        asymp_std_error: ase_r,
        approx_t: None,
        approx_sig: None,
    });

    let denom_c = n - cm;
    let lam_c = if denom_c > 0.0 {
        (sum_row_max - cm) / denom_c
    } else {
        0.0
    };
    let ase_c = if denom_c > 0.0 {
        let num = (n - sum_row_max) * (sum_row_max + cm - 2.0 * sum_row_max_in_cmax);
        Some((num.max(0.0) / denom_c.powi(3)).sqrt())
    } else {
        None
    };
    out.push(DirectionalMeasure {
        name: "Lambda".to_string(),
        direction: format!("{} Dependent", col_var),
        value: lam_c,
        asymp_std_error: ase_c,
        approx_t: None,
        approx_sig: None,
    });
}

#[allow(clippy::too_many_arguments)]
fn compute_uncertainty(
    obs: &[Vec<f64>],
    row_totals: &[f64],
    col_totals: &[f64],
    n: f64,
    df: usize,
    row_var: &str,
    col_var: &str,
    out: &mut Vec<DirectionalMeasure>,
) {
    let h_r: f64 = row_totals
        .iter()
        .filter(|&&r| r > 0.0)
        .map(|&r| {
            let p = r / n;
            -p * p.ln()
        })
        .sum();
    let h_c: f64 = col_totals
        .iter()
        .filter(|&&c| c > 0.0)
        .map(|&c| {
            let p = c / n;
            -p * p.ln()
        })
        .sum();
    let mut h_rc = 0.0;
    for row in obs {
        for &v in row {
            if v > 0.0 {
                let p = v / n;
                h_rc -= p * p.ln();
            }
        }
    }
    let mi = (h_r + h_c - h_rc).max(0.0);
    let sig = Some(chi_square_sf(2.0 * n * mi, df));

    let u_sym = if h_r + h_c > 0.0 {
        2.0 * mi / (h_r + h_c)
    } else {
        0.0
    };
    let u_r = if h_r > 0.0 { mi / h_r } else { 0.0 };
    let u_c = if h_c > 0.0 { mi / h_c } else { 0.0 };

    for (dir, val) in [
        ("Symmetric".to_string(), u_sym),
        (format!("{} Dependent", row_var), u_r),
        (format!("{} Dependent", col_var), u_c),
    ] {
        out.push(DirectionalMeasure {
            name: "Uncertainty Coefficient".to_string(),
            direction: dir,
            value: val,
            asymp_std_error: None,
            approx_t: None,
            approx_sig: sig,
        });
    }
}

fn compute_kappa(
    obs: &[Vec<f64>],
    row_totals: &[f64],
    col_totals: &[f64],
    n: f64,
    out: &mut Vec<AssociationMeasure>,
) {
    let k = obs.len();
    let p_o: f64 = (0..k).map(|i| obs[i][i]).sum::<f64>() / n;
    let p_e: f64 = (0..k).map(|i| row_totals[i] * col_totals[i]).sum::<f64>() / (n * n);
    if (1.0 - p_e).abs() < 1e-15 {
        return;
    }
    let kappa = (p_o - p_e) / (1.0 - p_e);
    let sum_rc_sum: f64 = (0..k)
        .map(|i| row_totals[i] * col_totals[i] * (row_totals[i] + col_totals[i]))
        .sum::<f64>()
        / (n * n * n);
    let var0 = (p_e + p_e * p_e - sum_rc_sum).max(0.0) / (n * (1.0 - p_e).powi(2));
    let se0 = var0.sqrt();
    let t = if se0 > 0.0 { Some(kappa / se0) } else { None };
    let sig = t.map(|z| 2.0 * (1.0 - normal_cdf(z.abs())));
    out.push(AssociationMeasure {
        name: "Kappa".to_string(),
        category: "Measure of Agreement".to_string(),
        value: kappa,
        asymp_std_error: Some(se0),
        approx_t: t,
        approx_sig: sig,
    });
}

fn compute_risk(obs: &[Vec<f64>], row_var: &str, col_var: &str, out: &mut Vec<RiskEstimate>) {
    let a = obs[0][0];
    let b = obs[0][1];
    let c = obs[1][0];
    let d = obs[1][1];
    let z = 1.959963984540054;

    if a > 0.0 && b > 0.0 && c > 0.0 && d > 0.0 {
        let or_val = (a * d) / (b * c);
        let se_ln = (1.0 / a + 1.0 / b + 1.0 / c + 1.0 / d).sqrt();
        out.push(RiskEstimate {
            name: format!("Odds Ratio for {}", row_var),
            value: or_val,
            ci_lower: Some((or_val.ln() - z * se_ln).exp()),
            ci_upper: Some((or_val.ln() + z * se_ln).exp()),
        });
    }
    if a > 0.0 && c > 0.0 && (a + b) > 0.0 && (c + d) > 0.0 {
        let rr1 = (a / (a + b)) / (c / (c + d));
        let se_ln = (b / (a * (a + b)) + d / (c * (c + d))).sqrt();
        out.push(RiskEstimate {
            name: format!("For cohort {} (Col 1)", col_var),
            value: rr1,
            ci_lower: Some((rr1.ln() - z * se_ln).exp()),
            ci_upper: Some((rr1.ln() + z * se_ln).exp()),
        });
    }
    if b > 0.0 && d > 0.0 && (a + b) > 0.0 && (c + d) > 0.0 {
        let rr2 = (b / (a + b)) / (d / (c + d));
        let se_ln = (a / (b * (a + b)) + c / (d * (c + d))).sqrt();
        out.push(RiskEstimate {
            name: format!("For cohort {} (Col 2)", col_var),
            value: rr2,
            ci_lower: Some((rr2.ln() - z * se_ln).exp()),
            ci_upper: Some((rr2.ln() + z * se_ln).exp()),
        });
    }
}

fn compute_mcnemar(obs: &[Vec<f64>]) -> ChiSquareTestRow {
    let k = obs.len();
    if k == 2 {
        let b = obs[0][1].round() as usize;
        let c = obs[1][0].round() as usize;
        let n_disc = b + c;
        if n_disc == 0 {
            return ChiSquareTestRow {
                name: "McNemar Test".to_string(),
                value: 0.0,
                df: None,
                asymp_sig: None,
                exact_sig: Some(1.0),
                ..Default::default()
            };
        }
        let k_min = b.min(c);
        let mut p_one_sided = 0.0;
        let log_half = -(n_disc as f64) * 2.0_f64.ln();
        for i in 0..=k_min {
            let log_binom = ln_gamma(n_disc as f64 + 1.0)
                - ln_gamma(i as f64 + 1.0)
                - ln_gamma((n_disc - i) as f64 + 1.0)
                + log_half;
            p_one_sided += log_binom.exp();
        }
        let exact_p = (2.0 * p_one_sided).min(1.0);
        let chisq = ((b as f64 - c as f64).abs() - 1.0).max(0.0).powi(2) / (n_disc as f64);
        ChiSquareTestRow {
            name: "McNemar Test".to_string(),
            value: chisq,
            df: Some(1),
            asymp_sig: Some(chi_square_sf(chisq, 1)),
            exact_sig: Some(exact_p),
            ..Default::default()
        }
    } else {
        let mut bowker = 0.0;
        for i in 0..k {
            for j in (i + 1)..k {
                let s = obs[i][j] + obs[j][i];
                if s > 0.0 {
                    bowker += (obs[i][j] - obs[j][i]).powi(2) / s;
                }
            }
        }
        let df = k * (k - 1) / 2;
        ChiSquareTestRow {
            name: "McNemar-Bowker Test".to_string(),
            value: bowker,
            df: Some(df),
            asymp_sig: Some(chi_square_sf(bowker, df)),
            ..Default::default()
        }
    }
}
