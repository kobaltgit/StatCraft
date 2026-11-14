use serde_json::json;
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};
use statcraft_syntax::commands::{
    MultivariateGlmCommand, RepeatedGlmCommand, UnivariateGlmCommand,
};
use statcraft_syntax::{CommandBus, SyntaxDispatcher};

fn make_test_dataset() -> DataSet {
    let csv = "\
score,y1,y2,group,gender,age,t1,t2,t3
10.0,5.0,15.0,A,M,20.0,10.0,14.0,18.0
11.0,6.0,14.0,A,F,22.0,11.0,15.0,19.0
12.0,7.0,13.0,A,M,24.0,12.0,16.0,20.0
13.0,8.0,12.0,A,F,26.0,13.0,17.0,21.0
14.0,9.0,11.0,A,M,28.0,14.0,18.0,22.0
15.0,10.0,10.0,B,F,30.0,15.0,19.0,23.0
16.0,11.0,9.0,B,M,32.0,16.0,20.0,24.0
17.0,12.0,8.0,B,F,34.0,17.0,21.0,25.0
18.0,13.0,7.0,B,M,36.0,18.0,22.0,26.0
19.0,14.0,6.0,B,F,38.0,19.0,23.0,27.0
";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    DataSet::from_raw_table(table).unwrap()
}

#[test]
fn test_unianova_sps_generation_and_execution() {
    let sps = UnivariateGlmCommand::to_sps(
        "score",
        &["group".to_string(), "gender".to_string()],
        &["age".to_string()],
        true,
    );
    assert!(sps.starts_with("UNIANOVA score BY group gender WITH age"));
    assert!(sps.contains("/METHOD = SSTYPE(3)"));

    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let val = bus
        .execute(
            "stat.glm.univariate",
            &mut ds,
            json!({
                "dependent": "score",
                "fixed_factors": ["group", "gender"],
                "covariates": ["age"],
                "intercept": true,
            }),
        )
        .expect("UNIANOVA command execution must succeed");

    assert_eq!(val["dependent"], "score");
    assert_eq!(val["n_valid"], 10);
    assert!(val["tests"].as_array().unwrap().len() >= 6);
}

#[test]
fn test_unianova_syntax_parser_dispatch() {
    let syntax = "UNIANOVA score BY group WITH age\n  /METHOD = SSTYPE(3)\n  /INTERCEPT = INCLUDE.";
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let res = SyntaxDispatcher::execute_batch(syntax, &mut ds, &mut bus)
        .expect("Dispatch UNIANOVA must succeed");

    assert_eq!(res.len(), 1);
    let val = &res[0].1;
    assert_eq!(val["dependent"], "score");
    assert_eq!(val["n_valid"], 10);
}

#[test]
fn test_multivariate_glm_sps_and_dispatch() {
    let sps = MultivariateGlmCommand::to_sps(
        &["y1".to_string(), "y2".to_string()],
        &["group".to_string()],
        &[],
        true,
    );
    assert!(sps.starts_with("GLM y1 y2 BY group"));
    assert!(sps.contains("/PRINT = TEST(MULTIVAR)"));

    let syntax = "GLM y1 y2 BY group\n  /METHOD = SSTYPE(3)\n  /INTERCEPT = INCLUDE.";
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let res = SyntaxDispatcher::execute_batch(syntax, &mut ds, &mut bus)
        .expect("Dispatch MANOVA must succeed");

    assert_eq!(res.len(), 1);
    let val = &res[0].1;
    assert_eq!(val["dependents"].as_array().unwrap().len(), 2);
    assert_eq!(val["n_valid"], 10);
    assert!(!val["effects"].as_array().unwrap().is_empty());
}

#[test]
fn test_repeated_measures_glm_sps_and_dispatch() {
    let sps = RepeatedGlmCommand::to_sps(
        "time",
        &["t1".to_string(), "t2".to_string(), "t3".to_string()],
    );
    assert!(sps.contains("/WSFACTOR = time 3 Polynomial"));

    let syntax = "GLM t1 t2 t3\n  /WSFACTOR = time 3\n  /METHOD = SSTYPE(3).";
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_test_dataset());

    let res = SyntaxDispatcher::execute_batch(syntax, &mut ds, &mut bus)
        .expect("Dispatch Repeated GLM must succeed");

    assert_eq!(res.len(), 1);
    let val = &res[0].1;
    assert_eq!(val["factor_name"], "time");
    assert_eq!(val["measure_variables"].as_array().unwrap().len(), 3);
    assert_eq!(val["n_valid"], 10);
    assert!(val["sphericity"]["mauchly_w"].as_f64().unwrap() > 0.0);
}
