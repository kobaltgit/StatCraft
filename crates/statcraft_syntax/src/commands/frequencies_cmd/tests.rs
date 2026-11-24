use super::*;
use serde_json::json;
use statcraft_algo::FrequencyChartResult;
use statcraft_core::DataSet;
use statcraft_io::{load_csv_from_reader, CsvOptions};

fn make_test_dataset() -> DataSet {
    let csv = "gender,score\n1,85\n2,90\n1,95\n2,85\n1,100\n";
    let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
    let mut ds = DataSet::from_raw_table(table).unwrap();
    let mut labels = statcraft_core::ValueLabels::new();
    labels.insert("1", "Male");
    labels.insert("2", "Female");
    ds.dictionary.set_value_labels(0, labels).unwrap();
    ds.dictionary
        .set_measure(0, statcraft_core::Measure::Nominal)
        .unwrap();
    ds.dictionary.set_label(0, "Respondent Gender").unwrap();
    ds
}

#[test]
fn test_frequencies_command_execution() {
    let cmd = FrequenciesCommand;
    let mut ds = Some(make_test_dataset());

    let params = json!({
        "vars": ["gender", "score"]
    });

    assert!(cmd.enabled(ds.as_ref()));

    let res = cmd
        .execute(&mut ds, params)
        .expect("Execution should succeed");
    let results: Vec<FrequencyResult> = serde_json::from_value(res).expect("Valid JSON");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].variable, "gender");
    assert_eq!(
        results[0].variable_label,
        Some("Respondent Gender".to_string())
    );
    assert_eq!(results[0].n_valid, 5);
    assert_eq!(results[0].categories.len(), 2);
    assert_eq!(results[0].categories[0].value, "1");
    assert_eq!(results[0].categories[0].label, Some("Male".to_string()));
    assert_eq!(results[0].categories[0].count, 3);
    assert!(results[0].quartiles.is_some());

    assert_eq!(results[1].variable, "score");
    assert_eq!(results[1].n_valid, 5);
    assert!(results[1].quartiles.is_some());
}

#[test]
fn test_frequencies_with_statistics_and_percentiles() {
    let cmd = FrequenciesCommand;
    let mut ds = Some(make_test_dataset());

    let params = json!({
        "vars": ["score"],
        "statistics": {
            "mean": true,
            "std_dev": true,
            "median": true,
            "min": true,
            "max": true
        },
        "percentiles": {
            "quartiles": true,
            "ntiles": 10,
            "custom": [10.0, 90.0]
        },
        "order": "dfreq"
    });

    let res = cmd.execute(&mut ds, params).expect("Execution success");
    let results: Vec<FrequencyResult> = serde_json::from_value(res).expect("Valid JSON");

    assert_eq!(results.len(), 1);
    let score_res = &results[0];
    let stats = score_res.statistics.as_ref().expect("Statistics computed");
    assert_eq!(stats.mean, Some(91.0));
    assert!(stats.std_dev.is_some());
    assert_eq!(stats.min, Some(85.0));
    assert_eq!(stats.max, Some(100.0));
    assert_eq!(stats.median, Some(90.0));

    // Check dfreq sorting
    assert_eq!(score_res.categories[0].value, "85");
    assert_eq!(score_res.categories[0].count, 2);
}

#[test]
fn test_to_sps_from_params() {
    let params = json!({
        "vars": ["salary", "educ"],
        "display_tables": false,
        "order": "dfreq",
        "statistics": {
            "mean": true,
            "std_dev": true,
            "min": true,
            "max": true
        },
        "percentiles": {
            "custom": [25.0, 75.0],
            "ntiles": 5
        }
    });

    let sps = FrequenciesCommand::to_sps_from_params(&params);
    assert!(sps.starts_with("FREQUENCIES VARIABLES=salary educ"));
    assert!(sps.contains("/FORMAT=NOTABLE DFREQ"));
    assert!(sps.contains("/STATISTICS=MEAN STDDEV MINIMUM MAXIMUM"));
    assert!(sps.contains("/PERCENTILES=25 75"));
    assert!(sps.contains("/NTILES=5"));
    assert!(sps.contains("/ORDER=ANALYSIS."));
}

