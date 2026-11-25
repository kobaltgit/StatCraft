use statcraft_algo::{
    BarCategoryData, FrequenciesStatisticsResult, FrequencyCategory, FrequencyChartResult,
    FrequencyResult, HistogramBinData, HistogramSummary, MultipleVarsMode, PercentileValue,
};
use statcraft_app::views::output_view::{
    extract_item_tsv, frequencies_to_tsv, render_output_view, OutputItem, OutputItemKind,
    OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};

fn sample_rich_frequencies() -> Vec<FrequencyResult> {
    vec![
        FrequencyResult {
            variable: "salary".to_string(),
            variable_label: None,
            n_valid: 474,
            n_missing: 0,
            categories: vec![
                FrequencyCategory {
                    value: "20000".to_string(),
                    label: None,
                    count: 100,
                    percent: 21.1,
                    valid_percent: Some(21.1),
                    cumulative_percent: Some(21.1),
                },
                FrequencyCategory {
                    value: "30000".to_string(),
                    label: None,
                    count: 374,
                    percent: 78.9,
                    valid_percent: Some(78.9),
                    cumulative_percent: Some(100.0),
                },
            ],
            quartiles: Some((20000.0, 30000.0, 35000.0)),
            statistics: Some(FrequenciesStatisticsResult {
                mean: Some(34419.57),
                se_mean: Some(784.31),
                median: Some(28875.0),
                mode: Some(30750.0),
                has_multiple_modes: true,
                std_dev: Some(17075.66),
                variance: Some(291578191.0),
                skewness: Some(2.125),
                se_skewness: Some(0.112),
                kurtosis: Some(5.378),
                se_kurtosis: Some(0.224),
                range: Some(119250.0),
                min: Some(15750.0),
                max: Some(135000.0),
                sum: Some(16314875.0),
                percentiles: vec![
                    PercentileValue::new(25.0, 24000.0),
                    PercentileValue::new(50.0, 28875.0),
                    PercentileValue::new(75.0, 37000.0),
                    PercentileValue::new(90.0, 60000.0),
                ],
            }),
            chart: None,
            table_suppressed: false,
            multiple_vars_mode: MultipleVarsMode::Compare,
            bootstrap: None,
        },
        FrequencyResult {
            variable: "educ".to_string(),
            variable_label: None,
            n_valid: 474,
            n_missing: 0,
            categories: vec![],
            quartiles: None,
            statistics: Some(FrequenciesStatisticsResult {
                mean: Some(13.49),
                se_mean: Some(0.13),
                median: Some(12.0),
                mode: Some(12.0),
                has_multiple_modes: false,
                std_dev: Some(2.88),
                variance: Some(8.32),
                skewness: None,
                se_skewness: None,
                kurtosis: None,
                se_kurtosis: None,
                range: Some(13.0),
                min: Some(8.0),
                max: Some(21.0),
                sum: Some(6395.0),
                percentiles: vec![],
            }),
            chart: None,
            table_suppressed: false,
            multiple_vars_mode: MultipleVarsMode::Compare,
            bootstrap: None,
        },
        FrequencyResult {
            variable: "gender".to_string(),
            variable_label: None,
            n_valid: 474,
            n_missing: 0,
            categories: vec![
                FrequencyCategory {
                    value: "m".to_string(),
                    label: Some("Male".to_string()),
                    count: 258,
                    percent: 54.4,
                    valid_percent: Some(54.4),
                    cumulative_percent: Some(54.4),
                },
                FrequencyCategory {
                    value: "f".to_string(),
                    label: Some("Female".to_string()),
                    count: 216,
                    percent: 45.6,
                    valid_percent: Some(45.6),
                    cumulative_percent: Some(100.0),
                },
            ],
            quartiles: None,
            statistics: None,
            chart: None,
            table_suppressed: false,
            multiple_vars_mode: MultipleVarsMode::Compare,
            bootstrap: None,
        },
    ]
}

