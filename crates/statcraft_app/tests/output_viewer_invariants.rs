#[allow(unused_imports)]
use statcraft_algo::{
    compute_bivariate_correlations, AnovaTableRow, AssociationMeasure,
    BivariateCorrelationsOptions, BivariateCorrelationsResult, ChiSquareTestRow, CrosstabCategory,
    CrosstabCell, CrosstabOptions, CrosstabResult, CrosstabRow, DescriptivesResult,
    FrequencyCategory, FrequencyResult, GroupStats, IndepTTestResult, KruskalWallisGroupStats,
    KruskalWallisResult, LeveneResult, LinearRegressionOptions, LinearRegressionResult,
    MannWhitneyResult, OneSampleTTestResult, OneWayAnovaResult, PairedCorrelation,
    PairedDifferences, PairedTTestResult, PairedVarStats, RankGroupStats, RegressionAnova,
    RegressionAnovaRow, RegressionCoefficient, RelatedPairTestResult, ReliabilityModel,
    ReliabilityResult, SignTestDifferences, SignTestResult, TTestTwoGroupDetails,
    WilcoxonRanksSummary, WilcoxonSignedRankResult,
};
use statcraft_app::views::output_view::{
    anova_to_tsv, correlations_to_tsv, crosstabs_to_tsv, descriptives_to_tsv, frequencies_to_tsv,
    indep_ttest_to_tsv, kruskal_wallis_to_tsv, mann_whitney_to_tsv, onesample_ttest_to_tsv,
    paired_ttest_to_tsv, regression_to_tsv, related_samples_to_tsv, reliability_to_tsv,
    render_output_view, OutputItem, OutputItemKind, OutputNotes, OutputProcedure,
    OutputViewerState, SelectedTarget,
};

#[path = "output_viewer_invariants/fixtures.rs"]
mod fixtures;
use fixtures::*;

fn sample_correlations() -> BivariateCorrelationsResult {
    let opts = BivariateCorrelationsOptions {
        descriptives: true,
        cross_prod_cov: true,
        ..Default::default()
    };
    compute_bivariate_correlations(
        &[
            ("salary", &[Some(50000.0), Some(60000.0), Some(70000.0)]),
            ("educ", &[Some(12.0), Some(14.0), Some(16.0)]),
        ],
        &opts,
    )
    .unwrap()
}

#[test]
fn test_continuous_history_accumulation() {
    let mut state = OutputViewerState::new();
    assert_eq!(state.procedures.len(), 0);

    // 1. Descriptives
    state.add_descriptives(vec![sample_descriptives()]);
    assert_eq!(state.procedures.len(), 1);
    assert_eq!(state.procedures[0].title_key, "output-descriptives-title");

    // 2. Frequencies
    state.add_frequencies(vec![sample_frequencies()]);
    assert_eq!(state.procedures.len(), 2);

    // 3. One-Sample T-Test
    state.add_onesample_ttest(vec![sample_onesample()], 45000.0, 0.95);
    assert_eq!(state.procedures.len(), 3);

    // 4. Independent-Samples T-Test
    state.add_indep_ttest(vec![sample_indep()], "gender", "Male", "Female", 0.95);
    assert_eq!(state.procedures.len(), 4);

    // 5. Paired-Samples T-Test
    state.add_paired_ttest(
        vec![sample_paired()],
        &[("pre_test".to_string(), "post_test".to_string())],
        0.95,
    );
    assert_eq!(state.procedures.len(), 5);

    // 6. One-Way ANOVA
    state.add_oneway_anova(vec![sample_anova()], 0.95);
    assert_eq!(state.procedures.len(), 6);

    // 7. Crosstabs
    let sample_c = sample_crosstabs();
    let opts = sample_c.options.clone();
    state.add_crosstabs(vec![sample_c], opts);
    assert_eq!(state.procedures.len(), 7);

    // Invariant: IDs strictly increasing
    for i in 0..6 {
        assert!(
            state.procedures[i].id < state.procedures[i + 1].id,
            "Procedure IDs must be strictly monotonic"
        );
    }

    // Invariant: History is accumulated, older entries not overwritten
    let proc0 = &state.procedures[0];
    assert_eq!(proc0.title_key, "output-descriptives-title");
    assert_eq!(proc0.items.len(), 3);
    assert!(matches!(&proc0.items[0].kind, OutputItemKind::Log(_)));
    assert!(matches!(&proc0.items[1].kind, OutputItemKind::Notes(_)));
    assert!(matches!(
        &proc0.items[2].kind,
        OutputItemKind::DescriptivesTable(_)
    ));

    let proc3 = &state.procedures[3];
    assert_eq!(proc3.title_key, "output-ttest-indep-title");
    assert_eq!(proc3.items.len(), 3);
    assert!(matches!(
        &proc3.items[2].kind,
        OutputItemKind::IndepTTest { .. }
    ));
}

