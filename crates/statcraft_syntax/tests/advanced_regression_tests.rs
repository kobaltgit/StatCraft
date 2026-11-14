use serde_json::json;
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};
use statcraft_syntax::commands::{CurveFitCommand, LogisticRegressionCommand, PartialCorrCommand};
use statcraft_syntax::{CommandBus, SyntaxDispatcher};

fn make_regression_dataset() -> DataSet {
    let csv = "id,salary,educ,jobtime,outcome\n\
               1,25000,12,24,0\n\
               2,28000,12,36,0\n\
               3,35000,14,48,0\n\
               4,42000,16,60,1\n\
               5,38000,14,30,0\n\
               6,55000,16,72,1\n\
               7,62000,18,84,1\n\
               8,75000,18,96,1\n\
               9,48000,16,50,0\n\
               10,82000,20,108,1\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).expect("valid csv");
    DataSet::from_raw_table(table).expect("valid dataset")
}

#[test]
fn test_partial_corr_syntax_and_command() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_regression_dataset());

    let sps = "PARTIAL CORR /VARIABLES = salary educ WITH jobtime /SIGNIFICANCE = TWOTAIL /MISSING = LISTWISE.";
    let res =
        SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).expect("batch should succeed");
    assert_eq!(res.len(), 1);

    let (cmd, val) = &res[0];
    assert_eq!(cmd.name, "PARTIAL CORR");
    assert_eq!(val["variables"].as_array().unwrap().len(), 2);
    assert_eq!(val["controls"].as_array().unwrap().len(), 1);

    // Test direct CommandBus execution
    let cmd_res = bus
        .execute(
            "stat.correlations.partial",
            &mut ds,
            json!({
                "vars": ["salary", "educ"],
                "controls": ["jobtime"],
                "two_tailed": true,
                "missing": "listwise"
            }),
        )
        .expect("bus execution should succeed");
    assert_eq!(cmd_res["variables"][0], "salary");

    // Test to_sps generator
    let gen_sps = PartialCorrCommand::to_sps(
        &["salary".to_string(), "educ".to_string()],
        &["jobtime".to_string()],
        true,
        "listwise",
    );
    assert!(gen_sps.contains("PARTIAL CORR"));
    assert!(gen_sps.contains("WITH jobtime"));
}

#[test]
fn test_logistic_regression_syntax_and_command() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_regression_dataset());

    let sps = "LOGISTIC REGRESSION VARIABLES outcome /METHOD = ENTER salary educ /CRITERIA = CUT(0.50) CI(95) /PRINT = GOODFIT CI(95).";
    let res =
        SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).expect("batch should succeed");
    assert_eq!(res.len(), 1);

    let (cmd, val) = &res[0];
    assert_eq!(cmd.name, "LOGISTIC REGRESSION");
    assert_eq!(val["dependent_var"], "outcome");
    assert_eq!(val["covariates"].as_array().unwrap().len(), 2);
    assert!(val["coefficients"].as_array().unwrap().len() >= 2);
    assert!(val["model_summary"]["minus_2ll"].is_number());
    assert!(val["classification_table"]["overall_percent_correct"].is_number());

    // Test to_sps generator
    let gen_sps = LogisticRegressionCommand::to_sps(
        "outcome",
        &["salary".to_string(), "educ".to_string()],
        0.5,
        0.95,
        true,
    );
    assert!(gen_sps.contains("LOGISTIC REGRESSION VARIABLES outcome"));
    assert!(gen_sps.contains("/METHOD = ENTER salary educ"));
}

#[test]
fn test_curvefit_syntax_and_command() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_regression_dataset());

    let sps = "CURVEFIT /VARIABLES = salary WITH jobtime /MODEL = LINEAR QUADRATIC /CONSTANT.";
    let res =
        SyntaxDispatcher::execute_batch(sps, &mut ds, &mut bus).expect("batch should succeed");
    assert_eq!(res.len(), 1);

    let (cmd, val) = &res[0];
    assert_eq!(cmd.name, "CURVEFIT");
    assert_eq!(val["dependent_var"], "salary");
    assert_eq!(val["independent_var"], "jobtime");
    let models = val["models"].as_array().unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[0]["model_type"], "Linear");
    assert_eq!(models[1]["model_type"], "Quadratic");

    // Test to_sps generator
    use statcraft_algo::CurveModelType;
    let gen_sps = CurveFitCommand::to_sps(
        "salary",
        "jobtime",
        &[CurveModelType::Linear, CurveModelType::Quadratic],
        true,
    );
    assert!(gen_sps.contains("CURVEFIT"));
    assert!(gen_sps.contains("/MODEL = LINEAR QUADRATIC"));
}
