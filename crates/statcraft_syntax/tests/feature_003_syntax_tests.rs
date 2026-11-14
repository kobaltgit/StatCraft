//! Integration tests for Feature 003 syntax parsing, execution, and Undo/Redo.

use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};
use statcraft_syntax::commands::{
    CodebookCommand, KMeansClusterCommand, KaplanMeierCommand, LifeTablesCommand, SummarizeCommand,
};
use statcraft_syntax::{CommandBus, SyntaxDispatcher};

fn make_test_dataset() -> DataSet {
    let csv = "\
x1,x2,group,time,status
1.0,2.0,G1,5.0,1.0
1.5,1.8,G1,10.0,0.0
10.0,10.5,G2,15.0,1.0
9.8,10.2,G2,20.0,1.0
";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    DataSet::from_raw_table(table).unwrap()
}

#[test]
fn test_quick_cluster_syntax_and_undo_redo() {
    let mut ds = Some(make_test_dataset());
    let mut bus = CommandBus::with_defaults();

    let sps = KMeansClusterCommand::to_sps(
        &["x1".to_string(), "x2".to_string()],
        2,
        10,
        0.0001,
        true,
        true,
    );
    assert!(sps.starts_with("QUICK CLUSTER x1 x2"));
    assert!(sps.contains("/SAVE = CLUSTER DISTANCE."));

    // Initial column count: 5
    assert_eq!(ds.as_ref().unwrap().col_count(), 5);

    // Execute via SyntaxDispatcher
    let res = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus).expect("dispatch ok");
    assert!(!res.is_empty());
    let val = &res[0].1;
    assert!(val.get("cluster_assignments").is_some());

    // After save: QCL_1 and QCL_2 added -> 7 columns
    assert_eq!(ds.as_ref().unwrap().col_count(), 7);
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("QCL_1")
        .is_some());
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("QCL_2")
        .is_some());

    // Undo -> back to 5 columns
    bus.undo(&mut ds).expect("undo ok");
    assert_eq!(ds.as_ref().unwrap().col_count(), 5);
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("QCL_1")
        .is_none());
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("QCL_2")
        .is_none());

    // Redo -> back to 7 columns
    bus.redo(&mut ds).expect("redo ok");
    assert_eq!(ds.as_ref().unwrap().col_count(), 7);
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("QCL_1")
        .is_some());
}

#[test]
fn test_codebook_and_summarize_syntax() {
    let mut ds = Some(make_test_dataset());
    let mut bus = CommandBus::with_defaults();

    // Codebook
    let sps_cb = CodebookCommand::to_sps(&["x1".to_string(), "group".to_string()], 50);
    let res_cb =
        SyntaxDispatcher::execute_batch(&sps_cb, &mut ds, &mut bus).expect("dispatch codebook ok");
    assert!(!res_cb.is_empty());
    assert_eq!(res_cb[0].1["total_variables"].as_u64(), Some(2));

    // Summarize
    let sps_sm = SummarizeCommand::to_sps(&["x1".to_string()], &["group".to_string()], Some(100));
    let res_sm =
        SyntaxDispatcher::execute_batch(&sps_sm, &mut ds, &mut bus).expect("dispatch summarize ok");
    assert!(!res_sm.is_empty());
    assert_eq!(res_sm[0].1["total_cases"].as_u64(), Some(4));
}

#[test]
fn test_km_and_survival_syntax() {
    let mut ds = Some(make_test_dataset());
    let mut bus = CommandBus::with_defaults();

    // Kaplan-Meier
    let sps_km = KaplanMeierCommand::to_sps("time", "status", 1.0, Some("group"));
    let res_km =
        SyntaxDispatcher::execute_batch(&sps_km, &mut ds, &mut bus).expect("dispatch km ok");
    assert!(!res_km.is_empty());
    assert!(res_km[0].1.get("groups").is_some());

    // Life Tables
    let sps_surv = LifeTablesCommand::to_sps("time", "status", 1.0, 30.0, 10.0);
    let res_surv = SyntaxDispatcher::execute_batch(&sps_surv, &mut ds, &mut bus)
        .expect("dispatch survival ok");
    assert!(!res_surv.is_empty());
    assert_eq!(res_surv[0].1["total_cases"].as_u64(), Some(4));
}
