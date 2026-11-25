use super::chi_square_sf;
use super::types::CmhResult;
use crate::cox::normal_quantile;

/// Stratified 2x2xK analysis: Cochran's, Mantel-Haenszel, Breslow-Day, Tarone's, and Common Odds Ratio.
pub fn compute_cmh(strata: &[[[f64; 2]; 2]], common_or_test_val: f64) -> Option<CmhResult> {
    let valid_strata: Vec<[[f64; 2]; 2]> = strata
        .iter()
        .copied()
        .filter(|s| {
            let n = s[0][0] + s[0][1] + s[1][0] + s[1][1];
            n > 1.0
                && (s[0][0] + s[0][1]) > 0.0
                && (s[1][0] + s[1][1]) > 0.0
                && (s[0][0] + s[1][0]) > 0.0
                && (s[0][1] + s[1][1]) > 0.0
        })
        .collect();

    if valid_strata.is_empty() {
        return None;
    }

    let mut sum_num = 0.0;
    let mut sum_var_cochran = 0.0;
    let mut sum_var_mh = 0.0;

    let mut sum_ad_n = 0.0;
    let mut sum_bc_n = 0.0;
    let mut r_sum = 0.0;
    let mut s_sum = 0.0;
    let mut pr_sum = 0.0;
    let mut ps_qr_sum = 0.0;
    let mut qs_sum = 0.0;

    for s in &valid_strata {
        let a = s[0][0];
        let b = s[0][1];
        let c = s[1][0];
        let d = s[1][1];
        let r1 = a + b;
        let r2 = c + d;
        let c1 = a + c;
        let c2 = b + d;
        let n = r1 + r2;

        let e_a = r1 * c1 / n;
        sum_num += a - e_a;
        sum_var_cochran += (r1 * r2 * c1 * c2) / (n * n * n);
        sum_var_mh += (r1 * r2 * c1 * c2) / (n * n * (n - 1.0));

        let r_k = (a * d) / n;
        let s_k = (b * c) / n;
        let p_k = (a + d) / n;
        let q_k = (b + c) / n;

        sum_ad_n += r_k;
        sum_bc_n += s_k;
        r_sum += r_k;
        s_sum += s_k;
        pr_sum += p_k * r_k;
        ps_qr_sum += p_k * s_k + q_k * r_k;
        qs_sum += q_k * s_k;
    }

    let cochran_stat = if sum_var_cochran > 0.0 {
        sum_num * sum_num / sum_var_cochran
    } else {
        0.0
    };
    let mh_stat = if sum_var_mh > 0.0 {
        (sum_num.abs() - 0.5).max(0.0).powi(2) / sum_var_mh
    } else {
        0.0
    };

    let or_mh = if sum_bc_n > 0.0 {
        sum_ad_n / sum_bc_n
    } else {
        1.0
    };

    // Robins-Breslow-Greenland variance for ln(OR_MH)
    let ln_se = if r_sum > 0.0 && s_sum > 0.0 {
        let var_ln = pr_sum / (2.0 * r_sum * r_sum)
            + ps_qr_sum / (2.0 * r_sum * s_sum)
            + qs_sum / (2.0 * s_sum * s_sum);
        var_ln.max(0.0).sqrt()
    } else {
        0.0
    };

    let z95 = normal_quantile(0.975);
    let (ci_low, ci_high) = if or_mh > 0.0 && ln_se > 0.0 {
        (
            (or_mh.ln() - z95 * ln_se).exp(),
            (or_mh.ln() + z95 * ln_se).exp(),
        )
    } else {
        (or_mh, or_mh)
    };

    let common_or_sig = if or_mh > 0.0 && common_or_test_val > 0.0 && ln_se > 0.0 {
        let z_test = (or_mh.ln() - common_or_test_val.ln()) / ln_se;
        chi_square_sf(z_test * z_test, 1)
    } else {
        1.0
    };

    // Breslow-Day & Tarone's test of homogeneity of OR
    let mut bd_stat = 0.0;
    let mut tarone_adj = 0.0;
    let mut sum_var_bd = 0.0;

    for s in &valid_strata {
        let r1 = s[0][0] + s[0][1];
        let r2 = s[1][0] + s[1][1];
        let c1 = s[0][0] + s[1][0];
        let n = r1 + r2;

        // Solve quadratic: (or_mh - 1) * a_hat^2 - (or_mh*(r1+c1) + (r2-c1)) * a_hat + or_mh*r1*c1 = 0
        let a_hat = if (or_mh - 1.0).abs() < 1.0e-10 {
            r1 * c1 / n
        } else {
            let qa = or_mh - 1.0;
            let qb = -(or_mh * (r1 + c1) + (r2 - c1));
            let qc = or_mh * r1 * c1;
            let disc = (qb * qb - 4.0 * qa * qc).max(0.0).sqrt();
            let root1 = (-qb - disc) / (2.0 * qa);
            let root2 = (-qb + disc) / (2.0 * qa);
            let low = c1.saturating_sub_f64(r2);
            let high = r1.min(c1);
            if root1 >= low - 1e-9 && root1 <= high + 1e-9 {
                root1
            } else {
                root2
            }
        };

        let b_hat = (r1 - a_hat).max(1e-12);
        let c_hat = (c1 - a_hat).max(1e-12);
        let d_hat = (r2 - c_hat).max(1e-12);
        let a_hat_pos = a_hat.max(1e-12);

        let var_a = 1.0 / (1.0 / a_hat_pos + 1.0 / b_hat + 1.0 / c_hat + 1.0 / d_hat);
        if var_a > 0.0 {
            let diff = s[0][0] - a_hat;
            bd_stat += (diff * diff) / var_a;
            tarone_adj += diff;
            sum_var_bd += var_a;
        }
    }

    let tarone_stat = if sum_var_bd > 0.0 {
        (bd_stat - (tarone_adj * tarone_adj) / sum_var_bd).max(0.0)
    } else {
        bd_stat
    };
    let k_df = valid_strata.len().saturating_sub(1);

    Some(CmhResult {
        cochran_stat,
        cochran_df: 1,
        cochran_sig: chi_square_sf(cochran_stat, 1),
        mh_stat,
        mh_df: 1,
        mh_sig: chi_square_sf(mh_stat, 1),
        breslow_day_stat: bd_stat,
        breslow_day_df: k_df,
        breslow_day_sig: if k_df > 0 {
            chi_square_sf(bd_stat, k_df)
        } else {
            1.0
        },
        tarone_stat,
        tarone_df: k_df,
        tarone_sig: if k_df > 0 {
            chi_square_sf(tarone_stat, k_df)
        } else {
            1.0
        },
        common_or_estimate: or_mh,
        common_or_ln_se: ln_se,
        common_or_ci_lower: ci_low,
        common_or_ci_upper: ci_high,
        common_or_test_val,
        common_or_sig,
    })
}

trait SaturatingSubF64 {
    fn saturating_sub_f64(self, rhs: Self) -> Self;
}

impl SaturatingSubF64 for f64 {
    fn saturating_sub_f64(self, rhs: Self) -> Self {
        (self - rhs).max(0.0)
    }
}
