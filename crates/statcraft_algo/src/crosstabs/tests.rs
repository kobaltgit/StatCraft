use super::*;

fn build_items(matrix: &[&[usize]], row_vals: &[&str], col_vals: &[&str]) -> Vec<RawCrosstabItem> {
    let mut items = Vec::new();
    for (i, row) in matrix.iter().enumerate() {
        for (j, &cnt) in row.iter().enumerate() {
            for _ in 0..cnt {
                items.push(RawCrosstabItem {
                    row_value: row_vals[i].to_string(),
                    row_label: None,
                    col_value: col_vals[j].to_string(),
                    col_label: None,
                    is_missing: false,
                    weight: 1.0,
                });
            }
        }
    }
    items
}

#[test]
fn test_chi_square_sf_analytical() {
    let p = chi_square_sf(5.991464547107979, 2);
    assert!((p - 0.05).abs() < 1e-6);
    let p1 = chi_square_sf(3.841458826890731, 1);
    assert!((p1 - 0.05).abs() < 1e-6);
    assert!(chi_square_sf(1000.0, 1) < 1e-15);
    assert_eq!(chi_square_sf(0.0, 2), 1.0);
}

#[test]
fn test_fisher_exact_2x2_standard() {
    assert!((fisher_exact_2x2(3, 1, 1, 3) - 0.4857142857).abs() < 1e-6);
    assert!((fisher_exact_2x2(4, 0, 0, 4) - 0.02857142857).abs() < 1e-6);
}

#[test]
fn test_crosstabs_2x2_golden_oracle_and_residuals() {
    let items = build_items(&[&[20, 30], &[40, 10]], &["0", "1"], &["0", "1"]);
    let opts = CrosstabOptions {
        resid_standardized: true,
        resid_adjusted: true,
        contingency_coef: true,
        risk: true,
        kappa: true,
        mcnemar: true,
        ..Default::default()
    };
    let res = compute_crosstabs("gender", "treatment", &items, &opts).unwrap();
    assert_eq!(res.n_valid, 100);
    // Cell (0,0): obs=20, exp=30, resid=-10, std_res = -10/sqrt(30), adj_res = -10/sqrt(30*0.5*0.4) = -10/sqrt(6)
    let c00 = &res.rows[0].cells[0];
    assert!((c00.expected.unwrap() - 30.0).abs() < 1e-7);
    assert!((c00.residual.unwrap() - (-10.0)).abs() < 1e-7);
    assert!((c00.std_residual.unwrap() - (-10.0 / 30.0_f64.sqrt())).abs() < 1e-6);
    assert!((c00.adj_residual.unwrap() - (-10.0 / 6.0_f64.sqrt())).abs() < 1e-6);

    // Pearson Chi-Square = 16.6666667
    let p_test = res
        .chi_square_tests
        .iter()
        .find(|t| t.name == "Pearson Chi-Square")
        .unwrap();
    assert!((p_test.value - 16.6666666667).abs() < 1e-6);

    // Contingency coef = sqrt(16.6666667 / 116.6666667) = 0.377964473
    let cc = res
        .association_measures
        .iter()
        .find(|m| m.name == "Contingency Coefficient")
        .unwrap();
    assert!((cc.value - (16.6666666667 / 116.6666666667_f64).sqrt()).abs() < 1e-6);

    // Risk OR = (20*10)/(30*40) = 200/1200 = 0.166666667
    let or = res
        .risk_estimates
        .iter()
        .find(|r| r.name.starts_with("Odds Ratio"))
        .unwrap();
    assert!((or.value - (1.0 / 6.0)).abs() < 1e-6);

    // Kappa: p_o = 30/100 = 0.3, p_e = (50*60 + 50*40)/10000 = 0.5 -> (0.3-0.5)/(1-0.5) = -0.4
    let kap = res
        .association_measures
        .iter()
        .find(|m| m.name == "Kappa")
        .unwrap();
    assert!((kap.value - (-0.4)).abs() < 1e-6);

    // McNemar: b=30, c=40
    assert!(res.mcnemar_sig.is_some());
}

