use crate::dialogs::bootstrap_dialog::BootstrapDialogState;
use crate::dialogs::table_style_dialog::TableStyleState;
use serde_json::Value;
use statcraft_reporting::styling::TableStyleRule;

#[derive(Debug, Clone)]
pub struct TheilSenDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_var: Option<String>,
    pub indep_var: Option<String>,
    pub target_dep_selected: bool,
    pub target_indep_selected: bool,
    pub ci_level_str: String,
    pub mann_kendall: bool,
    pub include_scatter_plot: bool,
    pub include_residual_plot: bool,
    pub save_predicted: bool,
    pub save_residuals: bool,
    pub show_options: bool,
    pub show_plots: bool,
    pub show_save: bool,
    pub table_style: TableStyleState,
    pub bootstrap: BootstrapDialogState,
}

impl Default for TheilSenDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            dep_var: None,
            indep_var: None,
            target_dep_selected: false,
            target_indep_selected: false,
            ci_level_str: "95".to_string(),
            mann_kendall: true,
            include_scatter_plot: true,
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

pub enum TheilSenDialogAction {
    Run {
        dependent_var: String,
        independent_var: String,
        ci: f64,
        mann_kendall: bool,
        include_scatter_plot: bool,
        include_residual_plot: bool,
        save_predicted: bool,
        save_residuals: bool,
        table_styles: Vec<TableStyleRule>,
        bootstrap: Option<Value>,
    },
    Paste {
        dependent_var: String,
        independent_var: String,
        ci: f64,
        mann_kendall: bool,
        include_scatter_plot: bool,
        include_residual_plot: bool,
        save_predicted: bool,
        save_residuals: bool,
        bootstrap: Option<Value>,
    },
    None,
}
