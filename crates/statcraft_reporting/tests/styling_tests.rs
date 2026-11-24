use statcraft_reporting::model::ReportTable;
use statcraft_reporting::styling::{
    parse_num, CellCondition, CellStyleFormat, ConditionalFormattingEngine, DimensionTarget,
    TableStyleRule, TableTarget, TargetValue,
};

fn create_sample_table() -> ReportTable {
    let mut table = ReportTable::new("Frequencies: Gender");
    table.headers = vec![
        "Category".to_string(),
        "Frequency".to_string(),
        "Percent".to_string(),
    ];
    table.rows = vec![
        vec!["Male".to_string(), "150".to_string(), "60.0%".to_string()],
        vec!["Female".to_string(), "100".to_string(), "40.0%".to_string()],
        vec!["Total".to_string(), "250".to_string(), "100.0%".to_string()],
    ];
    table
}

#[test]
fn test_safe_parse_num() {
    assert_eq!(parse_num("123"), Some(123.0));
    assert_eq!(parse_num("  123.45  "), Some(123.45));
    assert_eq!(parse_num("60.0%"), Some(60.0));
    assert_eq!(parse_num("1,234.56"), Some(1234.56));
    assert_eq!(parse_num("12,5"), Some(12.5));
    assert_eq!(parse_num("."), None);
    assert_eq!(parse_num(""), None);
    assert_eq!(parse_num("   "), None);
    assert_eq!(parse_num("Total"), None);
    assert_eq!(parse_num("sysmis"), None);
}

#[test]
fn test_table_target_matching() {
    assert!(ConditionalFormattingEngine::matches_table(
        &TableTarget::AllEligible,
        "Frequencies"
    ));
    assert!(ConditionalFormattingEngine::matches_table(
        &TableTarget::Subtype("frequencies".to_string()),
        "Frequencies"
    ));
    assert!(!ConditionalFormattingEngine::matches_table(
        &TableTarget::Subtype("Descriptives".to_string()),
        "Frequencies"
    ));
}

#[test]
fn test_apply_condition_greater_than_with_column_target() {
    let mut table = create_sample_table();
    let rule = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Frequency".to_string()),
        dimension: DimensionTarget::Columns,
        condition: CellCondition::GreaterThan(120.0),
        format: CellStyleFormat {
            bold: true,
            italic: false,
            text_color: Some([0, 128, 0]),
            bg_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    let modified = ConditionalFormattingEngine::apply_rules(&mut table, "Frequencies", &[rule]);
    // Rows 0 ("150") and 2 ("250") in column 1 ("Frequency") match > 120.0
    assert_eq!(modified, 2);
    assert!(table.cell_styles.contains_key(&(0, 1)));
    assert!(!table.cell_styles.contains_key(&(1, 1)));
    assert!(table.cell_styles.contains_key(&(2, 1)));

    let format = table.cell_styles.get(&(0, 1)).unwrap();
    assert!(format.bold);
    assert_eq!(format.text_color, Some([0, 128, 0]));
}

#[test]
fn test_apply_condition_between_and_not_between() {
    let mut table = create_sample_table();
    let rule_between = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::AllDataCells,
        dimension: DimensionTarget::Both,
        condition: CellCondition::Between(90.0, 160.0),
        format: CellStyleFormat {
            bold: true,
            italic: false,
            text_color: None,
            bg_color: Some([200, 200, 255]),
            reset_defaults: false,
            ..Default::default()
        },
    };

    let modified = ConditionalFormattingEngine::apply(&mut table, &[rule_between]);
    // "150" and "100" are between 90 and 160. "100.0%" also parses to 100.0.
    assert!(modified >= 2);
    assert!(table.cell_styles.contains_key(&(0, 1))); // 150
    assert!(table.cell_styles.contains_key(&(1, 1))); // 100
}

#[test]
fn test_rule_priority_override() {
    let mut table = create_sample_table();
    let rule1 = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::AllDataCells,
        dimension: DimensionTarget::Both,
        condition: CellCondition::All,
        format: CellStyleFormat {
            bold: false,
            italic: false,
            text_color: Some([10, 10, 10]),
            bg_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    let rule2 = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Total".to_string()),
        dimension: DimensionTarget::Rows,
        condition: CellCondition::All,
        format: CellStyleFormat {
            bold: true,
            italic: true,
            text_color: Some([255, 0, 0]),
            bg_color: Some([240, 240, 240]),
            reset_defaults: false,
            ..Default::default()
        },
    };

    ConditionalFormattingEngine::apply(&mut table, &[rule1, rule2]);

    // Total row is row 2. It should have rule2 applied.
    let total_cell_format = table.cell_styles.get(&(2, 0)).unwrap();
    assert!(total_cell_format.bold);
    assert!(total_cell_format.italic);
    assert_eq!(total_cell_format.text_color, Some([255, 0, 0]));

    // Row 0, col 0 ("Male") should retain rule1.
    let male_cell_format = table.cell_styles.get(&(0, 0)).unwrap();
    assert!(!male_cell_format.bold);
    assert_eq!(male_cell_format.text_color, Some([10, 10, 10]));
}

