#[cfg(test)]
mod tests {
    use crate::correlations::*;

    #[test]
    fn test_pearson_perfect_linear() {
        let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        let y: Vec<Option<f64>> = vec![Some(2.0), Some(4.0), Some(6.0), Some(8.0), Some(10.0)];
        let vars = vec![("X", x.as_slice()), ("Y", y.as_slice())];

        let opts = BivariateCorrelationsOptions {
            cross_prod_cov: true,
            descriptives: true,
            ..Default::default()
        };

        let res = compute_bivariate_correlations(&vars, &opts).expect("calculation should succeed");

        let desc = res.descriptives.as_ref().unwrap();
        assert_eq!(desc.len(), 2);
        assert!((desc[0].mean.unwrap() - 3.0).abs() < 1e-6);
        assert!((desc[1].mean.unwrap() - 6.0).abs() < 1e-6);

        let mat = &res.matrices[0];
        assert_eq!(mat.corr_type, CorrelationType::Pearson);
        // Cell (0, 0)
        assert_eq!(mat.cells[0][0].r, Some(1.0));
        assert!(mat.cells[0][0].sig.is_none());
        assert_eq!(mat.cells[0][0].n, 5);

        // Cell (0, 1)
        let cell_01 = &mat.cells[0][1];
        assert!((cell_01.r.unwrap() - 1.0).abs() < 1e-6);
        assert!(cell_01.sig.unwrap() < 1e-5);
        assert_eq!(cell_01.n, 5);
        assert_eq!(cell_01.flag, SignificanceFlag::DoubleStar);
        assert!((cell_01.covariance.unwrap() - 5.0).abs() < 1e-6);
        assert!((cell_01.ss_cross_prod.unwrap() - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_pearson_golden_oracle() {
        // X = [1, 2, 3, 4, 5], Y = [1, 4, 9, 16, 25]
        let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
        let y: Vec<Option<f64>> = vec![Some(1.0), Some(4.0), Some(9.0), Some(16.0), Some(25.0)];
        let vars = vec![("X", x.as_slice()), ("Y", y.as_slice())];

        let opts = BivariateCorrelationsOptions {
            cross_prod_cov: true,
            ..Default::default()
        };

        let res = compute_bivariate_correlations(&vars, &opts).unwrap();
        let cell = &res.matrices[0].cells[0][1];

        // Pearson r = 60 / sqrt(3740) = 0.9811049...
        assert!((cell.r.unwrap() - 0.981105).abs() < 1e-5);
        // p-value ~ 0.003117
        assert!((cell.sig.unwrap() - 0.003117).abs() < 1e-4);
        assert_eq!(cell.flag, SignificanceFlag::DoubleStar);
        assert!((cell.covariance.unwrap() - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_spearman_and_kendall_oracle() {
        // X = [1, 2, 2, 3], Y = [1, 2, 3, 2]
        let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(2.0), Some(3.0)];
        let y: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(2.0)];
        let vars = vec![("X", x.as_slice()), ("Y", y.as_slice())];

        let opts = BivariateCorrelationsOptions {
            pearson: false,
            spearman: true,
            kendall: true,
            ..Default::default()
        };

        let res = compute_bivariate_correlations(&vars, &opts).unwrap();
        assert_eq!(res.matrices.len(), 2);

        // 1. Kendall tau-b
        let kendall_mat = &res.matrices[0];
        assert_eq!(kendall_mat.corr_type, CorrelationType::KendallTauB);
        let k_cell = &kendall_mat.cells[0][1];
        // tau_b = 2 / 5 = 0.4
        assert!((k_cell.r.unwrap() - 0.4).abs() < 1e-6);
        assert_eq!(k_cell.n, 4);

        // 2. Spearman rho
        let spearman_mat = &res.matrices[1];
        assert_eq!(spearman_mat.corr_type, CorrelationType::Spearman);
        let s_cell = &spearman_mat.cells[0][1];
        // rho = 0.5
        assert!((s_cell.r.unwrap() - 0.5).abs() < 1e-6);
        assert_eq!(s_cell.n, 4);
    }

    #[test]
    fn test_missing_values_pairwise_vs_listwise() {
        let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), None, Some(4.0)];
        let y: Vec<Option<f64>> = vec![Some(2.0), None, Some(3.0), Some(4.0)];
        let z: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0)];
        let vars = vec![
            ("X", x.as_slice()),
            ("Y", y.as_slice()),
            ("Z", z.as_slice()),
        ];

        // Pairwise
        let opts_pair = BivariateCorrelationsOptions {
            missing: MissingHandling::Pairwise,
            ..Default::default()
        };
        let res_pair = compute_bivariate_correlations(&vars, &opts_pair).unwrap();
        let p_mat = &res_pair.matrices[0];
        // Pair (X, Y) has 2 valid pairs (idx 0 and 3)
        assert_eq!(p_mat.cells[0][1].n, 2);
        // Pair (X, Z) has 3 valid pairs (idx 0, 1, 3)
        assert_eq!(p_mat.cells[0][2].n, 3);
        // Pair (Y, Z) has 3 valid pairs (idx 0, 2, 3)
        assert_eq!(p_mat.cells[1][2].n, 3);

        // Listwise
        let opts_list = BivariateCorrelationsOptions {
            missing: MissingHandling::Listwise,
            ..Default::default()
        };
        let res_list = compute_bivariate_correlations(&vars, &opts_list).unwrap();
        let l_mat = &res_list.matrices[0];
        // All pairs in listwise mode must have n = 2 (only rows 0 and 3 are valid across all 3 vars)
        assert_eq!(l_mat.cells[0][1].n, 2);
        assert_eq!(l_mat.cells[0][2].n, 2);
        assert_eq!(l_mat.cells[1][2].n, 2);
    }
}
