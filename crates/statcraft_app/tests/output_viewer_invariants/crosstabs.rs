use super::fixtures::sample_crosstabs;
use statcraft_app::views::output_view::{crosstabs_to_tsv, render_output_view, OutputViewerState};

#[test]
fn test_crosstabs_stage3_layered_stats_charts_and_export() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let mut res = sample_crosstabs();
    res.layer_keys = vec![statcraft_algo::CrosstabLayerKey {
        var_name: "dept".to_string(),
        value: "1".to_string(),
        label: Some("Sales".to_string()),
    }];
    res.options.hide_small_counts = true;
    res.options.hide_small_threshold = 15;
    res.options.compare_col_proportions = true;
    res.options.resid_unstandardized = true;
    res.options.resid_standardized = true;
    res.options.resid_adjusted = true;
    res.options.display_clustered_bar_charts = true;
    res.rows[0].cells[0].col_prop_group = Some("a".to_string());
    res.rows[0].cells[1].suppressed = true;
    res.rows[0].cells[1].std_residual = Some(-2.236);
    res.rows[0].cells[1].adj_residual = Some(-2.828);
    res.directional_measures
        .push(statcraft_algo::DirectionalMeasure {
            name: "Lambda".to_string(),
            direction: "Symmetric".to_string(),
            value: 0.25,
            asymp_std_error: Some(0.05),
            approx_t: Some(4.0),
            approx_sig: Some(0.001),
        });
    res.risk_estimates.push(statcraft_algo::RiskEstimate {
        name: "Odds Ratio".to_string(),
        value: 2.5,
        ci_lower: Some(1.2),
        ci_upper: Some(5.2),
    });
    res.mcnemar_sig = Some(0.031);
    res.cmh_result = Some(statcraft_algo::CmhResult {
        cochran_stat: 4.5,
        cochran_df: 1,
        cochran_sig: 0.034,
        mh_stat: 4.1,
        mh_df: 1,
        mh_sig: 0.043,
        breslow_day_stat: 0.8,
        breslow_day_df: 1,
        breslow_day_sig: 0.371,
        tarone_stat: 0.79,
        tarone_df: 1,
        tarone_sig: 0.374,
        common_or_estimate: 2.1,
        common_or_ln_se: 0.35,
        common_or_ci_lower: 1.05,
        common_or_ci_upper: 4.18,
        common_or_test_val: 2.12,
        common_or_sig: 0.034,
    });

    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();
    let opts = res.options.clone();
    state.add_crosstabs(vec![res.clone()], opts);

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = crosstabs_to_tsv(&res);
    assert!(tsv.contains("dept = Sales"));
    assert!(tsv.contains("<15"));
    assert!(tsv.contains("40 (a)"));
    assert!(tsv.contains("Directional Measures"));
    assert!(tsv.contains("Risk Estimate"));
    assert!(tsv.contains("McNemar Test"));
    assert!(tsv.contains("Cochran's and Mantel-Haenszel Statistics"));

    let mut suppressed_res = res.clone();
    suppressed_res.options.suppress_tables = true;
    let suppressed_tsv = crosstabs_to_tsv(&suppressed_res);
    assert!(!suppressed_tsv.contains("gender * jobcat Crosstabulation"));
    assert!(suppressed_tsv.contains("Chi-Square Tests"));
}
