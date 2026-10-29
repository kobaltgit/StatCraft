use serde_json::json;
use statcraft_core::{DataSet, RawValue};
use statcraft_syntax::command::EngineCommand;
use statcraft_syntax::commands::{
    DescriptivesCommand, SelectCasesAction, SelectCasesCommand, SelectCasesMode,
};

fn create_test_dataset() -> DataSet {
    let mut ds = DataSet::blank(5, 2).unwrap();
    // VAR00001: age -> 15, 20, 25, 30, 10
    // VAR00002: salary -> 1000, 2000, 3000, 4000, 500
    let ages = [15.0, 20.0, 25.0, 30.0, 10.0];
    let salaries = [1000.0, 2000.0, 3000.0, 4000.0, 500.0];

    for r in 0..5 {
        ds.set_cell(r, 0, RawValue::Number(ages[r])).unwrap();
        ds.set_cell(r, 1, RawValue::Number(salaries[r])).unwrap();
    }
    ds
}

#[test]
fn test_select_cases_by_condition_and_undo() {
    let mut ds = Some(create_test_dataset());
    let cmd = SelectCasesCommand::new();

    let params = json!({
        "mode": "condition",
        "condition": "VAR00001 >= 20",
        "action": "filter"
    });

    let res = cmd.execute(&mut ds, params).unwrap();
    assert_eq!(res["mode"], "condition");
    assert_eq!(res["selected"], 3);

    let active_ds = ds.as_ref().unwrap();
    assert_eq!(active_ds.active_filter_var(), Some("filter_$"));
    assert_eq!(active_ds.active_row_indices(), vec![1, 2, 3]);

    // Test undo
    cmd.undo(&mut ds).unwrap();
    let restored_ds = ds.as_ref().unwrap();
    assert_eq!(restored_ds.active_filter_var(), None);
    assert_eq!(restored_ds.active_row_indices(), vec![0, 1, 2, 3, 4]);
}

#[test]
fn test_select_cases_delete_unselected_and_undo() {
    let mut ds = Some(create_test_dataset());
    let cmd = SelectCasesCommand::new();

    let params = json!({
        "mode": "condition",
        "condition": "VAR00001 >= 25",
        "action": "delete"
    });

    let res = cmd.execute(&mut ds, params).unwrap();
    assert_eq!(res["mode"], "condition");
    assert_eq!(res["kept_rows"], 2);

    let active_ds = ds.as_ref().unwrap();
    assert_eq!(active_ds.row_count(), 2);
    assert_eq!(active_ds.active_filter_var(), None);

    // Test undo restores deleted cases
    cmd.undo(&mut ds).unwrap();
    let restored_ds = ds.as_ref().unwrap();
    assert_eq!(restored_ds.row_count(), 5);
}

#[test]
fn test_descriptives_respects_filter() {
    let mut ds = Some(create_test_dataset());
    let select_cmd = SelectCasesCommand::new();
    let desc_cmd = DescriptivesCommand;

    // Descriptives before filter
    let desc_params = json!({
        "vars": ["VAR00001"]
    });
    let res1 = desc_cmd.execute(&mut ds, desc_params.clone()).unwrap();
    assert_eq!(res1[0]["n_valid"], 5);
    // Mean of 15, 20, 25, 30, 10 is 20.0
    assert_eq!(res1[0]["mean"], 20.0);

    // Apply filter: age >= 20 (keeps 20, 25, 30 -> mean 25.0)
    let filter_params = json!({
        "mode": "condition",
        "condition": "VAR00001 >= 20",
        "action": "filter"
    });
    select_cmd.execute(&mut ds, filter_params).unwrap();

    let res2 = desc_cmd.execute(&mut ds, desc_params).unwrap();
    assert_eq!(res2[0]["n_valid"], 3);
    assert_eq!(res2[0]["mean"], 25.0);

    // Turn filter off (mode "all")
    let all_params = json!({
        "mode": "all"
    });
    select_cmd.execute(&mut ds, all_params).unwrap();
    let active_ds = ds.as_ref().unwrap();
    assert_eq!(active_ds.active_filter_var(), None);
    assert_eq!(active_ds.active_row_indices().len(), 5);
}

#[test]
fn test_select_cases_to_sps() {
    let sps_all = SelectCasesCommand::to_sps(&SelectCasesMode::All, &SelectCasesAction::Filter);
    assert!(sps_all.contains("FILTER OFF."));

    let sps_cond = SelectCasesCommand::to_sps(
        &SelectCasesMode::Condition("age >= 18".into()),
        &SelectCasesAction::Filter,
    );
    assert!(sps_cond.contains("COMPUTE filter_$ = (age >= 18)."));
    assert!(sps_cond.contains("FILTER BY filter_$."));

    let sps_del = SelectCasesCommand::to_sps(
        &SelectCasesMode::Condition("age >= 18".into()),
        &SelectCasesAction::Delete,
    );
    assert!(sps_del.contains("SELECT IF (age >= 18)."));
}