#[test]
fn test_ordinal_nominal_and_correlations_oracle() {
    // 3x3 ordinal table:
    // [12, 8, 5]
    // [6, 15, 9]
    // [4, 7, 14]
    let items = build_items(
        &[&[12, 8, 5], &[6, 15, 9], &[4, 7, 14]],
        &["1", "2", "3"],
        &["1", "2", "3"],
    );
    let opts = CrosstabOptions {
        lambda: true,
        uncertainty_coef: true,
        gamma: true,
        somers_d: true,
        kendall_tau_b: true,
        kendall_tau_c: true,
        eta: true,
        correlations: true,
        ..Default::default()
    };
    let res = compute_crosstabs("row", "col", &items, &opts).unwrap();

    // P = 12*(15+9+7+14) + 8*(9+14) + 6*(7+14) + 15*14 = 12*45 + 8*23 + 6*21 + 210 = 540+184+126+210 = 1060
    // Q = 5*(6+15+4+7) + 8*(6+4) + 9*(4+7) + 15*4 = 5*32 + 8*10 + 9*11 + 60 = 160+80+99+60 = 399
    // Gamma = (1060 - 399) / (1060 + 399) = 661 / 1459 = 0.453050034
    let gamma = res
        .association_measures
        .iter()
        .find(|m| m.name == "Gamma")
        .unwrap();
    assert!((gamma.value - (661.0 / 1459.0)).abs() < 1e-6);

    // tau-c: m=3, N=80 -> 2*3*661 / (6400*2) = 3966 / 12800 = 0.30984375
    let tau_c = res
        .association_measures
        .iter()
        .find(|m| m.name == "Kendall's tau-c")
        .unwrap();
    assert!((tau_c.value - 0.30984375).abs() < 1e-6);

    // Linear-by-Linear: N=80, r^2 * 79
    let lbl = res
        .chi_square_tests
        .iter()
        .find(|t| t.name == "Linear-by-Linear Association")
        .unwrap();
    let r = res
        .association_measures
        .iter()
        .find(|m| m.name == "Pearson's R")
        .unwrap();
    assert!((lbl.value - 79.0 * r.value * r.value).abs() < 1e-6);

    // Lambda, Uncertainty, Somers' d, Eta all present
    assert!(res.directional_measures.iter().any(|m| m.name == "Lambda"));
    assert!(res
        .directional_measures
        .iter()
        .any(|m| m.name == "Uncertainty Coefficient"));
    assert!(res
        .directional_measures
        .iter()
        .any(|m| m.name == "Somers' d"));
    assert!(res.directional_measures.iter().any(|m| m.name == "Eta"));
    assert!(res
        .association_measures
        .iter()
        .any(|m| m.name == "Spearman Correlation"));
}

#[test]
fn test_exact_mc_and_ztest_and_options() {
    let items = build_items(&[&[10, 2, 1], &[1, 10, 2]], &["1", "2"], &["1", "2", "3"]);
    let opts = CrosstabOptions {
        row_order: RowOrder::Descending,
        hide_small_counts: true,
        hide_small_threshold: 5,
        compare_col_proportions: true,
        bonferroni_adjust: true,
        exact_mode: ExactMode::MonteCarlo {
            confidence: 0.99,
            samples: 2000,
        },
        ..Default::default()
    };
    let res = compute_crosstabs("r", "c", &items, &opts).unwrap();
    // Row order descending: first row should be "2"
    assert_eq!(res.rows[0].category.value, "2");
    // Small count suppressed (< 5)
    assert!(res.rows[0].cells[0].suppressed);
    assert!(!res.rows[0].cells[1].suppressed);
    // Column prop group assigned
    assert!(res.rows[0].cells[0].col_prop_group.is_some());
    // MC p-value populated and small
    let p_test = res
        .chi_square_tests
        .iter()
        .find(|t| t.name == "Pearson Chi-Square")
        .unwrap();
    assert!(p_test.monte_carlo_sig.unwrap() < 0.01);
    assert!(p_test.mc_ci_lower.is_some());

    // Exact rxc on small 2x3 table
    let opts_ex = CrosstabOptions {
        exact_mode: ExactMode::Exact {
            time_limit_min: Some(1),
        },
        ..Default::default()
    };
    let res_ex = compute_crosstabs("r", "c", &items, &opts_ex).unwrap();
    let p_ex = res_ex
        .chi_square_tests
        .iter()
        .find(|t| t.name == "Pearson Chi-Square")
        .unwrap();
    assert!(p_ex.exact_sig.unwrap() < 0.01);
}

