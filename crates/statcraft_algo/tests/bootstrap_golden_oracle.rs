use statcraft_algo::bootstrap::{
    normal_cdf, BootstrapCiMethod, BootstrapOptions, BootstrapSamplingMethod, BootstrapSummary,
    SamplingPlan,
};
use statcraft_algo::rank_cases::inv_normal_cdf;
use statcraft_algo::{
    compute_bivariate_correlations_bootstrap, compute_descriptives_bootstrap,
    compute_indep_ttest_bootstrap, compute_linear_regression, compute_linear_regression_bootstrap,
    BivariateCorrelationsOptions, DescriptivesOptions, IndepTTestOptions, LinearRegressionOptions,
};
use statcraft_core::Mt19937;

fn scipy_reference_summary(
    estimate: f64,
    replicates: &[f64],
    jackknife: &[f64],
    ci_level: f64,
    method: BootstrapCiMethod,
) -> (f64, f64, f64, f64) {
    let b = replicates.len() as f64;
    let boot_mean = replicates.iter().sum::<f64>() / b;
    let bias = boot_mean - estimate;
    let ss = replicates
        .iter()
        .map(|&v| (v - boot_mean).powi(2))
        .sum::<f64>();
    let std_error = (ss / (b - 1.0)).sqrt();

    let mut sorted = replicates.to_vec();
    sorted.sort_by(f64::total_cmp);

    let alpha = 1.0 - ci_level / 100.0;
    let p_lo = alpha * 0.5;
    let p_hi = 1.0 - alpha * 0.5;

    let interp = |p: f64| -> f64 {
        let pos = p * (sorted.len() - 1) as f64;
        let lo = pos.floor() as usize;
        let hi = (lo + 1).min(sorted.len() - 1);
        sorted[lo] + (pos - lo as f64) * (sorted[hi] - sorted[lo])
    };

    let (ci_lo, ci_hi) = match method {
        BootstrapCiMethod::Percentile => (interp(p_lo), interp(p_hi)),
        BootstrapCiMethod::Bca => {
            let less = sorted.iter().filter(|&&v| v < estimate).count() as f64;
            let eq = sorted.iter().filter(|&&v| v == estimate).count() as f64;
            let z0 = inv_normal_cdf((less + 0.5 * eq) / b);

            let jk_mean = jackknife.iter().sum::<f64>() / jackknife.len() as f64;
            let sum_sq: f64 = jackknife.iter().map(|&v| (jk_mean - v).powi(2)).sum();
            let sum_cu: f64 = jackknife.iter().map(|&v| (jk_mean - v).powi(3)).sum();
            let a_hat = if sum_sq > 0.0 {
                sum_cu / (6.0 * sum_sq.powf(1.5))
            } else {
                0.0
            };

            let z_lo = inv_normal_cdf(p_lo);
            let z_hi = inv_normal_cdf(p_hi);
            let a1 = normal_cdf(z0 + (z0 + z_lo) / (1.0 - a_hat * (z0 + z_lo)));
            let a2 = normal_cdf(z0 + (z0 + z_hi) / (1.0 - a_hat * (z0 + z_hi)));
            (interp(a1), interp(a2))
        }
    };

    (bias, std_error, ci_lo, ci_hi)
}

fn assert_matches_oracle(
    summary: &BootstrapSummary,
    estimate: f64,
    replicates: &[f64],
    jackknife: &[f64],
    ci_level: f64,
    method: BootstrapCiMethod,
) {
    let (exp_bias, exp_se, exp_lo, exp_hi) =
        scipy_reference_summary(estimate, replicates, jackknife, ci_level, method);
    assert!(
        (summary.estimate - estimate).abs() < 1e-6,
        "estimate mismatch: {} vs {}",
        summary.estimate,
        estimate
    );
    assert!(
        (summary.bias - exp_bias).abs() < 1e-6,
        "bias mismatch: {} vs {}",
        summary.bias,
        exp_bias
    );
    assert!(
        (summary.std_error - exp_se).abs() < 1e-6,
        "std_error mismatch: {} vs {}",
        summary.std_error,
        exp_se
    );
    assert!(
        (summary.ci.lower - exp_lo).abs() < 1e-6,
        "ci.lower mismatch: {} vs {}",
        summary.ci.lower,
        exp_lo
    );
    assert!(
        (summary.ci.upper - exp_hi).abs() < 1e-6,
        "ci.upper mismatch: {} vs {}",
        summary.ci.upper,
        exp_hi
    );
}

