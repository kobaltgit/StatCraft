use statcraft_algo::{DescriptivesOptions, DescriptivesResult};
use statcraft_app::views::output_view::{
    descriptives_to_tsv, export_output_to_html, export_output_to_txt, render_output_view,
    OutputItemKind, OutputViewerState,
};

fn make_full_descriptives() -> DescriptivesResult {
    DescriptivesResult {
        variable: "salary".to_string(),
        n_valid: 100,
        n_missing: 0,
        mean: Some(50000.0),
        se_mean: Some(1200.0),
        std_dev: Some(12000.0),
        variance: Some(144000000.0),
        min: Some(25000.0),
        max: Some(85000.0),
        range: Some(60000.0),
        sum: Some(5000000.0),
        skewness: Some(0.4),
        se_skewness: Some(0.24),
        kurtosis: Some(-0.2),
        se_kurtosis: Some(0.48),
        bootstrap: None,
        listwise_n: Some(92),
        applied_options: Some(DescriptivesOptions::all()),
    }
}

#[test]
fn test_descriptives_dynamic_columns_listwise_n_and_reporting() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    // 1. Only Mean + Sum selected via applied_options
    let opts_mean_sum = DescriptivesOptions {
        mean: true,
        sum: true,
        ..Default::default()
    };
    let res_mean_sum = DescriptivesResult {
        variable: "age".to_string(),
        n_valid: 10,
        n_missing: 2,
        mean: Some(30.5),
        se_mean: None,
        std_dev: None,
        variance: None,
        min: None,
        max: None,
        range: None,
        sum: Some(305.0),
        skewness: None,
        se_skewness: None,
        kurtosis: None,
        se_kurtosis: None,
        bootstrap: None,
        listwise_n: Some(8),
        applied_options: Some(opts_mean_sum),
    };

    let tsv_partial = descriptives_to_tsv(std::slice::from_ref(&res_mean_sum));
    let header_line = tsv_partial.lines().next().unwrap();
    assert_eq!(header_line, "Variable\tN\tSum\tMean");
    assert!(tsv_partial.contains("Valid N (listwise)\t8"));
    assert!(!tsv_partial.contains("Minimum"));
    assert!(!tsv_partial.contains("Std. Deviation"));

    // 2. All statistics selected, including Range, S.E. Mean, SE Skewness, SE Kurtosis
    let res_all = make_full_descriptives();

    let tsv_all = descriptives_to_tsv(std::slice::from_ref(&res_all));
    let header_all = tsv_all.lines().next().unwrap();
    assert_eq!(
        header_all,
        "Variable\tN\tRange\tMinimum\tMaximum\tSum\tMean\tS.E. Mean\tStd. Deviation\tVariance\tSkewness\tSE Skewness\tKurtosis\tSE Kurtosis"
    );
    assert!(tsv_all.contains("Valid N (listwise)\t92"));

    // 3. Headless render + HTML/TXT/TSV export round-trip
    let mut state = OutputViewerState::new();
    let custom_log = "DESCRIPTIVES VARIABLES=salary\n  /STATISTICS=MEAN SUM RANGE.".to_string();
    state.add_descriptives_with_log(vec![res_all], custom_log.clone(), None, false);

    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    if let OutputItemKind::Log(log) = &state.procedures[0].items[0].kind {
        assert_eq!(log, &custom_log);
    } else {
        panic!("Expected Log item");
    }

    let html = export_output_to_html(&state);
    assert!(html.contains("S.E. Mean"));
    assert!(html.contains("SE Skewness"));
    assert!(html.contains("SE Kurtosis"));
    assert!(html.contains("Valid N (listwise)"));
    assert!(html.contains("92"));

    let txt = export_output_to_txt(&state);
    assert!(txt.contains("Range"));
    assert!(txt.contains("Valid N (listwise)"));
}
