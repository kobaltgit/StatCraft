use statcraft_algo::DescriptivesOptions;

#[derive(Debug, Clone)]
pub struct DescriptivesDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub options: DescriptivesOptions,
    pub show_options: bool,
    pub table_style: crate::dialogs::table_style_dialog::TableStyleState,
    pub bootstrap: crate::dialogs::bootstrap_dialog::BootstrapDialogState,
}

impl Default for DescriptivesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            options: DescriptivesOptions::spps_default(),
            show_options: false,
            table_style: crate::dialogs::table_style_dialog::TableStyleState::default(),
            bootstrap: crate::dialogs::bootstrap_dialog::BootstrapDialogState::default(),
        }
    }
}

impl DescriptivesDialogState {
    /// Resets the dialog state to SPSS defaults.
    pub fn reset(&mut self) {
        self.selected_vars.clear();
        self.source_selected = None;
        self.target_selected = None;
        self.options = DescriptivesOptions::spps_default();
        self.show_options = false;
        self.table_style.reset();
        self.bootstrap.reset();
    }
}

pub enum DescriptivesDialogAction {
    Run {
        vars: Vec<String>,
        options: DescriptivesOptions,
        table_styles: Option<serde_json::Value>,
        create_style_report: bool,
        bootstrap: Option<serde_json::Value>,
    },
    Paste {
        vars: Vec<String>,
        options: DescriptivesOptions,
        bootstrap: Option<serde_json::Value>,
    },
    None,
}
