use statcraft_core::dataset::merge::MergeMethod;
use statcraft_core::DataSet;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MergeSourceKind {
    OpenSession(usize),
    #[default]
    ExternalFile,
}

#[derive(Debug, Clone)]
pub struct AddCasesDialogState {
    pub is_open: bool,
    pub source_kind: MergeSourceKind,
    pub external_file_path: String,
    pub selected_session_id: Option<usize>,
    pub secondary_dataset: Option<DataSet>,

    pub unpaired_selected: Option<usize>,
    pub included_selected: Option<usize>,
    pub included_vars: Vec<String>,
    pub unpaired_vars: Vec<(String, bool)>, // (var_name, is_from_active)

    pub in_variable_enabled: bool,
    pub in_variable_name: String,

    pub error_msg: Option<String>,
}

impl Default for AddCasesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_kind: MergeSourceKind::ExternalFile,
            external_file_path: String::new(),
            selected_session_id: None,
            secondary_dataset: None,
            unpaired_selected: None,
            included_selected: None,
            included_vars: Vec::new(),
            unpaired_vars: Vec::new(),
            in_variable_enabled: false,
            in_variable_name: "source01".to_string(),
            error_msg: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AddVarsDialogState {
    pub is_open: bool,
    pub source_kind: MergeSourceKind,
    pub external_file_path: String,
    pub selected_session_id: Option<usize>,
    pub secondary_dataset: Option<DataSet>,

    pub method: MergeMethod,
    pub auto_sort_keys: bool,

    pub excluded_selected: Option<usize>,
    pub new_active_selected: Option<usize>,
    pub key_selected: Option<usize>,

    pub excluded_vars: Vec<String>,
    pub new_active_vars: Vec<String>,
    pub key_vars: Vec<String>,

    pub in_variable_enabled: bool,
    pub in_variable_name: String,

    pub error_msg: Option<String>,
}

impl Default for AddVarsDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_kind: MergeSourceKind::ExternalFile,
            external_file_path: String::new(),
            selected_session_id: None,
            secondary_dataset: None,
            method: MergeMethod::OneToOneWithKeys,
            auto_sort_keys: true,
            excluded_selected: None,
            new_active_selected: None,
            key_selected: None,
            excluded_vars: Vec::new(),
            new_active_vars: Vec::new(),
            key_vars: Vec::new(),
            in_variable_enabled: false,
            in_variable_name: "in_second".to_string(),
            error_msg: None,
        }
    }
}

pub enum MergeCasesAction {
    Run {
        file_path: String,
        in_variable: Option<String>,
        renames: HashMap<String, String>,
        interleave_by: Option<Vec<String>>,
    },
    Paste {
        file_path: String,
        in_variable: Option<String>,
        renames: HashMap<String, String>,
        interleave_by: Option<Vec<String>>,
    },
    None,
}

pub enum MergeVarsAction {
    Run {
        file_path: String,
        method: MergeMethod,
        keys: Vec<String>,
        renames: HashMap<String, String>,
        drop_second: Vec<String>,
        in_second: Option<String>,
    },
    Paste {
        file_path: String,
        method: MergeMethod,
        keys: Vec<String>,
        renames: HashMap<String, String>,
        drop_second: Vec<String>,
        in_second: Option<String>,
    },
    None,
}
