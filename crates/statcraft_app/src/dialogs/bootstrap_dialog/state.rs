use serde::{Deserialize, Serialize};
use serde_json::Value;
use statcraft_algo::{BootstrapCiMethod, BootstrapOptions, BootstrapSamplingMethod};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BootstrapSamplingUi {
    #[default]
    Simple,
    Stratified,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapBackup {
    pub enabled: bool,
    pub samples_input: String,
    pub use_seed: bool,
    pub seed_input: String,
    pub ci_level_input: String,
    pub ci_method: BootstrapCiMethod,
    pub sampling_method: BootstrapSamplingUi,
    pub strata_vars: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapDialogState {
    pub is_open: bool,
    pub enabled: bool,
    pub samples_input: String,
    pub use_seed: bool,
    pub seed_input: String,
    pub ci_level_input: String,
    pub ci_method: BootstrapCiMethod,
    pub sampling_method: BootstrapSamplingUi,
    pub strata_vars: Vec<String>,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub backup: Option<BootstrapBackup>,
}

impl Default for BootstrapDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            enabled: false,
            samples_input: "1000".to_string(),
            use_seed: false,
            seed_input: "2000000".to_string(),
            ci_level_input: "95".to_string(),
            ci_method: BootstrapCiMethod::Percentile,
            sampling_method: BootstrapSamplingUi::Simple,
            strata_vars: Vec::new(),
            source_selected: None,
            target_selected: None,
            backup: None,
        }
    }
}

impl BootstrapDialogState {
    pub fn open(&mut self) {
        self.backup = Some(BootstrapBackup {
            enabled: self.enabled,
            samples_input: self.samples_input.clone(),
            use_seed: self.use_seed,
            seed_input: self.seed_input.clone(),
            ci_level_input: self.ci_level_input.clone(),
            ci_method: self.ci_method,
            sampling_method: self.sampling_method,
            strata_vars: self.strata_vars.clone(),
        });
        self.source_selected = None;
        self.target_selected = None;
        self.is_open = true;
    }

    pub fn commit(&mut self) {
        self.backup = None;
        self.is_open = false;
    }

    pub fn cancel(&mut self) {
        if let Some(b) = self.backup.take() {
            self.enabled = b.enabled;
            self.samples_input = b.samples_input;
            self.use_seed = b.use_seed;
            self.seed_input = b.seed_input;
            self.ci_level_input = b.ci_level_input;
            self.ci_method = b.ci_method;
            self.sampling_method = b.sampling_method;
            self.strata_vars = b.strata_vars;
        }
        self.source_selected = None;
        self.target_selected = None;
        self.is_open = false;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn can_continue(&self) -> bool {
        if !self.enabled {
            return true;
        }
        let samples_ok = self
            .samples_input
            .trim()
            .parse::<usize>()
            .is_ok_and(|n| n >= 2);
        let ci_ok = self
            .ci_level_input
            .trim()
            .parse::<f64>()
            .is_ok_and(|v| v.is_finite() && v > 0.0 && v < 100.0);
        let seed_ok = !self.use_seed
            || self
                .seed_input
                .trim()
                .parse::<u64>()
                .is_ok_and(|s| (1..=2_000_000_000).contains(&s));
        let strata_ok =
            self.sampling_method != BootstrapSamplingUi::Stratified || !self.strata_vars.is_empty();

        samples_ok && ci_ok && seed_ok && strata_ok
    }

    pub fn to_bootstrap_options(&self) -> Option<BootstrapOptions> {
        if !self.enabled {
            return None;
        }
        let samples = self
            .samples_input
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|&n| n >= 2)
            .unwrap_or(1000);
        let ci_level = self
            .ci_level_input
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|&v| v.is_finite() && v > 0.0 && v < 100.0)
            .unwrap_or(95.0);
        let seed = if self.use_seed {
            self.seed_input
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|&s| (1..=2_000_000_000).contains(&s))
        } else {
            None
        };
        let sampling_method = match self.sampling_method {
            BootstrapSamplingUi::Simple => BootstrapSamplingMethod::Simple,
            BootstrapSamplingUi::Stratified => BootstrapSamplingMethod::Stratified {
                strata_vars: self.strata_vars.clone(),
            },
        };

        Some(BootstrapOptions {
            enabled: true,
            samples,
            seed,
            ci_level,
            ci_method: self.ci_method,
            sampling_method,
        })
    }

    pub fn to_json(&self) -> Option<Value> {
        self.to_bootstrap_options()
            .and_then(|opts| serde_json::to_value(&opts).ok())
    }
}
