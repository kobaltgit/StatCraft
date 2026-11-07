use statcraft_app::syntax_runner::run_syntax_batch;
use statcraft_app::views::OutputViewerState;
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};
use statcraft_syntax::CommandBus;

fn make_sample_dataset() -> DataSet {
    let csv = "id,salary,educ,gender\n1,50000,1,1\n2,60000,1,2\n3,75000,2,1\n4,85000,2,2\n5,90000,3,1\n6,110000,3,2\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    DataSet::from_raw_table(table).unwrap()
}

#[test]
fn test_run_syntax_batch_oneway_returns_ascii_tables() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());
    let mut output_view = OutputViewerState::default();

    let sps = "ONEWAY salary BY educ /STATISTICS DESCRIPTIVES HOMOGENEITY.";
    let outcome = run_syntax_batch(sps, &mut ds, &mut bus, &mut output_view).unwrap();

    assert_eq!(outcome.count, 1);
    assert!(!outcome.output_text.is_empty());
    assert!(outcome.output_text.contains("ONEWAY"));
    assert!(outcome.output_text.contains("ANOVA"));
    assert!(outcome.output_text.contains("Between Groups"));
    assert!(outcome.output_text.contains("Within Groups"));
}

#[test]
fn test_run_syntax_batch_help_returns_help_text() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());
    let mut output_view = OutputViewerState::default();

    let sps = "HELP ONEWAY.";
    let outcome = run_syntax_batch(sps, &mut ds, &mut bus, &mut output_view).unwrap();

    assert_eq!(outcome.count, 1);
    assert!(!outcome.output_text.is_empty());
    assert!(outcome.output_text.contains("PROCEDURE: ONEWAY"));
    assert!(outcome.output_text.contains("HOMOGENEITY"));
}

#[test]
fn test_run_syntax_batch_variables_and_anova_combined() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());
    let mut output_view = OutputViewerState::default();

    let sps = r#"
    VARIABLE LEVEL salary (SCALE) /educ (NOMINAL).
    VARIABLE LABELS salary 'Annual Compensation'.
    ONEWAY salary BY educ.
    "#;

    let outcome = run_syntax_batch(sps, &mut ds, &mut bus, &mut output_view).unwrap();

    assert_eq!(outcome.count, 3);
    assert!(!outcome.output_text.is_empty());
    assert!(outcome.output_text.contains("ANOVA"));
}

#[test]
fn test_run_syntax_batch_run_command_directive() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());
    let mut output_view = OutputViewerState::default();

    let sps = r#"RUN COMMAND "transform.shift_values" { "source_var": "salary", "target_var": "salary_lag", "direction": "lag", "step": 1 }."#;
    let outcome = run_syntax_batch(sps, &mut ds, &mut bus, &mut output_view).unwrap();

    assert_eq!(outcome.count, 1);
    assert!(!outcome.output_text.is_empty());
    assert!(outcome.output_text.contains("transform.shift_values"));
}