#[test]
fn test_exhaustive_enum_payloads_render_without_panics() {
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    // Edge cases and standard cases across ALL OutputItemKind variants
    let items = vec![
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-log",
            custom_title: None,
            kind: OutputItemKind::Log("DESCRIPTIVES /VARIABLES=x.".to_string()),
        },
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-notes",
            custom_title: None,
            kind: OutputItemKind::Notes(OutputNotes {
                dataset_name: "TestDS".to_string(),
                timestamp: "2026-10-28".to_string(),
                active_cases: 0,
                active_vars: 0,
            }),
        },
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-dataset-summary",
            custom_title: None,
            kind: OutputItemKind::DatasetSummary {
                name: "empty.csv".to_string(),
                rows: 0,
                cols: 0,
            },
        },
        // Descriptives with None stats (edge case)
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-descriptives-table",
            custom_title: None,
            kind: OutputItemKind::DescriptivesTable(vec![DescriptivesResult {
                variable: "empty_var".to_string(),
                n_valid: 0,
                n_missing: 10,
                mean: None,
                std_dev: None,
                variance: None,
                min: None,
                max: None,
                range: None,
                sum: None,
                skewness: None,
                se_skewness: None,
                kurtosis: None,
                se_kurtosis: None,
                se_mean: None,
            }]),
        },
        // Frequencies with empty categories
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-frequencies-table",
            custom_title: None,
            kind: OutputItemKind::FrequenciesTables(vec![FrequencyResult {
                variable: "missing_var".to_string(),
                n_valid: 0,
                n_missing: 5,
                categories: vec![],
                quartiles: None,
            }]),
        },
        // One-Sample T-Test with 0 df and None stats
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-ttest-onesample-title",
            custom_title: None,
            kind: OutputItemKind::OneSampleTTest {
                results: vec![OneSampleTTestResult {
                    variable: "single".to_string(),
                    n_valid: 1,
                    n_missing: 0,
                    mean: Some(42.0),
                    std_dev: None,
                    se_mean: None,
                    test_value: 0.0,
                    t_stat: None,
                    df: None,
                    sig_2tailed: None,
                    mean_diff: Some(42.0),
                    ci_level: 0.95,
                    ci_lower: None,
                    ci_upper: None,
                }],
                test_value: 0.0,
                ci: 0.95,
            },
        },
        // Indep T-Test with empty results
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-ttest-indep-title",
            custom_title: None,
            kind: OutputItemKind::IndepTTest {
                results: vec![],
                grouping_var: "group".to_string(),
                group1: "A".to_string(),
                group2: "B".to_string(),
                ci: 0.95,
            },
        },
        // Paired T-Test with zero variance / None t-stat
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-ttest-paired-title",
            custom_title: None,
            kind: OutputItemKind::PairedTTest {
                results: vec![PairedTTestResult {
                    pair_num: 1,
                    var1_stats: PairedVarStats {
                        var_name: "v1".to_string(),
                        n: 2,
                        mean: Some(10.0),
                        std_dev: Some(0.0),
                        se_mean: Some(0.0),
                    },
                    var2_stats: PairedVarStats {
                        var_name: "v2".to_string(),
                        n: 2,
                        mean: Some(10.0),
                        std_dev: Some(0.0),
                        se_mean: Some(0.0),
                    },
                    correlation: PairedCorrelation {
                        var1: "v1".to_string(),
                        var2: "v2".to_string(),
                        n: 2,
                        correlation: None,
                        sig: None,
                    },
                    differences: PairedDifferences {
                        mean_diff: Some(0.0),
                        std_dev_diff: Some(0.0),
                        se_diff: Some(0.0),
                        ci_lower: Some(0.0),
                        ci_upper: Some(0.0),
                        t_stat: None,
                        df: Some(1.0),
                        sig_2tailed: None,
                    },
                    confidence_level: 0.95,
                }],
                pairs: vec![("v1".to_string(), "v2".to_string())],
                ci: 0.95,
            },
        },
        // One-Way ANOVA with empty results
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-anova-title",
            custom_title: None,
            kind: OutputItemKind::OneWayAnova {
                results: vec![],
                ci: 0.95,
            },
        },
        // Warning
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-log",
            custom_title: Some("Warning".to_string()),
            kind: OutputItemKind::Warning("Empty dataset provided".to_string()),
        },
        // Custom Text
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-log",
            custom_title: Some("Log Output".to_string()),
            kind: OutputItemKind::CustomText {
                title: "Custom Title".to_string(),
                content: "Detailed syntax execution log".to_string(),
            },
        },
        // Correlations
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-correlations-title",
            custom_title: None,
            kind: OutputItemKind::Correlations(sample_correlations()),
        },
        // Two Related Samples
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-npar-related-title",
            custom_title: None,
            kind: OutputItemKind::TwoRelatedSamples(vec![]),
        },
        // Kruskal-Wallis
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-npar-kruskal-title",
            custom_title: None,
            kind: OutputItemKind::KruskalWallis {
                results: vec![],
                grouping_var: "group".to_string(),
            },
        },
    ];

    let proc_id = state.alloc_id();
    state.procedures.push(OutputProcedure {
        id: proc_id,
        title_key: "output-descriptives-title",
        custom_title: None,
        is_expanded: true,
        items,
    });

    // Run headless rendering in egui context
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    // Asserts: 0 panics, state remains intact
    assert_eq!(state.procedures.len(), 1);
    assert_eq!(state.procedures[0].items.len(), 14);
}