#[test]
fn test_frequencies_statistics_summary_table_rendering_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let results = sample_rich_frequencies();
    let tsv = frequencies_to_tsv(&results);

    // 1. TSV assertions for Statistics summary table
    assert!(
        tsv.contains("Statistics"),
        "TSV must contain 'Statistics' title:\n{}",
        tsv
    );
    assert!(
        tsv.contains("salary\teduc\tgender"),
        "TSV must contain variable headers:\n{}",
        tsv
    );
    assert!(
        tsv.contains("Valid\t474\t474\t474"),
        "TSV must contain Valid counts:\n{}",
        tsv
    );
    assert!(
        tsv.contains("Missing\t0\t0\t0"),
        "TSV must contain Missing counts:\n{}",
        tsv
    );
    assert!(
        tsv.contains("Mean\t34419.5700\t13.4900"),
        "TSV must contain Mean values:\n{}",
        tsv
    );
    assert!(
        tsv.contains("Std. Error of Mean\t784.3100"),
        "TSV must contain SEM:\n{}",
        tsv
    );
    assert!(
        tsv.contains("Median\t28875.0000\t12.0000"),
        "TSV must contain Median values:\n{}",
        tsv
    );
    assert!(
        tsv.contains("Mode\t30750.0000 a\t12.0000"),
        "TSV must contain Mode with 'a' flag:\n{}",
        tsv
    );
    assert!(
        tsv.contains("Percentiles"),
        "TSV must contain Percentiles header:\n{}",
        tsv
    );
    assert!(
        tsv.contains("  25\t24000.0000"),
        "TSV must contain 25th percentile:\n{}",
        tsv
    );
    assert!(
        tsv.contains("  90\t60000.0000"),
        "TSV must contain 90th percentile:\n{}",
        tsv
    );
    assert!(
        tsv.contains("Multiple modes exist. The smallest value is shown."),
        "TSV must contain footnote for multiple modes:\n{}",
        tsv
    );

    // 2. Category tables assertions in TSV
    assert!(
        tsv.contains("salary\nValue\tFrequency"),
        "TSV must contain salary frequency table:\n{}",
        tsv
    );
    assert!(
        tsv.contains("gender\nValue\tFrequency"),
        "TSV must contain gender frequency table:\n{}",
        tsv
    );
    assert!(
        tsv.contains("Male\t258\t54.4"),
        "TSV must contain value label 'Male':\n{}",
        tsv
    );

    // 3. UI rendering test without panic
    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();
    let proc_id = state.alloc_id();
    let notes_id = state.alloc_id();
    let item_id = state.alloc_id();
    state.procedures.push(OutputProcedure {
        id: proc_id,
        title_key: "output-frequencies",
        custom_title: None,
        dataset_name: Some("TestDataset".to_string()),
        is_expanded: true,
        items: vec![
            OutputItem {
                id: notes_id,
                title_key: "output-notes",
                custom_title: None,
                kind: OutputItemKind::Notes(OutputNotes {
                    dataset_name: "TestDataset".to_string(),
                    timestamp: "2026-11-22".to_string(),
                    active_cases: 474,
                    active_vars: 3,
                }),
            },
            OutputItem {
                id: item_id,
                title_key: "output-frequencies-table",
                custom_title: None,
                kind: OutputItemKind::FrequenciesTables(results),
            },
        ],
    });
    state.selected_target = Some(SelectedTarget::Item {
        proc_idx: 0,
        item_idx: 1,
    });

    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}

#[test]
fn test_frequencies_notable_suppresses_category_tables() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let mut results = sample_rich_frequencies();
    // In SPSS /FORMAT=NOTABLE, categories are cleared/suppressed
    for res in &mut results {
        res.categories.clear();
    }

    let tsv = frequencies_to_tsv(&results);
    assert!(
        tsv.contains("Statistics"),
        "Must contain summary table in NOTABLE mode"
    );
    assert!(
        !tsv.contains("Value\tFrequency"),
        "Must NOT contain category frequency tables in NOTABLE mode"
    );

    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();
    let proc_id = state.alloc_id();
    let item_id = state.alloc_id();
    state.procedures.push(OutputProcedure {
        id: proc_id,
        title_key: "output-frequencies",
        custom_title: None,
        dataset_name: Some("TestDataset".to_string()),
        is_expanded: true,
        items: vec![OutputItem {
            id: item_id,
            title_key: "output-frequencies-table",
            custom_title: None,
            kind: OutputItemKind::FrequenciesTables(results),
        }],
    });

    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}

