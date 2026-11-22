use statcraft_algo::{
    FrequenciesStatisticsResult, FrequencyCategory, FrequencyResult, PercentileValue,
};
use statcraft_app::views::output_view::{
    frequencies_to_tsv, render_output_view, OutputItem, OutputItemKind, OutputNotes,
    OutputProcedure, OutputViewerState, SelectedTarget,
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
