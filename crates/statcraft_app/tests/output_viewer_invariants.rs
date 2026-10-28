use statcraft_algo::{
    AnovaTableRow, DescriptivesResult, FrequencyCategory, FrequencyResult, GroupStats,
    IndepTTestResult, LeveneResult, OneSampleTTestResult, OneWayAnovaResult, PairedCorrelation,
    PairedDifferences, PairedTTestResult, PairedVarStats, TTestTwoGroupDetails,
};
use statcraft_app::views::output_view::{
    anova_to_tsv, descriptives_to_tsv, frequencies_to_tsv, indep_ttest_to_tsv,
    onesample_ttest_to_tsv, paired_ttest_to_tsv, render_output_view, OutputItem, OutputItemKind,
    OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};

fn sample_descriptives() -> DescriptivesResult {
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
    }
}

fn sample_frequencies() -> FrequencyResult {
    FrequencyResult {
        variable: "gender".to_string(),
        n_valid: 100,
        n_missing: 0,
        categories: vec![
            FrequencyCategory {
                value: "1".to_string(),
                label: Some("Male".to_string()),
                count: 45,
                percent: 45.0,
                valid_percent: Some(45.0),
                cumulative_percent: Some(45.0),
            },
            FrequencyCategory {
                value: "2".to_string(),
                label: Some("Female".to_string()),
                count: 55,
                percent: 55.0,
                valid_percent: Some(55.0),
                cumulative_percent: Some(100.0),
            },
        ],
        quartiles: None,
    }
}

fn sample_onesample() -> OneSampleTTestResult {
    OneSampleTTestResult {
        variable: "salary".to_string(),
        n_valid: 100,
        n_missing: 0,
        mean: Some(50000.0),
        std_dev: Some(12000.0),
        se_mean: Some(1200.0),
        test_value: 45000.0,
        t_stat: Some(4.167),
        df: Some(99.0),
        sig_2tailed: Some(0.0001),
        mean_diff: Some(5000.0),
        ci_level: 0.95,
        ci_lower: Some(2618.0),
        ci_upper: Some(7382.0),
    }
}

fn sample_indep() -> IndepTTestResult {
    IndepTTestResult {
        variable: "salary".to_string(),
        grouping_var: "gender".to_string(),
        group1_stats: GroupStats {
            group_val: "1".to_string(),
            n: 45,
            mean: Some(52000.0),
            std_dev: Some(11000.0),
            se_mean: Some(1640.0),
        },
        group2_stats: GroupStats {
            group_val: "2".to_string(),
            n: 55,
            mean: Some(48000.0),
            std_dev: Some(12500.0),
            se_mean: Some(1685.0),
        },
        levene: LeveneResult {
            f_stat: Some(0.85),
            sig: Some(0.358),
            df1: 1.0,
            df2: 98.0,
        },
        equal_variances_assumed: TTestTwoGroupDetails {
            t_stat: Some(1.68),
            df: Some(98.0),
            sig_2tailed: Some(0.096),
            mean_diff: Some(4000.0),
            se_diff: Some(2380.0),
            ci_lower: Some(-723.0),
            ci_upper: Some(8723.0),
        },
        equal_variances_not_assumed: TTestTwoGroupDetails {
            t_stat: Some(1.70),
            df: Some(96.4),
            sig_2tailed: Some(0.092),
            mean_diff: Some(4000.0),
            se_diff: Some(2351.0),
            ci_lower: Some(-667.0),
            ci_upper: Some(8667.0),
        },
        ci_level: 0.95,
    }
}

fn sample_paired() -> PairedTTestResult {
    PairedTTestResult {
        pair_num: 1,
        var1_stats: PairedVarStats {
            var_name: "pre_test".to_string(),
            n: 50,
            mean: Some(120.0),
            std_dev: Some(15.0),
            se_mean: Some(2.12),
        },
        var2_stats: PairedVarStats {
            var_name: "post_test".to_string(),
            n: 50,
            mean: Some(115.0),
            std_dev: Some(14.0),
            se_mean: Some(1.98),
        },
        correlation: PairedCorrelation {
            var1: "pre_test".to_string(),
            var2: "post_test".to_string(),
            n: 50,
            correlation: Some(0.85),
            sig: Some(0.0001),
        },
        differences: PairedDifferences {
            mean_diff: Some(5.0),
            std_dev_diff: Some(8.0),
            se_diff: Some(1.13),
            ci_lower: Some(2.73),
            ci_upper: Some(7.27),
            t_stat: Some(4.42),
            df: Some(49.0),
            sig_2tailed: Some(0.0001),
        },
        confidence_level: 0.95,
    }
}

fn sample_anova() -> OneWayAnovaResult {
    OneWayAnovaResult {
        dep_var: "salary".to_string(),
        factor_var: "education".to_string(),
        between_groups: AnovaTableRow {
            sum_of_squares: 15000000.0,
            df: 2,
            mean_square: Some(7500000.0),
            f_stat: Some(12.5),
            p_value: Some(0.0001),
        },
        within_groups: AnovaTableRow {
            sum_of_squares: 58200000.0,
            df: 97,
            mean_square: Some(600000.0),
            f_stat: None,
            p_value: None,
        },
        total: AnovaTableRow {
            sum_of_squares: 73200000.0,
            df: 99,
            mean_square: None,
            f_stat: None,
            p_value: None,
        },
        descriptives: vec![],
        total_descriptives: None,
        homogeneity: None,
    }
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

    // Invariant: IDs strictly increasing
    for i in 0..5 {
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
    assert_eq!(state.procedures[0].items.len(), 11);
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