#[test]
fn test_frequencies_with_variable_labels_rendering_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let results = vec![FrequencyResult {
        variable: "EDU_HOMEWORK_TIME_CAT".to_string(),
        variable_label: Some("8.17. Daily homework time".to_string()),
        n_valid: 100,
        n_missing: 5,
        categories: vec![
            FrequencyCategory {
                value: "1".to_string(),
                label: Some("< 1 hour".to_string()),
                count: 40,
                percent: 38.1,
                valid_percent: Some(40.0),
                cumulative_percent: Some(40.0),
            },
            FrequencyCategory {
                value: "2".to_string(),
                label: Some("1-2 hours".to_string()),
                count: 60,
                percent: 57.1,
                valid_percent: Some(60.0),
                cumulative_percent: Some(100.0),
            },
        ],
        quartiles: Some((1.0, 2.0, 2.0)),
        statistics: Some(FrequenciesStatisticsResult {
            mean: Some(1.6),
            se_mean: Some(0.05),
            median: Some(2.0),
            mode: Some(2.0),
            has_multiple_modes: false,
            std_dev: Some(0.49),
            variance: Some(0.24),
            skewness: None,
            se_skewness: None,
            kurtosis: None,
            se_kurtosis: None,
            range: Some(1.0),
            min: Some(1.0),
            max: Some(2.0),
            sum: Some(160.0),
            percentiles: vec![],
        }),
        chart: None,
        table_suppressed: false,
        multiple_vars_mode: MultipleVarsMode::Compare,
        bootstrap: None,
    }];

    let tsv = frequencies_to_tsv(&results);
    // Summary table column header should use label
    assert!(tsv.contains("8.17. Daily homework time"));
    // Category table title should contain Label [Variable]
    assert!(tsv.contains("8.17. Daily homework time [EDU_HOMEWORK_TIME_CAT]"));

    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();
    state.add_frequencies_with_log(
        results,
        "FREQUENCIES VARIABLES=EDU_HOMEWORK_TIME_CAT\n  /STATISTICS=MEAN MEDIAN MODE.\n"
            .to_string(),
    );

    assert_eq!(state.procedures.len(), 1);
    let proc = &state.procedures[0];
    assert_eq!(proc.items.len(), 3);
    // Item 0 is Log
    match &proc.items[0].kind {
        OutputItemKind::Log(log_str) => {
            assert!(log_str.contains("/STATISTICS=MEAN MEDIAN MODE."));
        }
        _ => panic!("Item 0 must be Log"),
    }

    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}

#[test]
fn test_frequencies_charts_bar_and_pie_rendering_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let bar_chart = FrequencyChartResult::Bar {
        var_name: "gender".to_string(),
        var_label: Some("Gender Identity".to_string()),
        is_percentages: true,
        categories: vec![
            BarCategoryData {
                label: "Female".to_string(),
                value_repr: "f".to_string(),
                count: 216.0,
                percent: 45.6,
            },
            BarCategoryData {
                label: "Male".to_string(),
                value_repr: "m".to_string(),
                count: 258.0,
                percent: 54.4,
            },
        ],
    };

    let pie_chart = FrequencyChartResult::Pie {
        var_name: "jobcat".to_string(),
        var_label: Some("Employment Category".to_string()),
        is_percentages: false,
        slices: vec![
            BarCategoryData {
                label: "Clerical".to_string(),
                value_repr: "1".to_string(),
                count: 363.0,
                percent: 76.6,
            },
            BarCategoryData {
                label: "Custodial".to_string(),
                value_repr: "2".to_string(),
                count: 27.0,
                percent: 5.7,
            },
            BarCategoryData {
                label: "Manager".to_string(),
                value_repr: "3".to_string(),
                count: 84.0,
                percent: 17.7,
            },
        ],
    };

    let charts_kind = OutputItemKind::FrequencyCharts(vec![bar_chart, pie_chart]);
    let tsv = extract_item_tsv(&charts_kind).expect("Charts must produce TSV");
    assert!(tsv.contains("Bar Chart: gender"));
    assert!(tsv.contains("Female\t216\t45.60%"));
    assert!(tsv.contains("Pie Chart: jobcat"));
    assert!(tsv.contains("Clerical\t363\t76.60%"));

    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();
    let proc_id = state.alloc_id();
    let item_id = state.alloc_id();
    state.procedures.push(OutputProcedure {
        id: proc_id,
        title_key: "output-frequencies",
        custom_title: None,
        dataset_name: Some("EmployeeData".to_string()),
        is_expanded: true,
        items: vec![OutputItem {
            id: item_id,
            title_key: "output-frequencies-charts",
            custom_title: None,
            kind: charts_kind,
        }],
    });

    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}

