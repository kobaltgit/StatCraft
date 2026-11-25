use super::BootstrapError;
use serde::{Deserialize, Serialize};

/// Confidence interval method for bootstrap resampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapCiMethod {
    #[default]
    Percentile,
    Bca,
}

/// Resampling strategy for bootstrap replicates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum BootstrapSamplingMethod {
    #[default]
    Simple,
    Stratified {
        strata_vars: Vec<String>,
    },
}

/// Configuration options for the bootstrap resampling engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BootstrapOptions {
    pub enabled: bool,
    pub samples: usize,
    pub seed: Option<u64>,
    pub ci_level: f64,
    pub ci_method: BootstrapCiMethod,
    pub sampling_method: BootstrapSamplingMethod,
}

impl Default for BootstrapOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            samples: 1000,
            seed: None,
            ci_level: 95.0,
            ci_method: BootstrapCiMethod::Percentile,
            sampling_method: BootstrapSamplingMethod::Simple,
        }
    }
}

impl BootstrapOptions {
    /// Validates all bootstrap configuration parameters without panicking.
    pub fn validate(&self) -> Result<(), BootstrapError> {
        if self.samples < 2 {
            return Err(BootstrapError::InsufficientSamples {
                samples: self.samples,
            });
        }
        if !self.ci_level.is_finite() || self.ci_level <= 0.0 || self.ci_level >= 100.0 {
            return Err(BootstrapError::InvalidCiLevel {
                ci_level: self.ci_level,
            });
        }
        if let Some(seed) = self.seed {
            if !(1..=2_000_000_000).contains(&seed) {
                return Err(BootstrapError::InvalidSeed { seed });
            }
        }
        if let BootstrapSamplingMethod::Stratified { strata_vars } = &self.sampling_method {
            if strata_vars.is_empty() {
                return Err(BootstrapError::EmptyStrataVariables);
            }
        }
        Ok(())
    }
}
