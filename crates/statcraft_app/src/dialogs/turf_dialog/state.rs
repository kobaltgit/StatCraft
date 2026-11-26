use statcraft_algo::TurfOptions;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TurfDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,

    // Main options fields
    pub max_combinations_text: String,
    pub num_to_display_text: String,
    pub min_positive_pct_text: String,
    pub positive_values_text: String,
    pub always_required_text: String,
    pub removal_cycles_text: String,
    pub reach_and_frequency_chart: bool,
    pub show_only_max_combo: bool,
    pub limit_exceeded_abort: bool,
    pub max_operations_text: String,

    // Subdialog 1: Importance Weights
    pub weights_open: bool,
    pub weights_strict: bool,
    pub weights_spec_text: String,
    pub backup_weights_strict: bool,
    pub backup_weights_spec_text: String,

    // Subdialog 2: Advanced Selection
    pub advanced_open: bool,
    pub use_advanced_selection: bool,
    pub advanced_auto: bool,
    pub advanced_size_text: String,
    pub advanced_winners_text: String,
    pub backup_use_advanced_selection: bool,
    pub backup_advanced_auto: bool,
    pub backup_advanced_size_text: String,
    pub backup_advanced_winners_text: String,
}

impl Default for TurfDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            max_combinations_text: String::new(),
            num_to_display_text: "10".to_string(),
            min_positive_pct_text: String::new(),
            positive_values_text: "1".to_string(),
            always_required_text: "0".to_string(),
            removal_cycles_text: "0".to_string(),
            reach_and_frequency_chart: true,
            show_only_max_combo: false,
            limit_exceeded_abort: true,
            max_operations_text: "5000000".to_string(),

            weights_open: false,
            weights_strict: true,
            weights_spec_text: String::new(),
            backup_weights_strict: true,
            backup_weights_spec_text: String::new(),

            advanced_open: false,
            use_advanced_selection: true,
            advanced_auto: true,
            advanced_size_text: "18".to_string(),
            advanced_winners_text: "2".to_string(),
            backup_use_advanced_selection: true,
            backup_advanced_auto: true,
            backup_advanced_size_text: "18".to_string(),
            backup_advanced_winners_text: "2".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TurfDialogAction {
    Run { options: TurfOptions },
    Paste { options: TurfOptions },
    None,
}

impl TurfDialogState {
    pub fn open_weights(&mut self) {
        self.backup_weights_strict = self.weights_strict;
        self.backup_weights_spec_text = self.weights_spec_text.clone();
        self.weights_open = true;
    }

    pub fn close_weights(&mut self, accept: bool) {
        if !accept {
            self.weights_strict = self.backup_weights_strict;
            self.weights_spec_text = self.backup_weights_spec_text.clone();
        }
        self.weights_open = false;
    }

    pub fn open_advanced(&mut self) {
        self.backup_use_advanced_selection = self.use_advanced_selection;
        self.backup_advanced_auto = self.advanced_auto;
        self.backup_advanced_size_text = self.advanced_size_text.clone();
        self.backup_advanced_winners_text = self.advanced_winners_text.clone();
        self.advanced_open = true;
    }

    pub fn close_advanced(&mut self, accept: bool) {
        if !accept {
            self.use_advanced_selection = self.backup_use_advanced_selection;
            self.advanced_auto = self.backup_advanced_auto;
            self.advanced_size_text = self.backup_advanced_size_text.clone();
            self.advanced_winners_text = self.backup_advanced_winners_text.clone();
        }
        self.advanced_open = false;
    }

    pub fn reset(&mut self) {
        let is_open = self.is_open;
        *self = Self::default();
        self.is_open = is_open;
    }

