use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration options for Add Cases (`ADD FILES`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AddCasesConfig {
    /// Variables to include in the output dataset. If empty, all paired and unpaired are included.
    pub included_variables: Vec<String>,
    /// Variable renames for the secondary dataset: old_name -> new_name.
    pub renames_second: HashMap<String, String>,
    /// Optional indicator variable name (SPSS `/IN=source01`).
    /// 0 for active cases, 1 for added secondary cases.
    pub in_variable: Option<String>,
    /// Optional key variables for sorting / interleaving cases (SPSS `/BY key1...`).
    pub interleave_by: Option<Vec<String>>,
}

/// Merge method for Add Variables (`MATCH FILES`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MergeMethod {
    /// One-to-one match by key values (`/BY key...`).
    #[default]
    OneToOneWithKeys,
    /// One-to-many keyed table lookup (`/TABLE='file' /BY key...`).
    OneToManyTable,
    /// Sequential match by case order (row-by-row, no keys).
    Sequential,
}

/// Configuration options for Add Variables (`MATCH FILES`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AddVarsConfig {
    /// Merge method: 1:1 with keys, 1:N table lookup, or sequential.
    pub method: MergeMethod,
    /// Key variables present in both datasets (`/BY key1 key2...`).
    pub keys: Vec<String>,
    /// Variable renames for the secondary dataset: old_name -> new_name.
    pub renames_second: HashMap<String, String>,
    /// Variables from secondary dataset to drop / exclude.
    pub drop_second: Vec<String>,
    /// Optional indicator variable for active cases presence (`/IN=in_active`).
    pub in_active_variable: Option<String>,
    /// Optional indicator variable for secondary cases presence (`/IN=in_second`).
    pub in_second_variable: Option<String>,
}

/// Pairing status between variables in two datasets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariablePairStatus {
    Paired,
    UnpairedActive,
    UnpairedSecond,
    TypeMismatch,
}

/// Metadata summary of paired variables for UI analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct VariablePairInfo {
    pub name: String,
    pub status: VariablePairStatus,
    pub active_type: Option<crate::variable::DataType>,
    pub second_type: Option<crate::variable::DataType>,
}
