use statcraft_algo::{FrequencyCategory, FrequencyResult};
use statcraft_app::views::output_view::{
    export_output_to_html, render_output_view, OutputItemKind, OutputViewerState,
};
use statcraft_reporting::styling::{
    CellCondition, CellStyleFormat, DimensionTarget, TableStyleRule, TableTarget, TargetValue,
};
use statcraft_syntax::commands::FrequenciesCommand;

fn make_sample_freq_result() -> FrequencyResult {
    FrequencyResult {
        variable: "salary_grp".to_string(),
        variable_label: Some("Salary Group".to_string()),
        n_valid: 250,
        n_missing: 0,
        categories: vec![
            FrequencyCategory {
                value: "High".to_string(),
                label: Some("High Income".to_string()),
                count: 180,
                percent: 72.0,
                valid_percent: Some(72.0),
                cumulative_percent: Some(72.0),
            },
            FrequencyCategory {
                value: "Low".to_string(),
                label: Some("Low Income".to_string()),
                count: 70,
                percent: 28.0,
                valid_percent: Some(28.0),
                cumulative_percent: Some(100.0),
            },
        ],
        quartiles: None,
        statistics: None,
        chart: None,
        table_suppressed: false,
        multiple_vars_mode: Default::default(),
    }
}

#[test]
fn test_frequencies_with_table_style_rule() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let mut state = OutputViewerState::new();
    let res = make_sample_freq_result();

    let rule = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Frequency".to_string()),
        dimension: DimensionTarget::Columns,
        condition: CellCondition::GreaterThan(100.0),
        format: CellStyleFormat {
            bold: true,
            italic: false,
            bg_color: Some([220, 245, 220]),
            text_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    let sps_log = "FREQUENCIES VARIABLES=salary_grp\n  /ORDER=ANALYSIS.".to_string();
    state.add_frequencies_styled(vec![res], sps_log, Some(&[rule]), false);

    assert_eq!(state.procedures.len(), 1);
    let proc = &state.procedures[0];
    assert_eq!(proc.items.len(), 3); // Log, Notes, ReportTable

    match &proc.items[2].kind {
        OutputItemKind::ReportTable(table) => {
            assert_eq!(table.headers.len(), 5);
            // Row 0 has count 180 > 100, column 1 ("Frequency") should be styled!
            let style_r0_c1 = table.cell_styles.get(&(0, 1));
            assert!(style_r0_c1.is_some(), "Cell (0, 1) should be styled");
            let s = style_r0_c1.unwrap();
            assert!(s.bold);
            assert_eq!(s.bg_color, Some([220, 245, 220]));

            // Row 1 has count 70 <= 100, column 1 should NOT be styled
            let style_r1_c1 = table.cell_styles.get(&(1, 1));
            assert!(style_r1_c1.is_none(), "Cell (1, 1) should not be styled");
        }
        _ => panic!("Expected ReportTable item"),
    }
}

#[test]
fn test_create_style_report_generates_audit_table() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let mut state = OutputViewerState::new();
    let res = make_sample_freq_result();

    let rule = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Frequency".to_string()),
        dimension: DimensionTarget::Columns,
        condition: CellCondition::GreaterThan(100.0),
        format: CellStyleFormat {
            bold: true,
            italic: false,
            bg_color: Some([220, 245, 220]),
            text_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    let sps_log = "FREQUENCIES VARIABLES=salary_grp\n  /ORDER=ANALYSIS.".to_string();
    state.add_frequencies_styled(vec![res], sps_log, Some(&[rule]), true);

    assert_eq!(state.procedures.len(), 1);
    let proc = &state.procedures[0];
    assert_eq!(proc.items.len(), 4); // Log, Notes, ReportTable, Conditional Styles Report

    // Check 4th item is the report table
    match &proc.items[3].kind {
        OutputItemKind::ReportTable(report_tab) => {
            assert_eq!(report_tab.headers.len(), 5);
            assert_eq!(report_tab.rows.len(), 1);
            let row = &report_tab.rows[0];
            // Table name, target, condition, format, count
            assert!(row[0].contains("Salary Group"));
            assert_eq!(row[1], "Frequency");
            assert_eq!(row[2], "> 100");
            assert!(row[3].contains("Bold"));
            assert_eq!(row[4], "3"); // 3 cells modified: category High (180), Total Valid (250), Total (250)
        }
        _ => panic!("Expected ReportTable for audit report"),
    }
}

#[test]
fn test_output_modify_syntax_paste_generation() {
    let params = serde_json::json!({
        "vars": ["salary"],
        "table_styles": [
            {
                "table_target": { "Subtype": "Frequencies" },
                "value_target": { "Label": "Frequency" },
                "dimension": "Columns",
                "condition": { "GreaterThan": 100.0 },
                "format": {
                    "bold": true,
                    "italic": false,
                    "bg_color": [220, 245, 220],
                    "text_color": null,
                    "reset_defaults": false
                }
            }
        ]
    });

    let sps = FrequenciesCommand::to_sps_from_params(&params);
    assert!(sps.contains("FREQUENCIES VARIABLES=salary"));
    assert!(sps.contains("OUTPUT MODIFY"));
    assert!(sps.contains("/SELECT TABLES"));
    assert!(sps.contains("/IF SUBTYPES=['Frequencies']"));
    assert!(sps.contains("/TABLECELLS SELECT=['Frequency'] SELECTDIMENSION=COLUMNS"));
    assert!(sps.contains("CONDITION=GT(100)"));
    assert!(sps.contains("STYLE=BACKGROUNDCOLOR(RGB(220, 245, 220)) FONTSTYLE=BOLD."));
}

#[test]
fn test_html_export_preserves_table_styles() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let mut state = OutputViewerState::new();
    let res = make_sample_freq_result();

    let rule = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Frequency".to_string()),
        dimension: DimensionTarget::Columns,
        condition: CellCondition::GreaterThan(100.0),
        format: CellStyleFormat {
            bold: true,
            italic: false,
            bg_color: Some([220, 245, 220]),
            text_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    state.add_frequencies_styled(vec![res], "LOG".to_string(), Some(&[rule]), false);

    let html = export_output_to_html(&state);
    assert!(html.contains("background-color: rgb(220, 245, 220);"));
    assert!(html.contains("font-weight: bold;"));
}

#[test]
fn test_headless_egui_render_with_styled_table() {
    let _ = statcraft_i18n::init_i18n();
    statcraft_i18n::set_language("en");

    let mut state = OutputViewerState::new();
    let res = make_sample_freq_result();

    let rule = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Frequency".to_string()),
        dimension: DimensionTarget::Columns,
        condition: CellCondition::GreaterThan(100.0),
        format: CellStyleFormat {
            bold: true,
            italic: false,
            bg_color: Some([220, 245, 220]),
            text_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    state.add_frequencies_styled(vec![res], "LOG".to_string(), Some(&[rule]), true);

    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });

    assert_eq!(state.procedures.len(), 1);
    assert_eq!(state.procedures[0].items.len(), 4);
}
