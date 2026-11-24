use statcraft_app::dialogs::table_style_dialog::{
    render_condition_subdialog, render_format_subdialog, render_table_style_dialog, TableStyleState,
};
use statcraft_app::dialogs::{
    render_frequencies_dialog, FrequenciesDialogAction, FrequenciesDialogState,
};
use statcraft_reporting::styling::{CellCondition, CellStyleFormat};

use super::fixtures::create_test_dataset;

#[test]
fn test_headless_render_table_style_dialog_basic() {
    let ctx = egui::Context::default();
    let mut state = TableStyleState {
        is_open: true,
        ..Default::default()
    };

    let _ = ctx.run(Default::default(), |ctx| {
        render_table_style_dialog(ctx, &mut state);
    });

    assert!(state.is_open);
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.selected_rule_idx, Some(0));
}

#[test]
fn test_table_style_rules_manipulation() {
    let mut state = TableStyleState::default();
    assert_eq!(state.rules.len(), 1);

    // 1. Add rule
    state.add_rule();
    assert_eq!(state.rules.len(), 2);
    assert_eq!(state.selected_rule_idx, Some(1));

    // 2. Duplicate rule
    state.duplicate_rule();
    assert_eq!(state.rules.len(), 3);
    assert_eq!(state.selected_rule_idx, Some(2));

    // 3. Move rule up
    state.move_rule_up();
    assert_eq!(state.selected_rule_idx, Some(1));

    // 4. Move rule down
    state.move_rule_down();
    assert_eq!(state.selected_rule_idx, Some(2));

    // Boundary: move down at bottom does not panic or change
    state.move_rule_down();
    assert_eq!(state.selected_rule_idx, Some(2));

    // Boundary: move up to 0 then move up again does not panic
    state.selected_rule_idx = Some(0);
    state.move_rule_up();
    assert_eq!(state.selected_rule_idx, Some(0));

    // 5. Delete rule
    state.delete_rule();
    assert_eq!(state.rules.len(), 2);
    assert_eq!(state.selected_rule_idx, Some(0));

    state.delete_rule();
    state.delete_rule();
    assert_eq!(state.rules.len(), 0);
    assert_eq!(state.selected_rule_idx, None);

    // Boundary: delete when empty does not panic
    state.delete_rule();
    assert_eq!(state.rules.len(), 0);
}

#[test]
fn test_table_style_transactionality_on_cancel() {
    let mut state = TableStyleState::default();
    let original_rules = state.rules.clone();

    // Backup on open
    state.backup = Some(original_rules.clone());
    state.backup_report = state.create_style_report;
    state.is_open = true;

    // Mutate state
    state.add_rule();
    state.add_rule();
    state.create_style_report = true;
    assert_eq!(state.rules.len(), 3);

    // Cancel restores original
    state.cancel();
    assert!(!state.is_open);
    assert_eq!(state.rules, original_rules);
    assert!(!state.create_style_report);
}

#[test]
fn test_table_style_condition_subdialog_headless() {
    let ctx = egui::Context::default();
    let mut state = TableStyleState {
        is_open: true,
        ..Default::default()
    };

    state.open_condition_editor(0);
    assert_eq!(state.editing_condition_idx, Some(0));

    // Configure condition in subdialog
    state.cond_is_conditional = true;
    state.cond_operator_idx = 4; // GreaterThan
    state.cond_val1_input = "42.5".to_string();

    let _ = ctx.run(Default::default(), |ctx| {
        render_condition_subdialog(ctx, &mut state);
    });

    assert_eq!(state.editing_condition_idx, Some(0));
}

#[test]
fn test_table_style_format_subdialog_headless() {
    let ctx = egui::Context::default();
    let mut state = TableStyleState {
        is_open: true,
        ..Default::default()
    };

    state.open_format_editor(0);
    assert_eq!(state.editing_format_idx, Some(0));

    state.fmt_bold = true;
    state.fmt_italic = true;
    state.fmt_bg_color = Some([255, 225, 225]);
    state.fmt_text_color = Some([180, 20, 20]);

    let _ = ctx.run(Default::default(), |ctx| {
        render_format_subdialog(ctx, &mut state);
    });

    assert_eq!(state.editing_format_idx, Some(0));
}