#[test]
fn test_frequencies_charts_histogram_rendering_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let hist_chart = FrequencyChartResult::Histogram {
        var_name: "salary".to_string(),
        var_label: Some("Current Salary".to_string()),
        summary: HistogramSummary {
            mean: 34419.57,
            std_dev: 17075.66,
            n_valid: 474,
            bin_width: 10000.0,
            bins: vec![
                HistogramBinData {
                    left: 10000.0,
                    right: 20000.0,
                    midpoint: 15000.0,
                    count: 65,
                    percent: 13.7,
                },
                HistogramBinData {
                    left: 20000.0,
                    right: 30000.0,
                    midpoint: 25000.0,
                    count: 220,
                    percent: 46.4,
                },
                HistogramBinData {
                    left: 30000.0,
                    right: 40000.0,
                    midpoint: 35000.0,
                    count: 105,
                    percent: 22.2,
                },
            ],
            normal_curve_points: Some(vec![
                (10000.0, 15.0),
                (20000.0, 120.0),
                (30000.0, 210.0),
                (40000.0, 95.0),
            ]),
        },
    };

    let charts_kind = OutputItemKind::FrequencyCharts(vec![hist_chart]);
    let tsv = extract_item_tsv(&charts_kind).expect("Histogram must produce TSV");
    assert!(tsv.contains("Histogram: salary"));
    assert!(tsv.contains("Mean=34419.57"));
    assert!(tsv.contains("StdDev=17075.660"));
    assert!(tsv.contains("N=474"));
    assert!(tsv.contains("10000.000\t20000.000\t15000.000\t65\t13.70%"));

    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();
    let proc_id = state.alloc_id();
    let item_id = state.alloc_id();
    state.procedures.push(OutputProcedure {
        id: proc_id,
        title_key: "output-frequencies",
        custom_title: None,
        dataset_name: Some("EmployeeData".to_string()),
        is_expanded: true,
        items: vec![OutputItem {
            id: item_id,
            title_key: "output-frequencies-charts",
            custom_title: None,
            kind: charts_kind,
        }],
    });

    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}

#[test]
fn test_frequencies_builder_with_charts_adds_chart_item() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let results = vec![FrequencyResult {
        variable: "gender".to_string(),
        variable_label: Some("Gender".to_string()),
        n_valid: 474,
        n_missing: 0,
        categories: vec![
            FrequencyCategory {
                value: "f".to_string(),
                label: Some("Female".to_string()),
                count: 216,
                percent: 45.6,
                valid_percent: Some(45.6),
                cumulative_percent: Some(45.6),
            },
            FrequencyCategory {
                value: "m".to_string(),
                label: Some("Male".to_string()),
                count: 258,
                percent: 54.4,
                valid_percent: Some(54.4),
                cumulative_percent: Some(100.0),
            },
        ],
        quartiles: None,
        statistics: None,
        chart: Some(FrequencyChartResult::Bar {
            var_name: "gender".to_string(),
            var_label: Some("Gender".to_string()),
            is_percentages: false,
            categories: vec![
                BarCategoryData {
                    label: "Female".to_string(),
                    value_repr: "f".to_string(),
                    count: 216.0,
                    percent: 45.6,
                },
                BarCategoryData {
                    label: "Male".to_string(),
                    value_repr: "m".to_string(),
                    count: 258.0,
                    percent: 54.4,
                },
            ],
        }),
        table_suppressed: false,
        multiple_vars_mode: MultipleVarsMode::Compare,
        bootstrap: None,
    }];

    let mut state = OutputViewerState::new();
    state.add_frequencies_with_log(
        results,
        "FREQUENCIES VARIABLES=gender\n  /BARCHART FREQ.\n".to_string(),
    );

    assert_eq!(state.procedures.len(), 1);
    let proc = &state.procedures[0];
    assert_eq!(
        proc.items.len(),
        4,
        "Should have Log, Notes, FrequenciesTables, and FrequencyCharts"
    );

    // Item 3 must be FrequencyCharts
    match &proc.items[3].kind {
        OutputItemKind::FrequencyCharts(charts) => {
            assert_eq!(charts.len(), 1);
            assert_eq!(charts[0].var_name(), "gender");
        }
        _ => panic!("Item 3 must be FrequencyCharts"),
    }

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}

