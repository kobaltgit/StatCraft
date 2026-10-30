mod view;

use statcraft_algo::{DatePart, DateUnit};
pub use view::render_date_time_dialog;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateTimeDialogTask {
    #[default]
    Extract,
    Diff,
    Add,
    Construct,
}

#[derive(Debug, Clone)]
pub struct DateTimeDialogState {
    pub is_open: bool,
    pub task: DateTimeDialogTask,
    pub available_selected: Option<usize>,
    pub primary_var: Option<String>,
    pub primary_selected: bool,
    pub secondary_var: Option<String>,
    pub secondary_selected: bool,
    pub part: DatePart,
    pub unit: DateUnit,
    pub duration_amount: f64,
    pub target_var: String,
    pub target_label: String,
}

impl Default for DateTimeDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            task: DateTimeDialogTask::Extract,
            available_selected: None,
            primary_var: None,
            primary_selected: false,
            secondary_var: None,
            secondary_selected: false,
            part: DatePart::Year,
            unit: DateUnit::Days,
            duration_amount: 1.0,
            target_var: String::new(),
            target_label: String::new(),
        }
    }
}

impl DateTimeDialogState {
    pub fn reset(&mut self) {
        self.task = DateTimeDialogTask::Extract;
        self.available_selected = None;
        self.primary_var = None;
        self.primary_selected = false;
        self.secondary_var = None;
        self.secondary_selected = false;
        self.part = DatePart::Year;
        self.unit = DateUnit::Days;
        self.duration_amount = 1.0;
        self.target_var.clear();
        self.target_label.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DateTimeDialogAction {
    Run {
        operation: String,
        target_var: String,
        target_label: Option<String>,
        primary_var: String,
        secondary_var: Option<String>,
        part: Option<String>,
        unit: Option<String>,
        duration_value: Option<f64>,
    },
    Paste {
        operation: String,
        target_var: String,
        target_label: Option<String>,
        primary_var: String,
        secondary_var: Option<String>,
        part: Option<String>,
        unit: Option<String>,
        duration_value: Option<f64>,
    },
    None,
}
