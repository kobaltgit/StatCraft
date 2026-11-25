use serde_json::json;
use statcraft_algo::{
    BivariateCorrelationsResult, DescriptivesResult, IndepTTestResult, LinearRegressionResult,
    MeansOptions, MeansProcedureResult, OneSampleTTestResult, OneWayAnovaResult, PairedTTestResult,
};
use statcraft_app::views::output_view::{
    anova_to_tsv, correlations_to_tsv, descriptives_to_tsv, indep_ttest_to_tsv, means_to_tsv,
    onesample_ttest_to_tsv, paired_ttest_to_tsv, regression_to_tsv, render_output_view,
    OutputViewerState,
};
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};
use statcraft_syntax::commands::{
    CorrelationsCommand, DescriptivesCommand, IndependentSamplesTTestCommand,
    LinearRegressionCommand, MeansCommand, OneSampleTTestCommand, OneWayAnovaCommand,
    PairedSamplesTTestCommand,
};
use statcraft_syntax::EngineCommand;

fn sample_dataset() -> Option<DataSet> {
    let csv = concat!(
        "x,y,z,grp\n",
        "10,12,100,1\n",
        "12,15,110,1\n",
        "14,16,120,1\n",
        "16,19,130,1\n",
        "18,21,140,2\n",
        "20,24,150,2\n",
        "22,25,160,2\n",
        "24,29,170,2\n",
    );
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).expect("parse csv");
    Some(DataSet::from_raw_table(table).expect("create dataset"))
}

fn boot_json() -> serde_json::Value {
    json!({
        "enabled": true,
        "samples": 40,
        "seed": 2_000_000,
        "ci_level": 95.0,
        "ci_method": "percentile",
        "sampling_method": { "method": "simple" }
    })
}

fn render_headless(state: &mut OutputViewerState) {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, state);
        });
    });
}

fn assert_boot_tsv(tsv: &str, proc_name: &str) {
    assert!(
        tsv.contains("Bootstrap Specifications"),
        "{proc_name} TSV missing Bootstrap Specifications"
    );
    assert!(
        tsv.contains("95% CI Lower"),
        "{proc_name} TSV missing 95% CI Lower"
    );
}

#[test]
fn test_stage3_procedures_bootstrap_output_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let mut ds = sample_dataset();
    let mut state = OutputViewerState::new();

    // 1. Descriptives
    let desc_val = DescriptivesCommand::new()
        .execute(&mut ds, json!({ "vars": ["x"], "bootstrap": boot_json() }))
        .expect("descriptives");
    let desc_res: Vec<DescriptivesResult> = serde_json::from_value(desc_val).unwrap();
    assert!(desc_res[0].bootstrap.is_some());
    assert_boot_tsv(&descriptives_to_tsv(&desc_res), "Descriptives");
    state.add_descriptives(desc_res);

    // 2. Means
    let means_val = MeansCommand
        .execute(
            &mut ds,
            json!({ "dependent_vars": ["x"], "factor_vars": ["grp"], "bootstrap": boot_json() }),
        )
        .expect("means");
    let means_res: MeansProcedureResult = serde_json::from_value(means_val).unwrap();
    assert!(means_res.tables[0].bootstrap.is_some());
    assert_boot_tsv(&means_to_tsv(&means_res), "Means");
    state.add_means(
        means_res,
        vec!["x".to_string()],
        vec!["grp".to_string()],
        &MeansOptions::default(),
    );

    // 3. One-Sample T-Test
    let os_val = OneSampleTTestCommand
        .execute(
            &mut ds,
            json!({ "vars": ["x"], "test_value": 15.0, "bootstrap": boot_json() }),
        )
        .expect("onesample ttest");
    let os_res: Vec<OneSampleTTestResult> = serde_json::from_value(os_val).unwrap();
    assert!(os_res[0].bootstrap.is_some());
    assert_boot_tsv(&onesample_ttest_to_tsv(&os_res), "OneSampleTTest");
    state.add_onesample_ttest(os_res, 15.0, 0.95);

    // 4. Independent-Samples T-Test
    let ind_val = IndependentSamplesTTestCommand
        .execute(
            &mut ds,
            json!({ "vars": ["x"], "grouping_var": "grp", "group1": "1", "group2": "2", "bootstrap": boot_json() }),
        )
        .expect("indep ttest");
    let ind_res: Vec<IndepTTestResult> = serde_json::from_value(ind_val).unwrap();
    assert!(ind_res[0].bootstrap.is_some());
    assert_boot_tsv(&indep_ttest_to_tsv(&ind_res), "IndepTTest");
    state.add_indep_ttest(ind_res, "grp", "1", "2", 0.95);

    // 5. Paired-Samples T-Test
    let pair_val = PairedSamplesTTestCommand
        .execute(
            &mut ds,
            json!({ "pairs": [["x", "y"]], "bootstrap": boot_json() }),
        )
        .expect("paired ttest");
    let pair_res: Vec<PairedTTestResult> = serde_json::from_value(pair_val).unwrap();
    assert!(pair_res[0].bootstrap.is_some());
    assert_boot_tsv(&paired_ttest_to_tsv(&pair_res), "PairedTTest");
    state.add_paired_ttest(pair_res, &[("x".to_string(), "y".to_string())], 0.95);

    // 6. One-Way ANOVA
    let anova_val = OneWayAnovaCommand
        .execute(
            &mut ds,
            json!({ "dependent_vars": ["x"], "factor_var": "grp", "bootstrap": boot_json() }),
        )
        .expect("oneway anova");
    let anova_res: Vec<OneWayAnovaResult> = serde_json::from_value(anova_val).unwrap();
    assert!(anova_res[0].bootstrap.is_some());
    assert_boot_tsv(&anova_to_tsv(&anova_res), "OneWayAnova");
    state.add_oneway_anova(anova_res, 0.95);

    // 7. Bivariate Correlations
    let corr_val = CorrelationsCommand
        .execute(
            &mut ds,
            json!({ "vars": ["x", "y"], "bootstrap": boot_json() }),
        )
        .expect("correlations");
    let corr_res: BivariateCorrelationsResult = serde_json::from_value(corr_val).unwrap();
    assert!(corr_res.bootstrap.is_some());
    assert_boot_tsv(&correlations_to_tsv(&corr_res), "Correlations");
    state.add_correlations(corr_res, &["x".to_string(), "y".to_string()]);

    // 8. Linear Regression
    let reg_val = LinearRegressionCommand
        .execute(
            &mut ds,
            json!({ "dependent_var": "y", "independent_vars": ["x", "z"], "bootstrap": boot_json() }),
        )
        .expect("linear regression");
    let reg_res: LinearRegressionResult = serde_json::from_value(reg_val).unwrap();
    assert!(reg_res.bootstrap.is_some());
    assert_boot_tsv(&regression_to_tsv(&reg_res), "LinearRegression");
    state.add_linear_regression(reg_res);

    render_headless(&mut state);
}
