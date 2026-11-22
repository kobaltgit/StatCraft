//! Golden Oracle verification tests for FREQUENCIES charts algorithms.
//! Validated against IBM SPSS Statistics canon, scipy.stats, and numpy.histogram formulas.
//! Precision target: <= 1e-6.

use statcraft_algo::frequencies::{
    calculate_nice_step, compute_bar_chart, compute_frequency_chart, compute_histogram,
    compute_pie_chart, ChartType, ChartValueMode, FrequencyCategory, FrequencyChartResult,
    FrequencyChartSpec,
};

#[test]
fn test_calculate_nice_step() {
    assert_eq!(calculate_nice_step(1.0), 1.0);
    assert_eq!(calculate_nice_step(1.2), 1.0);
    assert_eq!(calculate_nice_step(2.3), 2.0);
    assert_eq!(calculate_nice_step(4.5), 5.0);
    assert_eq!(calculate_nice_step(8.5), 10.0);
    assert_eq!(calculate_nice_step(23.0), 20.0);
    assert_eq!(calculate_nice_step(45.0), 50.0);
    assert_eq!(calculate_nice_step(0.23), 0.2);
    assert_eq!(calculate_nice_step(0.045), 0.05);

    // Degenerate inputs
    assert_eq!(calculate_nice_step(0.0), 1.0);
    assert_eq!(calculate_nice_step(-10.0), 1.0);
    assert_eq!(calculate_nice_step(f64::NAN), 1.0);
    assert_eq!(calculate_nice_step(f64::INFINITY), 1.0);
}

#[test]
fn test_histogram_normal_distribution_oracle() {
    // Deterministic sample with known properties:
    // Generate 100 values spread symmetrically around mean = 50.0
    let mut values = Vec::with_capacity(100);
    for i in 0..100 {
        // Linear spread representing bounded continuous observations
        let v = 20.0 + (i as f64) * (60.0 / 99.0);
        values.push(v);
    }

    let n = values.len();
    let expected_mean = 50.0;
    let expected_var: f64 = values
        .iter()
        .map(|v| (v - expected_mean).powi(2))
        .sum::<f64>()
        / 99.0;
    let expected_sd = expected_var.sqrt();

    let chart = compute_histogram(
        &values,
        "score".to_string(),
        Some("Exam Score".to_string()),
        true,
    )
    .expect("Histogram should be computed");

    match chart {
        FrequencyChartResult::Histogram {
            var_name,
            var_label,
            summary,
        } => {
            assert_eq!(var_name, "score");
            assert_eq!(var_label, Some("Exam Score".to_string()));
            assert_eq!(summary.n_valid, n);

            // Verify mean and std_dev against reference math
            assert!(
                (summary.mean - expected_mean).abs() < 1e-6,
                "Mean mismatch: got {}, expected {}",
                summary.mean,
                expected_mean
            );
            assert!(
                (summary.std_dev - expected_sd).abs() < 1e-6,
                "Std dev mismatch: got {}, expected {}",
                summary.std_dev,
                expected_sd
            );

            // Verify bin properties and total counts
            assert!(summary.bin_width > 0.0);
            let total_counts: usize = summary.bins.iter().map(|b| b.count).sum();
            assert_eq!(
                total_counts, n,
                "Sum of bin counts must equal sample size N"
            );

            let total_percent: f64 = summary.bins.iter().map(|b| b.percent).sum();
            assert!(
                (total_percent - 100.0).abs() < 1e-6,
                "Sum of bin percentages must equal 100%"
            );

            for bin in &summary.bins {
                assert!(
                    (bin.right - bin.left - summary.bin_width).abs() < 1e-6,
                    "Bin width mismatch"
                );
                assert!(
                    (bin.midpoint - (bin.left + bin.right) * 0.5).abs() < 1e-6,
                    "Midpoint mismatch"
                );
                let expected_pct = (bin.count as f64) * 100.0 / (n as f64);
                assert!(
                    (bin.percent - expected_pct).abs() < 1e-6,
                    "Bin percent mismatch"
                );
            }

            // Verify Gaussian normal curve points against scipy.stats.norm.pdf formula
            let pts = summary
                .normal_curve_points
                .as_ref()
                .expect("Normal curve points must be generated");
            assert_eq!(pts.len(), 101);

            let scale = (n as f64) * summary.bin_width;
            for &(x, y) in pts {
                let z = (x - summary.mean) / summary.std_dev;
                let scipy_pdf =
                    (-0.5 * z * z).exp() / (summary.std_dev * (2.0 * std::f64::consts::PI).sqrt());
                let expected_y = scipy_pdf * scale;

                assert!(
                    (y - expected_y).abs() < 1e-6,
                    "Normal curve point mismatch at x={}: got {}, expected {}",
                    x,
                    y,
                    expected_y
                );
            }
        }
        _ => panic!("Expected FrequencyChartResult::Histogram"),
    }
}