#[test]
fn test_tsv_export_golden_headers() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    // 1. Descriptives
    let desc_results = vec![sample_descriptives()];
    let desc_tsv = descriptives_to_tsv(&desc_results);
    assert!(
        desc_tsv.contains("Mean"),
        "Descriptives TSV must contain 'Mean':\n{}",
        desc_tsv
    );
    assert!(
        desc_tsv.contains("Std. Deviation"),
        "Descriptives TSV must contain 'Std. Deviation':\n{}",
        desc_tsv
    );
    assert!(
        desc_tsv.contains("Valid N (listwise)"),
        "Descriptives TSV must contain 'Valid N (listwise)':\n{}",
        desc_tsv
    );

    // 2. Frequencies
    let freq_results = vec![sample_frequencies()];
    let freq_tsv = frequencies_to_tsv(&freq_results);
    assert!(
        freq_tsv.contains("Valid Percent"),
        "Frequencies TSV must contain 'Valid Percent':\n{}",
        freq_tsv
    );
    assert!(
        freq_tsv.contains("Cumulative Percent"),
        "Frequencies TSV must contain 'Cumulative Percent':\n{}",
        freq_tsv
    );
    assert!(
        freq_tsv.contains("Total"),
        "Frequencies TSV must contain 'Total':\n{}",
        freq_tsv
    );

    // 3. One-Sample T-Test
    let ost_results = vec![sample_onesample()];
    let ost_tsv = onesample_ttest_to_tsv(&ost_results);
    assert!(ost_tsv.contains("One-Sample Statistics"));
    assert!(ost_tsv.contains("One-Sample Test"));
    assert!(ost_tsv.contains("Variable"));
    assert!(ost_tsv.contains("Sig. (2-tailed)"));

    // 4. Independent-Samples T-Test
    let indep_results = vec![sample_indep()];
    let indep_tsv = indep_ttest_to_tsv(&indep_results);
    assert!(indep_tsv.contains("Group Statistics"));
    assert!(indep_tsv.contains("Independent Samples Test"));
    assert!(
        indep_tsv.contains("Levene"),
        "Indep T-Test TSV must contain 'Levene':\n{}",
        indep_tsv
    );
    assert!(
        indep_tsv.contains("Equal variances assumed"),
        "Indep T-Test TSV must contain 'Equal variances assumed':\n{}",
        indep_tsv
    );
    assert!(
        indep_tsv.contains("Equal variances not assumed"),
        "Indep T-Test TSV must contain 'Equal variances not assumed':\n{}",
        indep_tsv
    );

    // 5. Paired-Samples T-Test
    let paired_results = vec![sample_paired()];
    let paired_tsv = paired_ttest_to_tsv(&paired_results);
    assert!(paired_tsv.contains("Paired Samples Statistics"));
    assert!(paired_tsv.contains("Paired Samples Correlations"));
    assert!(paired_tsv.contains("Paired Samples Test"));
    assert!(
        paired_tsv.contains("Paired Differences"),
        "Paired T-Test TSV must contain 'Paired Differences':\n{}",
        paired_tsv
    );
    assert!(
        paired_tsv.contains("Correlation"),
        "Paired T-Test TSV must contain 'Correlation':\n{}",
        paired_tsv
    );
    assert!(
        paired_tsv.contains("Sig."),
        "Paired T-Test TSV must contain 'Sig.':\n{}",
        paired_tsv
    );

    // 6. One-Way ANOVA
    let anova_results = vec![sample_anova()];
    let anova_tsv = anova_to_tsv(&anova_results);
    assert!(
        anova_tsv.contains("Between Groups"),
        "ANOVA TSV must contain 'Between Groups':\n{}",
        anova_tsv
    );
    assert!(
        anova_tsv.contains("Within Groups"),
        "ANOVA TSV must contain 'Within Groups':\n{}",
        anova_tsv
    );
    assert!(
        anova_tsv.contains("Sum of Squares"),
        "ANOVA TSV must contain 'Sum of Squares':\n{}",
        anova_tsv
    );
    assert!(
        anova_tsv.contains("Mean Square"),
        "ANOVA TSV must contain 'Mean Square':\n{}",
        anova_tsv
    );
    assert!(
        anova_tsv.contains("F"),
        "ANOVA TSV must contain 'F':\n{}",
        anova_tsv
    );
    assert!(
        anova_tsv.contains("Sig."),
        "ANOVA TSV must contain 'Sig.':\n{}",
        anova_tsv
    );

    // 7. Crosstabs
    let sample_c = sample_crosstabs();
    let crosstabs_tsv = crosstabs_to_tsv(&sample_c);
    assert!(
        crosstabs_tsv.contains("gender * jobcat Crosstabulation"),
        "Crosstabs TSV must contain table header:\n{}",
        crosstabs_tsv
    );
    assert!(
        crosstabs_tsv.contains("Count"),
        "Crosstabs TSV must contain 'Count':\n{}",
        crosstabs_tsv
    );
    assert!(
        crosstabs_tsv.contains("Pearson Chi-Square"),
        "Crosstabs TSV must contain 'Pearson Chi-Square':\n{}",
        crosstabs_tsv
    );
    assert!(
        crosstabs_tsv.contains("Phi"),
        "Crosstabs TSV must contain 'Phi':\n{}",
        crosstabs_tsv
    );

    // 8. Correlations
    let sample_corr = sample_correlations();
    let corr_tsv = correlations_to_tsv(&sample_corr);
    assert!(
        corr_tsv.contains("Correlations (Pearson)"),
        "Correlations TSV must contain table header:\n{}",
        corr_tsv
    );
    assert!(
        corr_tsv.contains("salary"),
        "Correlations TSV must contain variable 'salary':\n{}",
        corr_tsv
    );
    assert!(
        corr_tsv.contains("Correlation"),
        "Correlations TSV must contain 'Correlation':\n{}",
        corr_tsv
    );
}

