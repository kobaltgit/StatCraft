use super::*;
use crate::glm::repeated::build_helmert_contrasts;

#[test]
fn test_univariate_two_way_anova_type3() {
    // 2x2 Factorial design
    // Factor A: "a1", "a2"
    // Factor B: "b1", "b2"
    let y = vec![
        10.0, 12.0, 11.0, 13.0, // a1, b1
        14.0, 15.0, 13.0, 16.0, // a1, b2
        8.0, 9.0, 7.0, 10.0, // a2, b1
        18.0, 20.0, 19.0, 21.0, // a2, b2
    ];
    let factor_a = vec![
        "a1".to_string(),
        "a1".to_string(),
        "a1".to_string(),
        "a1".to_string(),
        "a1".to_string(),
        "a1".to_string(),
        "a1".to_string(),
        "a1".to_string(),
        "a2".to_string(),
        "a2".to_string(),
        "a2".to_string(),
        "a2".to_string(),
        "a2".to_string(),
        "a2".to_string(),
        "a2".to_string(),
        "a2".to_string(),
    ];
    let factor_b = vec![
        "b1".to_string(),
        "b1".to_string(),
        "b1".to_string(),
        "b1".to_string(),
        "b2".to_string(),
        "b2".to_string(),
        "b2".to_string(),
        "b2".to_string(),
        "b1".to_string(),
        "b1".to_string(),
        "b1".to_string(),
        "b1".to_string(),
        "b2".to_string(),
        "b2".to_string(),
        "b2".to_string(),
        "b2".to_string(),
    ];

    let options = UnivariateGlmOptions {
        dependent: "Score".to_string(),
        fixed_factors: vec!["FactorA".to_string(), "FactorB".to_string()],
        covariates: Vec::new(),
        model_type: GlmModelType::FullFactorial,
        custom_terms: Vec::new(),
        sum_of_squares: SumOfSquaresType::Type3,
        intercept: true,
        descriptives: true,
        partial_eta_squared: true,
        homogeneity_levene: true,
        confidence_level: 0.95,
    };

    let factors = vec![
        ("FactorA".to_string(), factor_a),
        ("FactorB".to_string(), factor_b),
    ];

    let result = compute_univariate_glm("Score", &y, &factors, &[], &options)
        .expect("Univariate GLM must succeed");

    assert_eq!(result.n_valid, 16);
    assert!(result.r_squared > 0.85);

    // Verify presence of all expected rows in Tests of Between-Subjects Effects
    let source_names: Vec<&str> = result.tests.iter().map(|r| r.source.as_str()).collect();
    assert!(source_names.contains(&"Corrected Model"));
    assert!(source_names.contains(&"Intercept"));
    assert!(source_names.contains(&"FactorA"));
    assert!(source_names.contains(&"FactorB"));
    assert!(source_names.contains(&"FactorA * FactorB"));
    assert!(source_names.contains(&"Error"));
    assert!(source_names.contains(&"Total"));
    assert!(source_names.contains(&"Corrected Total"));

    // Check Factor B effect is strongly significant
    let row_b = result.tests.iter().find(|r| r.source == "FactorB").unwrap();
    assert_eq!(row_b.df, 1);
    assert!(row_b.p_value.unwrap() < 0.001);

    // Check interaction effect is significant
    let row_ab = result
        .tests
        .iter()
        .find(|r| r.source == "FactorA * FactorB")
        .unwrap();
    assert_eq!(row_ab.df, 1);
    assert!(row_ab.f_stat.unwrap() > 5.0);
}

#[test]
fn test_univariate_ancova_with_covariate() {
    let y = vec![25.0, 30.0, 28.0, 36.0, 40.0, 38.0, 22.0, 24.0, 26.0];
    let group = vec![
        "ctrl".to_string(),
        "ctrl".to_string(),
        "ctrl".to_string(),
        "treat".to_string(),
        "treat".to_string(),
        "treat".to_string(),
        "low".to_string(),
        "low".to_string(),
        "low".to_string(),
    ];
    let pre_score = vec![10.0, 12.0, 11.0, 15.0, 18.0, 16.0, 8.0, 9.0, 10.0];

    let options = UnivariateGlmOptions::default();
    let factors = vec![("Group".to_string(), group)];
    let covariates = vec![("PreScore".to_string(), pre_score)];

    let result = compute_univariate_glm("PostScore", &y, &factors, &covariates, &options)
        .expect("ANCOVA must succeed");

    assert_eq!(result.n_valid, 9);
    let cov_row = result
        .tests
        .iter()
        .find(|r| r.source == "PreScore")
        .unwrap();
    assert_eq!(cov_row.df, 1);
    assert!(cov_row.type3_ss > 0.0);
}