#[test]
fn test_to_sps_from_params_with_charts() {
    // Bar chart with percent
    let params_bar = json!({
        "vars": ["salary"],
        "charts": {
            "chart_type": "bar",
            "values": "percentages",
            "normal_curve": false
        }
    });
    let sps = FrequenciesCommand::to_sps_from_params(&params_bar);
    assert!(sps.contains("  /BARCHART PERCENT"));

    // Pie chart with frequencies
    let params_pie = json!({
        "vars": ["gender"],
        "charts": {
            "chart_type": "pie",
            "values": "frequencies",
            "normal_curve": false
        }
    });
    let sps = FrequenciesCommand::to_sps_from_params(&params_pie);
    assert!(sps.contains("  /PIECHART FREQ"));

    // Histogram with normal curve
    let params_hist = json!({
        "vars": ["score"],
        "charts": {
            "chart_type": "histogram",
            "values": "frequencies",
            "normal_curve": true
        }
    });
    let sps = FrequenciesCommand::to_sps_from_params(&params_hist);
    assert!(sps.contains("  /HISTOGRAM NORMAL"));
}

#[test]
fn test_frequencies_nominal_numeric_stats_and_label() {
    let cmd = FrequenciesCommand;
    let mut ds = Some(make_test_dataset());

    let params = json!({
        "vars": ["gender"],
        "statistics": {
            "mean": true,
            "median": true,
            "mode": true
        },
        "percentiles": {
            "quartiles": true
        }
    });

    let res = cmd.execute(&mut ds, params).expect("Execution success");
    let results: Vec<FrequencyResult> = serde_json::from_value(res).expect("Valid JSON");

    assert_eq!(results.len(), 1);
    let gender_res = &results[0];
    assert_eq!(gender_res.variable, "gender");
    assert_eq!(
        gender_res.variable_label,
        Some("Respondent Gender".to_string())
    );
    assert_eq!(gender_res.display_title(), "Respondent Gender [gender]");
    assert_eq!(gender_res.display_label_or_name(), "Respondent Gender");

    // Statistics and quartiles must NOT be suppressed for numeric nominal variables
    let stats = gender_res
        .statistics
        .as_ref()
        .expect("Statistics must be computed for numeric nominal variable");
    assert!(stats.mean.is_some());
    assert!(stats.median.is_some());
    assert!(gender_res.quartiles.is_some());
}

#[test]
fn test_frequencies_with_charts_execution() {
    let cmd = FrequenciesCommand;
    let mut ds = Some(make_test_dataset());

    // 1. Bar chart with percentages
    let params_bar = json!({
        "vars": ["gender"],
        "charts": {
            "chart_type": "bar",
            "values": "percentages"
        }
    });
    let res_bar = cmd.execute(&mut ds, params_bar).expect("Execution success");
    let results_bar: Vec<FrequencyResult> = serde_json::from_value(res_bar).expect("Valid JSON");
    assert!(results_bar[0].chart.is_some());
    if let Some(FrequencyChartResult::Bar {
        is_percentages,
        categories,
        ..
    }) = &results_bar[0].chart
    {
        assert!(is_percentages);
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].label, "Male");
        assert_eq!(categories[0].count, 3.0);
        assert!((categories[0].percent - 60.0).abs() < 1e-6);
        assert_eq!(categories[1].label, "Female");
        assert_eq!(categories[1].count, 2.0);
        assert!((categories[1].percent - 40.0).abs() < 1e-6);
    } else {
        panic!("Expected Bar chart");
    }

    // 2. Histogram with normal curve
    let params_hist = json!({
        "vars": ["score"],
        "charts": {
            "chart_type": "histogram",
            "values": "frequencies",
            "normal_curve": true
        }
    });
    let res_hist = cmd
        .execute(&mut ds, params_hist)
        .expect("Execution success");
    let results_hist: Vec<FrequencyResult> = serde_json::from_value(res_hist).expect("Valid JSON");
    assert!(results_hist[0].chart.is_some());
    if let Some(FrequencyChartResult::Histogram { summary, .. }) = &results_hist[0].chart {
        assert_eq!(summary.n_valid, 5);
        assert!((summary.mean - 91.0).abs() < 1e-6);
        assert!(summary.normal_curve_points.is_some());
        assert!(!summary.bins.is_empty());
    } else {
        panic!("Expected Histogram chart");
    }
}

