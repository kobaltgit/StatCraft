use statcraft_algo::bootstrap::{
    compute_bca_ci, compute_percentile_ci, normal_cdf, run_bootstrap_scalar, summarize_bootstrap,
    BootstrapCiMethod, BootstrapError, BootstrapOptions, BootstrapSamplingMethod, SamplingPlan,
};
use statcraft_algo::rank_cases::inv_normal_cdf;
use statcraft_core::Mt19937;

fn sample_mean(data: &[f64], indices: &[usize]) -> Option<f64> {
    if indices.is_empty() {
        return None;
    }
    let sum: f64 = indices.iter().map(|&i| data[i]).sum();
    Some(sum / indices.len() as f64)
}

#[test]
fn test_normal_cdf_high_precision() {
    assert!((normal_cdf(0.0) - 0.5).abs() < 1e-12);
    assert!((normal_cdf(1.959963984540054) - 0.975).abs() < 1e-10);
    assert!((normal_cdf(-1.959963984540054) - 0.025).abs() < 1e-10);
    assert!((normal_cdf(inv_normal_cdf(0.1234)) - 0.1234).abs() < 1e-10);
    assert!((normal_cdf(inv_normal_cdf(0.9876)) - 0.9876).abs() < 1e-10);
}

#[test]
fn test_bootstrap_options_validation() {
    let mut opts = BootstrapOptions::default();
    assert!(opts.validate().is_ok());

    opts.samples = 1;
    assert_eq!(
        opts.validate(),
        Err(BootstrapError::InsufficientSamples { samples: 1 })
    );

    opts.samples = 1000;
    opts.ci_level = 100.0;
    assert_eq!(
        opts.validate(),
        Err(BootstrapError::InvalidCiLevel { ci_level: 100.0 })
    );

    opts.ci_level = 95.0;
    opts.seed = Some(0);
    assert_eq!(
        opts.validate(),
        Err(BootstrapError::InvalidSeed { seed: 0 })
    );

    opts.seed = Some(2_000_000);
    opts.sampling_method = BootstrapSamplingMethod::Stratified {
        strata_vars: vec![],
    };
    assert_eq!(opts.validate(), Err(BootstrapError::EmptyStrataVariables));
}

#[test]
fn test_stratified_resampling_preserves_stratum_sizes() {
    let strata_keys = vec![
        "A".to_string(),
        "B".to_string(),
        "A".to_string(),
        "__MISSING__".to_string(),
        "B".to_string(),
        "A".to_string(),
    ];
    let plan = SamplingPlan::stratified(&strata_keys).unwrap();
    assert_eq!(plan.len(), 6);

    let mut rng = Mt19937::new(2_000_000);
    let mut buf = Vec::new();
    for _ in 0..50 {
        plan.sample_indices(&mut rng, &mut buf);
        assert_eq!(buf.len(), 6);
        let count_a = buf.iter().filter(|&&i| strata_keys[i] == "A").count();
        let count_b = buf.iter().filter(|&&i| strata_keys[i] == "B").count();
        let count_m = buf
            .iter()
            .filter(|&&i| strata_keys[i] == "__MISSING__")
            .count();
        assert_eq!(count_a, 3);
        assert_eq!(count_b, 2);
        assert_eq!(count_m, 1);
    }
}

#[test]
fn test_bootstrap_determinism_and_degenerate_bca() {
    let data = vec![5.0, 5.0, 5.0, 5.0, 5.0];
    let plan = SamplingPlan::simple(data.len()).unwrap();
    let opts = BootstrapOptions {
        enabled: true,
        samples: 200,
        seed: Some(2_000_000),
        ci_level: 95.0,
        ci_method: BootstrapCiMethod::Bca,
        sampling_method: BootstrapSamplingMethod::Simple,
    };

    let summary = run_bootstrap_scalar(&plan, &opts, |idx| sample_mean(&data, idx)).unwrap();
    assert!((summary.estimate - 5.0).abs() < 1e-12);
    assert!(summary.bias.abs() < 1e-12);
    assert!(summary.std_error.abs() < 1e-12);
    assert!((summary.ci.lower - 5.0).abs() < 1e-12);
    assert!((summary.ci.upper - 5.0).abs() < 1e-12);
}

