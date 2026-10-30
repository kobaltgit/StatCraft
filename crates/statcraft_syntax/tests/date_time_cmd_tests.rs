use serde_json::json;
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use statcraft_syntax::command::EngineCommand;
use statcraft_syntax::commands::DateTimeWizardCommand;

fn create_test_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    let col1_idx = ds
        .add_variable("date1", DataType::String { max_len: 20 }, Measure::Nominal)
        .unwrap();
    let col2_idx = ds
        .add_variable("date2", DataType::String { max_len: 20 }, Measure::Nominal)
        .unwrap();

    ds.table.columns[col1_idx].values = vec![
        RawValue::Text("2024-05-15".to_string()),
        RawValue::Text("2023-01-01".to_string()),
    ];
    ds.table.columns[col2_idx].values = vec![
        RawValue::Text("2020-05-15".to_string()),
        RawValue::Text("2022-01-01".to_string()),
    ];
    ds
}

#[test]
fn test_date_time_cmd_extract_and_undo() {
    let cmd = DateTimeWizardCommand::new();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "operation": "extract",
        "target_var": "year_out",
        "primary_var": "date1",
        "part": "year",
        "target_label": "Extracted Year"
    });

    let res = cmd.execute(&mut ds, params).unwrap();
    assert_eq!(res["status"], "success");

    let current_ds = ds.as_ref().unwrap();
    assert_eq!(current_ds.col_count(), 3);
    let col = &current_ds.table.columns[2];
    assert_eq!(col.values[0], RawValue::Number(2024.0));
    assert_eq!(col.values[1], RawValue::Number(2023.0));

    cmd.undo(&mut ds).unwrap();
    let undid_ds = ds.as_ref().unwrap();
    assert_eq!(undid_ds.col_count(), 2);
}

#[test]
fn test_date_time_cmd_diff() {
    let cmd = DateTimeWizardCommand::new();
    let mut ds = Some(create_test_dataset());

    let params = json!({
        "operation": "diff",
        "target_var": "diff_years",
        "primary_var": "date1",
        "secondary_var": "date2",
        "unit": "years"
    });

    let res = cmd.execute(&mut ds, params).unwrap();
    assert_eq!(res["status"], "success");

    let current_ds = ds.as_ref().unwrap();
    let col = &current_ds.table.columns[2];
    assert_eq!(col.values[0], RawValue::Number(4.0));
    assert_eq!(col.values[1], RawValue::Number(1.0));
}

#[test]
fn test_date_time_cmd_sps_syntax() {
    let sps_extract = DateTimeWizardCommand::to_sps(
        "extract",
        "birth_year",
        "birth_date",
        None,
        Some("year"),
        None,
        None,
        Some("Birth Year"),
    );
    assert!(sps_extract.contains("COMPUTE birth_year = XDATE.YEAR(birth_date)."));
    assert!(sps_extract.contains("VARIABLE LABELS birth_year \"Birth Year\"."));

    let sps_diff = DateTimeWizardCommand::to_sps(
        "diff",
        "age_years",
        "date2",
        Some("date1"),
        None,
        Some("years"),
        None,
        None,
    );
    assert!(sps_diff.contains("COMPUTE age_years = DATEDIFF(date2, date1, \"years\")."));

    let sps_add = DateTimeWizardCommand::to_sps(
        "add",
        "due_date",
        "start_date",
        None,
        None,
        Some("days"),
        Some(30.0),
        None,
    );
    assert!(sps_add.contains("COMPUTE due_date = DATESUM(start_date, 30, \"days\")."));
}
