use crate::anova_oneway::AlgoError;
use crate::correlations_types::{MissingHandling, SignificanceTest};
use crate::partial_corr::{compute_partial_correlations, PartialCorrOptions};

#[test]
fn test_partial_corr_golden_oracle_two_controls() {
    let x: Vec<Option<f64>> = vec![2.3, 1.9, 3.4, 4.5, 2.1, 5.2, 3.8, 4.1, 2.9, 3.6]
        .into_iter()
        .map(Some)
        .collect();
    let y: Vec<Option<f64>> = vec![12.1, 14.5, 10.2, 8.4, 15.1, 7.2, 9.8, 8.9, 11.4, 10.5]
        .into_iter()
        .map(Some)
        .collect();
    let z1: Vec<Option<f64>> = vec![34.0, 28.0, 45.0, 52.0, 23.0, 61.0, 44.0, 48.0, 38.0, 41.0]
        .into_iter()
        .map(Some)
        .collect();
    let z2: Vec<Option<f64>> = vec![
        120.0, 110.0, 140.0, 160.0, 105.0, 180.0, 150.0, 155.0, 130.0, 135.0,
    ]
    .into_iter()
    .map(Some)
    .collect();

    let vars = vec![("x", x.as_slice()), ("y", y.as_slice())];
    let ctrls = vec![("z1", z1.as_slice()), ("z2", z2.as_slice())];

    let opts = PartialCorrOptions {
        significance: SignificanceTest::TwoTailed,
        missing: MissingHandling::Listwise,
        descriptives: true,
        zero_order_correlations: true,
    };

    let res = compute_partial_correlations(&vars, &ctrls, &opts).expect("should compute");
    assert_eq!(res.variables, vec!["x", "y"]);
    assert_eq!(res.controls, vec!["z1", "z2"]);

    // Diagonal
    assert_eq!(res.cells[0][0].r, Some(1.0));
    assert_eq!(res.cells[1][1].r, Some(1.0));

    // Off-diagonal cell (0, 1)
    let cell = &res.cells[0][1];
    let r = cell.r.expect("r should be some");
    let p = cell.p_val.expect("p should be some");
    assert_eq!(cell.df, 6);
    assert_eq!(cell.n, 10);

    // Golden oracle comparison with SciPy (precision <= 1e-6)
    let expected_r = -0.1719967316;
    let expected_p = 0.6838098798;
    assert!(
        (r - expected_r).abs() < 1e-6,
        "r diff: {}",
        (r - expected_r).abs()
    );
    assert!(
        (p - expected_p).abs() < 1e-6,
        "p diff: {}",
        (p - expected_p).abs()
    );

    // Symmetry
    let r01 = res.cells[0][1].r.expect("r01");
    let r10 = res.cells[1][0].r.expect("r10");
    assert!((r01 - r10).abs() < 1e-12);
    assert_eq!(res.cells[0][1].p_val, res.cells[1][0].p_val);

    // Descriptives check
    let desc = res.descriptives.expect("descriptives requested");
    assert_eq!(desc.len(), 4);
    assert_eq!(desc[0].var_name, "x");

    // Zero order check
    let zero = res.zero_order.expect("zero order requested");
    assert_eq!(zero.variables, vec!["x", "y"]);
}

#[test]
fn test_partial_corr_one_control_analytical() {
    let x: Vec<Option<f64>> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
        .into_iter()
        .map(Some)
        .collect();
    let y: Vec<Option<f64>> = vec![2.0, 1.0, 4.0, 3.0, 6.0, 5.0, 8.0, 7.0]
        .into_iter()
        .map(Some)
        .collect();
    let z: Vec<Option<f64>> = vec![1.5, 2.5, 3.0, 4.5, 5.0, 6.5, 7.0, 8.5]
        .into_iter()
        .map(Some)
        .collect();

    let vars = vec![("x", x.as_slice()), ("y", y.as_slice())];
    let ctrls = vec![("z", z.as_slice())];

    let opts = PartialCorrOptions {
        significance: SignificanceTest::OneTailed,
        missing: MissingHandling::Listwise,
        descriptives: false,
        zero_order_correlations: false,
    };

    let res = compute_partial_correlations(&vars, &ctrls, &opts).expect("should compute");
    let cell = &res.cells[0][1];
    assert_eq!(cell.df, 8 - 1 - 2); // 5
    assert!(cell.r.is_some());
    assert!(cell.p_val.is_some());
}

#[test]
fn test_partial_corr_missing_handling() {
    let x: Vec<Option<f64>> = vec![Some(1.0), None, Some(3.0), Some(4.0), Some(5.0), Some(6.0)];
    let y: Vec<Option<f64>> = vec![Some(2.0), Some(3.0), Some(4.0), None, Some(6.0), Some(7.0)];
    let z: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0), None];

    let vars = vec![("x", x.as_slice()), ("y", y.as_slice())];
    let ctrls = vec![("z", z.as_slice())];

    // Listwise: valid rows are only row 0, 2, 4 -> N = 3. But N <= k + 2 (3 <= 1 + 2 = 3), so InsufficientData
    let opts_list = PartialCorrOptions {
        missing: MissingHandling::Listwise,
        ..Default::default()
    };
    let res_list = compute_partial_correlations(&vars, &ctrls, &opts_list);
    assert!(matches!(res_list, Err(AlgoError::InsufficientData(_))));

    // Pairwise
    let opts_pair = PartialCorrOptions {
        missing: MissingHandling::Pairwise,
        ..Default::default()
    };
    let res_pair = compute_partial_correlations(&vars, &ctrls, &opts_pair).expect("pairwise works");
    assert_eq!(res_pair.cells[0][0].n, 5); // x valid
    assert_eq!(res_pair.cells[1][1].n, 5); // y valid
}

#[test]
fn test_partial_corr_validation_errors() {
    let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let y: Vec<Option<f64>> = vec![Some(2.0), Some(3.0), Some(4.0), Some(5.0), Some(6.0)];
    let z: Vec<Option<f64>> = vec![Some(1.0), Some(1.0), Some(1.0), Some(1.0), Some(1.0)];

    let opts = PartialCorrOptions::default();

    // Less than 2 variables
    assert!(
        compute_partial_correlations(&[("x", x.as_slice())], &[("z", z.as_slice())], &opts)
            .is_err()
    );

    // Empty controls
    assert!(
        compute_partial_correlations(&[("x", x.as_slice()), ("y", y.as_slice())], &[], &opts)
            .is_err()
    );

    // Duplicate variable name
    assert!(compute_partial_correlations(
        &[("x", x.as_slice()), ("y", y.as_slice())],
        &[("x", z.as_slice())],
        &opts
    )
    .is_err());

    // Zero variance in control variable
    let res_zero_var = compute_partial_correlations(
        &[("x", x.as_slice()), ("y", y.as_slice())],
        &[("z", z.as_slice())],
        &opts,
    );
    assert!(res_zero_var.is_err());
}