#[test]
fn test_frequencies_dialog_style_subdialog_integration() {
    let ds = create_test_dataset();
    let ctx = egui::Context::default();
    let mut state = FrequenciesDialogState {
        is_open: true,
        selected_vars: vec!["salary".to_string()],
        ..Default::default()
    };

    // Open style subdialog
    state.table_style.backup = Some(state.table_style.rules.clone());
    state.table_style.backup_report = state.table_style.create_style_report;
    state.table_style.is_open = true;

    // Mutate a rule
    state.table_style.rules[0].condition = CellCondition::GreaterThan(50000.0);
    state.table_style.rules[0].format = CellStyleFormat {
        bold: true,
        italic: false,
        text_color: Some([180, 20, 20]),
        bg_color: Some([255, 250, 205]),
        reset_defaults: false,
        ..Default::default()
    };
    state.table_style.create_style_report = true;

    let _ = ctx.run(Default::default(), |ctx| {
        let _ = render_frequencies_dialog(ctx, Some(&ds), &mut state);
    });

    assert!(state.table_style.is_open);

    // Commit subdialog
    state.table_style.confirm();
    assert!(!state.table_style.is_open);

    let rules_json = state.table_style.rules_to_json();
    assert!(rules_json.is_some());

    let action = FrequenciesDialogAction::Run {
        vars: state.selected_vars.clone(),
        display_tables: state.display_tables,
        apa_tables: state.apa_tables,
        statistics: state.stats.to_statistics_json(),
        percentiles: state.stats.to_percentiles_json(),
        group_midpoints: state.stats.group_midpoints,
        charts: state.charts.to_json(),
        order: None,
        multiple_vars_mode: None,
        suppress_tables_limit: None,
        table_styles: rules_json,
        create_style_report: state.table_style.create_style_report,
    };

    match action {
        FrequenciesDialogAction::Run {
            table_styles,
            create_style_report,
            ..
        } => {
            assert!(table_styles.is_some());
            assert!(create_style_report);
        }
        _ => panic!("Expected FrequenciesDialogAction::Run"),
    }
}

#[test]
fn test_table_style_format_subdialog_commit_full_spss_parity() {
    use statcraft_app::dialogs::table_style_dialog::format_masks::{
        get_format_mask_label, FORMAT_MASK_KEYS,
    };

    assert_eq!(FORMAT_MASK_KEYS.len(), 34);
    for idx in 0..34 {
        let label = get_format_mask_label(idx);
        assert!(
            !label.is_empty(),
            "Format mask idx {} must have a non-empty label",
            idx
        );
    }

    let ctx = egui::Context::default();
    let mut state = TableStyleState {
        is_open: true,
        ..Default::default()
    };

    state.open_format_editor(0);
    assert_eq!(state.editing_format_idx, Some(0));

    // Configure all parity fields
    state.fmt_use_defaults = false;
    state.fmt_bold = true;
    state.fmt_italic = true;
    state.fmt_underline = true;
    state.fmt_text_color = Some([139, 0, 0]);
    state.fmt_bg_color = Some([255, 255, 204]);
    state.fmt_format_idx = 30; // January, February...
    state.fmt_decimals_enabled = true;
    state.fmt_decimals_val = "3".to_string();
    state.fmt_apply_to_idx = 1; // EntireColumn
    state.fmt_replace_enabled = true;
    state.fmt_replace_val = "N/A".to_string();

    let _ = ctx.run(Default::default(), |ctx| {
        render_format_subdialog(ctx, &mut state);
    });

    // Subdialog is open and rendered without error
    assert_eq!(state.editing_format_idx, Some(0));
}
