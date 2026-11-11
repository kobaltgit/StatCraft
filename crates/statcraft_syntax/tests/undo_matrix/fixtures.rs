use serde_json::Value;
use statcraft_core::variable::{Alignment, DataType, Measure, MissingValues, ValueLabels};
use statcraft_core::DataSet;
use statcraft_io::{RawColumn, RawDataType, RawTable, RawValue};
use statcraft_syntax::bus::CommandBus;

pub fn create_test_dataset() -> DataSet {
    let col_id = RawColumn {
        name: "id".to_string(),
        data_type: RawDataType::Numeric,
        values: vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(3.0),
            RawValue::Number(4.0),
            RawValue::Number(5.0),
            RawValue::Number(6.0),
        ],
    };

    let col_gender = RawColumn {
        name: "gender".to_string(),
        data_type: RawDataType::Numeric,
        values: vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(1.0),
            RawValue::Number(2.0),
        ],
    };

    let col_salary = RawColumn {
        name: "salary".to_string(),
        data_type: RawDataType::Numeric,
        values: vec![
            RawValue::Number(2500.0),
            RawValue::Number(3200.0),
            RawValue::Number(4100.0),
            RawValue::Number(2900.0),
            RawValue::Number(5500.0),
            RawValue::Null,
        ],
    };

    let col_dept = RawColumn {
        name: "dept".to_string(),
        data_type: RawDataType::Text,
        values: vec![
            RawValue::Text("Sales".to_string()),
            RawValue::Text("Eng".to_string()),
            RawValue::Text("Sales".to_string()),
            RawValue::Text("HR".to_string()),
            RawValue::Text("Eng".to_string()),
            RawValue::Text("HR".to_string()),
        ],
    };

    let col_date = RawColumn {
        name: "hiredate".to_string(),
        data_type: RawDataType::Text,
        values: vec![
            RawValue::Text("2020-01-15".to_string()),
            RawValue::Text("2021-03-20".to_string()),
            RawValue::Text("2019-11-05".to_string()),
            RawValue::Text("2022-07-01".to_string()),
            RawValue::Text("2018-05-12".to_string()),
            RawValue::Text("2023-09-30".to_string()),
        ],
    };

    let table = RawTable {
        columns: vec![col_id, col_gender, col_salary, col_dept, col_date],
        row_count: 6,
    };

    let mut ds = DataSet::from_raw_table(table).expect("Should create dataset from raw table");

    // Configure metadata for id
    if let Some(m) = ds.dictionary.get_mut(0) {
        m.var_type = DataType::Numeric { decimals: 0 };
        m.measure = Measure::Scale;
        m.decimals = 0;
    }

    // Configure metadata for gender
    if let Some(m) = ds.dictionary.get_mut(1) {
        m.measure = Measure::Nominal;
        m.label = "Respondent Gender".to_string();
        let mut vl = ValueLabels::new();
        vl.insert("1", "Male");
        vl.insert("2", "Female");
        m.values = vl;
    }

    // Configure metadata for salary
    if let Some(m) = ds.dictionary.get_mut(2) {
        m.measure = Measure::Scale;
        m.label = "Monthly Salary (USD)".to_string();
        m.decimals = 2;
        m.missing = MissingValues {
            discrete: vec!["-99".to_string()],
            range: None,
        };
    }

    // Configure metadata for dept
    if let Some(m) = ds.dictionary.get_mut(3) {
        m.var_type = DataType::String { max_len: 12 };
        m.width = 12;
        m.measure = Measure::Nominal;
        m.align = Alignment::Left;
        m.label = "Department".to_string();
    }

    // Configure metadata for hiredate
    if let Some(m) = ds.dictionary.get_mut(4) {
        m.var_type = DataType::Date {
            format: "yyyy-mm-dd".to_string(),
        };
        m.measure = Measure::Scale;
        m.label = "Date of Employment".to_string();
    }

    ds
}

pub fn assert_reversibility_cycle(
    bus: &mut CommandBus,
    cmd_id: &str,
    params: Value,
    dataset: &mut Option<DataSet>,
) {
    let initial_ds = dataset
        .as_ref()
        .expect("Dataset must be present before execute")
        .clone();

    // 1. Execute
    let res = bus
        .execute(cmd_id, dataset, params.clone())
        .unwrap_or_else(|e| panic!("Execution of '{}' failed: {:?}", cmd_id, e));
    assert!(
        res.is_object() || res.is_array() || res.is_null() || res.is_boolean(),
        "Command '{}' returned invalid JSON",
        cmd_id
    );

    let mutated_ds = dataset
        .as_ref()
        .expect("Dataset must be present after execute")
        .clone();

    // 2. Undo -> Must match initial byte-for-byte
    bus.undo(dataset)
        .unwrap_or_else(|e| panic!("Undo of '{}' failed: {:?}", cmd_id, e));
    let restored_ds = dataset
        .as_ref()
        .expect("Dataset must be present after undo");
    assert_eq!(
        restored_ds, &initial_ds,
        "Undo of '{}' failed mathematical reversibility invariant (restored != initial)",
        cmd_id
    );

    // 3. Redo -> Must match mutated byte-for-byte
    bus.redo(dataset)
        .unwrap_or_else(|e| panic!("Redo of '{}' failed: {:?}", cmd_id, e));
    let redone_ds = dataset
        .as_ref()
        .expect("Dataset must be present after redo");
    assert_eq!(
        redone_ds, &mutated_ds,
        "Redo of '{}' failed mathematical reversibility invariant (redone != mutated)",
        cmd_id
    );

    // 4. Second Undo -> Return back to clean state
    bus.undo(dataset)
        .unwrap_or_else(|e| panic!("Second undo of '{}' failed: {:?}", cmd_id, e));
    let final_ds = dataset
        .as_ref()
        .expect("Dataset must be present after second undo");
    assert_eq!(
        final_ds, &initial_ds,
        "Second undo of '{}' failed to restore original state",
        cmd_id
    );
}
