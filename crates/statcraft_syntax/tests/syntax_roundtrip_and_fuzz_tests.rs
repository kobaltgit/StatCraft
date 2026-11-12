use statcraft_algo::{
    DescriptivesResult, ExploreResult, FrequencyResult, SortDirection, SortKeySpec,
};
use statcraft_core::{DataSet, RawValue, SplitFileMode};
use statcraft_io::{load_csv_from_reader, CsvOptions};
use statcraft_syntax::commands::{
    DescriptivesCommand, ExploreCommand, FrequenciesCommand, IndependentSamplesTTestCommand,
    SelectCasesAction, SelectCasesCommand, SelectCasesMode, SortCasesCommand, SplitFileCommand,
    WeightCasesCommand,
};
use statcraft_syntax::{CommandBus, SyntaxDispatcher};

fn make_test_dataset() -> DataSet {
    let csv = r#"gender,salary,age,educ
1,35000,25,12
2,45000,30,16
1,55000,35,16
2,65000,40,18
1,75000,45,20
"#;
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    DataSet::from_raw_table(table).unwrap()
}

#[test]
fn test_roundtrip_descriptives() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let vars = vec!["salary".to_string(), "age".to_string()];
    let sps = DescriptivesCommand::to_sps(&vars);
    let results = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus)
        .expect("Descriptives batch should succeed");

    assert_eq!(results.len(), 1);
    let desc: Vec<DescriptivesResult> = serde_json::from_value(results[0].1.clone()).unwrap();
    assert_eq!(desc.len(), 2);
    assert_eq!(desc[0].variable, "salary");
    assert_eq!(desc[0].n_valid, 5);
    assert_eq!(desc[0].mean, Some(55000.0));
    assert_eq!(desc[1].variable, "age");
    assert_eq!(desc[1].mean, Some(35.0));
}

#[test]
fn test_roundtrip_frequencies() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let vars = vec!["gender".to_string()];
    let sps = FrequenciesCommand::to_sps(&vars);
    let results = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus)
        .expect("Frequencies batch should succeed");

    assert_eq!(results.len(), 1);
    let freq: Vec<FrequencyResult> = serde_json::from_value(results[0].1.clone()).unwrap();
    assert_eq!(freq.len(), 1);
    assert_eq!(freq[0].variable, "gender");
    assert_eq!(freq[0].n_valid, 5);
}

#[test]
fn test_roundtrip_sort_cases() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let keys = vec![
        SortKeySpec::new("salary", SortDirection::Descending),
        SortKeySpec::new("age", SortDirection::Ascending),
    ];
    let sps = SortCasesCommand::to_sps(&keys);
    let results = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus)
        .expect("Sort cases batch should succeed");

    assert_eq!(results.len(), 1);
    let active_ds = ds.as_ref().unwrap();
    let col_idx = active_ds.dictionary.find_index("salary").unwrap();
    assert_eq!(
        active_ds.get_cell(0, col_idx),
        Some(&RawValue::Number(75000.0))
    );
}

#[test]
fn test_roundtrip_split_file() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let vars = vec!["gender".to_string()];
    let sps = SplitFileCommand::to_sps(SplitFileMode::CompareGroups, &vars, false);
    let results = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus)
        .expect("Split file batch should succeed");

    assert!(!results.is_empty());
    assert_eq!(ds.as_ref().unwrap().split_file().unwrap().variables, vars);
}

#[test]
fn test_roundtrip_weight_cases() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let sps = WeightCasesCommand::to_sps(Some("salary"));
    let results = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus)
        .expect("Weight cases batch should succeed");

    assert!(!results.is_empty());
    assert_eq!(ds.as_ref().unwrap().weight_var, Some("salary".to_string()));
}

#[test]
fn test_roundtrip_select_cases() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let sps = SelectCasesCommand::to_sps(
        &SelectCasesMode::FilterVar("gender".to_string()),
        &SelectCasesAction::Filter,
    );
    let results = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus)
        .expect("Select cases batch should succeed");

    assert!(!results.is_empty());
    assert_eq!(
        ds.as_ref().unwrap().active_filter_var,
        Some("gender".to_string())
    );
}

#[test]
fn test_roundtrip_ttest_independent() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let vars = vec!["salary".to_string()];
    let sps = IndependentSamplesTTestCommand::to_sps(&vars, "gender", "1", "2", 0.95);
    let results = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus)
        .expect("T-test batch should succeed");

    assert_eq!(results.len(), 1);
    let val = &results[0].1;
    assert!(val.is_array());
}

#[test]
fn test_never_crash_on_fuzzed_and_malformed_syntax() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let malformed_inputs = [
        "",
        "   ",
        "\n\n\n\t",
        ".",
        "...",
        "* Just a comment without terminating dot",
        "GET FILE='unclosed_quote.sav",
        "GET FILE=\"unclosed_double_quote.sav",
        "DESCRIPTIVES VARIABLES=.",
        "DESCRIPTIVES /STATISTICS=.",
        "CROSSTABS /TABLES=salary.",
        "T-TEST GROUPS=gender(1) /VARIABLES=salary.",
        "T-TEST GROUPS=gender /VARIABLES=salary.",
        "ONEWAY salary.",
        "SORT CASES BY.",
        "COMPUTE invalid_expr",
        "UNKNOWN_PROCEDURE_NAME_XYZ /PARAM=1.",
        "/// / / = = = .",
        "DESCRIPTIVES VARIABLES=var1 TO var999.",
        "WEIGHT BY .",
        "FILTER BY .",
    ];

    for (idx, malformed) in malformed_inputs.iter().enumerate() {
        // Must NEVER crash or panic! Either Ok (for empty/comment) or Err(SyntaxError).
        let res = SyntaxDispatcher::execute_batch(malformed, &mut ds, &mut bus);
        match res {
            Ok(_cmds) => {
                // If it succeeded, verify it was safe
                assert!(
                    malformed.trim().is_empty()
                        || malformed.trim().starts_with('*')
                        || malformed.trim().starts_with("DESCRIPTIVES")
                        || malformed.trim() == ".",
                    "Case #{} unexpectedly succeeded: {}",
                    idx,
                    malformed
                );
            }
            Err(err) => {
                // Verify that a typed SyntaxError is produced without panics
                let msg = err.to_string();
                assert!(!msg.is_empty());
            }
        }
    }
}

#[test]
fn test_roundtrip_explore() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let deps = vec!["salary".to_string()];
    let factors = vec!["gender".to_string()];
    let sps = ExploreCommand::to_sps(&deps, &factors);
    let results = SyntaxDispatcher::execute_batch(&sps, &mut ds, &mut bus)
        .expect("Explore batch should succeed");

    assert_eq!(results.len(), 1);
    let res: ExploreResult = serde_json::from_value(results[0].1.clone()).unwrap();
    assert_eq!(res.variables.len(), 1);
    assert_eq!(res.variables[0].variable, "salary");
    assert_eq!(res.variables[0].factor_var.as_deref(), Some("gender"));
    assert_eq!(res.variables[0].groups.len(), 2);
}
