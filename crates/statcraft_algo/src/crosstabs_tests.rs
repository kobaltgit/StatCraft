#[cfg(test)]
mod tests {
    use crate::crosstabs::*;

    #[test]
    fn test_chi_square_sf_analytical() {
        // Analytical test: df=2, sf(x) = exp(-x / 2)
        let df = 2;
        let x = 5.991464547107979; // chi2_sf(5.99146..., 2) = 0.05
        let p = chi_square_sf(x, df);
        assert!((p - 0.05).abs() < 1e-6);

        // df=1, x=3.841458826890731 -> p = 0.05
        let p1 = chi_square_sf(3.841458826890731, 1);
        assert!((p1 - 0.05).abs() < 1e-6);

        // Very large statistic -> virtually 0.0
        assert!(chi_square_sf(1000.0, 1) < 1e-15);
        // Zero statistic -> p = 1.0
        assert_eq!(chi_square_sf(0.0, 2), 1.0);
    }

    #[test]
    fn test_fisher_exact_2x2_standard() {
        // Standard Tea-tasting experiment / Lady tasting tea:
        // [3, 1]
        // [1, 3]
        // Total N = 8
        // SciPy fisher_exact([[3, 1], [1, 3]]) gives p = 0.4857142857142857
        let p = fisher_exact_2x2(3, 1, 1, 3);
        assert!((p - 0.4857142857).abs() < 1e-6);

        // Extreme table:
        // [4, 0]
        // [0, 4]
        // SciPy gives p = 0.02857142857142857
        let p_extreme = fisher_exact_2x2(4, 0, 0, 4);
        assert!((p_extreme - 0.02857142857).abs() < 1e-6);
    }

