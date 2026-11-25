use super::*;
use crate::bus::CommandBus;
use crate::syntax_parser::SyntaxDispatcher;
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};

fn make_test_dataset() -> DataSet {
    let csv = "age,income\n20,25000\n30,35000\n40,45000\n50,55000\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    DataSet::from_raw_table(table).unwrap()
}

fn make_missing_dataset() -> DataSet {
    let csv = "x,y\n10,100\n20,\n30,300\n40,400\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    DataSet::from_raw_table(table).unwrap()
}

#[test]
fn test_descriptives_command_execution() {
    let cmd = DescriptivesCommand::new();
    let mut ds = Some(make_test_dataset());

    let params = json!({
        "vars": ["age", "income"]
    });

    assert!(cmd.enabled(ds.as_ref()));

    let res = cmd
        .execute(&mut ds, params)
        .expect("Execution should succeed");
    let results: Vec<DescriptivesResult> = serde_json::from_value(res).expect("Valid JSON");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].variable, "age");
    assert_eq!(results[0].n_valid, 4);
    assert_eq!(results[0].mean, Some(35.0));
    assert_eq!(results[0].min, Some(20.0));
    assert_eq!(results[0].max, Some(50.0));
    assert_eq!(results[0].sum, None);
    assert_eq!(results[0].listwise_n, Some(4));

    assert_eq!(results[1].variable, "income");
    assert_eq!(results[1].mean, Some(40000.0));
}

#[test]
fn test_descriptives_command_missing_variable() {
    let cmd = DescriptivesCommand::new();
    let mut ds = Some(make_test_dataset());

    let params = json!({
        "vars": ["non_existent"]
    });

    let err = cmd.execute(&mut ds, params).unwrap_err();
    match err {
        CommandError::InvalidParams { id, .. } => assert_eq!(id, "stat.descriptives"),
        _ => panic!("Expected InvalidParams error"),
    }
}

#[test]
fn test_descriptives_sps_syntax() {
    let vars = vec!["age".to_string(), "income".to_string()];
    let sps = DescriptivesCommand::to_sps(&vars);
    assert_eq!(
        sps,
        "DESCRIPTIVES VARIABLES=age income\n  /STATISTICS=MEAN STDDEV MIN MAX."
    );
}

#[test]
fn test_descriptives_save_zscores_and_undo() {
    let cmd = DescriptivesCommand::new();
    let mut ds = Some(make_test_dataset());

    let params = json!({
        "vars": ["age"],
        "options": {
            "mean": true,
            "std_dev": true,
            "save_standardized": true
        }
    });

    cmd.execute(&mut ds, params).expect("Execute with SAVE");
    let dataset = ds.as_ref().unwrap();
    let z_idx = dataset
        .dictionary
        .find_index("Zage")
        .expect("Zage column should be created");
    let z0 = dataset.get_cell_numeric(0, z_idx).unwrap();
    let z3 = dataset.get_cell_numeric(3, z_idx).unwrap();
    assert!((z0 + z3).abs() < 1e-6);

    // Collision handling: second run creates ZSC001
    let params2 = json!({
        "vars": ["age"],
        "options": { "mean": true, "save_standardized": true }
    });
    cmd.execute(&mut ds, params2)
        .expect("Second execute with SAVE");
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("ZSC001")
        .is_some());

    // Undo removes ZSC001, then Zage
    cmd.undo(&mut ds).expect("Undo second");
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("ZSC001")
        .is_none());
    assert!(ds.as_ref().unwrap().dictionary.find_index("Zage").is_some());

    cmd.undo(&mut ds).expect("Undo first");
    assert!(ds.as_ref().unwrap().dictionary.find_index("Zage").is_none());
}

#[test]
fn test_descriptives_missing_listwise_and_sort() {
    let cmd = DescriptivesCommand::new();
    let mut ds = Some(make_missing_dataset());

    let params = json!({
        "vars": ["x", "y"],
        "options": {
            "mean": true,
            "sort": "DescendingMeans",
            "missing": "Listwise"
        }
    });

    let res = cmd.execute(&mut ds, params).expect("Execute listwise");
    let results: Vec<DescriptivesResult> = serde_json::from_value(res).unwrap();

    assert_eq!(results[0].variable, "y");
    assert_eq!(results[1].variable, "x");
    assert_eq!(results[0].listwise_n, Some(3));
    assert_eq!(results[1].n_valid, 3);
    // x listwise values: [10, 30, 40] -> mean = 80/3
    assert!((results[1].mean.unwrap() - (80.0 / 3.0)).abs() < 1e-6);
}

#[test]
fn test_descriptives_sps_roundtrip() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());
    let opts = DescriptivesOptions {
        mean: true,
        std_dev: true,
        sum: true,
        save_standardized: true,
        sort: DescriptivesSortOrder::DescendingMeans,
        missing: DescriptivesMissingMode::Listwise,
        ..Default::default()
    };
    let sps = DescriptivesCommand::to_sps_full(&["age".into(), "income".into()], &opts, None);
    let results = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus)
        .expect("Execute generated SPS batch");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.name, "DESCRIPTIVES");

    let desc: Vec<DescriptivesResult> = serde_json::from_value(results[0].1.clone()).unwrap();
    assert_eq!(desc.len(), 2);
    assert_eq!(desc[0].variable, "income");
    assert_eq!(desc[1].variable, "age");
    assert!(desc[0].sum.is_some());
    assert!(desc[0].min.is_none());
    let applied = desc[0].applied_options.as_ref().expect("applied_options");
    assert_eq!(applied, &opts);

    // Z-variables created via SPS round-trip
    assert!(ds.as_ref().unwrap().dictionary.find_index("Zage").is_some());
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("Zincome")
        .is_some());
}