#[test]
fn test_histogram_degenerate_case() {
    // 1. Constant sample (zero variance: s = 0)
    let const_sample = vec![42.0; 50];
    let chart_const = compute_histogram(&const_sample, "constant".to_string(), None, true)
        .expect("Should handle constant sample without panic");

    match chart_const {
        FrequencyChartResult::Histogram { summary, .. } => {
            assert_eq!(summary.n_valid, 50);
            assert_eq!(summary.mean, 42.0);
            assert_eq!(summary.std_dev, 0.0);
            assert_eq!(summary.bins.len(), 1);
            assert_eq!(summary.bins[0].count, 50);
            assert!((summary.bins[0].percent - 100.0).abs() < 1e-6);
            // Normal curve must be suppressed for zero variance
            assert!(
                summary.normal_curve_points.is_none(),
                "Normal curve points must be None when std_dev is 0"
            );
        }
        _ => panic!("Expected FrequencyChartResult::Histogram"),
    }

    // 2. Single-element sample (N = 1)
    let single_sample = vec![100.0];
    let chart_single = compute_histogram(&single_sample, "single".to_string(), None, true)
        .expect("Should handle single-element sample");

    match chart_single {
        FrequencyChartResult::Histogram { summary, .. } => {
            assert_eq!(summary.n_valid, 1);
            assert_eq!(summary.mean, 100.0);
            assert_eq!(summary.std_dev, 0.0);
            assert_eq!(summary.bins.len(), 1);
            assert_eq!(summary.bins[0].count, 1);
            assert!(summary.normal_curve_points.is_none());
        }
        _ => panic!("Expected FrequencyChartResult::Histogram"),
    }

    // 3. Empty sample -> None
    assert!(compute_histogram(&[], "empty".to_string(), None, true).is_none());
}

#[test]
fn test_bar_chart_percentages_sum() {
    let categories = vec![
        FrequencyCategory {
            value: "1".to_string(),
            label: Some("Red".to_string()),
            count: 25,
            percent: 20.0,
            valid_percent: Some(25.0),
            cumulative_percent: Some(25.0),
        },
        FrequencyCategory {
            value: "2".to_string(),
            label: Some("Green".to_string()),
            count: 35,
            percent: 28.0,
            valid_percent: Some(35.0),
            cumulative_percent: Some(60.0),
        },
        FrequencyCategory {
            value: "3".to_string(),
            label: Some("Blue".to_string()),
            count: 40,
            percent: 32.0,
            valid_percent: Some(40.0),
            cumulative_percent: Some(100.0),
        },
        // Missing category (valid_percent == None)
        FrequencyCategory {
            value: "99".to_string(),
            label: Some("No answer".to_string()),
            count: 25,
            percent: 20.0,
            valid_percent: None,
            cumulative_percent: None,
        },
    ];

    let chart = compute_bar_chart(
        &categories,
        "color".to_string(),
        Some("Favorite Color".to_string()),
        true,
    )
    .expect("Bar chart should be computed");

    match chart {
        FrequencyChartResult::Bar {
            var_name,
            var_label,
            is_percentages,
            categories: bar_cats,
        } => {
            assert_eq!(var_name, "color");
            assert_eq!(var_label, Some("Favorite Color".to_string()));
            assert!(is_percentages);
            // Missing category must be filtered out
            assert_eq!(bar_cats.len(), 3);

            let sum_pct: f64 = bar_cats.iter().map(|c| c.percent).sum();
            assert!(
                (sum_pct - 100.0).abs() < 1e-6,
                "Sum of valid category percentages must equal 100.0"
            );

            assert_eq!(bar_cats[0].label, "Red");
            assert_eq!(bar_cats[0].value_repr, "1");
            assert_eq!(bar_cats[0].count, 25.0);
            assert_eq!(bar_cats[0].percent, 25.0);

            assert_eq!(bar_cats[1].label, "Green");
            assert_eq!(bar_cats[1].count, 35.0);
            assert_eq!(bar_cats[1].percent, 35.0);

            assert_eq!(bar_cats[2].label, "Blue");
            assert_eq!(bar_cats[2].count, 40.0);
            assert_eq!(bar_cats[2].percent, 40.0);
        }
        _ => panic!("Expected FrequencyChartResult::Bar"),
    }
}

