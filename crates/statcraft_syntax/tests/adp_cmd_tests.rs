use serde_json::json;
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use statcraft_syntax::commands::PrepareDataModelingCommand;
use statcraft_syntax::CommandBus;

fn create_sample_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    ds.add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("educ", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
        .unwrap();
    ds.add_variable("jobcat", DataType::String { max_len: 20 }, Measure::Nominal)
        .unwrap();

    let salaries = [
        30000.0, 35000.0, 42000.0, 48000.0, 55000.0, 62000.0, 71000.0, 80000.0,
    ];
    let educs = [12.0, 12.0, 14.0, 14.0, 16.0, 16.0, 18.0, 19.0];
    let jobcats = [
        "Clerical",
        "Clerical",
        "Clerical",
        "Custodial",
        "Manager",
        "Manager",
        "Manager",
        "Director",
    ];

    for _ in 0..8 {
        ds.add_empty_row();
    }

    for i in 0..8 {
        ds.set_cell(i, 0, RawValue::Number(salaries[i])).unwrap();
        ds.set_cell(i, 1, RawValue::Number(educs[i])).unwrap();
        ds.set_cell(i, 2, RawValue::Text(jobcats[i].to_string()))
            .unwrap();
    }

    ds
}

#[test]
fn test_adp_cmd_sps_syntax() {
    let opts = statcraft_algo::AdpOptions {
        objective: statcraft_algo::AdpObjective::OptimizeAccuracy,
        target_variable: Some("salary".to_string()),
        input_variables: vec!["educ".to_string(), "jobcat".to_string()],
        rescale: statcraft_algo::AdpRescaleMethod::ZScore,
        partition: statcraft_algo::AdpPartitionOptions {
            enabled: true,
            train_pct: 70.0,
            test_pct: 20.0,
            validate_pct: 10.0,
            seed: Some(2026),
        },
        ..Default::default()
    };

    let sps = PrepareDataModelingCommand::to_sps(&opts);
    assert!(sps.starts_with("ADP\n"));
    assert!(sps.contains("/FIELDS TARGET=salary INPUTS=educ jobcat"));
    assert!(sps.contains("/OBJECTIVE ACCURACY"));
    assert!(sps.contains("/OUTLIERS METHOD=ZSCORE(3.0) ACTION=WINSORIZE"));
    assert!(sps.contains("/RESCALE METHOD=ZSCORE"));
    assert!(sps.contains("/PARTITION TRAIN=70 TEST=20 VALIDATE=10 SEED=2026."));
    assert!(sps.ends_with("EXECUTE."));
}

#[test]
fn test_adp_cmd_execute_and_undo() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(create_sample_dataset());

    let initial_cols = ds.as_ref().unwrap().col_count();
    assert_eq!(initial_cols, 3);

    let params = json!({
        "inputs": ["educ", "jobcat"],
        "target": "salary",
        "objective": "accuracy",
        "rescale_method": "zscore",
        "partition": true,
        "seed": 2026
    });

    let res = bus.execute("transform.adp", &mut ds, params).unwrap();
    assert_eq!(res["total_inputs"], 2);
    assert_eq!(res["recommended_count"], 2);

    let current_cols = ds.as_ref().unwrap().col_count();
    // Created P_educ, P_jobcat, and _Partition_
    assert_eq!(current_cols, initial_cols + 3);

    // Verify created variable names exist in dictionary
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("P_educ")
        .is_some());
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("P_jobcat")
        .is_some());
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("_Partition_")
        .is_some());

    // Test Undo
    bus.undo(&mut ds).unwrap();
    assert_eq!(ds.as_ref().unwrap().col_count(), initial_cols);
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("P_educ")
        .is_none());
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("P_jobcat")
        .is_none());
    assert!(ds
        .as_ref()
        .unwrap()
        .dictionary
        .find_index("_Partition_")
        .is_none());
}