#[test]
fn test_outline_tree_navigation_integrity() {
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    // Add 2 procedures
    state.add_entry("Log 1".to_string(), "Syntax 1".to_string());
    state.add_entry("Log 2".to_string(), "Syntax 2".to_string());
    assert_eq!(state.procedures.len(), 2);

    // Select second procedure
    let target = SelectedTarget::Procedure(1);
    state.selected_target = Some(target);
    state.scroll_to_target = Some(target);

    // Headless render
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    // Invariant: scroll_to_target consumed, selected_target retained
    assert_eq!(state.scroll_to_target, None);
    assert_eq!(state.selected_target, Some(target));

    // Invariant: Deletion integrity
    state.delete_selected();
    assert_eq!(
        state.procedures.len(),
        1,
        "Targeted procedure should be deleted"
    );
    assert_eq!(state.procedures[0].items[0].title(), "Log 1");
}

#[test]
fn test_mann_whitney_output_and_tsv_integrity() {
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    let sample_mw = MannWhitneyResult {
        variable: "salary".to_string(),
        grouping_var: "gender".to_string(),
        group1_stats: RankGroupStats {
            group_val: "1".to_string(),
            n: 5,
            mean_rank: 4.0,
            sum_ranks: 20.0,
        },
        group2_stats: RankGroupStats {
            group_val: "2".to_string(),
            n: 5,
            mean_rank: 7.0,
            sum_ranks: 35.0,
        },
        total_n: 10,
        mann_whitney_u: 5.0,
        wilcoxon_w: 20.0,
        z_stat: -1.59598,
        asymp_sig_2tailed: 0.11049,
    };

    state.add_mann_whitney(vec![sample_mw.clone()], "gender", "1", "2");
    assert_eq!(state.procedures.len(), 1);
    assert_eq!(state.procedures[0].items.len(), 3);

    // Headless render
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = mann_whitney_to_tsv(&[sample_mw], "gender");
    assert!(tsv.contains("Ranks\n"));
    assert!(tsv.contains("salary\t1\t5\t4.00\t20.00\n"));
    assert!(tsv.contains("Test Statistics\n"));
    assert!(tsv.contains("Mann-Whitney U\t5.0000\n"));
    assert!(tsv.contains("Wilcoxon W\t20.0000\n"));
    assert!(tsv.contains("a. Grouping Variable: gender\n"));
}

