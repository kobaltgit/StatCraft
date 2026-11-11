use super::*;
use statcraft_io::{RawColumn, RawDataType, RawTable, RawValue};

fn make_test_table() -> RawTable {
    let col1 = RawColumn {
        name: "id".to_string(),
        data_type: RawDataType::Numeric,
        values: vec![RawValue::Number(1.0), RawValue::Number(2.0)],
    };
    let col2 = RawColumn {
        name: "gender".to_string(),
        data_type: RawDataType::Numeric,
        values: vec![RawValue::Number(1.0), RawValue::Number(2.0)],
    };
    RawTable {
        columns: vec![col1, col2],
        row_count: 2,
    }
}

#[test]
fn test_dataset_dimensions() {
    let table = make_test_table();
    let ds = DataSet::from_raw_table(table).unwrap();

    assert_eq!(ds.row_count(), 2);
    assert_eq!(ds.col_count(), 2);
    assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(1.0)));
    assert_eq!(ds.get_cell(5, 5), None);
}

#[test]
fn test_dataset_get_cell_bounds_check() {
    let mut ds = DataSet::empty();
    assert_eq!(ds.get_cell(0, 0), None);
    assert!(matches!(
        ds.set_cell(0, 0, RawValue::Number(42.0)),
        Err(CoreError::CellOutOfBounds { .. })
    ));
}

#[test]
fn test_dataset_cell_edit_and_undo_redo() {
    let table = make_test_table();
    let mut ds = DataSet::from_raw_table(table).unwrap();

    // Initial value is 1.0
    assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(1.0)));

    // Edit cell to 99.0
    ds.set_cell(0, 0, RawValue::Number(99.0)).unwrap();
    assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(99.0)));

    // Undo
    assert!(ds.undo());
    assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(1.0)));

    // Redo
    assert!(ds.redo());
    assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(99.0)));
}

#[test]
fn test_dataset_format_display_value() {
    let table = make_test_table();
    let mut ds = DataSet::from_raw_table(table).unwrap();

    // Without labels: 1.00 (since default numeric has decimals: 2)
    assert_eq!(ds.get_cell_display(0, 1, false), "1.00");

    // Add ValueLabel 1 => "Male"
    ds.dictionary.get_mut(1).unwrap().values.insert("1", "Male");

    // With labels: Male
    assert_eq!(ds.get_cell_display(0, 1, true), "Male");
}

#[test]
fn test_dataset_blank_and_add_remove_variable() {
    let mut ds = DataSet::blank(10, 2).unwrap();
    assert_eq!(ds.row_count(), 10);
    assert_eq!(ds.col_count(), 2);
    assert_eq!(ds.dictionary.get(0).unwrap().name, "VAR00001");
    assert_eq!(ds.dictionary.get(1).unwrap().name, "VAR00002");

    // Add variable
    let idx = ds
        .add_variable("income", DataType::Dollar { decimals: 2 }, Measure::Scale)
        .unwrap();
    assert_eq!(idx, 2);
    assert_eq!(ds.col_count(), 3);
    assert_eq!(ds.get_cell(0, 2), Some(&RawValue::Null));

    // Set cell parsed
    ds.set_cell_parsed(0, 2, "$1,250.50").unwrap();
    assert_eq!(ds.get_cell(0, 2), Some(&RawValue::Number(1250.5)));
    assert_eq!(ds.get_cell_display(0, 2, false), "$1,250.50");

    // Rename variable
    ds.rename_variable(2, "monthly_income").unwrap();
    assert_eq!(ds.dictionary.get(2).unwrap().name, "monthly_income");

    // Remove variable
    ds.remove_variable(0).unwrap();
    assert_eq!(ds.col_count(), 2);
    assert_eq!(ds.dictionary.get(0).unwrap().name, "VAR00002");
}

