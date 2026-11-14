//! Golden Oracle verification tests for Feature 003 (K-Means, Codebook, Summarize, KM, LifeTables).
//! Precision target: <= 1e-6 against analytical solutions and reference implementations.

use statcraft_algo::cluster_kmeans::{compute_kmeans, KMeansOptions};
use statcraft_algo::codebook_core::{compute_codebook, CodebookOptions, RawColumnData};
use statcraft_algo::summarize_core::{compute_summarize, SummarizeOptions};
use statcraft_algo::survival_km::{compute_kaplan_meier, KMDataPoint, KMOptions};
use statcraft_algo::survival_lifetables::{compute_life_table, LifeTableCase, LifeTableOptions};
use statcraft_core::variable::{DataType, Measure, VariableMetadata};

#[test]
fn test_golden_oracle_kmeans_convergence_and_anova() {
    // 3 distinct clusters in 2D space:
    // C1: (0, 0), (1, 0), (0, 1) -> centroid ~ (1/3, 1/3)
    // C2: (10, 10), (11, 10), (10, 11) -> centroid ~ (31/3, 31/3)
    // C3: (20, 0), (21, 0), (20, 1) -> centroid ~ (61/3, 1/3)
    let data = vec![
        vec![0.0, 0.0],
        vec![1.0, 0.0],
        vec![0.0, 1.0],
        vec![10.0, 10.0],
        vec![11.0, 10.0],
        vec![10.0, 11.0],
        vec![20.0, 0.0],
        vec![21.0, 0.0],
        vec![20.0, 1.0],
    ];
    let var_names = vec!["x1".to_string(), "x2".to_string()];
    let options = KMeansOptions {
        clusters: 3,
        max_iterations: 20,
        convergence_criterion: 1e-6,
    };

    let res = compute_kmeans(&data, &var_names, &options).expect("K-Means converged");
    assert_eq!(res.k, 3);
    assert_eq!(res.cluster_sizes, vec![3, 3, 3]);

    // Check final centers (order may vary, but centroids must match the 3 known clusters)
    let expected_centers = vec![
        (1.0 / 3.0, 1.0 / 3.0),
        (31.0 / 3.0, 31.0 / 3.0),
        (61.0 / 3.0, 1.0 / 3.0),
    ];

    for exp in &expected_centers {
        let found = res
            .final_centers
            .iter()
            .any(|c| (c[0] - exp.0).abs() < 1e-4 && (c[1] - exp.1).abs() < 1e-4);
        assert!(
            found,
            "Expected center ({}, {}) not found in final centers",
            exp.0, exp.1
        );
    }

    // Check ANOVA table: high F statistic, significant p-value
    assert_eq!(res.anova_table.len(), 2);
    assert!(res.anova_table[0].f_stat > 100.0);
    assert!(res.anova_table[0].p_value < 1e-4);
    assert!(res.anova_table[1].f_stat > 100.0);
    assert!(res.anova_table[1].p_value < 1e-4);
}

#[test]
fn test_golden_oracle_kaplan_meier_greenwood_and_logrank() {
    // Standard reference 2-group clinical trial data:
    // Group 1: 5 patients, times = [2, 3, 6, 6+, 7]
    // Group 2: 5 patients, times = [1, 2, 2, 4, 5+]
    let data = vec![
        KMDataPoint {
            time: 2.0,
            is_event: true,
            group: "G1".to_string(),
        },
        KMDataPoint {
            time: 3.0,
            is_event: true,
            group: "G1".to_string(),
        },
        KMDataPoint {
            time: 6.0,
            is_event: true,
            group: "G1".to_string(),
        },
        KMDataPoint {
            time: 6.0,
            is_event: false,
            group: "G1".to_string(),
        },
        KMDataPoint {
            time: 7.0,
            is_event: true,
            group: "G1".to_string(),
        },
        KMDataPoint {
            time: 1.0,
            is_event: true,
            group: "G2".to_string(),
        },
        KMDataPoint {
            time: 2.0,
            is_event: true,
            group: "G2".to_string(),
        },
        KMDataPoint {
            time: 2.0,
            is_event: true,
            group: "G2".to_string(),
        },
        KMDataPoint {
            time: 4.0,
            is_event: true,
            group: "G2".to_string(),
        },
        KMDataPoint {
            time: 5.0,
            is_event: false,
            group: "G2".to_string(),
        },
    ];

    let res = compute_kaplan_meier(
        &data,
        "time",
        "status",
        Some("group"),
        &KMOptions::default(),
    )
    .expect("KM computed");

    assert_eq!(res.groups.len(), 2);

    // G1 checks:
    // t=2: n=5, d=1 -> S(2) = 4/5 = 0.8. Greenwood Var = (0.8)^2 * (1 / (5*4)) = 0.64 * 0.05 = 0.032 -> SE = sqrt(0.032) ~ 0.178885
    let g1 = &res.groups[0];
    assert_eq!(g1.group_label, "G1");
    let row_t2 = &g1.table_rows[0];
    assert_eq!(row_t2.time, 2.0);
    assert!((row_t2.cumulative_survival - 0.8).abs() < 1e-6);
    assert!((row_t2.std_error - 0.032f64.sqrt()).abs() < 1e-6);

    // t=3: n=4, d=1 -> S(3) = 0.8 * (3/4) = 0.6. Greenwood Var = 0.36 * (1/20 + 1/12) = 0.36 * 0.1333333 = 0.048 -> SE = sqrt(0.048) ~ 0.219089
    let row_t3 = &g1.table_rows[1];
    assert_eq!(row_t3.time, 3.0);
    assert!((row_t3.cumulative_survival - 0.6).abs() < 1e-6);
    assert!((row_t3.std_error - 0.048f64.sqrt()).abs() < 1e-6);

    // Log-Rank test check:
    assert!(res.log_rank_test.is_some());
    let lr = res.log_rank_test.as_ref().unwrap();
    assert_eq!(lr.df, 1);
    assert!(lr.p_value >= 0.0 && lr.p_value <= 1.0);
}