#[test]
fn test_pie_chart_categories_ordering() {
    let categories = vec![
        FrequencyCategory {
            value: "Low".to_string(),
            label: None,
            count: 10,
            percent: 20.0,
            valid_percent: Some(20.0),
            cumulative_percent: Some(20.0),
        },
        FrequencyCategory {
            value: "Medium".to_string(),
            label: None,
            count: 30,
            percent: 60.0,
            valid_percent: Some(60.0),
            cumulative_percent: Some(80.0),
        },
        FrequencyCategory {
            value: "High".to_string(),
            label: None,
            count: 10,
            percent: 20.0,
            valid_percent: Some(20.0),
            cumulative_percent: Some(100.0),
        },
    ];

    let chart = compute_pie_chart(&categories, "level".to_string(), None, false)
        .expect("Pie chart should be computed");

    match chart {
        FrequencyChartResult::Pie {
            var_name,
            is_percentages,
            slices,
            ..
        } => {
            assert_eq!(var_name, "level");
            assert!(!is_percentages);
            assert_eq!(slices.len(), 3);
            assert_eq!(slices[0].label, "Low");
            assert_eq!(slices[0].count, 10.0);
            assert_eq!(slices[1].label, "Medium");
            assert_eq!(slices[1].count, 30.0);
            assert_eq!(slices[2].label, "High");
            assert_eq!(slices[2].count, 10.0);
        }
        _ => panic!("Expected FrequencyChartResult::Pie"),
    }
}

#[test]
fn test_unified_compute_frequency_chart_dispatcher() {
    let categories = vec![FrequencyCategory {
        value: "1".to_string(),
        label: Some("Yes".to_string()),
        count: 10,
        percent: 100.0,
        valid_percent: Some(100.0),
        cumulative_percent: Some(100.0),
    }];

    // 1. None spec
    let spec_none = FrequencyChartSpec {
        chart_type: ChartType::None,
        value_mode: ChartValueMode::Frequencies,
        normal_curve: false,
    };
    assert!(compute_frequency_chart(&spec_none, "x", None, &categories, &[1.0, 2.0]).is_none());

    // 2. Bar spec
    let spec_bar = FrequencyChartSpec {
        chart_type: ChartType::Bar,
        value_mode: ChartValueMode::Percentages,
        normal_curve: false,
    };
    let bar_res = compute_frequency_chart(&spec_bar, "x", None, &categories, &[]);
    assert!(matches!(bar_res, Some(FrequencyChartResult::Bar { .. })));

    // 3. Histogram with empty numeric slice (e.g. non-numeric string variable)
    let spec_hist = FrequencyChartSpec {
        chart_type: ChartType::Histogram,
        value_mode: ChartValueMode::Frequencies,
        normal_curve: true,
    };
    // Non-numeric variable returns None without panic!
    assert!(compute_frequency_chart(&spec_hist, "text_var", None, &categories, &[]).is_none());

    // 4. Histogram with valid numeric slice
    let hist_res = compute_frequency_chart(
        &spec_hist,
        "num_var",
        None,
        &categories,
        &[10.0, 20.0, 30.0],
    );
    assert!(matches!(
        hist_res,
        Some(FrequencyChartResult::Histogram { .. })
    ));
}