#[test]
fn test_dataset_rows_manipulation() {
    let mut ds = DataSet::blank(2, 1).unwrap();
    assert_eq!(ds.row_count(), 2);

    ds.add_empty_row();
    assert_eq!(ds.row_count(), 3);
    assert_eq!(ds.get_cell(2, 0), Some(&RawValue::Null));

    ds.remove_row(1).unwrap();
    assert_eq!(ds.row_count(), 2);
}

#[test]
fn test_dataset_reorder_cases() {
    let mut ds = DataSet::blank(3, 1).unwrap();
    ds.set_cell(0, 0, RawValue::Number(10.0)).unwrap();
    ds.set_cell(1, 0, RawValue::Number(20.0)).unwrap();
    ds.set_cell(2, 0, RawValue::Number(30.0)).unwrap();

    ds.reorder_cases(&[2, 0, 1]).unwrap();
    assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(30.0)));
    assert_eq!(ds.get_cell(1, 0), Some(&RawValue::Number(10.0)));
    assert_eq!(ds.get_cell(2, 0), Some(&RawValue::Number(20.0)));

    assert!(ds.reorder_cases(&[0, 1]).is_err());
    assert!(ds.reorder_cases(&[0, 1, 99]).is_err());
}

#[test]
fn test_formatting_variations() {
    assert_eq!(
        format_number_grouped(1234567.89, 2, ',', '.'),
        "1,234,567.89"
    );
    assert_eq!(
        format_number_grouped(1234567.89, 2, '.', ','),
        "1.234.567,89"
    );
    assert_eq!(format_number_grouped(-9876.5, 1, ',', '.'), "-9,876.5");
    assert_eq!(parse_number_loose("1,234.56"), Some(1234.56));
    assert_eq!(parse_number_loose("1.234,56"), Some(1234.56));
    assert_eq!(parse_number_loose("$ 999.00"), Some(999.0));
}

#[test]
fn test_dataset_filtering() {
    let mut ds = DataSet::blank(4, 2).unwrap();
    // col 0: VAR00001, col 1: VAR00002 (we use as filter)
    ds.set_cell(0, 1, RawValue::Number(1.0)).unwrap();
    ds.set_cell(1, 1, RawValue::Number(0.0)).unwrap();
    ds.set_cell(2, 1, RawValue::Number(1.0)).unwrap();
    ds.set_cell(3, 1, RawValue::Null).unwrap();

    assert_eq!(ds.active_row_indices(), vec![0, 1, 2, 3]);
    assert!(ds.is_row_active(1));

    ds.set_filter_variable(Some("VAR00002".to_string()))
        .unwrap();
    assert_eq!(ds.active_filter_var(), Some("VAR00002"));
    assert!(ds.is_row_active(0));
    assert!(!ds.is_row_active(1));
    assert!(ds.is_row_active(2));
    assert!(!ds.is_row_active(3));
    assert_eq!(ds.active_row_indices(), vec![0, 2]);

    // Non-existent variable
    assert!(ds.set_filter_variable(Some("NON_EXISTENT".into())).is_err());

    // Clear filter
    ds.set_filter_variable(None).unwrap();
    assert_eq!(ds.active_filter_var(), None);
    assert_eq!(ds.active_row_indices(), vec![0, 1, 2, 3]);
}