#[test]
fn test_related_samples_output_and_tsv_integrity() {
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    let sample_wilcoxon = WilcoxonSignedRankResult {
        var1: "pre".to_string(),
        var2: "post".to_string(),
        ranks: WilcoxonRanksSummary {
            negative_ranks: RankGroupStats {
                group_val: "Negative Ranks".to_string(),
                n: 1,
                mean_rank: 2.5,
                sum_ranks: 2.5,
            },
            positive_ranks: RankGroupStats {
                group_val: "Positive Ranks".to_string(),
                n: 3,
                mean_rank: 2.5,
                sum_ranks: 7.5,
            },
            ties_count: 1,
            total_n: 5,
        },
        test_statistic_w: 2.5,
        z_stat: -0.80,
        asymp_sig_2tailed: 0.4237,
        based_on_negative: true,
    };

    let sample_sign = SignTestResult {
        var1: "pre".to_string(),
        var2: "post".to_string(),
        diffs: SignTestDifferences {
            negative_count: 1,
            positive_count: 3,
            ties_count: 1,
            total_n: 5,
        },
        exact_sig_2tailed: 0.625,
    };

    let sample_res = RelatedPairTestResult {
        var1: "pre".to_string(),
        var2: "post".to_string(),
        wilcoxon: Some(sample_wilcoxon),
        sign: Some(sample_sign),
    };

    let pairs = vec![("pre".to_string(), "post".to_string())];
    state.add_related_samples(vec![sample_res.clone()], &pairs, true, true);
    assert_eq!(state.procedures.len(), 1);
    assert_eq!(state.procedures[0].items.len(), 3);

    // Headless render
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = related_samples_to_tsv(&[sample_res]);
    assert!(tsv.contains("Ranks\n"));
    assert!(tsv.contains("post - pre\tNegative Ranks\t1\t2.50\t2.50\n"));
    assert!(tsv.contains("Test Statistics (Wilcoxon Signed Ranks)\n"));
    assert!(tsv.contains("Z\t-0.800\n"));
    assert!(tsv.contains("Frequencies (Sign Test)\n"));
    assert!(tsv.contains("Exact Sig. (2-tailed)\t0.625\n"));
}