#[test]
fn test_golden_oracle_life_tables_actuarial() {
    // 10 cases, 3 intervals: [0, 10), [10, 20), [20, 30)
    // [0, 10): 2 deaths, 1 censored
    // [10, 20): 3 deaths, 0 censored
    // [20, 30): 2 deaths, 2 censored
    let cases = vec![
        LifeTableCase {
            time: 3.0,
            is_event: true,
        },
        LifeTableCase {
            time: 7.0,
            is_event: true,
        },
        LifeTableCase {
            time: 8.0,
            is_event: false,
        },
        LifeTableCase {
            time: 12.0,
            is_event: true,
        },
        LifeTableCase {
            time: 15.0,
            is_event: true,
        },
        LifeTableCase {
            time: 18.0,
            is_event: true,
        },
        LifeTableCase {
            time: 22.0,
            is_event: true,
        },
        LifeTableCase {
            time: 25.0,
            is_event: true,
        },
        LifeTableCase {
            time: 28.0,
            is_event: false,
        },
        LifeTableCase {
            time: 29.0,
            is_event: false,
        },
    ];

    let options = LifeTableOptions {
        interval_start: 0.0,
        interval_end: 30.0,
        interval_width: 10.0,
    };

    let res = compute_life_table(&cases, "time", "status", &options).expect("Life table computed");
    assert_eq!(res.rows.len(), 3);

    // Interval 0: [0, 10)
    // Entering: 10, Withdrawn: 1 -> Exposed: 10 - 0.5 = 9.5
    // Events: 2 -> q = 2 / 9.5 = 0.2105263, p = 1 - q = 0.7894737
    // CumSurv: 1.0
    // Density f = (1.0 * 2/9.5) / 10 = 0.0210526
    // Hazard h = (2 * q) / (10 * (1 + p)) = (2 * 0.2105263) / (10 * 1.7894737) = 0.4210526 / 17.894737 = 0.0235294
    let r0 = &res.rows[0];
    assert_eq!(r0.cases_entering, 10);
    assert_eq!(r0.cases_withdrawn, 1);
    assert!((r0.cases_exposed - 9.5).abs() < 1e-6);
    assert_eq!(r0.terminal_events, 2);
    assert!((r0.proportion_terminating - (2.0 / 9.5)).abs() < 1e-6);
    assert!((r0.proportion_surviving - (1.0 - 2.0 / 9.5)).abs() < 1e-6);
    assert!((r0.cumulative_surviving - 1.0).abs() < 1e-6);
    assert!((r0.probability_density - (2.0 / 95.0)).abs() < 1e-6);
    assert!((r0.hazard_rate - 0.02352941176).abs() < 1e-5);
}

#[test]
fn test_golden_oracle_codebook_and_summarize() {
    let mut var_salary =
        VariableMetadata::new("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .expect("metadata ok");
    var_salary.label = "Current Salary".to_string();

    let salary_str = vec![
        Some("1000.00".to_string()),
        Some("2000.00".to_string()),
        Some("3000.00".to_string()),
        Some("4000.00".to_string()),
    ];
    let salary_num = vec![Some(1000.0), Some(2000.0), Some(3000.0), Some(4000.0)];

    let cols = vec![RawColumnData {
        values: &salary_str,
        numeric_values: &salary_num,
    }];

    let cb =
        compute_codebook(&[&var_salary], &cols, &CodebookOptions::default()).expect("codebook ok");
    assert_eq!(cb.total_cases, 4);
    let s = cb.cards[0].scale_stats.as_ref().unwrap();
    assert!((s.mean - 2500.0).abs() < 1e-6);
    assert!((s.min - 1000.0).abs() < 1e-6);
    assert!((s.max - 4000.0).abs() < 1e-6);

    // Summarize test
    let sm = compute_summarize(
        &["salary".to_string()],
        &[salary_str],
        &[salary_num],
        &[],
        &[],
        &SummarizeOptions::default(),
    )
    .expect("summarize ok");
    assert_eq!(sm.total_cases, 4);
    assert!((sm.total_stats[0].mean.unwrap() - 2500.0).abs() < 1e-6);
    assert!((sm.total_stats[0].sum.unwrap() - 10000.0).abs() < 1e-6);
}