#[test]
fn test_frequencies_limit_suppression_renders_note_and_preserves_statistics() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let results = vec![FrequencyResult {
        variable: "salary".to_string(),
        variable_label: Some("Current Salary".to_string()),
        n_valid: 474,
        n_missing: 0,
        categories: vec![],
        quartiles: Some((24000.0, 28875.0, 37000.0)),
        statistics: Some(FrequenciesStatisticsResult {
            mean: Some(34419.57),
            se_mean: Some(784.31),
            median: Some(28875.0),
            mode: Some(30750.0),
            has_multiple_modes: false,
            std_dev: Some(17075.66),
            variance: Some(291578191.0),
            skewness: None,
            se_skewness: None,
            kurtosis: None,
            se_kurtosis: None,
            range: Some(119250.0),
            min: Some(15750.0),
            max: Some(135000.0),
            sum: Some(16314875.0),
            percentiles: vec![],
        }),
        chart: Some(FrequencyChartResult::Bar {
            var_name: "salary".to_string(),
            var_label: Some("Current Salary".to_string()),
            is_percentages: false,
            categories: vec![BarCategoryData {
                label: "Tier 1".to_string(),
                value_repr: "1".to_string(),
                count: 100.0,
                percent: 21.1,
            }],
        }),
        table_suppressed: true,
        multiple_vars_mode: MultipleVarsMode::Compare,
        bootstrap: None,
    }];

    let tsv = frequencies_to_tsv(&results);
    assert!(tsv.contains("Statistics"), "Must preserve statistics table");
    assert!(tsv.contains("Mean\t34419.5700"));
    assert!(
        tsv.contains("The frequency table for variable Current Salary [salary] is not displayed because the number of categories exceeds the specified limit."),
        "Must contain suppression note in TSV:\n{}",
        tsv
    );

    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();
    state.add_frequencies_with_log(
        results,
        "FREQUENCIES VARIABLES=salary\n  /FORMAT=LIMIT(10).\n".to_string(),
    );

    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}

#[test]
fn test_frequencies_multivars_compare_and_variable_modes() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let make_results = |mode: MultipleVarsMode| {
        vec![
            FrequencyResult {
                variable: "varA".to_string(),
                variable_label: Some("Variable A".to_string()),
                n_valid: 100,
                n_missing: 0,
                categories: vec![FrequencyCategory {
                    value: "1".to_string(),
                    label: Some("Yes".to_string()),
                    count: 60,
                    percent: 60.0,
                    valid_percent: Some(60.0),
                    cumulative_percent: Some(60.0),
                }],
                quartiles: None,
                statistics: Some(FrequenciesStatisticsResult {
                    mean: Some(1.4),
                    se_mean: None,
                    median: Some(1.0),
                    mode: Some(1.0),
                    has_multiple_modes: false,
                    std_dev: Some(0.49),
                    variance: None,
                    skewness: None,
                    se_skewness: None,
                    kurtosis: None,
                    se_kurtosis: None,
                    range: None,
                    min: Some(1.0),
                    max: Some(2.0),
                    sum: Some(140.0),
                    percentiles: vec![],
                }),
                chart: Some(FrequencyChartResult::Bar {
                    var_name: "varA".to_string(),
                    var_label: Some("Variable A".to_string()),
                    is_percentages: false,
                    categories: vec![BarCategoryData {
                        label: "Yes".to_string(),
                        value_repr: "1".to_string(),
                        count: 60.0,
                        percent: 60.0,
                    }],
                }),
                table_suppressed: false,
                multiple_vars_mode: mode,
                bootstrap: None,
            },
            FrequencyResult {
                variable: "varB".to_string(),
                variable_label: Some("Variable B".to_string()),
                n_valid: 100,
                n_missing: 0,
                categories: vec![FrequencyCategory {
                    value: "2".to_string(),
                    label: Some("No".to_string()),
                    count: 40,
                    percent: 40.0,
                    valid_percent: Some(40.0),
                    cumulative_percent: Some(40.0),
                }],
                quartiles: None,
                statistics: Some(FrequenciesStatisticsResult {
                    mean: Some(2.2),
                    se_mean: None,
                    median: Some(2.0),
                    mode: Some(2.0),
                    has_multiple_modes: false,
                    std_dev: Some(0.51),
                    variance: None,
                    skewness: None,
                    se_skewness: None,
                    kurtosis: None,
                    se_kurtosis: None,
                    range: None,
                    min: Some(1.0),
                    max: Some(3.0),
                    sum: Some(220.0),
                    percentiles: vec![],
                }),
                chart: Some(FrequencyChartResult::Bar {
                    var_name: "varB".to_string(),
                    var_label: Some("Variable B".to_string()),
                    is_percentages: false,
                    categories: vec![BarCategoryData {
                        label: "No".to_string(),
                        value_repr: "2".to_string(),
                        count: 40.0,
                        percent: 40.0,
                    }],
                }),
                table_suppressed: false,
                multiple_vars_mode: mode,
                bootstrap: None,
            },
        ]
    };

    // 1. Compare mode: single summary header with both columns
    let comp_results = make_results(MultipleVarsMode::Compare);
    let comp_tsv = frequencies_to_tsv(&comp_results);
    assert!(
        comp_tsv.contains("Variable A\tVariable B"),
        "Compare mode combines columns"
    );

    let ctx = egui::Context::default();
    let mut comp_state = OutputViewerState::new();
    comp_state.add_frequencies_with_log(
        comp_results,
        "FREQUENCIES VARIABLES=varA varB\n  /FORMAT=COMPARE.\n".to_string(),
    );
    assert_eq!(
        comp_state.procedures[0].items.len(),
        4,
        "Compare mode has separate Charts item"
    );

    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut comp_state);
        });
    });

    // 2. Variable mode: each variable has its own statistics followed by table and chart
    let var_results = make_results(MultipleVarsMode::Variable);
    let var_tsv = frequencies_to_tsv(&var_results);
    assert!(
        !var_tsv.contains("Variable A\tVariable B"),
        "Variable mode separates statistics tables"
    );
    assert!(var_tsv.contains("Bar Chart: varA"));
    assert!(var_tsv.contains("Bar Chart: varB"));

    let mut var_state = OutputViewerState::new();
    var_state.add_frequencies_with_log(
        var_results,
        "FREQUENCIES VARIABLES=varA varB\n  /FORMAT=VARIABLE.\n".to_string(),
    );
    assert_eq!(
        var_state.procedures[0].items.len(),
        3,
        "Variable mode embeds charts directly in variable blocks"
    );

    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut var_state);
        });
    });
}