#[test]
fn test_kruskal_wallis_output_and_tsv_integrity() {
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    let sample_kw = KruskalWallisResult {
        variable: "salary".to_string(),
        grouping_var: "jobcat".to_string(),
        groups: vec![
            KruskalWallisGroupStats {
                group_val: "1".to_string(),
                n: 5,
                mean_rank: 4.2,
                sum_ranks: 21.0,
            },
            KruskalWallisGroupStats {
                group_val: "2".to_string(),
                n: 5,
                mean_rank: 8.5,
                sum_ranks: 42.5,
            },
            KruskalWallisGroupStats {
                group_val: "3".to_string(),
                n: 5,
                mean_rank: 14.8,
                sum_ranks: 74.0,
            },
        ],
        total_n: 15,
        kruskal_wallis_h: 8.2998,
        df: 2,
        asymp_sig: 0.0158,
    };

    state.add_kruskal_wallis(vec![sample_kw.clone()], "jobcat", 1.0, 3.0);
    assert_eq!(state.procedures.len(), 1);
    assert_eq!(state.procedures[0].items.len(), 3);

    // Headless render
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = kruskal_wallis_to_tsv(&[sample_kw], "jobcat");
    assert!(tsv.contains("Ranks\n"));
    assert!(tsv.contains("salary\t1\t5\t4.20\n"));
    assert!(tsv.contains("\t2\t5\t8.50\n"));
    assert!(tsv.contains("\t3\t5\t14.80\n"));
    assert!(tsv.contains("Test Statistics\n"));
    assert!(tsv.contains("Kruskal-Wallis H\t8.2998\n"));
    assert!(tsv.contains("df\t2\n"));
}