#[test]
fn test_noninteger_weights_modes() {
    let items = vec![
        RawCrosstabItem {
            row_value: "1".to_string(),
            row_label: None,
            col_value: "1".to_string(),
            col_label: None,
            is_missing: false,
            weight: 1.4,
        },
        RawCrosstabItem {
            row_value: "1".to_string(),
            row_label: None,
            col_value: "1".to_string(),
            col_label: None,
            is_missing: false,
            weight: 1.4,
        },
        RawCrosstabItem {
            row_value: "2".to_string(),
            row_label: None,
            col_value: "2".to_string(),
            col_label: None,
            is_missing: false,
            weight: 2.6,
        },
    ];

    // RoundCell: (1.4+1.4)=2.8 -> 3; 2.6 -> 3
    let r_rc = compute_crosstabs(
        "r",
        "c",
        &items,
        &CrosstabOptions {
            noninteger_weights: NonIntegerWeights::RoundCell,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(r_rc.rows[0].cells[0].observed, 3);
    assert_eq!(r_rc.rows[1].cells[1].observed, 3);

    // TruncateCell: 2.8 -> 2; 2.6 -> 2
    let r_tc = compute_crosstabs(
        "r",
        "c",
        &items,
        &CrosstabOptions {
            noninteger_weights: NonIntegerWeights::TruncateCell,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(r_tc.rows[0].cells[0].observed, 2);
    assert_eq!(r_tc.rows[1].cells[1].observed, 2);

    // RoundCase: 1.4->1 + 1.4->1 = 2; 2.6->3
    let r_rk = compute_crosstabs(
        "r",
        "c",
        &items,
        &CrosstabOptions {
            noninteger_weights: NonIntegerWeights::RoundCase,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(r_rk.rows[0].cells[0].observed, 2);
    assert_eq!(r_rk.rows[1].cells[1].observed, 3);

    // NoAdjust: observed_f64 = 2.8 and 2.6
    let r_na = compute_crosstabs(
        "r",
        "c",
        &items,
        &CrosstabOptions {
            noninteger_weights: NonIntegerWeights::NoAdjust,
            ..Default::default()
        },
    )
    .unwrap();
    assert!((r_na.rows[0].cells[0].observed_f64 - 2.8).abs() < 1e-6);
    assert!((r_na.rows[1].cells[1].observed_f64 - 2.6).abs() < 1e-6);
}

#[test]
fn test_cmh_stratified_2x2xk_oracle() {
    // Classic 2x2x2 example:
    // Stratum 1: [[10, 5], [4, 12]]
    // Stratum 2: [[8, 3], [5, 10]]
    let strata = [[[10.0, 5.0], [4.0, 12.0]], [[8.0, 3.0], [5.0, 10.0]]];
    let cmh = compute_cmh(&strata, 1.0).expect("CMH must succeed");
    // OR_MH = (120/31 + 80/26) / (20/31 + 15/26) = 6.9478908 / 1.2220844 = 5.685279
    assert!((cmh.common_or_estimate - 5.685279).abs() < 1e-4);
    assert_eq!(cmh.cochran_df, 1);
    assert_eq!(cmh.mh_df, 1);
    assert_eq!(cmh.breslow_day_df, 1);
    assert!(cmh.cochran_sig < 0.01);
    assert!(cmh.mh_sig < 0.01);
    assert!(cmh.breslow_day_sig > 0.5); // Homogeneous odds ratios
}
