//! Procedures compute on full-precision values, never on the print format.
//!
//! Numeric CSV columns get a print format of two decimals (`F8.2`). That format must only govern
//! display: as in SPSS, `DESCRIPTIVES`, `REGRESSION`, `CORRELATIONS` and the other procedures work
//! on the stored values. The data here is exactly linear, y = 2x + 0.001, on values with three
//! decimals, so the reference results are exact: mean(x) = 0.6505, slope 2, intercept 0.001,
//! r = 1. Read through the two-decimal format, every one of them is off by more than 1e-4.

use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};
use statcraft_syntax::{CommandBus, SyntaxDispatcher};

const TOL: f64 = 1e-9;

fn dataset() -> DataSet {
    let csv = "x,y\n0.123,0.247\n0.456,0.913\n0.789,1.579\n1.234,2.469\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).expect("valid csv");
    DataSet::from_raw_table(table).expect("valid dataset")
}

fn run(sps: &str) -> serde_json::Value {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(dataset());
    let res =
        SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).expect("batch should succeed");
    assert_eq!(res.len(), 1);
    res[0].1.clone()
}

#[test]
fn print_format_does_not_round_stored_values() {
    let ds = dataset();
    assert_eq!(
        ds.get_cell_display(0, 0, false),
        "0.12",
        "the format still governs display"
    );
    assert_eq!(
        ds.get_cell_numeric(0, 0),
        Some(0.123),
        "the value keeps its precision"
    );
}

#[test]
fn descriptives_use_full_precision() {
    let v = run("DESCRIPTIVES /VARIABLES=x.");
    let x = v
        .as_array()
        .and_then(|a| a.iter().find(|r| r["variable"] == "x"))
        .expect("x row");
    let mean = x["mean"].as_f64().expect("mean");
    assert!(
        (mean - 0.6505).abs() < TOL,
        "mean {mean}, expected 0.6505 (0.65 from rounded values)"
    );
    let min = x["min"].as_f64().expect("min");
    assert!((min - 0.123).abs() < TOL, "min {min}, expected 0.123");
}

#[test]
fn regression_uses_full_precision() {
    let v = run("REGRESSION /DEPENDENT y /METHOD=ENTER x.");
    let coefs = v["coefficients"].as_array().expect("coefficients");
    let b = |name: &str| {
        coefs
            .iter()
            .find(|c| c["var_name"] == name)
            .and_then(|c| c["b"].as_f64())
            .expect(name)
    };
    assert!((b("x") - 2.0).abs() < TOL, "slope {}, expected 2", b("x"));
    assert!(
        (b("(Constant)") - 0.001).abs() < TOL,
        "intercept {}, expected 0.001",
        b("(Constant)")
    );
}

#[test]
fn correlations_use_full_precision() {
    let v = run("CORRELATIONS /VARIABLES=x y.");
    let r = v["matrices"][0]["cells"][0][1]["r"].as_f64().expect("r");
    assert!(
        (r - 1.0).abs() < TOL,
        "r {r}, expected 1 for an exactly linear pair"
    );
}
