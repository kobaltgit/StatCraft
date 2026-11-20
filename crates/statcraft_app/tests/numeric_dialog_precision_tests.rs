//! Tests for numeric precision preservation across dataset cells and dialog level-counting.

use statcraft_core::{DataSet, DataType};
use statcraft_io::{load_csv_from_reader, CsvOptions};

#[test]
fn test_small_decimal_levels_are_distinct_numeric_values() {
    let csv = "y,x\n0.001,1\n0.002,2\n0.001,3\n0.002,4\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).expect("valid csv");
    let ds = DataSet::from_raw_table(table).expect("valid dataset");

    // Display string rounds both 0.001 and 0.002 to "0.00" under default F8.2 format
    assert_eq!(ds.get_cell_display(0, 0, false), "0.00");
    assert_eq!(ds.get_cell_display(1, 0, false), "0.00");

    // But get_cell_numeric preserves full precision
    let active_rows = ds.active_row_indices();
    let mut unique_levels: Vec<f64> = Vec::new();
    for &r in &active_rows {
        if let Some(v) = ds.get_cell_numeric(r, 0) {
            if v.is_finite() && !unique_levels.iter().any(|&u| (u - v).abs() < 1e-9) {
                unique_levels.push(v);
            }
        }
    }

    assert_eq!(
        unique_levels.len(),
        2,
        "Levels 0.001 and 0.002 must be counted as 2 distinct levels, not collapsed to 1"
    );
}

#[test]
fn test_get_cell_numeric_preserves_currency_and_comma_formats() {
    let csv = "amount\n1234.56\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).expect("valid csv");
    let mut ds = DataSet::from_raw_table(table).expect("valid dataset");

    // Change format to Dollar { decimals: 2 }
    if let Some(meta) = ds.dictionary.get_mut(0) {
        meta.var_type = DataType::Dollar { decimals: 2 };
    }

    // Display string has dollar sign
    assert_eq!(ds.get_cell_display(0, 0, false), "$1,234.56");

    // get_cell_numeric returns the raw number directly without parse errors
    assert_eq!(ds.get_cell_numeric(0, 0), Some(1234.56));
}

#[test]
fn test_get_cell_numeric_handles_missing_and_text_safely() {
    let csv = "v\n.\nNA\nvalid\n99.9\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).expect("valid csv");
    let ds = DataSet::from_raw_table(table).expect("valid dataset");

    assert_eq!(ds.get_cell_numeric(0, 0), None); // "."
    assert_eq!(ds.get_cell_numeric(1, 0), None); // "NA"
    assert_eq!(ds.get_cell_numeric(2, 0), None); // non-numeric text "valid"
    assert_eq!(ds.get_cell_numeric(3, 0), Some(99.9)); // numeric string "99.9" parses to 99.9
}
