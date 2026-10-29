use serde_json::json;
use statcraft_algo::{NewValueSpec, OldValueRule, RecodeMapping, RecodePlan};
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions, RawValue};
use statcraft_syntax::commands::{RecodeIntoDifferentCommand, RecodeIntoSameCommand};
use statcraft_syntax::EngineCommand;

fn make_test_dataset() -> DataSet {
    let csv = "age,salary\n17,1000\n25,2500\n45,5000\n70,3000\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    DataSet::from_raw_table(table).unwrap()
}

#[test]
fn test_recode_different_and_undo() {
    let cmd = RecodeIntoDifferentCommand::new();
    let mut ds = Some(make_test_dataset());

    let plan = RecodePlan::new(vec![
        RecodeMapping::new(OldValueRule::RangeLowest(18.0), NewValueSpec::Value(1.0)),
        RecodeMapping::new(OldValueRule::Range(19.0, 60.0), NewValueSpec::Value(2.0)),
        RecodeMapping::new(OldValueRule::RangeHighest(61.0), NewValueSpec::Value(3.0)),
    ]);

    let params = json!({
        "source": "age",
        "target": "age_cat",
        "label": "Age Category",
        "mappings": plan.mappings
    });

    let res = cmd.execute(&mut ds, params).unwrap();
    assert_eq!(res["status"], "success");

    let dataset = ds.as_ref().unwrap();
    assert_eq!(dataset.col_count(), 3);
    let cat_col = dataset.dictionary.find_index("age_cat").unwrap();
    assert_eq!(
        dataset.table.columns[cat_col].values,
        vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(2.0),
            RawValue::Number(3.0)
        ]
    );

    // Test undo
    cmd.undo(&mut ds).unwrap();
    let dataset = ds.as_ref().unwrap();
    assert_eq!(dataset.col_count(), 2);
    assert!(dataset.dictionary.find_index("age_cat").is_none());
}

#[test]
fn test_recode_same_and_undo() {
    let cmd = RecodeIntoSameCommand::new();
    let mut ds = Some(make_test_dataset());

    let plan = RecodePlan::new(vec![
        RecodeMapping::new(OldValueRule::Exact(17.0), NewValueSpec::Value(18.0)),
        RecodeMapping::new(OldValueRule::AllOther, NewValueSpec::CopyOld),
    ]);

    let params = json!({
        "vars": ["age"],
        "mappings": plan.mappings
    });

    let res = cmd.execute(&mut ds, params).unwrap();
    assert_eq!(res["status"], "success");

    let dataset = ds.as_ref().unwrap();
    let age_col = dataset.dictionary.find_index("age").unwrap();
    assert_eq!(
        dataset.table.columns[age_col].values[0],
        RawValue::Number(18.0)
    );

    // Test undo
    cmd.undo(&mut ds).unwrap();
    let dataset = ds.as_ref().unwrap();
    assert_eq!(
        dataset.table.columns[age_col].values[0],
        RawValue::Number(17.0)
    );
}

#[test]
fn test_recode_sps_generation() {
    let plan = RecodePlan::new(vec![
        RecodeMapping::new(OldValueRule::RangeLowest(18.0), NewValueSpec::Value(1.0)),
        RecodeMapping::new(OldValueRule::Range(19.0, 35.0), NewValueSpec::Value(2.0)),
        RecodeMapping::new(OldValueRule::Range(36.0, 60.0), NewValueSpec::Value(3.0)),
        RecodeMapping::new(OldValueRule::RangeHighest(61.0), NewValueSpec::Value(4.0)),
        RecodeMapping::new(OldValueRule::AllOther, NewValueSpec::SysMissing),
    ]);

    let pairs = [("age", "age_group", Some("Age Group Categories"))];
    let diff_sps = RecodeIntoDifferentCommand::to_sps(&pairs, &plan);
    assert!(diff_sps.contains("RECODE age (LOWEST THRU 18=1) (19 THRU 35=2) (36 THRU 60=3) (61 THRU HIGHEST=4) (ELSE=SYSMIS) INTO age_group."));
    assert!(diff_sps.contains("VARIABLE LABELS age_group \"Age Group Categories\"."));
    assert!(diff_sps.contains("EXECUTE."));

    let same_sps = RecodeIntoSameCommand::to_sps(&["age", "salary"], &plan);
    assert!(same_sps.contains("RECODE age salary (LOWEST THRU 18=1) (19 THRU 35=2) (36 THRU 60=3) (61 THRU HIGHEST=4) (ELSE=SYSMIS)."));
    assert!(same_sps.contains("EXECUTE."));
}
