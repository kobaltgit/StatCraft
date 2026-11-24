use serde::{Deserialize, Serialize};
use serde_json::Value;
use statcraft_reporting::styling::{
    CellCondition, CellStyleFormat, DimensionTarget, TableStyleRule, TableTarget, TargetValue,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableStyleState {
    pub is_open: bool,
    pub rules: Vec<TableStyleRule>,
    pub selected_rule_idx: Option<usize>,
    pub create_style_report: bool,

    // Subdialog editing states
    pub editing_condition_idx: Option<usize>,
    pub cond_is_conditional: bool,
    pub cond_operator_idx: usize,
    pub cond_val1_input: String,
    pub cond_val2_input: String,
    pub cond_n_input: String,
    pub cond_abs_value: bool,

    pub editing_format_idx: Option<usize>,
    pub fmt_use_defaults: bool,
    pub fmt_bold: bool,
    pub fmt_italic: bool,
    pub fmt_underline: bool,
    pub fmt_bg_color: Option<[u8; 3]>,
    pub fmt_text_color: Option<[u8; 3]>,
    pub fmt_reset: bool,
    pub fmt_format_idx: usize,
    pub fmt_decimals_enabled: bool,
    pub fmt_decimals_val: String,
    pub fmt_apply_to_idx: usize,
    pub fmt_replace_enabled: bool,
    pub fmt_replace_val: String,

    // Backup for transactional cancellation
    pub backup: Option<Vec<TableStyleRule>>,
    pub backup_report: bool,
}

impl Default for TableStyleState {
    fn default() -> Self {
        Self {
            is_open: false,
            rules: vec![TableStyleRule {
                table_target: TableTarget::AllEligible,
                value_target: TargetValue::AllDataCells,
                dimension: DimensionTarget::Columns,
                condition: CellCondition::All,
                format: CellStyleFormat::default(),
            }],
            selected_rule_idx: Some(0),
            create_style_report: false,

            editing_condition_idx: None,
            cond_is_conditional: false,
            cond_operator_idx: 0,
            cond_val1_input: String::new(),
            cond_val2_input: String::new(),
            cond_n_input: "10".to_string(),
            cond_abs_value: false,

            editing_format_idx: None,
            fmt_use_defaults: false,
            fmt_bold: false,
            fmt_italic: false,
            fmt_underline: false,
            fmt_bg_color: None,
            fmt_text_color: None,
            fmt_reset: false,
            fmt_format_idx: 0,
            fmt_decimals_enabled: false,
            fmt_decimals_val: "0".to_string(),
            fmt_apply_to_idx: 0,
            fmt_replace_enabled: false,
            fmt_replace_val: String::new(),

            backup: None,
            backup_report: false,
        }
    }
}

impl TableStyleState {
    /// Resets the dialog state to default clean rules.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Adds a new default rule at the end and selects it.
    pub fn add_rule(&mut self) {
        let rule = TableStyleRule {
            table_target: TableTarget::AllEligible,
            value_target: TargetValue::AllDataCells,
            dimension: DimensionTarget::Columns,
            condition: CellCondition::All,
            format: CellStyleFormat::default(),
        };
        self.rules.push(rule);
        self.selected_rule_idx = Some(self.rules.len() - 1);
    }

    /// Duplicates currently selected rule, inserting immediately below it.
    pub fn duplicate_rule(&mut self) {
        if let Some(idx) = self.selected_rule_idx {
            if idx < self.rules.len() {
                let cloned = self.rules[idx].clone();
                self.rules.insert(idx + 1, cloned);
                self.selected_rule_idx = Some(idx + 1);
            }
        }
    }

    /// Moves selected rule up by one position if possible.
    pub fn move_rule_up(&mut self) {
        if let Some(idx) = self.selected_rule_idx {
            if idx > 0 && idx < self.rules.len() {
                self.rules.swap(idx, idx - 1);
                self.selected_rule_idx = Some(idx - 1);
            }
        }
    }

    /// Moves selected rule down by one position if possible.
    pub fn move_rule_down(&mut self) {
        if let Some(idx) = self.selected_rule_idx {
            if idx + 1 < self.rules.len() {
                self.rules.swap(idx, idx + 1);
                self.selected_rule_idx = Some(idx + 1);
            }
        }
    }

    /// Deletes currently selected rule.
    pub fn delete_rule(&mut self) {
        if let Some(idx) = self.selected_rule_idx {
            if idx < self.rules.len() {
                self.rules.remove(idx);
                if self.rules.is_empty() {
                    self.selected_rule_idx = None;
                } else if idx >= self.rules.len() {
                    self.selected_rule_idx = Some(self.rules.len() - 1);
                }
            }
        }
    }

    /// Commits and closes the main style dialog.
    pub fn confirm(&mut self) {
        self.backup = None;
        self.is_open = false;
        self.editing_condition_idx = None;
        self.editing_format_idx = None;
    }

    /// Cancels and reverts changes made to the rules list.
    pub fn cancel(&mut self) {
        if let Some(backup) = self.backup.take() {
            self.rules = backup;
            self.create_style_report = self.backup_report;
        }
        self.is_open = false;
        self.editing_condition_idx = None;
        self.editing_format_idx = None;
    }

    /// Prepares subdialog state for editing condition of a rule.
    pub fn open_condition_editor(&mut self, idx: usize) {
        if idx >= self.rules.len() {
            return;
        }
        self.editing_condition_idx = Some(idx);
        self.cond_abs_value = false;
        let cond = &self.rules[idx].condition;
        match cond {
            CellCondition::All => {
                self.cond_is_conditional = false;
                self.cond_operator_idx = 0;
                self.cond_val1_input.clear();
                self.cond_val2_input.clear();
                self.cond_n_input = "10".to_string();
            }
            CellCondition::Equal(v) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 0;
                self.cond_val1_input = v.to_string();
            }
            CellCondition::NotEqual(v) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 1;
                self.cond_val1_input = v.to_string();
            }
            CellCondition::LessThan(v) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 2;
                self.cond_val1_input = v.to_string();
            }
            CellCondition::LessOrEqual(v) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 3;
                self.cond_val1_input = v.to_string();
            }
            CellCondition::GreaterThan(v) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 4;
                self.cond_val1_input = v.to_string();
            }
            CellCondition::GreaterOrEqual(v) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 5;
                self.cond_val1_input = v.to_string();
            }
            CellCondition::Between(low, high) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 6;
                self.cond_val1_input = low.to_string();
                self.cond_val2_input = high.to_string();
            }
            CellCondition::NotBetween(low, high) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 7;
                self.cond_val1_input = low.to_string();
                self.cond_val2_input = high.to_string();
            }
            CellCondition::TopN(n) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 8;
                self.cond_n_input = n.to_string();
            }
            CellCondition::BottomN(n) => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 9;
                self.cond_n_input = n.to_string();
            }
            CellCondition::SysMis => {
                self.cond_is_conditional = true;
                self.cond_operator_idx = 10;
            }
        }
    }

    /// Prepares subdialog state for editing format of a rule.
    pub fn open_format_editor(&mut self, idx: usize) {
        if idx >= self.rules.len() {
            return;
        }
        self.editing_format_idx = Some(idx);
        let fmt = &self.rules[idx].format;
        self.fmt_bold = fmt.bold;
        self.fmt_italic = fmt.italic;
        self.fmt_underline = fmt.underline;
        self.fmt_bg_color = fmt.bg_color;
        self.fmt_text_color = fmt.text_color;
        self.fmt_reset = fmt.reset_defaults;
        self.fmt_use_defaults = fmt.reset_defaults;
        self.fmt_format_idx = fmt.format_mask_idx;
        if let Some(d) = fmt.decimals {
            self.fmt_decimals_enabled = true;
            self.fmt_decimals_val = d.to_string();
        } else {
            self.fmt_decimals_enabled = false;
            self.fmt_decimals_val = "0".to_string();
        }
        self.fmt_apply_to_idx = match fmt.target_scope {
            statcraft_reporting::styling::CellTargetScope::CellsOnly => 0,
            statcraft_reporting::styling::CellTargetScope::EntireColumn => 1,
            statcraft_reporting::styling::CellTargetScope::EntireRow => 2,
        };
        if let Some(ref r) = fmt.replace_value {
            self.fmt_replace_enabled = true;
            self.fmt_replace_val = r.clone();
        } else {
            self.fmt_replace_enabled = false;
            self.fmt_replace_val.clear();
        }
    }

    /// Converts active rules into JSON value for CommandBus.
    pub fn rules_to_json(&self) -> Option<Value> {
        if self.rules.is_empty() {
            return None;
        }
        serde_json::to_value(&self.rules).ok()
    }
}
