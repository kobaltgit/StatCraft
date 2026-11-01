use serde_json::json;
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use statcraft_syntax::commands::ProgrammabilityCommand;
use statcraft_syntax::CommandBus;

fn create_sample_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    ds.add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("educ", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
        .unwrap();

    let salaries = [30000.0, 45000.0, 60000.0, 80000.0];
    let educs = [12.0, 14.0, 16.0, 18.0];

    for _ in 0..4 {
        ds.add_empty_row();
    }

    for i in 0..4 {
        ds.set_cell(i, 0, RawValue::Number(salaries[i])).unwrap();
        ds.set_cell(i, 1, RawValue::Number(educs[i])).unwrap();
    }

    ds
}

#[test]
fn test_programmability_cmd_sps_syntax() {
    let script = "target = salary / 1000\nIF educ >= 16 THEN high_educ = 1 ELSE 0";
    let sps = ProgrammabilityCommand::to_sps(script);
    assert!(sps.starts_with("BEGIN PROGRAM."));
    assert!(sps.contains("target = salary / 1000"));
    assert!(sps.contains("IF educ >= 16 THEN high_educ = 1 ELSE 0"));
    assert!(sps.ends_with("END PROGRAM.\nEXECUTE."));
}

#[test]
fn test_programmability_cmd_bus_execution_and_undo() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_sample_dataset());

    let script = "scaled_salary = salary / 1000\nIF educ >= 16 THEN grad = 1 ELSE grad = 0";
    let params = json!({
        "script": script
    });

    let res = bus
        .execute("transform.programmability", &mut ds, params)
        .unwrap();
    assert_eq!(res["status"], "success");
    assert_eq!(res["statements"], 2);

    let active_ds = ds.as_ref().unwrap();
    assert_eq!(active_ds.col_count(), 4);

    let scaled_col = active_ds.dictionary.find_index("scaled_salary").unwrap();
    assert_eq!(
        active_ds.get_cell(0, scaled_col),
        Some(&RawValue::Number(30.0))
    );

    let grad_col = active_ds.dictionary.find_index("grad").unwrap();
    assert_eq!(
        active_ds.get_cell(0, grad_col),
        Some(&RawValue::Number(0.0))
    );
    assert_eq!(
        active_ds.get_cell(2, grad_col),
        Some(&RawValue::Number(1.0))
    );

    // Undo via CommandBus
    bus.undo(&mut ds).unwrap();
    let undone_ds = ds.as_ref().unwrap();
    assert_eq!(undone_ds.col_count(), 2);
    assert!(undone_ds.dictionary.find_index("scaled_salary").is_none());
    assert!(undone_ds.dictionary.find_index("grad").is_none());
}
