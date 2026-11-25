use super::BootstrapError;
use statcraft_core::Mt19937;
use std::collections::HashMap;
use std::hash::Hash;

/// Precomputed stratum index groups for stratified bootstrap resampling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StratifiedPlan {
    strata: Vec<Vec<usize>>,
    total_len: usize,
}

impl StratifiedPlan {
    /// Builds a stratified resampling plan from per-row stratum keys.
    /// Preserves the deterministic order of first appearance for each stratum.
    pub fn from_keys<K: Eq + Hash>(keys: &[K]) -> Result<Self, BootstrapError> {
        if keys.is_empty() {
            return Err(BootstrapError::EmptyDataset);
        }

        let mut key_to_idx: HashMap<&K, usize> = HashMap::new();
        let mut strata: Vec<Vec<usize>> = Vec::new();

        for (row_idx, key) in keys.iter().enumerate() {
            if let Some(&stratum_idx) = key_to_idx.get(key) {
                strata[stratum_idx].push(row_idx);
            } else {
                let stratum_idx = strata.len();
                key_to_idx.insert(key, stratum_idx);
                strata.push(vec![row_idx]);
            }
        }

        Ok(Self {
            strata,
            total_len: keys.len(),
        })
    }

    /// Returns the total number of observations across all strata.
    pub fn total_len(&self) -> usize {
        self.total_len
    }

    /// Returns the number of distinct strata.
    pub fn num_strata(&self) -> usize {
        self.strata.len()
    }

    /// Returns the index slices for each stratum.
    pub fn strata(&self) -> &[Vec<usize>] {
        &self.strata
    }
}

/// Resampling plan for simple or stratified bootstrap draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SamplingPlan {
    Simple { n: usize },
    Stratified(StratifiedPlan),
}

impl SamplingPlan {
    /// Creates a simple random sampling plan for `n` observations.
    pub fn simple(n: usize) -> Result<Self, BootstrapError> {
        if n == 0 {
            return Err(BootstrapError::EmptyDataset);
        }
        Ok(Self::Simple { n })
    }

    /// Creates a stratified sampling plan from per-row stratum keys.
    pub fn stratified<K: Eq + Hash>(keys: &[K]) -> Result<Self, BootstrapError> {
        Ok(Self::Stratified(StratifiedPlan::from_keys(keys)?))
    }

    /// Creates a sampling plan from `BootstrapOptions` and optional per-row stratum keys.
    pub fn from_options(
        n: usize,
        strata_keys: Option<&[String]>,
        options: &super::options::BootstrapOptions,
    ) -> Result<Self, BootstrapError> {
        match &options.sampling_method {
            super::options::BootstrapSamplingMethod::Simple => Self::simple(n),
            super::options::BootstrapSamplingMethod::Stratified { .. } => {
                let keys = strata_keys.ok_or(BootstrapError::EmptyStrataVariables)?;
                if keys.len() != n {
                    return Err(BootstrapError::StrataLengthMismatch {
                        expected: n,
                        actual: keys.len(),
                    });
                }
                Self::stratified(keys)
            }
        }
    }

    /// Returns the total number of observations $N$.
    pub fn len(&self) -> usize {
        match self {
            Self::Simple { n } => *n,
            Self::Stratified(plan) => plan.total_len(),
        }
    }

    /// Returns `true` if the plan has zero observations.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Samples a single bootstrap replicate of row indices into `out`.
    /// For `Stratified`, each stratum $h$ of size $n_h$ is sampled independently
    /// with replacement, preserving exact stratum sizes $n_h$.
    pub fn sample_indices(&self, rng: &mut Mt19937, out: &mut Vec<usize>) {
        out.clear();
        match self {
            Self::Simple { n } => {
                out.reserve(*n);
                for _ in 0..*n {
                    out.push(rng.gen_range_usize(*n));
                }
            }
            Self::Stratified(plan) => {
                out.reserve(plan.total_len());
                for stratum in plan.strata() {
                    let nh = stratum.len();
                    for _ in 0..nh {
                        let pick = rng.gen_range_usize(nh);
                        out.push(stratum[pick]);
                    }
                }
            }
        }
    }
}