#[test]
fn test_reset_defaults() {
    let mut table = create_sample_table();
    let rule_apply = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::AllDataCells,
        dimension: DimensionTarget::Both,
        condition: CellCondition::All,
        format: CellStyleFormat {
            bold: true,
            italic: false,
            text_color: Some([255, 0, 0]),
            bg_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    let rule_reset = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Male".to_string()),
        dimension: DimensionTarget::Rows,
        condition: CellCondition::All,
        format: CellStyleFormat {
            bold: false,
            italic: false,
            text_color: None,
            bg_color: None,
            reset_defaults: true,
            ..Default::default()
        },
    };

    ConditionalFormattingEngine::apply(&mut table, &[rule_apply, rule_reset]);

    // Row 0 ("Male") should have been removed from cell_styles
    assert!(!table.cell_styles.contains_key(&(0, 0)));
    assert!(!table.cell_styles.contains_key(&(0, 1)));
    // Row 1 ("Female") should still be styled
    assert!(table.cell_styles.contains_key(&(1, 0)));
}

#[test]
fn test_top_n_and_bottom_n() {
    let mut table = ReportTable::new("Scores");
    table.headers = vec!["Student".to_string(), "Score".to_string()];
    table.rows = vec![
        vec!["Alice".to_string(), "85".to_string()],
        vec!["Bob".to_string(), "95".to_string()],
        vec!["Charlie".to_string(), "70".to_string()],
        vec!["Dave".to_string(), "90".to_string()],
    ];

    let rule_top2 = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Score".to_string()),
        dimension: DimensionTarget::Columns,
        condition: CellCondition::TopN(2),
        format: CellStyleFormat {
            bold: true,
            italic: false,
            text_color: Some([0, 150, 0]),
            bg_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    let count = ConditionalFormattingEngine::apply(&mut table, &[rule_top2]);
    assert_eq!(count, 2);
    // Bob (95) at row 1, Dave (90) at row 3 should be styled
    assert!(table.cell_styles.contains_key(&(1, 1)));
    assert!(table.cell_styles.contains_key(&(3, 1)));
    assert!(!table.cell_styles.contains_key(&(0, 1)));
    assert!(!table.cell_styles.contains_key(&(2, 1)));

    // Test Bottom 1
    let rule_bot1 = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Score".to_string()),
        dimension: DimensionTarget::Columns,
        condition: CellCondition::BottomN(1),
        format: CellStyleFormat {
            bold: false,
            italic: true,
            text_color: Some([200, 0, 0]),
            bg_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    ConditionalFormattingEngine::apply(&mut table, &[rule_bot1]);
    // Charlie (70) at row 2 should have bottom style
    assert!(table.cell_styles.contains_key(&(2, 1)));
    assert!(table.cell_styles.get(&(2, 1)).unwrap().italic);
}

#[test]
fn test_sysmis_condition() {
    let mut table = ReportTable::new("Missing Data");
    table.headers = vec!["Var".to_string(), "Val".to_string()];
    table.rows = vec![
        vec!["A".to_string(), ".".to_string()],
        vec!["B".to_string(), "10".to_string()],
        vec!["C".to_string(), "".to_string()],
        vec!["D".to_string(), "SYSMIS".to_string()],
    ];

    let rule = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::AllDataCells,
        dimension: DimensionTarget::Both,
        condition: CellCondition::SysMis,
        format: CellStyleFormat {
            bold: false,
            italic: true,
            text_color: Some([128, 128, 128]),
            bg_color: None,
            reset_defaults: false,
            ..Default::default()
        },
    };

    let count = ConditionalFormattingEngine::apply(&mut table, &[rule]);
    // Rows 0 ("."), 2 (""), 3 ("SYSMIS") in col 1 match
    assert!(count >= 3);
    assert!(table.cell_styles.contains_key(&(0, 1)));
    assert!(table.cell_styles.contains_key(&(2, 1)));
    assert!(table.cell_styles.contains_key(&(3, 1)));
    assert!(!table.cell_styles.contains_key(&(1, 1)));
}

#[test]
fn test_cell_target_scope_entire_column_and_row() {
    use statcraft_reporting::styling::CellTargetScope;

    let mut table = create_sample_table();
    let col_rule = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::Label("Frequency".to_string()),
        dimension: DimensionTarget::Columns,
        condition: CellCondition::Equal(150.0),
        format: CellStyleFormat {
            bold: true,
            target_scope: CellTargetScope::EntireColumn,
            ..Default::default()
        },
    };

    ConditionalFormattingEngine::apply(&mut table, &[col_rule]);
    // Entire column 1 ("Frequency") should be styled for rows 0, 1, 2
    assert!(table.cell_styles.get(&(0, 1)).unwrap().bold);
    assert!(table.cell_styles.get(&(1, 1)).unwrap().bold);
    assert!(table.cell_styles.get(&(2, 1)).unwrap().bold);

    // Row scope test
    let mut table2 = create_sample_table();
    let row_rule = TableStyleRule {
        table_target: TableTarget::AllEligible,
        value_target: TargetValue::AllDataCells,
        dimension: DimensionTarget::Both,
        condition: CellCondition::Equal(100.0), // matches Female (100) at row 1
        format: CellStyleFormat {
            italic: true,
            target_scope: CellTargetScope::EntireRow,
            replace_value: Some("Replaced".to_string()),
            ..Default::default()
        },
    };

    ConditionalFormattingEngine::apply(&mut table2, &[row_rule]);
    // Entire row 1 should be styled and text replaced
    assert!(table2.cell_styles.get(&(1, 0)).unwrap().italic);
    assert!(table2.cell_styles.get(&(1, 1)).unwrap().italic);
    assert_eq!(table2.rows[1][0], "Replaced");
    assert_eq!(table2.rows[1][1], "Replaced");
}