#[test]
fn test_frequencies_roundtrip_format_limit_and_mode() {
    let params = json!({
        "vars": ["salary", "educ"],
        "order": "dfreq",
        "multiple_vars_mode": "variable",
        "suppress_tables_limit": 10,
        "statistics": {
            "mean": true,
            "median": true
        }
    });

    let sps = FrequenciesCommand::to_sps_from_params(&params);
    assert!(sps.starts_with("FREQUENCIES VARIABLES=salary educ"));
    assert!(sps.contains("/FORMAT=DFREQ VARIABLE LIMIT(10)"));
    assert!(sps.contains("/STATISTICS=MEAN MEDIAN"));
    assert!(sps.contains("/ORDER=ANALYSIS."));

    use crate::syntax_parser::lexer::SyntaxLexer;
    use crate::syntax_parser::mappers::stats::frequencies::map_frequencies;
    use crate::syntax_parser::parser::SyntaxParser;

    let blocks = SyntaxLexer::split_commands(&sps).unwrap();
    assert_eq!(blocks.len(), 1);
    let parsed_cmd = SyntaxParser::parse_command(&blocks[0]).unwrap();
    let (id, val) = map_frequencies(&parsed_cmd, None).unwrap().unwrap();

    assert_eq!(id, "stat.frequencies");
    assert_eq!(val["vars"], json!(["salary", "educ"]));
    assert_eq!(val["order"], "dfreq");
    assert_eq!(val["multiple_vars_mode"], "variable");
    assert_eq!(val["suppress_tables_limit"], 10);
    assert_eq!(val["statistics"]["mean"], true);
    assert_eq!(val["statistics"]["median"], true);
}

#[test]
fn test_frequencies_execution_with_suppress_tables_limit() {
    let cmd = FrequenciesCommand;
    let mut ds = Some(make_test_dataset());

    // Dataset has 2 categories for gender (1, 2)
    // If suppress_tables_limit = 1, since 2 > 1, table should be suppressed!
    let params = json!({
        "vars": ["gender"],
        "suppress_tables_limit": 1
    });

    let res = cmd.execute(&mut ds, params).expect("Execution success");
    let results: Vec<FrequencyResult> = serde_json::from_value(res).expect("Valid JSON");
    assert_eq!(results.len(), 1);
    assert!(results[0].table_suppressed);
    assert!(results[0].categories.is_empty());
    assert_eq!(results[0].n_valid, 5);

    // If suppress_tables_limit = 2, since 2 <= 2, table is NOT suppressed
    let params2 = json!({
        "vars": ["gender"],
        "suppress_tables_limit": 2
    });
    let res2 = cmd.execute(&mut ds, params2).expect("Execution success");
    let results2: Vec<FrequencyResult> = serde_json::from_value(res2).expect("Valid JSON");
    assert_eq!(results2.len(), 1);
    assert!(!results2[0].table_suppressed);
    assert_eq!(results2[0].categories.len(), 2);
}

#[test]
fn test_frequencies_sps_generation_with_table_styles() {
    let params = json!({
        "vars": ["salary"],
        "order": "analysis",
        "table_styles": [
            {
                "table_target": { "Subtype": "Frequencies" },
                "value_target": { "Label": "Frequency" },
                "dimension": "Columns",
                "condition": { "GreaterThan": 100.0 },
                "format": {
                    "bold": true,
                    "italic": false,
                    "bg_color": [220, 245, 220],
                    "text_color": null,
                    "reset_defaults": false
                }
            }
        ]
    });

    let sps = FrequenciesCommand::to_sps_from_params(&params);
    assert!(sps.contains("FREQUENCIES VARIABLES=salary"));
    assert!(sps.contains("OUTPUT MODIFY"));
    assert!(sps.contains("/SELECT TABLES"));
    assert!(sps.contains("/IF SUBTYPES=['Frequencies']"));
    assert!(sps.contains("/TABLECELLS SELECT=['Frequency'] SELECTDIMENSION=COLUMNS"));
    assert!(sps.contains("CONDITION=GT(100)"));
    assert!(sps.contains("STYLE=BACKGROUNDCOLOR(RGB(220, 245, 220)) FONTSTYLE=BOLD."));
}
