pub mod ci;
pub mod engine;
pub mod options;
pub mod procedures;
pub mod resampler;

pub use ci::{
    compute_bca_ci, compute_percentile_ci, normal_cdf, summarize_bootstrap, BootstrapCi,
    BootstrapSummary,
};
pub use engine::{run_bootstrap, run_bootstrap_scalar};
pub use options::{BootstrapCiMethod, BootstrapOptions, BootstrapSamplingMethod};
pub use procedures::{
    compute_bivariate_correlations_bootstrap, compute_descriptives_bootstrap,
    compute_indep_ttest_bootstrap, compute_oneway_anova_bootstrap, compute_paired_ttest_bootstrap,
    OneWayAnovaBootstrap, OneWayAnovaBootstrapGroup, OneWayAnovaBootstrapPair,
};
pub use resampler::{SamplingPlan, StratifiedPlan};

use thiserror::Error;

/// Errors returned by the bootstrap resampling engine.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum BootstrapError {
    #[error("Cannot run bootstrap on an empty dataset (N = 0)")]
    EmptyDataset,

    #[error("Bootstrap requires at least 2 samples, got {samples}")]
    InsufficientSamples { samples: usize },

    #[error("Confidence level must be strictly between 0 and 100, got {ci_level}")]
    InvalidCiLevel { ci_level: f64 },

    #[error("Bootstrap seed must be between 1 and 2,000,000,000, got {seed}")]
    InvalidSeed { seed: u64 },

    #[error("Stratified bootstrap requires at least one strata variable")]
    EmptyStrataVariables,

    #[error("Strata keys length ({actual}) does not match dataset size ({expected})")]
    StrataLengthMismatch { expected: usize, actual: usize },

    #[error("Point estimate for original sample is non-finite or unavailable")]
    InvalidPointEstimate,

    #[error("Too few valid bootstrap replicates ({valid}) to compute summary")]
    InsufficientValidReplicates { valid: usize },
}
