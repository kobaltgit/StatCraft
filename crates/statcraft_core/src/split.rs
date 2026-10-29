use serde::{Deserialize, Serialize};
use statcraft_io::RawValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitFileMode {
    Off,
    CompareGroups,
    OrganizeOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitFileConfig {
    pub mode: SplitFileMode,
    pub variables: Vec<String>,
}

impl SplitFileConfig {
    pub fn new(mode: SplitFileMode, variables: Vec<String>) -> Self {
        Self { mode, variables }
    }

    pub fn is_active(&self) -> bool {
        self.mode != SplitFileMode::Off && !self.variables.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplitKeyValue {
    pub var_name: String,
    pub raw_value: RawValue,
    pub display_value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplitSubgroup {
    pub keys: Vec<SplitKeyValue>,
    pub label: String,
    pub row_indices: Vec<usize>,
}

impl SplitSubgroup {
    pub fn new(keys: Vec<SplitKeyValue>, label: String, row_indices: Vec<usize>) -> Self {
        Self {
            keys,
            label,
            row_indices,
        }
    }
}
