use super::*;
use statcraft_algo::ExploreResult;
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};

fn make_test_dataset() -> DataSet {
    let csv = "salary,age,gender\n35000,25,1\n45000,30,2\n55000,35,1\n65000,40,2\n75000,45,1\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    DataSet::from_raw_table(table).unwrap()
}

#[test]
fn test_explore_specs() {
    let cmd = ExploreCommand;
    assert_eq!(cmd.id(), "stat.explore");
    assert_eq!(
        cmd.menu_path(),
        "Analyze > Descriptive Statistics > Explore"
    );
    let specs = cmd.specs();
    assert_eq!(specs["type"], "object");
    assert!(specs["properties"]["dependent_vars"].is_object());
    assert!(specs["properties"]["vars"].is_object());
    assert!(specs["properties"]["factor_vars"].is_object());
    assert!(specs["properties"]["bootstrap"].is_object());
}

#[test]
fn test_explore_execute_without_factor() {
    let cmd = ExploreCommand;
    let mut ds = Some(make_test_dataset());

    assert!(cmd.enabled(ds.as_ref()));

    let params = json!({
        "vars": ["salary", "age"],
        "compute_descriptives": true,
        "compute_normality": true,
        "compute_extremes": true,
        "confidence_level": 0.95
    });

    let res_val = cmd
        .execute(&mut ds, params)
        .expect("Execution without factor should succeed");
    let res: ExploreResult = serde_json::from_value(res_val).expect("Valid ExploreResult JSON");

    assert_eq!(res.variables.len(), 2);
    assert_eq!(res.variables[0].variable, "salary");
    assert!(res.variables[0].factor_var.is_none());
    assert_eq!(res.variables[0].groups.len(), 1);
    assert_eq!(res.variables[0].groups[0].case_processing.n_valid, 5);

    let desc = res.variables[0].groups[0].descriptives.as_ref().unwrap();
    assert_eq!(desc.mean, Some(55000.0));
    assert_eq!(desc.median, Some(55000.0));

    let norm = res.variables[0].groups[0].normality.as_ref().unwrap();
    assert!(norm.kolmogorov_smirnov.is_some());
    assert!(norm.shapiro_wilk.is_some());

    let ext = res.variables[0].groups[0].extremes.as_ref().unwrap();
    assert_eq!(ext.highest.len(), 5);
    assert_eq!(ext.lowest.len(), 5);
}

#[test]
fn test_explore_execute_with_factor_and_bootstrap() {
    let cmd = ExploreCommand;
    let mut ds = Some(make_test_dataset());

    let params = json!({
        "dependent_vars": ["salary"],
        "factor_vars": ["gender"],
        "compute_descriptives": true,
        "compute_normality": false,
        "compute_extremes": false,
        "bootstrap": {
            "enabled": true,
            "samples": 30,
            "seed": 123,
            "ci_level": 95.0,
            "ci_method": "percentile"
        }
    });

    let res_val = cmd
        .execute(&mut ds, params)
        .expect("Execution with factor should succeed");
    let res: ExploreResult = serde_json::from_value(res_val).expect("Valid ExploreResult JSON");

    assert_eq!(res.variables.len(), 1);
    assert_eq!(res.variables[0].variable, "salary");
    assert_eq!(res.variables[0].factor_var.as_deref(), Some("gender"));
    assert_eq!(res.variables[0].groups.len(), 2);

    let boot = res.bootstrap.expect("bootstrap should be computed");
    assert_eq!(boot.groups.len(), 2);
    assert!(boot.groups[0].mean.is_some());
}

#[test]
fn test_explore_to_sps() {
    let deps = vec!["salary".to_string(), "age".to_string()];
    let empty_factor: Vec<String> = vec![];
    let sps1 = ExploreCommand::to_sps(&deps, &empty_factor);
    assert_eq!(
        sps1,
        "EXAMINE VARIABLES=salary age\n  /PLOT NPPLOT\n  /STATISTICS DESCRIPTIVES EXTREME."
    );

    let factor = vec!["gender".to_string()];
    let sps2 = ExploreCommand::to_sps(&deps, &factor);
    assert_eq!(
        sps2,
        "EXAMINE VARIABLES=salary age BY gender\n  /PLOT NPPLOT\n  /STATISTICS DESCRIPTIVES EXTREME."
    );
}

#[test]
fn test_explore_to_sps_full_roundtrip() {
    use crate::syntax_parser::lexer::SyntaxLexer;
    use crate::syntax_parser::mappers::map_stat_command;
    use crate::syntax_parser::parser::SyntaxParser;

    let opts = ExploreOptions {
        dependent_vars: vec!["salary".to_string()],
        factor_vars: vec!["gender".to_string()],
        label_var: Some("id_col".to_string()),
        compute_descriptives: true,
        compute_m_estimators: true,
        compute_percentiles: true,
        compute_normality: true,
        compute_extremes: true,
        spread_level_mode: SpreadLevelMode::Power(0.5),
        confidence_level: 0.99,
        missing: ExploreMissingHandling::Report,
        ..Default::default()
    };
    let sps = ExploreCommand::to_sps_full(&opts);
    let blocks = SyntaxLexer::split_commands(&sps).expect("lex ok");
    let parsed = SyntaxParser::parse_command(&blocks[0]).expect("parse ok");
    let (id, val) = map_stat_command(&parsed, None)
        .expect("map ok")
        .expect("some");
    assert_eq!(id, "stat.explore");
    assert_eq!(val["label_var"], "id_col");
    assert_eq!(val["compute_m_estimators"], true);
    assert_eq!(val["compute_percentiles"], true);
    assert_eq!(val["spread_level_mode"], "power");
    assert_eq!(val["spread_level_power"], 0.5);
    assert_eq!(val["confidence_level"], 0.99);
    assert_eq!(val["missing"], "report");
}