    #[test]
    fn test_crosstabs_2x2_golden_oracle() {
        // Sample 2x2 table:
        // Gender: 0 (Male), 1 (Female)
        // Treatment: 0 (Control), 1 (Active)
        // [ 20, 30 ]
        // [ 40, 10 ]
        // Total = 100
        //
        // Observed:
        // a=20, b=30 (Row 0 sum = 50)
        // c=40, d=10 (Row 1 sum = 50)
        // Col 0 sum = 60, Col 1 sum = 40
        //
        // Expected:
        // E00 = 50 * 60 / 100 = 30
        // E01 = 50 * 40 / 100 = 20
        // E10 = 50 * 60 / 100 = 30
        // E11 = 50 * 40 / 100 = 20
        //
        // Chi-Square Pearson:
        // (20-30)^2/30 + (30-20)^2/20 + (40-30)^2/30 + (10-20)^2/20
        // = 100/30 + 100/20 + 100/30 + 100/20 = 3.333333 + 5 + 3.333333 + 5 = 16.6666667
        // df = 1
        //
        // Yates continuity correction:
        // num = 100 * (|20*10 - 30*40| - 50)^2 = 100 * (|200 - 1200| - 50)^2 = 100 * 950^2 = 90,250,000
        // denom = 50 * 50 * 60 * 40 = 6,000,000
        // chisq_yates = 90,250,000 / 6,000,000 = 15.0416667
        //
        // Phi = sqrt(16.6666667 / 100) = sqrt(0.1666667) = 0.408248
        // Cramer's V = 0.408248

        let mut items = Vec::new();
        // 20 cases of (0, 0)
        for _ in 0..20 {
            items.push(RawCrosstabItem {
                row_value: "0".to_string(),
                row_label: Some("Male".to_string()),
                col_value: "0".to_string(),
                col_label: Some("Control".to_string()),
                is_missing: false,
            });
        }
        // 30 cases of (0, 1)
        for _ in 0..30 {
            items.push(RawCrosstabItem {
                row_value: "0".to_string(),
                row_label: Some("Male".to_string()),
                col_value: "1".to_string(),
                col_label: Some("Active".to_string()),
                is_missing: false,
            });
        }
        // 40 cases of (1, 0)
        for _ in 0..40 {
            items.push(RawCrosstabItem {
                row_value: "1".to_string(),
                row_label: Some("Female".to_string()),
                col_value: "0".to_string(),
                col_label: Some("Control".to_string()),
                is_missing: false,
            });
        }
        // 10 cases of (1, 1)
        for _ in 0..10 {
            items.push(RawCrosstabItem {
                row_value: "1".to_string(),
                row_label: Some("Female".to_string()),
                col_value: "1".to_string(),
                col_label: Some("Active".to_string()),
                is_missing: false,
            });
        }
        // 5 missing cases
        for _ in 0..5 {
            items.push(RawCrosstabItem {
                row_value: "0".to_string(),
                row_label: Some("Male".to_string()),
                col_value: "".to_string(),
                col_label: None,
                is_missing: true,
            });
        }

        let opts = CrosstabOptions::default();
        let res = compute_crosstabs("gender", "treatment", &items, &opts)
            .expect("Crosstabs computation must succeed");

        assert_eq!(res.n_valid, 100);
        assert_eq!(res.n_missing, 5);
        assert_eq!(res.grand_total, 100);
        assert_eq!(res.rows.len(), 2);
        assert_eq!(res.col_categories.len(), 2);

        // Row 0
        assert_eq!(res.rows[0].row_total, 50);
        assert_eq!(res.rows[0].cells[0].observed, 20);
        assert!((res.rows[0].cells[0].expected.unwrap() - 30.0).abs() < 1e-7);
        assert!((res.rows[0].cells[0].row_pct.unwrap() - 40.0).abs() < 1e-7);
        assert!((res.rows[0].cells[0].col_pct.unwrap() - (20.0 / 60.0 * 100.0)).abs() < 1e-7);
        assert!((res.rows[0].cells[0].total_pct.unwrap() - 20.0).abs() < 1e-7);
        assert!((res.rows[0].cells[0].residual.unwrap() - (-10.0)).abs() < 1e-7);

        // Pearson Chi-Square test
        let p_test = res
            .chi_square_tests
            .iter()
            .find(|t| t.name == "Pearson Chi-Square")
            .expect("Pearson test exists");
        assert!((p_test.value - 16.6666667).abs() < 1e-5);
        assert_eq!(p_test.df, Some(1));
        assert!(p_test.asymp_sig.unwrap() < 0.0001);

        // Continuity correction
        let c_test = res
            .chi_square_tests
            .iter()
            .find(|t| t.name == "Continuity Correction")
            .expect("Yates test exists");
        assert!((c_test.value - 15.0416667).abs() < 1e-5);
        assert_eq!(c_test.df, Some(1));

        // Fisher exact test
        let f_test = res
            .chi_square_tests
            .iter()
            .find(|t| t.name == "Fisher's Exact Test")
            .expect("Fisher test exists");
        assert!(f_test.exact_sig.unwrap() < 0.0001);

        // Measures of association
        let phi = res
            .association_measures
            .iter()
            .find(|m| m.name == "Phi")
            .expect("Phi exists");
        assert!((phi.value - 0.408248).abs() < 1e-5);

        let cramer = res
            .association_measures
            .iter()
            .find(|m| m.name == "Cramer's V")
            .expect("Cramer exists");
        assert!((cramer.value - 0.408248).abs() < 1e-5);
    }

    #[test]
    fn test_crosstabs_3x3_table() {
        // 3x3 table
        let mut items = Vec::new();
        let matrix = [
            [12, 8, 5], // sum = 25
            [6, 15, 9], // sum = 30
            [4, 7, 14], // sum = 25
        ];

        for (r_idx, row) in matrix.iter().enumerate() {
            for (c_idx, &cnt) in row.iter().enumerate() {
                for _ in 0..cnt {
                    items.push(RawCrosstabItem {
                        row_value: format!("R{}", r_idx + 1),
                        row_label: None,
                        col_value: format!("C{}", c_idx + 1),
                        col_label: None,
                        is_missing: false,
                    });
                }
            }
        }

        let opts = CrosstabOptions::default();
        let res = compute_crosstabs("row", "col", &items, &opts).expect("Success");

        assert_eq!(res.n_valid, 80);
        assert_eq!(res.rows.len(), 3);
        assert_eq!(res.col_categories.len(), 3);

        let p_test = res
            .chi_square_tests
            .iter()
            .find(|t| t.name == "Pearson Chi-Square")
            .unwrap();
        assert_eq!(p_test.df, Some(4));
        assert!(p_test.value > 0.0);

        // 3x3 has min(3-1, 3-1) = 2
        let cramer = res
            .association_measures
            .iter()
            .find(|m| m.name == "Cramer's V")
            .unwrap();
        assert!(cramer.value > 0.0);
    }
}
