use statcraft_algo::{
    compute_curve_fit, compute_logistic_regression, compute_partial_correlations, CurveFitOptions,
    CurveModelType, LogisticRegressionOptions, PartialCorrOptions,
};
use statcraft_app::views::output_view::{
    export_output_to_html, export_output_to_tsv, export_output_to_txt, extract_item_tsv,
    OutputItemKind, OutputViewerState,
};

#[test]
fn test_output_viewer_partial_correlations_invariants() {
    let mut state = OutputViewerState::new();

    let x = [
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
    ];
    let y = [
        Some(2.0),
        Some(4.0),
        Some(5.0),
        Some(7.0),
        Some(8.0),
        Some(11.0),
    ];
    let z = [
        Some(1.0),
        Some(1.0),
        Some(2.0),
        Some(2.0),
        Some(3.0),
        Some(3.0),
    ];
    let vars = [("x", &x[..]), ("y", &y[..])];
    let controls = [("z", &z[..])];
    let opts = PartialCorrOptions::default();
    let result = compute_partial_correlations(&vars, &controls, &opts).unwrap();

    state.add_partial_correlations(result, true, "listwise");
    assert_eq!(state.procedures.len(), 1);
    let proc = &state.procedures[0];
    assert_eq!(proc.title_key, "output-partial-corr-title");
    assert_eq!(proc.items.len(), 3);
    assert!(matches!(&proc.items[0].kind, OutputItemKind::Log(_)));
    assert!(matches!(&proc.items[1].kind, OutputItemKind::Notes(_)));
    assert!(matches!(
        &proc.items[2].kind,
        OutputItemKind::PartialCorrelations(_)
    ));

    // TSV Export
    let tsv = extract_item_tsv(&proc.items[2].kind).expect("TSV must be extractable");
    assert!(tsv.contains("Control Variables"));
    assert!(tsv.contains("Correlation"));
    assert!(tsv.contains("Sig. (2-tailed)"));
    assert!(tsv.contains("df"));
}

#[test]
fn test_output_viewer_logistic_regression_invariants() {
    let mut state = OutputViewerState::new();

    let y = [
        Some(0.0),
        Some(0.0),
        Some(0.0),
        Some(0.0),
        Some(1.0),
        Some(1.0),
        Some(1.0),
        Some(1.0),
    ];
    let x = [
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
        Some(7.0),
        Some(8.0),
    ];
    let dep = ("y", &y[..]);
    let indeps = [("x", &x[..])];
    let opts = LogisticRegressionOptions::default();
    let result = compute_logistic_regression(dep, &indeps, &opts).unwrap();

    state.add_logistic_regression(result);
    assert_eq!(state.procedures.len(), 1);
    let proc = &state.procedures[0];
    assert_eq!(proc.title_key, "output-logistic-title");
    assert_eq!(proc.items.len(), 3);
    assert!(matches!(
        &proc.items[2].kind,
        OutputItemKind::LogisticRegression(_)
    ));

    // TSV Export
    let tsv = extract_item_tsv(&proc.items[2].kind).expect("TSV must be extractable");
    assert!(tsv.contains("Variable"));
    assert!(tsv.contains("Exp(B)"));
    assert!(tsv.contains("Wald"));
}

#[test]
fn test_output_viewer_curvefit_invariants() {
    let mut state = OutputViewerState::new();

    let y = [
        Some(2.0),
        Some(4.0),
        Some(7.0),
        Some(11.0),
        Some(16.0),
        Some(22.0),
    ];
    let x = [
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
    ];
    let dep = ("y", &y[..]);
    let indep = ("x", &x[..]);
    let opts = CurveFitOptions {
        models: vec![CurveModelType::Linear, CurveModelType::Quadratic],
        ..Default::default()
    };
    let result = compute_curve_fit(dep, indep, &opts).unwrap();

    state.add_curve_fit(result);
    assert_eq!(state.procedures.len(), 1);
    let proc = &state.procedures[0];
    assert_eq!(proc.title_key, "output-curvefit-title");
    assert_eq!(proc.items.len(), 3);
    assert!(matches!(&proc.items[2].kind, OutputItemKind::CurveFit(_)));

    // TSV Export
    let tsv = extract_item_tsv(&proc.items[2].kind).expect("TSV must be extractable");
    assert!(tsv.contains("Equation"));
    assert!(tsv.contains("R Square"));
    assert!(tsv.contains("Linear"));
    assert!(tsv.contains("Quadratic"));
}

#[test]
fn test_output_viewer_full_export_invariants() {
    let mut state = OutputViewerState::new();

    let y = [
        Some(2.0),
        Some(4.0),
        Some(7.0),
        Some(11.0),
        Some(16.0),
        Some(22.0),
    ];
    let x = [
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
    ];
    let dep = ("y", &y[..]);
    let indep = ("x", &x[..]);
    let opts = CurveFitOptions {
        models: vec![CurveModelType::Linear],
        ..Default::default()
    };
    let result = compute_curve_fit(dep, indep, &opts).unwrap();
    state.add_curve_fit(result);

    // Verify global exports work without crashing
    let tsv = export_output_to_tsv(&state);
    assert!(!tsv.is_empty());

    let txt = export_output_to_txt(&state);
    assert!(!txt.is_empty());

    let html = export_output_to_html(&state);
    assert!(!html.is_empty());
}
