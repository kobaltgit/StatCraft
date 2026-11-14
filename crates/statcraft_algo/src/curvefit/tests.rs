use crate::anova_oneway::AlgoError;
use crate::curvefit::{compute_curve_fit, CurveFitOptions, CurveModelType};

#[test]
fn test_curvefit_golden_oracle_all_six_models() {
    let x: Vec<Option<f64>> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        .into_iter()
        .map(Some)
        .collect();
    let y: Vec<Option<f64>> = vec![2.5, 4.1, 7.2, 11.5, 17.3, 24.8, 33.9, 44.7, 57.2, 71.4]
        .into_iter()
        .map(Some)
        .collect();

    let opts = CurveFitOptions {
        models: CurveModelType::all(),
        include_constant: true,
        confidence_level: 0.95,
    };

    let res = compute_curve_fit(
        ("response", y.as_slice()),
        ("predictor", x.as_slice()),
        &opts,
    )
    .expect("should compute");

    assert_eq!(res.dependent_var, "response");
    assert_eq!(res.independent_var, "predictor");
    assert_eq!(res.models.len(), 6);
    assert!(res.warnings.is_empty());

    // 1. Linear: R2=0.934708, F=114.5261, b0=-14.34, b1=7.60
    let m_lin = res
        .models
        .iter()
        .find(|m| m.model_type == CurveModelType::Linear)
        .unwrap();
    assert!((m_lin.r_squared - 0.934708).abs() < 1e-4);
    assert!((m_lin.f_stat - 114.5261).abs() < 1e-2);
    assert!((m_lin.b0 - (-14.34)).abs() < 1e-4);
    assert!((m_lin.b1 - 7.60).abs() < 1e-4);
    assert_eq!(m_lin.df1, 1);
    assert_eq!(m_lin.df2, 8);

    // 2. Logarithmic: R2=0.719755, F=20.5465, b0=-14.146337, b1=27.545816
    let m_log = res
        .models
        .iter()
        .find(|m| m.model_type == CurveModelType::Logarithmic)
        .unwrap();
    assert!((m_log.r_squared - 0.719755).abs() < 1e-4);
    assert!((m_log.f_stat - 20.5465).abs() < 1e-2);
    assert!((m_log.b0 - (-14.146337)).abs() < 1e-4);
    assert!((m_log.b1 - 27.545816).abs() < 1e-4);

    // 3. Inverse: R2=0.433100, F=6.1118, b0=44.006078, b1=-56.491149
    let m_inv = res
        .models
        .iter()
        .find(|m| m.model_type == CurveModelType::Inverse)
        .unwrap();
    assert!((m_inv.r_squared - 0.433100).abs() < 1e-4);
    assert!((m_inv.f_stat - 6.1118).abs() < 1e-2);
    assert!((m_inv.b0 - 44.006078).abs() < 1e-4);
    assert!((m_inv.b1 - (-56.491149)).abs() < 1e-4);

    // 4. Quadratic: R2=0.999929, F=49272.7368, b0=3.118333, b1=-1.129167, b2=0.793561
    let m_quad = res
        .models
        .iter()
        .find(|m| m.model_type == CurveModelType::Quadratic)
        .unwrap();
    assert!((m_quad.r_squared - 0.999929).abs() < 1e-4);
    assert!((m_quad.f_stat - 49272.7368).abs() < 1.0);
    assert!((m_quad.b0 - 3.118333).abs() < 1e-4);
    assert!((m_quad.b1 - (-1.129167)).abs() < 1e-4);
    assert!((m_quad.b2.unwrap() - 0.793561).abs() < 1e-4);
    assert_eq!(m_quad.df1, 2);
    assert_eq!(m_quad.df2, 7);

    // 5. Power: R2=0.967179, F=235.7442, b0=1.710075, b1=1.527906
    let m_pow = res
        .models
        .iter()
        .find(|m| m.model_type == CurveModelType::Power)
        .unwrap();
    assert!((m_pow.r_squared - 0.967179).abs() < 1e-4);
    assert!((m_pow.f_stat - 235.7442).abs() < 1e-1);
    assert!((m_pow.b0 - 1.710075).abs() < 1e-4);
    assert!((m_pow.b1 - 1.527906).abs() < 1e-4);

    // 6. Exponential: R2=0.977119, F=341.6342, b0=2.224106, b1=0.371817
    let m_exp = res
        .models
        .iter()
        .find(|m| m.model_type == CurveModelType::Exponential)
        .unwrap();
    assert!((m_exp.r_squared - 0.977119).abs() < 1e-4);
    assert!((m_exp.f_stat - 341.6342).abs() < 1e-1);
    assert!((m_exp.b0 - 2.224106).abs() < 1e-4);
    assert!((m_exp.b1 - 0.371817).abs() < 1e-4);
}

#[test]
fn test_curvefit_domain_handling() {
    // Data with non-positive values
    let x: Vec<Option<f64>> = vec![-2.0, -1.0, 0.0, 1.0, 2.0, 3.0]
        .into_iter()
        .map(Some)
        .collect();
    let y: Vec<Option<f64>> = vec![4.0, 1.0, 0.0, 1.0, 4.0, 9.0]
        .into_iter()
        .map(Some)
        .collect();

    let opts = CurveFitOptions {
        models: vec![
            CurveModelType::Linear,
            CurveModelType::Quadratic,
            CurveModelType::Logarithmic,
        ],
        include_constant: true,
        confidence_level: 0.95,
    };

    let res =
        compute_curve_fit(("y", y.as_slice()), ("x", x.as_slice()), &opts).expect("should compute");
    assert_eq!(res.models.len(), 3);
    let m_quad = res
        .models
        .iter()
        .find(|m| m.model_type == CurveModelType::Quadratic)
        .unwrap();
    assert!(m_quad.r_squared > 0.99); // y = x^2 fits quadratic perfectly
}

#[test]
fn test_curvefit_validation_errors() {
    let x: Vec<Option<f64>> = vec![Some(1.0), Some(2.0), Some(3.0)];
    let opts = CurveFitOptions::default();

    // Same variable for dependent and independent
    let res_same = compute_curve_fit(("x", x.as_slice()), ("x", x.as_slice()), &opts);
    assert!(matches!(res_same, Err(AlgoError::InvalidInput(_))));

    // Empty models
    let opts_empty = CurveFitOptions {
        models: vec![],
        ..Default::default()
    };
    let res_empty = compute_curve_fit(("y", x.as_slice()), ("x", x.as_slice()), &opts_empty);
    assert!(matches!(res_empty, Err(AlgoError::InvalidInput(_))));
}