#[test]
fn test_dataset_split_file() {
    let mut ds = DataSet::blank(4, 2).unwrap();
    // Col 0: gender (1, 1, 2, 2)
    // Col 1: salary (100, 200, 300, 400)
    ds.dictionary.get_mut(0).unwrap().name = "gender".to_string();
    ds.dictionary.get_mut(1).unwrap().name = "salary".to_string();
    ds.table.columns[0].name = "gender".to_string();
    ds.table.columns[1].name = "salary".to_string();

    ds.set_cell(0, 0, RawValue::Number(1.0)).unwrap();
    ds.set_cell(1, 0, RawValue::Number(1.0)).unwrap();
    ds.set_cell(2, 0, RawValue::Number(2.0)).unwrap();
    ds.set_cell(3, 0, RawValue::Number(2.0)).unwrap();

    // Default: 1 group containing all rows
    let groups = ds.split_groups();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].row_indices, vec![0, 1, 2, 3]);

    // Enable split file
    ds.set_split_file(Some(SplitFileConfig::new(
        SplitFileMode::CompareGroups,
        vec!["gender".to_string()],
    )))
    .unwrap();

    assert!(ds.split_file().unwrap().is_active());
    let groups = ds.split_groups();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].row_indices, vec![0, 1]);
    assert_eq!(groups[1].row_indices, vec![2, 3]);
    assert_eq!(groups[0].keys[0].var_name, "gender");
    assert_eq!(groups[0].keys[0].raw_value, RawValue::Number(1.0));

    // Rename variable: split_file should be updated
    ds.rename_variable(0, "sex").unwrap();
    assert_eq!(ds.split_file().unwrap().variables, vec!["sex".to_string()]);

    // Delete variable: split_file should be cleared
    ds.remove_variable(0).unwrap();
    assert_eq!(ds.split_file(), None);
}

#[test]
fn test_dataset_weighting() {
    let mut ds = DataSet::blank(5, 1).unwrap();
    ds.add_variable(
        "str_var",
        DataType::String { max_len: 20 },
        Measure::Nominal,
    )
    .unwrap();
    ds.add_variable(
        "filter_col",
        DataType::Numeric { decimals: 0 },
        Measure::Scale,
    )
    .unwrap();

    assert!(!ds.is_weighted());
    assert_eq!(ds.weight_var(), None);
    assert_eq!(ds.row_weight(0), 1.0);

    let err_missing = ds.set_weight_variable(Some("missing".to_string()));
    assert!(matches!(err_missing, Err(CoreError::VariableNotFound(_))));

    let err_str = ds.set_weight_variable(Some("str_var".to_string()));
    assert!(matches!(err_str, Err(CoreError::TypeMismatch(_))));

    ds.set_weight_variable(Some("VAR00001".to_string()))
        .unwrap();
    assert!(ds.is_weighted());
    assert_eq!(ds.weight_var(), Some("VAR00001"));

    ds.set_cell(0, 0, RawValue::Number(2.5)).unwrap();
    ds.set_cell(1, 0, RawValue::Number(0.0)).unwrap();
    ds.set_cell(2, 0, RawValue::Number(-3.0)).unwrap();
    ds.set_cell(3, 0, RawValue::Number(f64::NAN)).unwrap();
    ds.set_cell(4, 0, RawValue::Null).unwrap();

    assert_eq!(ds.row_weight(0), 2.5);
    assert_eq!(ds.row_weight(1), 0.0);
    assert_eq!(ds.row_weight(2), 0.0);
    assert_eq!(ds.row_weight(3), 0.0);
    assert_eq!(ds.row_weight(4), 0.0);

    ds.set_cell(0, 2, RawValue::Number(1.0)).unwrap();
    ds.set_cell(1, 2, RawValue::Number(1.0)).unwrap();
    ds.set_cell(2, 2, RawValue::Number(0.0)).unwrap();
    ds.set_filter_variable(Some("filter_col".to_string()))
        .unwrap();

    assert_eq!(ds.row_weight(0), 2.5);
    assert_eq!(ds.row_weight(1), 0.0);
    ds.set_cell(2, 0, RawValue::Number(10.0)).unwrap();
    assert_eq!(ds.row_weight(2), 0.0); // Inactive row -> 0.0

    ds.set_filter_variable(None).unwrap();
    assert_eq!(ds.row_weight(2), 10.0);

    ds.rename_variable(0, "w_cases").unwrap();
    assert_eq!(ds.weight_var(), Some("w_cases"));
    assert_eq!(ds.row_weight(0), 2.5);

    ds.remove_variable(0).unwrap();
    assert!(!ds.is_weighted());
    assert_eq!(ds.weight_var(), None);
    assert_eq!(ds.row_weight(0), 1.0);
}
