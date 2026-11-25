use super::types::{AssociationMeasure, CrosstabOptions, DirectionalMeasure};
use crate::bootstrap::normal_cdf;
use crate::ttest_onesample::student_t_sf_2tailed;

#[allow(clippy::too_many_arguments)]
pub fn compute_ordinal_measures(
    obs: &[Vec<f64>],
    row_totals: &[f64],
    col_totals: &[f64],
    n: f64,
    options: &CrosstabOptions,
    row_var: &str,
    col_var: &str,
    assoc_out: &mut Vec<AssociationMeasure>,
    dir_out: &mut Vec<DirectionalMeasure>,
) {
    let nr = obs.len();
    let nc = obs[0].len();
    let mut c_mat = vec![vec![0.0_f64; nc]; nr];
    let mut d_mat = vec![vec![0.0_f64; nc]; nr];

    for i in 0..nr {
        for j in 0..nc {
            let mut c_ij = 0.0;
            let mut d_ij = 0.0;
            for a in 0..nr {
                for b in 0..nc {
                    if (a > i && b > j) || (a < i && b < j) {
                        c_ij += obs[a][b];
                    } else if (a > i && b < j) || (a < i && b > j) {
                        d_ij += obs[a][b];
                    }
                }
            }
            c_mat[i][j] = c_ij;
            d_mat[i][j] = d_ij;
        }
    }

    let mut p_double = 0.0;
    let mut q_double = 0.0;
    let mut sum_diff_sq = 0.0;
    for i in 0..nr {
        for j in 0..nc {
            let o = obs[i][j];
            p_double += o * c_mat[i][j];
            q_double += o * d_mat[i][j];
            sum_diff_sq += o * (c_mat[i][j] - d_mat[i][j]).powi(2);
        }
    }

    let p = 0.5 * p_double;
    let q = 0.5 * q_double;
    let d_r = 0.5 * (n * n - row_totals.iter().map(|r| r * r).sum::<f64>());
    let d_c = 0.5 * (n * n - col_totals.iter().map(|c| c * c).sum::<f64>());
    let w = (sum_diff_sq - 4.0 * (p - q).powi(2) / n).max(0.0);

    let sig_from_t = |t: f64| -> f64 { 2.0 * (1.0 - normal_cdf(t.abs())) };

    if options.gamma {
        let val = if p + q > 0.0 { (p - q) / (p + q) } else { 0.0 };
        let ase = if p + q > 0.0 {
            let mut s = 0.0;
            for i in 0..nr {
                for j in 0..nc {
                    s += obs[i][j] * (q * c_mat[i][j] - p * d_mat[i][j]).powi(2);
                }
            }
            Some(s.sqrt() / (p + q).powi(2))
        } else {
            None
        };
        let ase0 = if p + q > 0.0 {
            w.sqrt() / (2.0 * (p + q))
        } else {
            0.0
        };
        let t = if ase0 > 0.0 { Some(val / ase0) } else { None };
        assoc_out.push(AssociationMeasure {
            name: "Gamma".to_string(),
            category: "Ordinal by Ordinal".to_string(),
            value: val,
            asymp_std_error: ase,
            approx_t: t,
            approx_sig: t.map(sig_from_t),
        });
    }

    if options.kendall_tau_b {
        let denom = (d_r * d_c).sqrt();
        let val = if denom > 0.0 { (p - q) / denom } else { 0.0 };
        let ase0 = if denom > 0.0 {
            w.sqrt() / (2.0 * denom)
        } else {
            0.0
        };
        let t = if ase0 > 0.0 { Some(val / ase0) } else { None };
        assoc_out.push(AssociationMeasure {
            name: "Kendall's tau-b".to_string(),
            category: "Ordinal by Ordinal".to_string(),
            value: val,
            asymp_std_error: if ase0 > 0.0 { Some(ase0) } else { None },
            approx_t: t,
            approx_sig: t.map(sig_from_t),
        });
    }

    if options.kendall_tau_c {
        let m = nr.min(nc) as f64;
        let val = if m > 1.0 && n > 0.0 {
            2.0 * m * (p - q) / (n * n * (m - 1.0))
        } else {
            0.0
        };
        let ase0 = if m > 1.0 && n > 0.0 {
            m * w.sqrt() / (n * n * (m - 1.0))
        } else {
            0.0
        };
        let t = if ase0 > 0.0 { Some(val / ase0) } else { None };
        assoc_out.push(AssociationMeasure {
            name: "Kendall's tau-c".to_string(),
            category: "Ordinal by Ordinal".to_string(),
            value: val,
            asymp_std_error: if ase0 > 0.0 { Some(ase0) } else { None },
            approx_t: t,
            approx_sig: t.map(sig_from_t),
        });
    }

    if options.somers_d {
        let add_somers = |dir: String, num: f64, den: f64, out: &mut Vec<DirectionalMeasure>| {
            let val = if den > 0.0 { num / den } else { 0.0 };
            let ase0 = if den > 0.0 {
                w.sqrt() / (2.0 * den)
            } else {
                0.0
            };
            let t = if ase0 > 0.0 { Some(val / ase0) } else { None };
            out.push(DirectionalMeasure {
                name: "Somers' d".to_string(),
                direction: dir,
                value: val,
                asymp_std_error: if ase0 > 0.0 { Some(ase0) } else { None },
                approx_t: t,
                approx_sig: t.map(sig_from_t),
            });
        };
        add_somers("Symmetric".to_string(), 2.0 * (p - q), d_r + d_c, dir_out);
        add_somers(format!("{} Dependent", row_var), p - q, d_c, dir_out);
        add_somers(format!("{} Dependent", col_var), p - q, d_r, dir_out);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn compute_eta(
    obs: &[Vec<f64>],
    row_totals: &[f64],
    col_totals: &[f64],
    n: f64,
    row_scores: &[f64],
    col_scores: &[f64],
    row_var: &str,
    col_var: &str,
    out: &mut Vec<DirectionalMeasure>,
) {
    let nr = obs.len();
    let nc = obs[0].len();

    let mean_r: f64 = (0..nr).map(|i| row_totals[i] * row_scores[i]).sum::<f64>() / n;
    let ss_tot_r: f64 = (0..nr)
        .map(|i| row_totals[i] * (row_scores[i] - mean_r).powi(2))
        .sum();
    let mut ss_within_r = 0.0;
    for j in 0..nc {
        if col_totals[j] > 0.0 {
            let mean_j: f64 =
                (0..nr).map(|i| obs[i][j] * row_scores[i]).sum::<f64>() / col_totals[j];
            for i in 0..nr {
                ss_within_r += obs[i][j] * (row_scores[i] - mean_j).powi(2);
            }
        }
    }
    let eta_r = if ss_tot_r > 0.0 {
        (1.0 - ss_within_r / ss_tot_r).max(0.0).sqrt()
    } else {
        0.0
    };
    out.push(DirectionalMeasure {
        name: "Eta".to_string(),
        direction: format!("{} Dependent", row_var),
        value: eta_r,
        asymp_std_error: None,
        approx_t: None,
        approx_sig: None,
    });

    let mean_c: f64 = (0..nc).map(|j| col_totals[j] * col_scores[j]).sum::<f64>() / n;
    let ss_tot_c: f64 = (0..nc)
        .map(|j| col_totals[j] * (col_scores[j] - mean_c).powi(2))
        .sum();
    let mut ss_within_c = 0.0;
    for i in 0..nr {
        if row_totals[i] > 0.0 {
            let mean_i: f64 =
                (0..nc).map(|j| obs[i][j] * col_scores[j]).sum::<f64>() / row_totals[i];
            for j in 0..nc {
                ss_within_c += obs[i][j] * (col_scores[j] - mean_i).powi(2);
            }
        }
    }
    let eta_c = if ss_tot_c > 0.0 {
        (1.0 - ss_within_c / ss_tot_c).max(0.0).sqrt()
    } else {
        0.0
    };
    out.push(DirectionalMeasure {
        name: "Eta".to_string(),
        direction: format!("{} Dependent", col_var),
        value: eta_c,
        asymp_std_error: None,
        approx_t: None,
        approx_sig: None,
    });
}

pub fn compute_correlations(
    obs: &[Vec<f64>],
    n: f64,
    row_scores: &[f64],
    col_scores: &[f64],
    out: &mut Vec<AssociationMeasure>,
) {
    let nr = obs.len();
    let nc = obs[0].len();
    let r_val = weighted_pearson(obs, n, row_scores, col_scores);
    let t_r = if (1.0 - r_val * r_val) > 1e-15 {
        r_val * ((n - 2.0) / (1.0 - r_val * r_val)).sqrt()
    } else {
        0.0
    };
    let sig_r = student_t_sf_2tailed(t_r, n - 2.0);
    out.push(AssociationMeasure {
        name: "Pearson's R".to_string(),
        category: "Interval by Interval".to_string(),
        value: r_val,
        asymp_std_error: None,
        approx_t: Some(t_r),
        approx_sig: Some(sig_r),
    });

    let mut row_ranks = vec![0.0; nr];
    let mut cum = 0.0;
    for i in 0..nr {
        let rt: f64 = obs[i].iter().sum();
        row_ranks[i] = cum + (rt + 1.0) * 0.5;
        cum += rt;
    }
    let mut col_ranks = vec![0.0; nc];
    cum = 0.0;
    for j in 0..nc {
        let ct: f64 = (0..nr).map(|i| obs[i][j]).sum();
        col_ranks[j] = cum + (ct + 1.0) * 0.5;
        cum += ct;
    }

    let rho = weighted_pearson(obs, n, &row_ranks, &col_ranks);
    let t_rho = if (1.0 - rho * rho) > 1e-15 {
        rho * ((n - 2.0) / (1.0 - rho * rho)).sqrt()
    } else {
        0.0
    };
    let sig_rho = student_t_sf_2tailed(t_rho, n - 2.0);
    out.push(AssociationMeasure {
        name: "Spearman Correlation".to_string(),
        category: "Ordinal by Ordinal".to_string(),
        value: rho,
        asymp_std_error: None,
        approx_t: Some(t_rho),
        approx_sig: Some(sig_rho),
    });
}

pub fn weighted_pearson(obs: &[Vec<f64>], n: f64, r_scores: &[f64], c_scores: &[f64]) -> f64 {
    let nr = obs.len();
    let nc = obs[0].len();
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    for i in 0..nr {
        for j in 0..nc {
            let w = obs[i][j];
            sum_x += w * r_scores[i];
            sum_y += w * c_scores[j];
        }
    }
    let mx = sum_x / n;
    let my = sum_y / n;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    let mut sxy = 0.0;
    for i in 0..nr {
        for j in 0..nc {
            let w = obs[i][j];
            let dx = r_scores[i] - mx;
            let dy = c_scores[j] - my;
            sxx += w * dx * dx;
            syy += w * dy * dy;
            sxy += w * dx * dy;
        }
    }
    if sxx > 0.0 && syy > 0.0 {
        (sxy / (sxx * syy).sqrt()).clamp(-1.0, 1.0)
    } else {
        0.0
    }
}
