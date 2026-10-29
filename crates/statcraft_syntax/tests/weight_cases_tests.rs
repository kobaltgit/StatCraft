use serde_json::json;
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use statcraft_syntax::bus::CommandBus;
use statcraft_syntax::commands::WeightCasesCommand;

fn make_sample_dataset() -> DataSet {
    let mut ds = DataSet::blank(3, 2).unwrap();
    // Col 0: score (10, 20, 30)
    // Col 1: count (2, 3, 5)
    ds.dictionary.get_mut(0).unwrap().name = "score".to_string();
    ds.dictionary.get_mut(1).unwrap().name = "count".to_string();
    ds.table.columns[0].name = "score".to_string();
    ds.table.columns[1].name = "count".to_string();

    ds.set_cell(0, 0, RawValue::Number(10.0)).unwrap();
    ds.set_cell(1, 0, RawValue::Number(20.0)).unwrap();
    ds.set_cell(2, 0, RawValue::Number(30.0)).unwrap();

    ds.set_cell(0, 1, RawValue::Number(2.0)).unwrap();
    ds.set_cell(1, 1, RawValue::Number(3.0)).unwrap();
    ds.set_cell(2, 1, RawValue::Number(5.0)).unwrap();

    ds
}

#[test]
fn test_weight_cases_sps_syntax_generation() {
    assert_eq!(
        WeightCasesCommand::to_sps(Some("count")),
        "WEIGHT BY count.\nEXECUTE."
    );
    assert_eq!(WeightCasesCommand::to_sps(None), "WEIGHT OFF.\nEXECUTE.");
    assert_eq!(
        WeightCasesCommand::to_sps(Some("   ")),
        "WEIGHT OFF.\nEXECUTE."
    );
}

#[test]
fn test_weight_cases_command_bus_integration() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());

    // 1. Unweighted descriptives
    let desc_unweighted = bus
        .execute("stat.descriptives", &mut ds, json!({ "vars": ["score"] }))
        .unwrap();
    let mean_unweighted = desc_unweighted[0]["mean"].as_f64().unwrap();
    // Unweighted mean of (10, 20, 30) = 20.0
    assert!((mean_unweighted - 20.0).abs() < 1e-6);
    assert_eq!(desc_unweighted[0]["n_valid"].as_u64().unwrap(), 3);

    // 2. Turn ON weighting by 'count'
    let weight_res = bus
        .execute("data.weight_cases", &mut ds, json!({ "variable": "count" }))
        .unwrap();
    assert_eq!(weight_res["status"], "ok");
    assert_eq!(weight_res["weight_var"], "count");
    assert!(ds.as_ref().unwrap().is_weighted());

    // 3. Weighted descriptives:
    // scores (10, 20, 30) with weights (2, 3, 5):
    // sum = 10*2 + 20*3 + 30*5 = 20 + 60 + 150 = 230
    // W = 2 + 3 + 5 = 10
    // mean = 230 / 10 = 23.0
    let desc_weighted = bus
        .execute("stat.descriptives", &mut ds, json!({ "vars": ["score"] }))
        .unwrap();
    let mean_weighted = desc_weighted[0]["mean"].as_f64().unwrap();
    assert!((mean_weighted - 23.0).abs() < 1e-6);
    assert_eq!(desc_weighted[0]["n_valid"].as_u64().unwrap(), 10);

    // 4. Weighted frequencies:
    let freq_weighted = bus
        .execute("stat.frequencies", &mut ds, json!({ "vars": ["score"] }))
        .unwrap();
    assert_eq!(freq_weighted[0]["n_valid"].as_u64().unwrap(), 10);
    let cats = freq_weighted[0]["categories"].as_array().unwrap();
    assert_eq!(cats.len(), 3);
    assert_eq!(cats[0]["count"].as_u64().unwrap(), 2);
    assert_eq!(cats[1]["count"].as_u64().unwrap(), 3);
    assert_eq!(cats[2]["count"].as_u64().unwrap(), 5);

    // 5. Undo operations back to unweighted state:
    // bus.undo pops in LIFO order: frequencies, descriptives, then weight_cases
    assert!(bus.undo(&mut ds).is_ok()); // pops stat.frequencies
    assert!(bus.undo(&mut ds).is_ok()); // pops stat.descriptives
    assert!(bus.undo(&mut ds).is_ok()); // pops data.weight_cases
    assert!(!ds.as_ref().unwrap().is_weighted());
    assert_eq!(ds.as_ref().unwrap().weight_var(), None);

    let desc_restored = bus
        .execute("stat.descriptives", &mut ds, json!({ "vars": ["score"] }))
        .unwrap();
    let mean_restored = desc_restored[0]["mean"].as_f64().unwrap();
    assert!((mean_restored - 20.0).abs() < 1e-6);
    assert_eq!(desc_restored[0]["n_valid"].as_u64().unwrap(), 3);
}

#[test]
fn test_weight_cases_validation_errors() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());

    // Non-existent variable
    let err_missing = bus.execute(
        "data.weight_cases",
        &mut ds,
        json!({ "variable": "does_not_exist" }),
    );
    assert!(err_missing.is_err());

    // String variable
    ds.as_mut()
        .unwrap()
        .add_variable(
            "text_var",
            DataType::String { max_len: 20 },
            Measure::Nominal,
        )
        .unwrap();
    let err_str = bus.execute(
        "data.weight_cases",
        &mut ds,
        json!({ "variable": "text_var" }),
    );
    assert!(err_str.is_err());
}