fn replay_indices(n: usize, samples: usize, seed: u64) -> Vec<Vec<usize>> {
    let plan = SamplingPlan::simple(n).unwrap();
    let mut rng = Mt19937::new(seed as u32);
    let mut all = Vec::with_capacity(samples);
    for _ in 0..samples {
        let mut buf = Vec::with_capacity(n);
        plan.sample_indices(&mut rng, &mut buf);
        all.push(buf);
    }
    all
}

#[test]
fn golden_oracle_univariate_mean_and_std_dev() {
    let data = [12.0_f64, 15.0, 18.0, 21.0, 25.0, 30.0, 36.0, 44.0];
    let opt_vals: Vec<Option<f64>> = data.iter().map(|&v| Some(v)).collect();
    let weights = vec![1.0; data.len()];
    let seed = 2_000_000;
    let samples = 200;

    let calc_mean = |vals: &[f64]| vals.iter().sum::<f64>() / vals.len() as f64;
    let calc_std = |vals: &[f64]| {
        let m = calc_mean(vals);
        let var = vals.iter().map(|&x| (x - m).powi(2)).sum::<f64>() / (vals.len() - 1) as f64;
        var.sqrt()
    };

    let idx_sets = replay_indices(data.len(), samples, seed);
    let rep_means: Vec<f64> = idx_sets
        .iter()
        .map(|idx| {
            let sub: Vec<f64> = idx.iter().map(|&i| data[i]).collect();
            calc_mean(&sub)
        })
        .collect();
    let rep_stds: Vec<f64> = idx_sets
        .iter()
        .map(|idx| {
            let sub: Vec<f64> = idx.iter().map(|&i| data[i]).collect();
            calc_std(&sub)
        })
        .collect();

    let jk_means: Vec<f64> = (0..data.len())
        .map(|skip| {
            let sub: Vec<f64> = data
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != skip)
                .map(|(_, &v)| v)
                .collect();
            calc_mean(&sub)
        })
        .collect();
    let jk_stds: Vec<f64> = (0..data.len())
        .map(|skip| {
            let sub: Vec<f64> = data
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != skip)
                .map(|(_, &v)| v)
                .collect();
            calc_std(&sub)
        })
        .collect();

    for method in [BootstrapCiMethod::Percentile, BootstrapCiMethod::Bca] {
        let boot_opts = BootstrapOptions {
            enabled: true,
            samples,
            seed: Some(seed),
            ci_level: 95.0,
            ci_method: method,
            sampling_method: BootstrapSamplingMethod::Simple,
        };
        let desc_opts = DescriptivesOptions {
            mean: true,
            std_dev: true,
            ..Default::default()
        };
        let res = compute_descriptives_bootstrap(
            "x",
            &opt_vals,
            &weights,
            None,
            Some(&desc_opts),
            &boot_opts,
        )
        .unwrap();

        assert_matches_oracle(
            res.mean.as_ref().unwrap(),
            calc_mean(&data),
            &rep_means,
            &jk_means,
            95.0,
            method,
        );
        assert_matches_oracle(
            res.std_dev.as_ref().unwrap(),
            calc_std(&data),
            &rep_stds,
            &jk_stds,
            95.0,
            method,
        );
    }
}