#[test]
fn test_frequencies_with_bootstrap_rendering_and_tsv() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let csv = "score,grp\n10,1\n20,1\n30,1\n40,2\n50,2\n60,2\n";
    let table =
        statcraft_io::load_csv_from_reader(csv.as_bytes(), statcraft_io::CsvOptions::default())
            .expect("parse csv");
    let ds = statcraft_core::DataSet::from_raw_table(table).expect("create dataset");

    let params = serde_json::json!({
        "vars": ["score"],
        "display_tables": true,
        "statistics": { "mean": true, "std_dev": true },
        "bootstrap": {
            "enabled": true,
            "samples": 50,
            "seed": 12345,
            "ci_level": 95.0,
            "ci_method": "percentile",
            "sampling_method": { "method": "stratified", "strata_vars": ["grp"] }
        }
    });

    let sps = statcraft_syntax::commands::FrequenciesCommand::to_sps_from_params(&params);
    assert!(sps.contains("SET MTINDEX=12345."));
    assert!(sps.contains("BOOTSTRAP"));
    assert!(sps.contains("/SAMPLING METHOD=STRATIFIED(STRATA=grp)"));
    assert!(sps.contains("FREQUENCIES VARIABLES=score"));

    let mut ds_opt = Some(ds);
    let val = statcraft_syntax::EngineCommand::execute(
        &statcraft_syntax::commands::FrequenciesCommand,
        &mut ds_opt,
        params,
    )
    .expect("frequencies with bootstrap should succeed");
    let results: Vec<FrequencyResult> =
        serde_json::from_value(val).expect("deserialize FrequencyResult");
    assert_eq!(results.len(), 1);
    let boot = results[0].bootstrap.as_ref().expect("bootstrap present");
    assert!(boot.stats.mean.is_some());
    assert!(boot.stats.std_dev.is_some());
    assert!(!boot.category_percents.is_empty());

    let tsv = frequencies_to_tsv(&results);
    assert!(tsv.contains("Bootstrap Specifications"));
    assert!(tsv.contains("Stratified (grp)"));
    assert!(tsv.contains("Statistics (Bootstrap)"));
    assert!(tsv.contains("95% CI Lower"));

    let ctx = egui::Context::default();
    let mut state = OutputViewerState::new();
    state.add_frequencies_with_log(results, sps);

    let _ = ctx.run(egui::RawInput::default(), |ui_ctx| {
        egui::CentralPanel::default().show(ui_ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}