    pub fn build_options(&self) -> Result<TurfOptions, String> {
        if self.selected_vars.is_empty() {
            return Err("No variables selected".to_string());
        }

        let max_combinations = if self.max_combinations_text.trim().is_empty() {
            None
        } else {
            let n = self
                .max_combinations_text
                .trim()
                .parse::<usize>()
                .map_err(|_| "Invalid max combinations".to_string())?;
            if n == 0 {
                return Err("Max combinations must be >= 1".to_string());
            }
            Some(n)
        };

        let num_to_display = if self.num_to_display_text.trim().is_empty() {
            None
        } else {
            let n = self
                .num_to_display_text
                .trim()
                .parse::<usize>()
                .map_err(|_| "Invalid num to display".to_string())?;
            if n == 0 {
                return Err("Number to display must be >= 1".to_string());
            }
            Some(n)
        };

        let min_positive_pct = if self.min_positive_pct_text.trim().is_empty() {
            None
        } else {
            let pct = self
                .min_positive_pct_text
                .trim()
                .parse::<f64>()
                .map_err(|_| "Invalid min positive pct".to_string())?;
            if !(0.0..=100.0).contains(&pct) {
                return Err("Minimum positive percent must be between 0 and 100".to_string());
            }
            Some(pct)
        };

        let positive_values: Vec<String> = if self.positive_values_text.trim().is_empty() {
            vec!["1".to_string()]
        } else {
            self.positive_values_text
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        };

        let always_required = if self.always_required_text.trim().is_empty() {
            0
        } else {
            self.always_required_text
                .trim()
                .parse::<usize>()
                .map_err(|_| "Invalid always required count".to_string())?
        };

        let removal_cycles = if self.removal_cycles_text.trim().is_empty() {
            0
        } else {
            self.removal_cycles_text
                .trim()
                .parse::<usize>()
                .map_err(|_| "Invalid removal cycles count".to_string())?
        };

        let max_operations = if self.max_operations_text.trim().is_empty() {
            5_000_000
        } else {
            let ops = self
                .max_operations_text
                .trim()
                .parse::<usize>()
                .map_err(|_| "Invalid max operations".to_string())?;
            if ops == 0 {
                return Err("Max operations must be >= 1".to_string());
            }
            ops
        };

        let advanced_size = if self.advanced_size_text.trim().is_empty() {
            18
        } else {
            let sz = self
                .advanced_size_text
                .trim()
                .parse::<usize>()
                .map_err(|_| "Invalid advanced size".to_string())?;
            if sz < 2 {
                return Err("Advanced size must be >= 2".to_string());
            }
            sz
        };

        let advanced_winners = if self.advanced_winners_text.trim().is_empty() {
            2
        } else {
            let w = self
                .advanced_winners_text
                .trim()
                .parse::<usize>()
                .map_err(|_| "Invalid advanced winners".to_string())?;
            if w == 0 {
                return Err("Advanced winners must be >= 1".to_string());
            }
            w
        };

        let mut importance_weights = HashMap::new();
        let tokens: Vec<&str> = self.weights_spec_text.split_whitespace().collect();
        if !tokens.is_empty() {
            if !tokens.len().is_multiple_of(2) {
                return Err(
                    "Importance weights must be pairs of variable name and weight".to_string(),
                );
            }
            for chunk in tokens.chunks(2) {
                let var_name = chunk[0].to_string();
                let weight = chunk[1]
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid weight value for variable {var_name}"))?;
                if weight < 0.0 {
                    return Err(format!("Weight for variable {var_name} must be >= 0"));
                }
                importance_weights.insert(var_name, weight);
            }
        }

        Ok(TurfOptions {
            variables: self.selected_vars.clone(),
            positive_values,
            max_combinations,
            num_to_display,
            min_positive_pct,
            always_required,
            removal_cycles,
            show_only_max_combo: self.show_only_max_combo,
            limit_exceeded_abort: self.limit_exceeded_abort,
            max_operations,
            importance_weights,
            weights_strict: self.weights_strict,
            use_advanced_selection: self.use_advanced_selection,
            advanced_auto: self.advanced_auto,
            advanced_size,
            advanced_winners,
            reach_and_frequency_chart: self.reach_and_frequency_chart,
        })
    }

    pub fn can_execute(&self) -> bool {
        !self.selected_vars.is_empty() && self.build_options().is_ok()
    }
}
