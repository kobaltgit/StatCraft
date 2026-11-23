use statcraft_algo::{
    apply_format_options, compute_frequencies, compute_frequencies_detailed, sort_categories,
    CategoryOrder, FrequenciesFormatOptions, FrequencyInputItem, MultipleVarsMode,
};

#[test]
fn test_category_ordering_and_cumulative_percent() {
    // Variable with categories of different counts:
    // "B" (count 10), "A" (count 30), "C" (count 20)
    let mut items = Vec::new();
    for _ in 0..10 {
        items.push(FrequencyInputItem {
            value: "B".to_string(),
            label: Some("Beta".to_string()),
            is_missing: false,
            numeric_value: None,
            weight: 1.0,
        });
    }
    for _ in 0..30 {
        items.push(FrequencyInputItem {
            value: "A".to_string(),
            label: Some("Alpha".to_string()),
            is_missing: false,
            numeric_value: None,
            weight: 1.0,
        });
    }
    for _ in 0..20 {
        items.push(FrequencyInputItem {
            value: "C".to_string(),
            label: Some("Gamma".to_string()),
            is_missing: false,
            numeric_value: None,
            weight: 1.0,
        });
    }

    let base_res = compute_frequencies_detailed("code", &items);
    assert_eq!(base_res.categories.len(), 3);
    // Base order is AValue (lexical for text): "A" (30), "B" (10), "C" (20)
    assert_eq!(base_res.categories[0].value, "A");
    assert_eq!(base_res.categories[0].count, 30);
    assert!((base_res.categories[0].cumulative_percent.unwrap() - 50.0).abs() < 1e-6);

    assert_eq!(base_res.categories[1].value, "B");
    assert_eq!(base_res.categories[1].count, 10);
    assert!(
        (base_res.categories[1].cumulative_percent.unwrap() - (50.0 + 16.6666667)).abs() < 1e-4
    );

    assert_eq!(base_res.categories[2].value, "C");
    assert_eq!(base_res.categories[2].count, 20);
    assert!((base_res.categories[2].cumulative_percent.unwrap() - 100.0).abs() < 1e-6);

    // 1. DValue (descending value): "C", "B", "A"
    let mut dvalue_res = base_res.clone();
    sort_categories(&mut dvalue_res.categories, CategoryOrder::DValue);
    assert_eq!(dvalue_res.categories[0].value, "C");
    assert_eq!(dvalue_res.categories[1].value, "B");
    assert_eq!(dvalue_res.categories[2].value, "A");
    // Cumulative percents must be recalculated in new order:
    // C (20/60 = 33.333%) -> cum = 33.333%
    assert!(
        (dvalue_res.categories[0].cumulative_percent.unwrap() - (20.0 / 60.0 * 100.0)).abs() < 1e-5
    );
    // B (10/60 = 16.667%) -> cum = 50.0%
    assert!((dvalue_res.categories[1].cumulative_percent.unwrap() - 50.0).abs() < 1e-5);
    // A (30/60 = 50.0%) -> cum = 100.0%
    assert!((dvalue_res.categories[2].cumulative_percent.unwrap() - 100.0).abs() < 1e-5);

    // 2. AFreq (ascending frequency): "B" (10), "C" (20), "A" (30)
    let mut afreq_res = base_res.clone();
    sort_categories(&mut afreq_res.categories, CategoryOrder::AFreq);
    assert_eq!(afreq_res.categories[0].value, "B");
    assert_eq!(afreq_res.categories[1].value, "C");
    assert_eq!(afreq_res.categories[2].value, "A");
    assert!(
        (afreq_res.categories[0].cumulative_percent.unwrap() - (10.0 / 60.0 * 100.0)).abs() < 1e-5
    );
    assert!(
        (afreq_res.categories[1].cumulative_percent.unwrap() - (30.0 / 60.0 * 100.0)).abs() < 1e-5
    );
    assert!((afreq_res.categories[2].cumulative_percent.unwrap() - 100.0).abs() < 1e-5);

    // 3. DFreq (descending frequency): "A" (30), "C" (20), "B" (10)
    let mut dfreq_res = base_res.clone();
    sort_categories(&mut dfreq_res.categories, CategoryOrder::DFreq);
    assert_eq!(dfreq_res.categories[0].value, "A");
    assert_eq!(dfreq_res.categories[1].value, "C");
    assert_eq!(dfreq_res.categories[2].value, "B");
    assert!((dfreq_res.categories[0].cumulative_percent.unwrap() - 50.0).abs() < 1e-5);
    assert!(
        (dfreq_res.categories[1].cumulative_percent.unwrap() - (50.0 / 60.0 * 100.0)).abs() < 1e-5
    );
    assert!((dfreq_res.categories[2].cumulative_percent.unwrap() - 100.0).abs() < 1e-5);
}

#[test]
fn test_suppress_tables_limit() {
    // 15 distinct categories
    let sample: Vec<Option<String>> = (1..=15).map(|i| Some(format!("Item_{:02}", i))).collect();

    let mut res = compute_frequencies("items", &sample);
    assert_eq!(res.categories.len(), 15);
    assert!(!res.table_suppressed);

    // Limit = 10 -> 15 > 10 => suppressed
    let fmt_suppress = FrequenciesFormatOptions {
        order: CategoryOrder::AValue,
        multiple_vars_mode: MultipleVarsMode::Compare,
        suppress_tables_limit: Some(10),
    };
    apply_format_options(&mut res, &fmt_suppress);
    assert!(res.table_suppressed);
    assert!(res.categories.is_empty());
    assert_eq!(res.n_valid, 15);

    // When limit = 20 -> 15 <= 20 => not suppressed
    let mut res2 = compute_frequencies("items", &sample);
    let fmt_keep = FrequenciesFormatOptions {
        order: CategoryOrder::AValue,
        multiple_vars_mode: MultipleVarsMode::Compare,
        suppress_tables_limit: Some(20),
    };
    apply_format_options(&mut res2, &fmt_keep);
    assert!(!res2.table_suppressed);
    assert_eq!(res2.categories.len(), 15);
}
