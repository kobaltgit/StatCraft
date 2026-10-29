use crate::factor::{compute_factor_analysis, jacobi_eigen_symmetric};
use crate::factor_types::{FactorExtractionCriteria, FactorExtractionMethod, FactorOptions};

#[test]
fn test_jacobi_eigen_symmetric_identity() {
    let eye = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
    ];
    let (evals, _) = jacobi_eigen_symmetric(&eye);
    assert_eq!(evals.len(), 3);
    for e in evals {
        assert!((e - 1.0).abs() < 1e-9);
    }
}

#[test]
fn test_jacobi_eigen_symmetric_known_matrix() {
    // 2x2 symmetric matrix: [[2, 1], [1, 2]]
    // Eigenvalues are 3 and 1
    let mat = vec![vec![2.0, 1.0], vec![1.0, 2.0]];
    let (evals, evecs) = jacobi_eigen_symmetric(&mat);
    assert!((evals[0] - 3.0).abs() < 1e-9);
    assert!((evals[1] - 1.0).abs() < 1e-9);

    // Verify orthogonality
    let dot = evecs[0][0] * evecs[0][1] + evecs[1][0] * evecs[1][1];
    assert!(dot.abs() < 1e-9);
}

#[test]
fn test_factor_analysis_golden_oracle() {
    // 5 variables, 10 observations with strong correlation between (q1, q2, q5) and (q3, q4)
    let q1 = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(5.0),
        Some(4.0),
        Some(6.0),
        Some(2.0),
        Some(5.0),
        Some(3.0),
        Some(4.0),
    ];
    let q2 = vec![
        Some(2.0),
        Some(3.0),
        Some(3.0),
        Some(6.0),
        Some(5.0),
        Some(7.0),
        Some(1.0),
        Some(4.0),
        Some(4.0),
        Some(5.0),
    ];
    let q3 = vec![
        Some(5.0),
        Some(6.0),
        Some(4.0),
        Some(2.0),
        Some(3.0),
        Some(1.0),
        Some(7.0),
        Some(2.0),
        Some(5.0),
        Some(3.0),
    ];
    let q4 = vec![
        Some(6.0),
        Some(7.0),
        Some(5.0),
        Some(1.0),
        Some(2.0),
        Some(2.0),
        Some(8.0),
        Some(3.0),
        Some(4.0),
        Some(4.0),
    ];
    let q5 = vec![
        Some(3.0),
        Some(4.0),
        Some(2.0),
        Some(6.0),
        Some(5.0),
        Some(7.0),
        Some(1.0),
        Some(4.0),
        Some(3.0),
        Some(5.0),
    ];

    let items = vec![
        ("q1", q1.as_slice()),
        ("q2", q2.as_slice()),
        ("q3", q3.as_slice()),
        ("q4", q4.as_slice()),
        ("q5", q5.as_slice()),
    ];

    let options = FactorOptions {
        extraction: FactorExtractionMethod::PC,
        criteria: FactorExtractionCriteria::Mineigen(1.0),
        kmo_bartlett: true,
    };

    let res =
        compute_factor_analysis(&items, &[], &options).expect("Factor analysis should succeed");

    assert_eq!(res.n_cases, 10);
    assert_eq!(res.variables.len(), 5);

    // Sum of all initial eigenvalues must equal p = 5.0
    let sum_eigen: f64 = res
        .total_variance_explained
        .iter()
        .map(|r| r.initial_total)
        .sum();
    assert!((sum_eigen - 5.0).abs() < 1e-7);

    // Cumulative percent of variance for last component must be 100.0%
    let last_cum = res
        .total_variance_explained
        .last()
        .unwrap()
        .initial_cumulative_pct;
    assert!((last_cum - 100.0).abs() < 1e-7);

    // In this dataset, component 1 captures large variance (> 4.0)
    assert!(res.total_variance_explained[0].initial_total > 3.8);

    // Initial communalities are all 1.0
    for comm in &res.communalities {
        assert_eq!(comm.initial, 1.0);
        assert!(comm.extraction > 0.0 && comm.extraction <= 1.0);
    }

    // Component matrix dimensions
    assert_eq!(res.component_matrix.len(), 5);
    for row in &res.component_matrix {
        assert_eq!(row.loadings.len(), res.n_components_extracted);
    }

    // KMO & Bartlett
    let kb = res.kmo_bartlett.as_ref().unwrap();
    assert!(kb.kmo > 0.0 && kb.kmo <= 1.0);
    assert_eq!(kb.bartlett_df, 10); // 5 * 4 / 2 = 10
    assert!(kb.bartlett_approx_chi_square > 0.0);
    assert!(kb.bartlett_sig >= 0.0 && kb.bartlett_sig <= 1.0);
}

#[test]
fn test_factor_analysis_fixed_factors() {
    let q1 = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let q2 = vec![Some(2.0), Some(1.0), Some(4.0), Some(3.0), Some(5.0)];
    let q3 = vec![Some(5.0), Some(4.0), Some(3.0), Some(2.0), Some(1.0)];

    let items = vec![
        ("q1", q1.as_slice()),
        ("q2", q2.as_slice()),
        ("q3", q3.as_slice()),
    ];

    let options = FactorOptions {
        extraction: FactorExtractionMethod::PC,
        criteria: FactorExtractionCriteria::Factors(2),
        kmo_bartlett: false,
    };

    let res = compute_factor_analysis(&items, &[], &options).unwrap();
    assert_eq!(res.n_components_extracted, 2);
    assert!(res.kmo_bartlett.is_none());
    assert_eq!(res.component_matrix[0].loadings.len(), 2);
}

#[test]
fn test_factor_analysis_missing_and_zero_variance() {
    let q1 = vec![Some(1.0), None, Some(3.0), Some(4.0)];
    let q2 = vec![Some(2.0), Some(2.0), Some(2.0), Some(2.0)]; // zero variance

    let items = vec![("q1", q1.as_slice()), ("q2", q2.as_slice())];
    let options = FactorOptions::default();

    let err = compute_factor_analysis(&items, &[], &options);
    assert!(err.is_err());
}