#[test]
fn test_output_viewer_linear_regression_item_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    let sample_reg = LinearRegressionResult {
        dep_var: "salary".to_string(),
        indep_vars: vec!["educ".to_string(), "prevexp".to_string()],
        method: "Enter".to_string(),
        n_valid: 10,
        n_missing: 0,
        r: 0.9974,
        r_squared: 0.9948,
        adj_r_squared: 0.9933,
        std_error_estimate: 0.8268,
        anova: RegressionAnova {
            regression: RegressionAnovaRow {
                sum_of_squares: 916.81,
                df: 2,
                mean_square: Some(458.41),
                f_stat: Some(670.57),
                p_value: Some(0.00001),
            },
            residual: RegressionAnovaRow {
                sum_of_squares: 4.79,
                df: 7,
                mean_square: Some(0.68),
                f_stat: None,
                p_value: None,
            },
            total: RegressionAnovaRow {
                sum_of_squares: 921.60,
                df: 9,
                mean_square: None,
                f_stat: None,
                p_value: None,
            },
        },
        coefficients: vec![
            RegressionCoefficient {
                var_name: "(Constant)".to_string(),
                b: 8.8223,
                std_error: 2.1297,
                beta: None,
                t_stat: 4.1425,
                p_value: 0.0043,
                ci_lower: 3.7864,
                ci_upper: 13.8582,
                tolerance: None,
                vif: None,
            },
            RegressionCoefficient {
                var_name: "educ".to_string(),
                b: 3.9500,
                std_error: 1.3283,
                beta: Some(1.1818),
                t_stat: 2.9738,
                p_value: 0.0207,
                ci_lower: 0.8092,
                ci_upper: 7.0908,
                tolerance: Some(0.0047),
                vif: Some(212.9155),
            },
        ],
        options: LinearRegressionOptions::default(),
    };

    state.add_linear_regression(sample_reg.clone());
    assert_eq!(state.procedures.len(), 1);
    assert_eq!(state.procedures[0].items.len(), 3);

    // Headless render
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = regression_to_tsv(&sample_reg);
    assert!(tsv.contains("Model Summary"));
    assert!(tsv.contains("ANOVA"));
    assert!(tsv.contains("Coefficients"));
    assert!(tsv.contains("0.9974"));
    assert!(tsv.contains("916.8100"));
}

#[test]
fn test_reliability_output_and_tsv_integrity() {
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();

    let sample_rel = ReliabilityResult {
        scale_name: "SCALE_1".to_string(),
        model: ReliabilityModel::Alpha,
        case_processing: statcraft_algo::CaseProcessingSummary {
            n_valid: 20,
            percent_valid: 80.0,
            n_excluded: 5,
            percent_excluded: 20.0,
            n_total: 25,
            percent_total: 100.0,
        },
        reliability_stats: statcraft_algo::ReliabilityStatistics {
            cronbach_alpha: Some(0.854),
            cronbach_alpha_standardized: Some(0.862),
            n_of_items: 3,
        },
        item_stats: vec![
            statcraft_algo::ItemStatistic {
                item_name: "q1".to_string(),
                mean: 3.5,
                std_dev: 1.1,
                n: 20,
            },
            statcraft_algo::ItemStatistic {
                item_name: "q2".to_string(),
                mean: 3.8,
                std_dev: 1.0,
                n: 20,
            },
        ],
        item_total_stats: vec![statcraft_algo::ItemTotalStatistic {
            item_name: "q1".to_string(),
            scale_mean_if_deleted: 3.8,
            scale_variance_if_deleted: 1.0,
            corrected_item_total_correlation: Some(0.72),
            cronbach_alpha_if_deleted: Some(0.81),
        }],
        scale_stats: Some(statcraft_algo::ScaleStatistics {
            mean: 7.3,
            variance: 3.5,
            std_dev: 1.87,
            n_of_items: 2,
        }),
    };

    state.add_reliability(sample_rel.clone(), vec!["q1".to_string(), "q2".to_string()]);
    assert_eq!(state.procedures.len(), 1);
    assert_eq!(state.procedures[0].items.len(), 3);

    // Headless render
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    let tsv = reliability_to_tsv(&sample_rel);
    assert!(tsv.contains("Case Processing Summary"));
    assert!(tsv.contains("Reliability Statistics"));
    assert!(tsv.contains("Item Statistics"));
    assert!(tsv.contains("Item-Total Statistics"));
    assert!(tsv.contains("Scale Statistics"));
    assert!(tsv.contains("0.854"));
    assert!(tsv.contains("0.862"));
}
