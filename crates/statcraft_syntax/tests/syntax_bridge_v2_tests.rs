use statcraft_core::{DataSet, Measure};
use statcraft_io::{load_csv_from_reader, CsvOptions};
use statcraft_syntax::{CommandBus, SyntaxDispatcher};

fn make_sample_dataset() -> DataSet {
    let csv = "id,salary,educ,gender\n1,50000,1,1\n2,60000,1,2\n3,75000,2,1\n4,85000,2,2\n5,90000,3,1\n6,110000,3,2\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    DataSet::from_raw_table(table).unwrap()
}

#[test]
fn test_help_syntax_command_general_and_specific() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());

    // 1. General HELP
    let sps_gen = "HELP.";
    let res_gen = SyntaxDispatcher::execute_batch(sps_gen, &mut ds, &mut bus).unwrap();
    assert_eq!(res_gen.len(), 1);
    let (cmd, val) = &res_gen[0];
    assert_eq!(cmd.name, "HELP");
    let out_str = val["output"].as_str().unwrap();
    assert!(out_str.contains("SPSS Syntax Quick Reference"));
    assert!(out_str.contains("DESCRIPTIVES"));
    assert!(out_str.contains("ONEWAY"));

    // 2. Specific HELP ONEWAY
    let sps_oneway = "HELP ONEWAY.";
    let res_oneway = SyntaxDispatcher::execute_batch(sps_oneway, &mut ds, &mut bus).unwrap();
    assert_eq!(res_oneway.len(), 1);
    let out_oneway = res_oneway[0].1["output"].as_str().unwrap();
    assert!(out_oneway.contains("PROCEDURE: ONEWAY"));
    assert!(out_oneway.contains("HOMOGENEITY"));
}

#[test]
fn test_variable_metadata_syntax_commands() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());

    let sps = r#"
    VARIABLE LEVEL salary (SCALE) /gender educ (NOMINAL).
    VARIABLE LABELS salary 'Annual Base Salary' gender 'Biological Sex'.
    VALUE LABELS gender 1 'Male' 2 'Female'.
    MISSING VALUES salary (99999).
    "#;

    let res = SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).unwrap();
    assert_eq!(res.len(), 4);

    let active_ds = ds.as_ref().unwrap();
    let dict = &active_ds.dictionary;

    // Check measures
    let salary_idx = dict.find_index("salary").unwrap();
    assert_eq!(dict.get(salary_idx).unwrap().measure, Measure::Scale);
    let gender_idx = dict.find_index("gender").unwrap();
    assert_eq!(dict.get(gender_idx).unwrap().measure, Measure::Nominal);

    // Check labels
    assert_eq!(dict.get(salary_idx).unwrap().label, "Annual Base Salary");
    assert_eq!(dict.get(gender_idx).unwrap().label, "Biological Sex");

    // Check value labels
    let val_labels = &dict.get(gender_idx).unwrap().values;
    assert_eq!(
        val_labels.mapping.get("1").map(|s| s.as_str()),
        Some("Male")
    );
    assert_eq!(
        val_labels.mapping.get("2").map(|s| s.as_str()),
        Some("Female")
    );

    // Check missing values
    let missing = &dict.get(salary_idx).unwrap().missing;
    assert!(missing.discrete.contains(&"99999".to_string()));
}

#[test]
fn test_oneway_anova_syntax_execution() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());

    let sps = "ONEWAY salary BY educ /STATISTICS DESCRIPTIVES HOMOGENEITY.";
    let res = SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).unwrap();
    assert_eq!(res.len(), 1);

    let (cmd, val) = &res[0];
    assert_eq!(cmd.name, "ONEWAY");
    let anova_res = val.as_array().expect("Expected array of OneWayAnovaResult");
    assert_eq!(anova_res.len(), 1);
    let item = &anova_res[0];
    assert_eq!(item["dep_var"], "salary");
    assert_eq!(item["factor_var"], "educ");
    assert!(item["between_groups"]["f_stat"].as_f64().unwrap() > 0.0);
    assert!(item["between_groups"]["p_value"].as_f64().unwrap() >= 0.0);
}

#[test]
fn test_unknown_command_did_you_mean_suggestion() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());

    let sps = "ONEWAY_TEST salary BY educ.";
    let err = SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).unwrap_err();

    let err_msg = err.to_string();
    assert!(err_msg.contains("Did you mean 'ONEWAY'?"));
    assert!(err_msg.contains("HELP."));
}