#[test]
fn test_multivariate_manova_4_tests() {
    // 2 dependent variables: y1, y2 across 2 groups
    let y1 = vec![10.0, 12.0, 11.0, 9.0, 20.0, 22.0, 21.0, 19.0];
    let y2 = vec![5.0, 6.0, 4.0, 5.0, 15.0, 16.0, 14.0, 15.0];
    let group = vec![
        "G1".to_string(),
        "G1".to_string(),
        "G1".to_string(),
        "G1".to_string(),
        "G2".to_string(),
        "G2".to_string(),
        "G2".to_string(),
        "G2".to_string(),
    ];

    let options = MultivariateGlmOptions::default();
    let dependents = vec![("Y1", y1.as_slice()), ("Y2", y2.as_slice())];
    let factors = vec![("Group".to_string(), group)];

    let result = compute_multivariate_glm(&dependents, &factors, &[], &options)
        .expect("MANOVA must succeed");

    assert_eq!(result.n_valid, 8);
    assert_eq!(result.dependents.len(), 2);

    // Group effect tests
    let group_tests: Vec<_> = result
        .effects
        .iter()
        .filter(|r| r.effect == "Group")
        .collect();
    assert_eq!(group_tests.len(), 4);

    let pillai = group_tests
        .iter()
        .find(|r| r.stat_name == MultivariateStatName::PillaiTrace)
        .unwrap();
    let wilks = group_tests
        .iter()
        .find(|r| r.stat_name == MultivariateStatName::WilksLambda)
        .unwrap();
    let hotelling = group_tests
        .iter()
        .find(|r| r.stat_name == MultivariateStatName::HotellingTrace)
        .unwrap();
    let roy = group_tests
        .iter()
        .find(|r| r.stat_name == MultivariateStatName::RoyLargestRoot)
        .unwrap();

    assert!(pillai.value > 0.8);
    assert!(wilks.value < 0.1);
    assert!(hotelling.value > 10.0);
    assert!(roy.value > 10.0);

    // All p-values should be strongly significant
    assert!(wilks.p_value < 0.001);
    assert!(pillai.p_value < 0.001);
}

#[test]
fn test_repeated_measures_mauchly_sphericity() {
    // 3 time points across 5 subjects
    let t1 = vec![10.0, 12.0, 9.0, 11.0, 8.0];
    let t2 = vec![15.0, 18.0, 14.0, 16.0, 13.0];
    let t3 = vec![20.0, 25.0, 19.0, 22.0, 17.0];

    let measure_vars = vec![
        ("Time1", t1.as_slice()),
        ("Time2", t2.as_slice()),
        ("Time3", t3.as_slice()),
    ];

    let options = RepeatedGlmOptions {
        factor_name: "Time".to_string(),
        measure_variables: vec![
            "Time1".to_string(),
            "Time2".to_string(),
            "Time3".to_string(),
        ],
        between_factors: Vec::new(),
        sum_of_squares: SumOfSquaresType::Type3,
        partial_eta_squared: true,
    };

    let result =
        compute_repeated_glm(&measure_vars, &options).expect("Repeated measures must succeed");

    assert_eq!(result.n_valid, 5);
    assert_eq!(result.sphericity.df, 2);
    assert!(result.sphericity.mauchly_w > 0.0 && result.sphericity.mauchly_w <= 1.0);
    assert!(
        result.sphericity.greenhouse_geisser >= 0.5 && result.sphericity.greenhouse_geisser <= 1.0
    );
    assert!(result.sphericity.huynh_feldt >= result.sphericity.greenhouse_geisser);

    // Time effect must be significant
    let eff = &result.within_subjects_tests[0];
    assert!(eff.sphericity_assumed_f.unwrap() > 20.0);
    assert!(eff.sphericity_assumed_p.unwrap() < 0.001);
}

#[test]
#[allow(clippy::needless_range_loop)]
fn test_helmert_contrasts_orthonormal() {
    let k = 4;
    let c = build_helmert_contrasts(k);
    assert_eq!(c.len(), 4);
    assert_eq!(c[0].len(), 3);

    // Check sum of each column is 0
    for j in 0..3 {
        let sum: f64 = (0..k).map(|i| c[i][j]).sum();
        assert!(sum.abs() < 1e-12);
    }

    // Check C^T C = I
    for j1 in 0..3 {
        for j2 in 0..3 {
            let dot: f64 = (0..k).map(|i| c[i][j1] * c[i][j2]).sum();
            let expected = if j1 == j2 { 1.0 } else { 0.0 };
            assert!((dot - expected).abs() < 1e-12);
        }
    }
}