#[test]
fn test_golden_oracle_percentile_and_bca() {
    // Skewed dataset: [1.0, 2.0, 4.0, 7.0, 12.0], mean = 5.2
    let data = [1.0_f64, 2.0, 4.0, 7.0, 12.0];
    let estimate = 5.2_f64;

    // Leave-one-out jackknife means for the 5 observations:
    // skip 0 (1.0): (2+4+7+12)/4 = 6.25
    // skip 1 (2.0): (1+4+7+12)/4 = 6.00
    // skip 2 (4.0): (1+2+7+12)/4 = 5.50
    // skip 3 (7.0): (1+2+4+12)/4 = 4.75
    // skip 4 (12.0): (1+2+4+7)/4 = 3.50
    let jackknife = [6.25_f64, 6.00, 5.50, 4.75, 3.50];

    // Deterministic 100-replicate set for exact analytical/SciPy verification
    let mut replicates = Vec::with_capacity(100);
    for i in 0..100 {
        // Monotonically increasing skewed replicates from 2.0 to 10.5
        let x = 2.0 + (i as f64) * 0.085 + ((i as f64) / 99.0).powi(2) * 1.5;
        replicates.push(x);
    }

    // Percentile 95% CI: p=0.025 -> pos=2.475, p=0.975 -> pos=96.525
    let mut sorted = replicates.clone();
    sorted.sort_by(f64::total_cmp);
    let pct_ci = compute_percentile_ci(&sorted, 95.0).unwrap();
    let expected_lo = sorted[2] + 0.475 * (sorted[3] - sorted[2]);
    let expected_hi = sorted[96] + 0.525 * (sorted[97] - sorted[96]);
    assert!((pct_ci.lower - expected_lo).abs() < 1e-12);
    assert!((pct_ci.upper - expected_hi).abs() < 1e-12);

    // BCa 95% CI: verify exact Efron & Tibshirani / scipy.stats.bootstrap formula
    let jk_mean = jackknife.iter().sum::<f64>() / 5.0; // 5.2
    let sum_sq: f64 = jackknife.iter().map(|&v| (jk_mean - v).powi(2)).sum();
    let sum_cu: f64 = jackknife.iter().map(|&v| (jk_mean - v).powi(3)).sum();
    let a_hat = sum_cu / (6.0 * sum_sq.powf(1.5));

    let less = sorted.iter().filter(|&&v| v < estimate).count() as f64;
    let eq = sorted.iter().filter(|&&v| v == estimate).count() as f64;
    let z0 = inv_normal_cdf((less + 0.5 * eq) / 100.0);

    let z_lo = inv_normal_cdf(0.025);
    let z_hi = inv_normal_cdf(0.975);
    let a1 = normal_cdf(z0 + (z0 + z_lo) / (1.0 - a_hat * (z0 + z_lo)));
    let a2 = normal_cdf(z0 + (z0 + z_hi) / (1.0 - a_hat * (z0 + z_hi)));

    let pos1 = a1 * 99.0;
    let lo1 = pos1.floor() as usize;
    let exp_bca_lo = sorted[lo1] + (pos1 - lo1 as f64) * (sorted[lo1 + 1] - sorted[lo1]);

    let pos2 = a2 * 99.0;
    let lo2 = pos2.floor() as usize;
    let exp_bca_hi = sorted[lo2] + (pos2 - lo2 as f64) * (sorted[lo2 + 1] - sorted[lo2]);

    let bca_ci = compute_bca_ci(estimate, &sorted, Some(&jackknife), 95.0).unwrap();
    assert!((bca_ci.lower - exp_bca_lo).abs() < 1e-9);
    assert!((bca_ci.upper - exp_bca_hi).abs() < 1e-9);

    let summary = summarize_bootstrap(
        estimate,
        &replicates,
        Some(&jackknife),
        95.0,
        BootstrapCiMethod::Bca,
    )
    .unwrap();
    assert_eq!(summary.valid_samples, 100);
    assert!((summary.ci.lower - exp_bca_lo).abs() < 1e-9);
    assert!((summary.ci.upper - exp_bca_hi).abs() < 1e-9);
    assert_eq!(data.len(), 5);
}

#[test]
fn test_run_bootstrap_reproducible_across_runs() {
    let data = vec![10.0, 12.0, 15.0, 18.0, 22.0, 30.0, 45.0];
    let plan = SamplingPlan::simple(data.len()).unwrap();
    let opts = BootstrapOptions {
        enabled: true,
        samples: 500,
        seed: Some(2_000_000),
        ci_level: 95.0,
        ci_method: BootstrapCiMethod::Bca,
        sampling_method: BootstrapSamplingMethod::Simple,
    };

    let s1 = run_bootstrap_scalar(&plan, &opts, |idx| sample_mean(&data, idx)).unwrap();
    let s2 = run_bootstrap_scalar(&plan, &opts, |idx| sample_mean(&data, idx)).unwrap();
    assert_eq!(s1, s2);
    assert!(s1.ci.lower < s1.estimate);
    assert!(s1.ci.upper > s1.estimate);
    assert!(s1.std_error > 0.0);
}
