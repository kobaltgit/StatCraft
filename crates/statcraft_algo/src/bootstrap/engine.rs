use super::ci::{summarize_bootstrap, BootstrapSummary};
use super::options::{BootstrapCiMethod, BootstrapOptions};
use super::resampler::SamplingPlan;
use super::BootstrapError;
use statcraft_core::Mt19937;

/// Runs a generic bootstrap resampling procedure for any scalar statistic.
///
/// - `plan`: Simple or Stratified sampling plan of total size `N`.
/// - `options`: Bootstrap configuration (`samples`, `seed`, `ci_level`, `ci_method`).
/// - `estimate_fn`: Computes the statistic from a slice of row indices in `0..N`.
/// - `jackknife_fn`: Optional leave-one-out estimator used for BCa acceleration.
///   When `None` and `ci_method == Bca`, `estimate_fn` is called on each leave-one-out
///   index slice `0..N \ {i}`.
pub fn run_bootstrap<F, J>(
    plan: &SamplingPlan,
    options: &BootstrapOptions,
    estimate_fn: F,
    jackknife_fn: Option<J>,
) -> Result<BootstrapSummary, BootstrapError>
where
    F: Fn(&[usize]) -> Option<f64>,
    J: Fn(&[usize]) -> Option<f64>,
{
    options.validate()?;
    let n = plan.len();
    if n == 0 {
        return Err(BootstrapError::EmptyDataset);
    }

    let full_indices: Vec<usize> = (0..n).collect();
    let estimate = estimate_fn(&full_indices)
        .filter(|v| v.is_finite())
        .ok_or(BootstrapError::InvalidPointEstimate)?;

    let mut rng = Mt19937::from_optional_seed(options.seed);
    let mut index_buf = Vec::with_capacity(n);
    let mut replicates = Vec::with_capacity(options.samples);

    for _ in 0..options.samples {
        plan.sample_indices(&mut rng, &mut index_buf);
        if let Some(val) = estimate_fn(&index_buf) {
            if val.is_finite() {
                replicates.push(val);
            }
        }
    }

    let jackknife_vals = if options.ci_method == BootstrapCiMethod::Bca && n >= 2 {
        let mut jk = Vec::with_capacity(n);
        let mut loo_buf = Vec::with_capacity(n - 1);
        for skip in 0..n {
            loo_buf.clear();
            loo_buf.extend(0..skip);
            loo_buf.extend((skip + 1)..n);
            let val = match &jackknife_fn {
                Some(jf) => jf(&loo_buf),
                None => estimate_fn(&loo_buf),
            };
            if let Some(v) = val {
                if v.is_finite() {
                    jk.push(v);
                }
            }
        }
        Some(jk)
    } else {
        None
    };

    summarize_bootstrap(
        estimate,
        &replicates,
        jackknife_vals.as_deref(),
        options.ci_level,
        options.ci_method,
    )
}

/// Convenience wrapper for `run_bootstrap` when no custom jackknife function is needed.
pub fn run_bootstrap_scalar<F>(
    plan: &SamplingPlan,
    options: &BootstrapOptions,
    estimate_fn: F,
) -> Result<BootstrapSummary, BootstrapError>
where
    F: Fn(&[usize]) -> Option<f64>,
{
    run_bootstrap(
        plan,
        options,
        estimate_fn,
        None::<fn(&[usize]) -> Option<f64>>,
    )
}
