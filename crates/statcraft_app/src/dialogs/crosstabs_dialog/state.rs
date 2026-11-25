use crate::dialogs::bootstrap_dialog::BootstrapDialogState;
use crate::dialogs::table_style_dialog::TableStyleState;
use serde_json::Value;
use statcraft_algo::{CrosstabOptions, ExactMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrosstabsTargetList {
    Row,
    Col,
    Layer,
}

#[derive(Debug, Clone)]
pub struct CrosstabsDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub row_selected: Option<usize>,
    pub col_selected: Option<usize>,
    pub layer_selected: Option<usize>,
    pub active_target: CrosstabsTargetList,
    pub row_vars: Vec<String>,
    pub col_vars: Vec<String>,
    pub layers: Vec<Vec<String>>,
    pub current_layer_idx: usize,
    pub options: CrosstabOptions,

    // Subdialog visibility flags
    pub show_exact: bool,
    pub show_stats: bool,
    pub show_cells: bool,
    pub show_format: bool,

    // Reused subdialog states
    pub table_style: TableStyleState,
    pub bootstrap: BootstrapDialogState,

    // Numeric input text buffers
    pub mc_confidence_input: String,
    pub mc_samples_input: String,
    pub exact_time_enabled: bool,
    pub exact_time_input: String,
    pub hide_small_input: String,
    pub cmh_or_input: String,

    // Transactional backups for subdialog Cancel
    pub exact_backup: Option<(ExactMode, String, String, bool, String)>,
    pub stats_backup: Option<(CrosstabOptions, String)>,
    pub cells_backup: Option<(CrosstabOptions, String)>,
    pub format_backup: Option<CrosstabOptions>,
}

impl Default for CrosstabsDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            row_selected: None,
            col_selected: None,
            layer_selected: None,
            active_target: CrosstabsTargetList::Row,
            row_vars: Vec::new(),
            col_vars: Vec::new(),
            layers: vec![Vec::new()],
            current_layer_idx: 0,
            options: CrosstabOptions::default(),
            show_exact: false,
            show_stats: false,
            show_cells: false,
            show_format: false,
            table_style: TableStyleState::default(),
            bootstrap: BootstrapDialogState::default(),
            mc_confidence_input: "99".to_string(),
            mc_samples_input: "10000".to_string(),
            exact_time_enabled: true,
            exact_time_input: "5".to_string(),
            hide_small_input: "5".to_string(),
            cmh_or_input: "1".to_string(),
            exact_backup: None,
            stats_backup: None,
            cells_backup: None,
            format_backup: None,
        }
    }
}

impl CrosstabsDialogState {
    pub fn ensure_valid_layer(&mut self) {
        if self.layers.is_empty() {
            self.layers.push(Vec::new());
        }
        if self.current_layer_idx >= self.layers.len() {
            self.current_layer_idx = self.layers.len() - 1;
        }
    }

    pub fn current_layer_vars(&self) -> &[String] {
        self.layers
            .get(self.current_layer_idx)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn current_layer_vars_mut(&mut self) -> &mut Vec<String> {
        self.ensure_valid_layer();
        &mut self.layers[self.current_layer_idx]
    }

    pub fn has_any_layer_vars(&self) -> bool {
        self.layers.iter().any(|l| !l.is_empty())
    }

    pub fn is_var_selected(&self, name: &str) -> bool {
        self.row_vars.iter().any(|v| v == name)
            || self.col_vars.iter().any(|v| v == name)
            || self.layers.iter().any(|l| l.iter().any(|v| v == name))
    }

    pub fn prev_layer(&mut self) {
        if self.current_layer_idx > 0 {
            if self.layers.last().is_some_and(|l| l.is_empty())
                && self.current_layer_idx == self.layers.len() - 1
                && self.layers.len() > 1
            {
                self.layers.pop();
            }
            self.current_layer_idx -= 1;
            self.layer_selected = None;
        }
    }

    pub fn next_layer(&mut self) {
        self.ensure_valid_layer();
        if self.current_layer_idx + 1 < self.layers.len() {
            self.current_layer_idx += 1;
            self.layer_selected = None;
        } else if !self.layers[self.current_layer_idx].is_empty() {
            self.layers.push(Vec::new());
            self.current_layer_idx += 1;
            self.layer_selected = None;
        }
    }

    pub fn non_empty_layers(&self) -> Vec<Vec<String>> {
        self.layers
            .iter()
            .filter(|l| !l.is_empty())
            .cloned()
            .collect()
    }

    pub fn sync_exact_from_buffers(&mut self) {
        match &mut self.options.exact_mode {
            ExactMode::AsymptoticOnly => {}
            ExactMode::MonteCarlo {
                confidence,
                samples,
            } => {
                if let Ok(pct) = self.mc_confidence_input.trim().parse::<f64>() {
                    if (50.0..100.0).contains(&pct) {
                        *confidence = pct / 100.0;
                    }
                }
                if let Ok(n) = self.mc_samples_input.trim().parse::<usize>() {
                    if n > 0 {
                        *samples = n;
                    }
                }
            }
            ExactMode::Exact { time_limit_min } => {
                if self.exact_time_enabled {
                    let mins = self
                        .exact_time_input
                        .trim()
                        .parse::<u32>()
                        .ok()
                        .filter(|&m| m > 0)
                        .unwrap_or(5);
                    *time_limit_min = Some(mins);
                } else {
                    *time_limit_min = None;
                }
            }
        }
    }

    pub fn sync_cells_from_buffers(&mut self) {
        if let Ok(thresh) = self.hide_small_input.trim().parse::<usize>() {
            if thresh > 0 {
                self.options.hide_small_threshold = thresh;
            }
        }
    }

    pub fn sync_stats_from_buffers(&mut self) {
        if let Ok(or_val) = self.cmh_or_input.trim().parse::<f64>() {
            if or_val > 0.0 && or_val.is_finite() {
                self.options.cmh_common_odds_ratio = or_val;
            }
        }
    }
}

pub enum CrosstabsDialogAction {
    Run {
        rows: Vec<String>,
        cols: Vec<String>,
        layers: Vec<Vec<String>>,
        options: CrosstabOptions,
        table_styles: Option<Value>,
        create_style_report: bool,
        bootstrap: Option<Value>,
    },
    Paste {
        rows: Vec<String>,
        cols: Vec<String>,
        layers: Vec<Vec<String>>,
        options: CrosstabOptions,
        table_styles: Option<Value>,
        create_style_report: bool,
        bootstrap: Option<Value>,
    },
    None,
}
