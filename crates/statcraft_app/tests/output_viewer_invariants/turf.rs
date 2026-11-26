use statcraft_algo::{TurfCombinationRow, TurfCycleResult, TurfOptions, TurfResult};
use statcraft_app::views::output_view::{
    export_output_to_html, export_output_to_txt, turf_to_report_tables, turf_to_tsv,
    OutputItemKind, OutputViewerState,
};

pub fn sample_turf() -> (TurfResult, TurfOptions) {
    let opts = TurfOptions {
        variables: vec!["v1".to_string(), "v2".to_string(), "v3".to_string()],
        positive_values: vec!["1".to_string()],
        max_combinations: Some(2),
        num_to_display: Some(5),
        min_positive_pct: Some(5.0),
        always_required: 0,
        removal_cycles: 1,
        show_only_max_combo: false,
        limit_exceeded_abort: true,
        max_operations: 5_000_000,
        importance_weights: [("v1".to_string(), 1.5), ("v2".to_string(), 2.0)]
            .into_iter()
            .collect(),
        weights_strict: false,
        use_advanced_selection: true,
        advanced_auto: true,
        advanced_size: 18,
        advanced_winners: 2,
        reach_and_frequency_chart: true,
    };

    let rows_k1 = vec![
        TurfCombinationRow {
            combo_size: 1,
            variables: vec!["v2".to_string()],
            reach_count: 80.0,
            reach_pct: 80.0,
            frequency_count: 80.0,
            frequency_pct: 44.4,
            mean_responses_per_reached: 1.0,
            weighted_score: Some(160.0),
        },
        TurfCombinationRow {
            combo_size: 1,
            variables: vec!["v1".to_string()],
            reach_count: 60.0,
            reach_pct: 60.0,
            frequency_count: 60.0,
            frequency_pct: 33.3,
            mean_responses_per_reached: 1.0,
            weighted_score: Some(90.0),
        },
    ];

    let rows_k2 = vec![TurfCombinationRow {
        combo_size: 2,
        variables: vec!["v1".to_string(), "v2".to_string()],
        reach_count: 95.0,
        reach_pct: 95.0,
        frequency_count: 140.0,
        frequency_pct: 77.8,
        mean_responses_per_reached: 1.47,
        weighted_score: Some(250.0),
    }];

    let cycle0 = TurfCycleResult {
        cycle_index: 0,
        removed_variables: Vec::new(),
        rows_by_size: vec![(1, rows_k1), (2, rows_k2)],
        used_heats: false,
        estimated_operations: 3,
    };

    let cycle1 = TurfCycleResult {
        cycle_index: 1,
        removed_variables: vec!["v2".to_string()],
        rows_by_size: vec![(
            1,
            vec![TurfCombinationRow {
                combo_size: 1,
                variables: vec!["v1".to_string()],
                reach_count: 60.0,
                reach_pct: 60.0,
                frequency_count: 60.0,
                frequency_pct: 33.3,
                mean_responses_per_reached: 1.0,
                weighted_score: Some(90.0),
            }],
        )],
        used_heats: false,
        estimated_operations: 2,
    };

    let result = TurfResult {
        total_valid_cases: 100.0,
        total_cases_unweighted: 100,
        excluded_variables: vec![("v_low".to_string(), 2.0)],
        cycles: vec![cycle0, cycle1],
    };

    (result, opts)
}

#[test]
fn test_turf_stage3_output_invariants_and_export() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let (res, opts) = sample_turf();
    let mut state = OutputViewerState::new();

    let sps = statcraft_syntax::commands::TurfCommand::to_sps(&opts);
    state.add_turf(res.clone(), opts.clone(), sps);

    assert_eq!(state.procedures.len(), 1);
    let proc = &state.procedures[0];
    assert_eq!(proc.title_key, "output-turf-title");
    assert_eq!(proc.items.len(), 3);
    assert!(matches!(&proc.items[0].kind, OutputItemKind::Log(_)));
    assert!(matches!(&proc.items[1].kind, OutputItemKind::Notes(_)));
    assert!(matches!(&proc.items[2].kind, OutputItemKind::Turf(_)));

    if let OutputItemKind::Turf(item) = &proc.items[2].kind {
        assert_eq!(item.result.total_valid_cases, 100.0);
        assert_eq!(item.options.variables.len(), 3);
        assert_eq!(item.result.cycles.len(), 2);

        // Test TSV generation
        let tsv = turf_to_tsv(item);
        assert!(tsv.contains("Valid Cases\t100"));
        assert!(tsv.contains("v_low (2.0%)"));
        assert!(tsv.contains("v1 + v2\t95\t95.0%\t140\t77.8%\t1.47\t250.0000"));
        assert!(tsv.contains("Cycle 1"));

        // Test ReportTables generation
        let tables = turf_to_report_tables(item);
        assert_eq!(tables.len(), 3); // Summary + Cycle 0 + Cycle 1
        assert_eq!(tables[0].headers, vec!["Parameter", "Value"]);
        assert!(tables[1].headers.contains(&"Weighted Score".to_string()));
    } else {
        panic!("Expected OutputItemKind::Turf");
    }

    // Test HTML and TXT export
    let html = export_output_to_html(&state);
    assert!(html.contains("TURF Analysis"));
    assert!(html.contains("v1 + v2"));

    let txt = export_output_to_txt(&state);
    assert!(txt.contains("TURF Analysis"));
    assert!(txt.contains("Valid Cases"));
}