#[test]
fn golden_oracle_independent_mean_diff() {
    let vals = [
        Some(10.0),
        Some(12.0),
        Some(14.0),
        Some(16.0),
        Some(19.0),
        Some(20.0),
        Some(23.0),
        Some(25.0),
        Some(28.0),
        Some(32.0),
    ];
    let grps: Vec<Option<String>> = ["1", "1", "1", "1", "1", "2", "2", "2", "2", "2"]
        .iter()
        .map(|s| Some((*s).to_string()))
        .collect();
    let seed = 123_456;
    let samples = 200;

    let calc_diff = |indices: &[usize]| -> Option<f64> {
        let mut s1 = 0.0;
        let mut n1 = 0_usize;
        let mut s2 = 0.0;
        let mut n2 = 0_usize;
        for &i in indices {
            if grps[i].as_deref() == Some("1") {
                s1 += vals[i].unwrap();
                n1 += 1;
            } else {
                s2 += vals[i].unwrap();
                n2 += 1;
            }
        }
        if n1 >= 2 && n2 >= 2 {
            Some((s1 / n1 as f64) - (s2 / n2 as f64))
        } else {
            None
        }
    };

    let all_idx: Vec<usize> = (0..vals.len()).collect();
    let est = calc_diff(&all_idx).unwrap();
    let idx_sets = replay_indices(vals.len(), samples, seed);
    let reps: Vec<f64> = idx_sets.iter().filter_map(|idx| calc_diff(idx)).collect();
    let jk: Vec<f64> = (0..vals.len())
        .filter_map(|skip| {
            let sub: Vec<usize> = (0..vals.len()).filter(|&i| i != skip).collect();
            calc_diff(&sub)
        })
        .collect();

    for method in [BootstrapCiMethod::Percentile, BootstrapCiMethod::Bca] {
        let boot_opts = BootstrapOptions {
            enabled: true,
            samples,
            seed: Some(seed),
            ci_level: 95.0,
            ci_method: method,
            sampling_method: BootstrapSamplingMethod::Simple,
        };
        let res = compute_indep_ttest_bootstrap(
            "score",
            "group",
            &vals,
            &grps,
            &IndepTTestOptions::default(),
            None,
            &boot_opts,
        )
        .unwrap();

        assert_matches_oracle(
            res.mean_diff.as_ref().unwrap(),
            est,
            &reps,
            &jk,
            95.0,
            method,
        );
    }
}

#[test]
fn golden_oracle_pearson_correlation() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 11.0, 14.0];
    let y = [2.1, 3.9, 6.2, 7.8, 10.5, 11.8, 14.2, 17.5, 22.1, 27.0];
    let vx: Vec<Option<f64>> = x.iter().map(|&v| Some(v)).collect();
    let vy: Vec<Option<f64>> = y.iter().map(|&v| Some(v)).collect();
    let seed = 42_000;
    let samples = 200;

    let calc_r = |indices: &[usize]| -> f64 {
        let n = indices.len() as f64;
        let mx = indices.iter().map(|&i| x[i]).sum::<f64>() / n;
        let my = indices.iter().map(|&i| y[i]).sum::<f64>() / n;
        let mut sxx = 0.0;
        let mut syy = 0.0;
        let mut sxy = 0.0;
        for &i in indices {
            let dx = x[i] - mx;
            let dy = y[i] - my;
            sxx += dx * dx;
            syy += dy * dy;
            sxy += dx * dy;
        }
        sxy / (sxx * syy).sqrt()
    };

    let all_idx: Vec<usize> = (0..x.len()).collect();
    let est = calc_r(&all_idx);
    let idx_sets = replay_indices(x.len(), samples, seed);
    let reps: Vec<f64> = idx_sets.iter().map(|idx| calc_r(idx)).collect();
    let jk: Vec<f64> = (0..x.len())
        .map(|skip| {
            let sub: Vec<usize> = (0..x.len()).filter(|&i| i != skip).collect();
            calc_r(&sub)
        })
        .collect();

    for method in [BootstrapCiMethod::Percentile, BootstrapCiMethod::Bca] {
        let boot_opts = BootstrapOptions {
            enabled: true,
            samples,
            seed: Some(seed),
            ci_level: 95.0,
            ci_method: method,
            sampling_method: BootstrapSamplingMethod::Simple,
        };
        let input = [("x", vx.as_slice()), ("y", vy.as_slice())];
        let res = compute_bivariate_correlations_bootstrap(
            &input,
            &BivariateCorrelationsOptions::default(),
            None,
            &boot_opts,
        )
        .unwrap();

        assert_matches_oracle(
            res.pairs[0].r.as_ref().unwrap(),
            est,
            &reps,
            &jk,
            95.0,
            method,
        );
    }
}

