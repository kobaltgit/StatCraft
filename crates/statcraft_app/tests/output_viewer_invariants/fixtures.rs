use statcraft_algo::{
    AnovaTableRow, AssociationMeasure, ChiSquareTestRow, CrosstabCategory, CrosstabCell,
    CrosstabOptions, CrosstabResult, CrosstabRow, DescriptivesResult, FrequencyCategory,
    FrequencyResult, GroupStats, IndepTTestResult, LeveneResult, OneSampleTTestResult,
    OneWayAnovaResult, PairedCorrelation, PairedDifferences, PairedTTestResult, PairedVarStats,
    TTestTwoGroupDetails,
};

pub fn sample_descriptives() -> DescriptivesResult {
    DescriptivesResult {
        variable: "salary".to_string(),
        n_valid: 100,
        n_missing: 0,
        mean: Some(50000.0),
        se_mean: Some(1200.0),
        std_dev: Some(12000.0),
        variance: Some(144000000.0),
        min: Some(25000.0),
        max: Some(85000.0),
        range: Some(60000.0),
        sum: Some(5000000.0),
        skewness: Some(0.4),
        se_skewness: Some(0.24),
        kurtosis: Some(-0.2),
        se_kurtosis: Some(0.48),
    }
}

pub fn sample_frequencies() -> FrequencyResult {
    FrequencyResult {
        variable: "gender".to_string(),
        n_valid: 100,
        n_missing: 0,
        categories: vec![
            FrequencyCategory {
                value: "1".to_string(),
                label: Some("Male".to_string()),
                count: 45,
                percent: 45.0,
                valid_percent: Some(45.0),
                cumulative_percent: Some(45.0),
            },
            FrequencyCategory {
                value: "2".to_string(),
                label: Some("Female".to_string()),
                count: 55,
                percent: 55.0,
                valid_percent: Some(55.0),
                cumulative_percent: Some(100.0),
            },
        ],
        quartiles: None,
    }
}

pub fn sample_onesample() -> OneSampleTTestResult {
    OneSampleTTestResult {
        variable: "salary".to_string(),
        n_valid: 100,
        n_missing: 0,
        mean: Some(50000.0),
        std_dev: Some(12000.0),
        se_mean: Some(1200.0),
        test_value: 45000.0,
        t_stat: Some(4.167),
        df: Some(99.0),
        sig_2tailed: Some(0.0001),
        mean_diff: Some(5000.0),
        ci_level: 0.95,
        ci_lower: Some(2618.0),
        ci_upper: Some(7382.0),
    }
}

pub fn sample_indep() -> IndepTTestResult {
    IndepTTestResult {
        variable: "salary".to_string(),
        grouping_var: "gender".to_string(),
        group1_stats: GroupStats {
            group_val: "1".to_string(),
            n: 45,
            mean: Some(52000.0),
            std_dev: Some(11000.0),
            se_mean: Some(1640.0),
        },
        group2_stats: GroupStats {
            group_val: "2".to_string(),
            n: 55,
            mean: Some(48000.0),
            std_dev: Some(12500.0),
            se_mean: Some(1685.0),
        },
        levene: LeveneResult {
            f_stat: Some(0.85),
            sig: Some(0.358),
            df1: 1.0,
            df2: 98.0,
        },
        equal_variances_assumed: TTestTwoGroupDetails {
            t_stat: Some(1.68),
            df: Some(98.0),
            sig_2tailed: Some(0.096),
            mean_diff: Some(4000.0),
            se_diff: Some(2380.0),
            ci_lower: Some(-723.0),
            ci_upper: Some(8723.0),
        },
        equal_variances_not_assumed: TTestTwoGroupDetails {
            t_stat: Some(1.70),
            df: Some(96.4),
            sig_2tailed: Some(0.092),
            mean_diff: Some(4000.0),
            se_diff: Some(2351.0),
            ci_lower: Some(-667.0),
            ci_upper: Some(8667.0),
        },
        ci_level: 0.95,
    }
}

pub fn sample_paired() -> PairedTTestResult {
    PairedTTestResult {
        pair_num: 1,
        var1_stats: PairedVarStats {
            var_name: "pre_test".to_string(),
            n: 50,
            mean: Some(120.0),
            std_dev: Some(15.0),
            se_mean: Some(2.12),
        },
        var2_stats: PairedVarStats {
            var_name: "post_test".to_string(),
            n: 50,
            mean: Some(115.0),
            std_dev: Some(14.0),
            se_mean: Some(1.98),
        },
        correlation: PairedCorrelation {
            var1: "pre_test".to_string(),
            var2: "post_test".to_string(),
            n: 50,
            correlation: Some(0.85),
            sig: Some(0.0001),
        },
        differences: PairedDifferences {
            mean_diff: Some(5.0),
            std_dev_diff: Some(8.0),
            se_diff: Some(1.13),
            ci_lower: Some(2.73),
            ci_upper: Some(7.27),
            t_stat: Some(4.42),
            df: Some(49.0),
            sig_2tailed: Some(0.0001),
        },
        confidence_level: 0.95,
    }
}

pub fn sample_anova() -> OneWayAnovaResult {
    OneWayAnovaResult {
        dep_var: "salary".to_string(),
        factor_var: "education".to_string(),
        between_groups: AnovaTableRow {
            sum_of_squares: 15000000.0,
            df: 2,
            mean_square: Some(7500000.0),
            f_stat: Some(12.5),
            p_value: Some(0.0001),
        },
        within_groups: AnovaTableRow {
            sum_of_squares: 58200000.0,
            df: 97,
            mean_square: Some(600000.0),
            f_stat: None,
            p_value: None,
        },
        total: AnovaTableRow {
            sum_of_squares: 73200000.0,
            df: 99,
            mean_square: None,
            f_stat: None,
            p_value: None,
        },
        descriptives: vec![],
        total_descriptives: None,
        homogeneity: None,
    }
}

pub fn sample_crosstabs() -> CrosstabResult {
    CrosstabResult {
        row_var: "gender".to_string(),
        col_var: "jobcat".to_string(),
        n_valid: 100,
        n_missing: 5,
        col_categories: vec![
            CrosstabCategory {
                value: "1".to_string(),
                label: Some("Clerical".to_string()),
            },
            CrosstabCategory {
                value: "2".to_string(),
                label: Some("Custodial".to_string()),
            },
        ],
        rows: vec![CrosstabRow {
            category: CrosstabCategory {
                value: "1".to_string(),
                label: Some("Female".to_string()),
            },
            cells: vec![
                CrosstabCell {
                    observed: 40,
                    expected: Some(30.0),
                    row_pct: Some(80.0),
                    col_pct: Some(66.7),
                    total_pct: Some(40.0),
                    residual: Some(10.0),
                },
                CrosstabCell {
                    observed: 10,
                    expected: Some(20.0),
                    row_pct: Some(20.0),
                    col_pct: Some(25.0),
                    total_pct: Some(10.0),
                    residual: Some(-10.0),
                },
            ],
            row_total: 50,
            row_total_pct: Some(50.0),
        }],
        col_totals: vec![60, 40],
        col_total_pcts: Some(vec![60.0, 40.0]),
        grand_total: 100,
        chi_square_tests: vec![ChiSquareTestRow {
            name: "Pearson Chi-Square".to_string(),
            value: 8.333,
            df: Some(1),
            asymp_sig: Some(0.004),
            exact_sig: None,
        }],
        association_measures: vec![AssociationMeasure {
            name: "Phi".to_string(),
            value: 0.289,
            approx_sig: Some(0.004),
        }],
        options: CrosstabOptions::default(),
    }
}
