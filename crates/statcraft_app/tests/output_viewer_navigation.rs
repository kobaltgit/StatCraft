use egui::Context;
use statcraft_app::views::output_view::{render_output_view, OutputViewerState, SelectedTarget};

#[test]
fn test_output_viewer_procedure_and_item_auto_scroll() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let ctx = Context::default();
    let mut state = OutputViewerState::new();

    // 1. Add Log procedure
    state.add_entry("Log 1".to_string(), "GET DATA /TYPE=CSV.".to_string());

    // 2. Add Frequencies procedure with a table (contains nested ScrollArea::horizontal)
    let freq_sample = vec![statcraft_algo::FrequencyResult {
        variable: "gender".to_string(),
        variable_label: None,
        n_valid: 100,
        n_missing: 0,
        categories: vec![
            statcraft_algo::FrequencyCategory {
                value: "1".to_string(),
                label: Some("Male".to_string()),
                count: 45,
                percent: 45.0,
                valid_percent: Some(45.0),
                cumulative_percent: Some(45.0),
            },
            statcraft_algo::FrequencyCategory {
                value: "2".to_string(),
                label: Some("Female".to_string()),
                count: 55,
                percent: 55.0,
                valid_percent: Some(55.0),
                cumulative_percent: Some(100.0),
            },
        ],
        quartiles: None,
        statistics: None,
        chart: None,
        table_suppressed: false,
        multiple_vars_mode: statcraft_algo::MultipleVarsMode::Compare,
    }];
    state.add_frequencies(freq_sample);

    // 3. Add second Log procedure
    state.add_entry(
        "Log 2".to_string(),
        "DESCRIPTIVES VARIABLES=all.".to_string(),
    );

    assert_eq!(state.procedures.len(), 3);

    // Verify Procedure Title Localization does not return raw missing key
    let freq_proc_title = state.procedures[1].title();
    assert_eq!(freq_proc_title, "Frequencies");
    assert!(!freq_proc_title.contains('['));

    // Switch to Russian and verify Russian title
    statcraft_i18n::set_language("ru");
    let freq_proc_title_ru = state.procedures[1].title();
    assert_eq!(freq_proc_title_ru, "Частоты");
    assert!(!freq_proc_title_ru.contains('['));

    // Test Target: Procedure
    let proc_target = SelectedTarget::Procedure(1);
    state.selected_target = Some(proc_target);
    state.scroll_to_target = Some(proc_target);

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    assert_eq!(
        state.scroll_to_target, None,
        "Procedure scroll target must be consumed"
    );
    assert_eq!(state.selected_target, Some(proc_target));

    // Test Target: Child Table Item when subsequent procedures exist
    let item_target = SelectedTarget::Item {
        proc_idx: 1,
        item_idx: 2,
    };
    state.selected_target = Some(item_target);
    state.scroll_to_target = Some(item_target);

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    assert_eq!(
        state.scroll_to_target, None,
        "Item scroll target must be consumed even with nested tables"
    );
    assert_eq!(state.selected_target, Some(item_target));
}