#[test]
fn golden_oracle_linear_regression_ols_coefficients() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let y = [3.2, 5.1, 6.8, 9.2, 10.9, 13.1, 14.8, 17.3];
    let vy: Vec<Option<f64>> = y.iter().map(|&v| Some(v)).collect();
    let vx: Vec<Vec<Option<f64>>> = vec![x.iter().map(|&v| Some(v)).collect()];
    let indep_names = vec!["x".to_string()];
    let weights = vec![1.0; x.len()];
    let reg_opts = LinearRegressionOptions::default();
    let base_res =
        compute_linear_regression("y", &vy, &indep_names, &vx, &weights, &reg_opts).unwrap();

    let seed = 987_654;
    let samples = 200;

    let calc_ols = |indices: &[usize]| -> (f64, f64) {
        let n = indices.len() as f64;
        let mx = indices.iter().map(|&i| x[i]).sum::<f64>() / n;
        let my = indices.iter().map(|&i| y[i]).sum::<f64>() / n;
        let mut sxx = 0.0;
        let mut sxy = 0.0;
        for &i in indices {
            sxx += (x[i] - mx).powi(2);
            sxy += (x[i] - mx) * (y[i] - my);
        }
        let b1 = sxy / sxx;
        let b0 = my - b1 * mx;
        (b0, b1)
    };

    let all_idx: Vec<usize> = (0..x.len()).collect();
    let (est_b0, est_b1) = calc_ols(&all_idx);
    let idx_sets = replay_indices(x.len(), samples, seed);
    let reps_b0: Vec<f64> = idx_sets.iter().map(|idx| calc_ols(idx).0).collect();
    let reps_b1: Vec<f64> = idx_sets.iter().map(|idx| calc_ols(idx).1).collect();
    let jk_b0: Vec<f64> = (0..x.len())
        .map(|skip| {
            let sub: Vec<usize> = (0..x.len()).filter(|&i| i != skip).collect();
            calc_ols(&sub).0
        })
        .collect();
    let jk_b1: Vec<f64> = (0..x.len())
        .map(|skip| {
            let sub: Vec<usize> = (0..x.len()).filter(|&i| i != skip).collect();
            calc_ols(&sub).1
        })
        .collect();

    for method in [BootstrapCiMethod::Percentile, BootstrapCiMethod::Bca] {
        let boot_opts = BootstrapOptions {
            enabled: true,
            samples,
            seed: Some(seed),
            ci_level: 95.0,
            ci_method: method,
            sampling_method: BootstrapSamplingMethod::Simple,
        };
        let res = compute_linear_regression_bootstrap(
            "y",
            &vy,
            &indep_names,
            &vx,
            &weights,
            &reg_opts,
            &base_res,
            None,
            &boot_opts,
        )
        .unwrap();

        assert_matches_oracle(
            res.coefficients[0].b.as_ref().unwrap(),
            est_b0,
            &reps_b0,
            &jk_b0,
            95.0,
            method,
        );
        assert_matches_oracle(
            res.coefficients[1].b.as_ref().unwrap(),
            est_b1,
            &reps_b1,
            &jk_b1,
            95.0,
            method,
        );
    }
}
