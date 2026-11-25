use crate::dialogs::bootstrap_dialog::BootstrapDialogState;
use crate::dialogs::table_style_dialog::TableStyleState;
use serde_json::Value;
use statcraft_reporting::styling::TableStyleRule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemingMode {
    Orthogonal,
    CustomRatio,
}

#[derive(Debug, Clone)]
pub struct DemingRegressionDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_var: Option<String>,
    pub indep_var: Option<String>,
    pub target_dep_selected: bool,
    pub target_indep_selected: bool,
    pub mode: DemingMode,
    pub lambda_str: String,
    pub ci_level_str: String,
    pub include_scatter_plot: bool,
    pub include_identity_line: bool,
    pub include_residual_plot: bool,
    pub save_predicted: bool,
    pub save_residuals: bool,
    pub show_options: bool,
    pub show_plots: bool,
    pub show_save: bool,
    pub table_style: TableStyleState,
    pub bootstrap: BootstrapDialogState,
}

impl Default for DemingRegressionDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            dep_var: None,
            indep_var: None,
            target_dep_selected: false,
            target_indep_selected: false,
            mode: DemingMode::Orthogonal,
            lambda_str: "1.0".to_string(),
            ci_level_str: "95".to_string(),
            include_scatter_plot: true,
            include_identity_line: true,
            include_residual_plot: false,
            save_predicted: false,
            save_residuals: false,
            show_options: false,
            show_plots: false,
            show_save: false,
            table_style: TableStyleState::default(),
            bootstrap: BootstrapDialogState::default(),
        }
    }
}

pub enum DemingRegressionDialogAction {
    Run {
        dependent_var: String,
        independent_var: String,
        lambda: f64,
        ci: f64,
        include_scatter_plot: bool,
        include_identity_line: bool,
        include_residual_plot: bool,
        save_predicted: bool,
        save_residuals: bool,
        table_styles: Vec<TableStyleRule>,
        bootstrap: Option<Value>,
    },
    Paste {
        dependent_var: String,
        independent_var: String,
        lambda: f64,
        ci: f64,
        include_scatter_plot: bool,
        include_identity_line: bool,
        include_residual_plot: bool,
        save_predicted: bool,
        save_residuals: bool,
        bootstrap: Option<Value>,
    },
    None,
}
