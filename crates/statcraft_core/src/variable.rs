use serde::{Deserialize, Serialize};
use statcraft_io::RawValue;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CoreError {
    #[error("Variable index {index} out of bounds (max: {total})")]
    IndexOutOfBounds { index: usize, total: usize },

    #[error("Variable name '{0}' is invalid: must be non-empty and start with letter")]
    InvalidVariableName(String),

    #[error("Variable '{0}' already exists in dictionary")]
    DuplicateVariableName(String),

    #[error("Variable '{0}' not found in dictionary")]
    VariableNotFound(String),

    #[error("Row {row} or column {col} out of dataset bounds")]
    CellOutOfBounds { row: usize, col: usize },

    #[error("Cannot parse value '{0}' as numeric")]
    TypeMismatch(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Measure {
    Scale,
    Ordinal,
    Nominal,
}

impl Measure {
    pub fn as_key(&self) -> &'static str {
        match self {
            Measure::Scale => "measure-scale",
            Measure::Ordinal => "measure-ordinal",
            Measure::Nominal => "measure-nominal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Numeric { decimals: u8 },
    Comma { decimals: u8 },
    Dot { decimals: u8 },
    Scientific { decimals: u8 },
    Date { format: String },
    Dollar { decimals: u8 },
    CustomCurrency { format_code: String, decimals: u8 },
    String { max_len: usize },
    RestrictedNumeric { width: usize },
}

impl DataType {
    pub fn as_key(&self) -> &'static str {
        match self {
            DataType::Numeric { .. } => "type-numeric",
            DataType::Comma { .. } => "type-comma",
            DataType::Dot { .. } => "type-dot",
            DataType::Scientific { .. } => "type-scientific",
            DataType::Date { .. } => "type-date",
            DataType::Dollar { .. } => "type-dollar",
            DataType::CustomCurrency { .. } => "type-custom-currency",
            DataType::String { .. } => "type-string",
            DataType::RestrictedNumeric { .. } => "type-restricted-numeric",
        }
    }

    pub fn is_numeric(&self) -> bool {
        !matches!(self, DataType::String { .. } | DataType::Date { .. })
    }

    pub fn allows_decimals(&self) -> bool {
        matches!(
            self,
            DataType::Numeric { .. }
                | DataType::Comma { .. }
                | DataType::Dot { .. }
                | DataType::Scientific { .. }
                | DataType::Dollar { .. }
                | DataType::CustomCurrency { .. }
        )
    }

    pub fn decimals(&self) -> u8 {
        match self {
            DataType::Numeric { decimals }
            | DataType::Comma { decimals }
            | DataType::Dot { decimals }
            | DataType::Scientific { decimals }
            | DataType::Dollar { decimals }
            | DataType::CustomCurrency { decimals, .. } => *decimals,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Input,
    Target,
    Both,
    None,
    Partition,
    Split,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueLabels {
    pub mapping: BTreeMap<String, String>,
}

impl ValueLabels {
    pub fn new() -> Self {
        Self {
            mapping: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, code: &str, label: &str) {
        self.mapping
            .insert(code.trim().to_string(), label.trim().to_string());
    }

    pub fn get_label(&self, code: &str) -> Option<&str> {
        self.mapping.get(code.trim()).map(|s| s.as_str())
    }

    pub fn get_code(&self, label: &str) -> Option<&str> {
        let trimmed = label.trim();
        self.mapping
            .iter()
            .find(|(_, l)| l.as_str() == trimmed)
            .map(|(c, _)| c.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MissingValues {
    pub discrete: Vec<String>,
    pub range: Option<(f64, f64)>,
}

impl MissingValues {
    pub fn is_missing(&self, raw: &RawValue) -> bool {
        match raw {
            RawValue::Null => true,
            RawValue::Number(n) => {
                if let Some((low, high)) = self.range {
                    if *n >= low && *n <= high {
                        return true;
                    }
                }
                let s = if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                };
                self.discrete.iter().any(|d| d == &s)
            }
            RawValue::Text(s) => self.discrete.iter().any(|d| d.eq_ignore_ascii_case(s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableMetadata {
    pub name: String,
    pub var_type: DataType,
    pub width: usize,
    pub decimals: u8,
    pub label: String,
    pub values: ValueLabels,
    pub missing: MissingValues,
    pub columns: usize,
    pub align: Alignment,
    pub measure: Measure,
    pub role: Role,
}

impl VariableMetadata {
    pub fn new(name: &str, var_type: DataType, measure: Measure) -> Result<Self, CoreError> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(CoreError::InvalidVariableName(name.to_string()));
        }

        let decimals = var_type.decimals();

        let align = match var_type {
            DataType::String { .. } => Alignment::Left,
            DataType::Date { .. } => Alignment::Center,
            _ => Alignment::Right,
        };

        Ok(Self {
            name: trimmed_name.to_string(),
            var_type,
            width: 8,
            decimals,
            label: String::new(),
            values: ValueLabels::new(),
            missing: MissingValues::default(),
            columns: 8,
            align,
            measure,
            role: Role::Input,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VariableDictionary {
    pub variables: Vec<VariableMetadata>,
}

impl VariableDictionary {
    pub fn new() -> Self {
        Self { variables: vec![] }
    }

    pub fn count(&self) -> usize {
        self.variables.len()
    }

    pub fn get(&self, index: usize) -> Option<&VariableMetadata> {
        self.variables.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut VariableMetadata> {
        self.variables.get_mut(index)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&VariableMetadata> {
        let trimmed = name.trim();
        self.variables
            .iter()
            .find(|v| v.name.eq_ignore_ascii_case(trimmed))
    }

    pub fn find_index(&self, name: &str) -> Option<usize> {
        let trimmed = name.trim();
        self.variables
            .iter()
            .position(|v| v.name.eq_ignore_ascii_case(trimmed))
    }

    pub fn add(&mut self, var: VariableMetadata) -> Result<usize, CoreError> {
        if self.find_by_name(&var.name).is_some() {
            return Err(CoreError::DuplicateVariableName(var.name));
        }
        let idx = self.variables.len();
        self.variables.push(var);
        Ok(idx)
    }

    pub fn rename(&mut self, index: usize, new_name: &str) -> Result<(), CoreError> {
        let total = self.variables.len();
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return Err(CoreError::InvalidVariableName(new_name.to_string()));
        }
        if let Some(existing_idx) = self.find_index(trimmed) {
            if existing_idx != index {
                return Err(CoreError::DuplicateVariableName(trimmed.to_string()));
            }
        }
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.name = trimmed.to_string();
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<VariableMetadata, CoreError> {
        let total = self.variables.len();
        if index >= total {
            return Err(CoreError::IndexOutOfBounds { index, total });
        }
        Ok(self.variables.remove(index))
    }

    pub fn set_type(
        &mut self,
        index: usize,
        var_type: DataType,
        width: usize,
        decimals: u8,
    ) -> Result<(), CoreError> {
        let total = self.variables.len();
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.var_type = var_type;
        var.width = width;
        var.decimals = decimals;
        Ok(())
    }

    pub fn set_decimals(&mut self, index: usize, decimals: u8) -> Result<(), CoreError> {
        let total = self.variables.len();
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.decimals = decimals;
        match &mut var.var_type {
            DataType::Numeric { decimals: d }
            | DataType::Comma { decimals: d }
            | DataType::Dot { decimals: d }
            | DataType::Scientific { decimals: d }
            | DataType::Dollar { decimals: d }
            | DataType::CustomCurrency { decimals: d, .. } => {
                *d = decimals;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn set_width(&mut self, index: usize, width: usize) -> Result<(), CoreError> {
        let total = self.variables.len();
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.width = width;
        if let DataType::String { max_len } = &mut var.var_type {
            *max_len = width;
        } else if let DataType::RestrictedNumeric { width: w } = &mut var.var_type {
            *w = width;
        }
        Ok(())
    }

    pub fn set_align(&mut self, index: usize, align: Alignment) -> Result<(), CoreError> {
        let total = self.variables.len();
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.align = align;
        Ok(())
    }

    pub fn set_role(&mut self, index: usize, role: Role) -> Result<(), CoreError> {
        let total = self.variables.len();
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.role = role;
        Ok(())
    }

    pub fn set_label(&mut self, index: usize, label: &str) -> Result<(), CoreError> {
        let total = self.variables.len();
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.label = label.trim().to_string();
        Ok(())
    }

    pub fn set_measure(&mut self, index: usize, measure: Measure) -> Result<(), CoreError> {
        let total = self.variables.len();
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.measure = measure;
        Ok(())
    }

    pub fn set_value_labels(&mut self, index: usize, values: ValueLabels) -> Result<(), CoreError> {
        let total = self.variables.len();
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.values = values;
        Ok(())
    }

    pub fn set_missing_values(
        &mut self,
        index: usize,
        missing: MissingValues,
    ) -> Result<(), CoreError> {
        let total = self.variables.len();
        let var = self
            .variables
            .get_mut(index)
            .ok_or(CoreError::IndexOutOfBounds { index, total })?;
        var.missing = missing;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_metadata_creation() {
        let var = VariableMetadata::new("age", DataType::Numeric { decimals: 0 }, Measure::Scale)
            .unwrap();
        assert_eq!(var.name, "age");
        assert_eq!(var.measure, Measure::Scale);
        assert_eq!(var.align, Alignment::Right);

        // Invalid empty name
        assert!(matches!(
            VariableMetadata::new("", DataType::Numeric { decimals: 0 }, Measure::Scale),
            Err(CoreError::InvalidVariableName(_))
        ));
    }

    #[test]
    fn test_value_labels_lookup() {
        let mut labels = ValueLabels::new();
        labels.insert("1", "Male");
        labels.insert("2", "Female");

        assert_eq!(labels.get_label("1"), Some("Male"));
        assert_eq!(labels.get_label("2"), Some("Female"));
        assert_eq!(labels.get_label("3"), None);

        assert_eq!(labels.get_code("Female"), Some("2"));
    }

    #[test]
    fn test_missing_value_check() {
        let missing = MissingValues {
            discrete: vec!["999".to_string(), "-1".to_string()],
            range: Some((900.0, 999.0)),
        };

        assert!(missing.is_missing(&RawValue::Null));
        assert!(missing.is_missing(&RawValue::Number(999.0)));
        assert!(missing.is_missing(&RawValue::Number(950.0)));
        assert!(!missing.is_missing(&RawValue::Number(25.0)));
    }

    #[test]
    fn test_variable_dictionary_operations() {
        let mut dict = VariableDictionary::new();
        let v1 = VariableMetadata::new("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
        let v2 = VariableMetadata::new(
            "gender",
            DataType::Numeric { decimals: 0 },
            Measure::Nominal,
        )
        .unwrap();

        dict.add(v1).unwrap();
        dict.add(v2).unwrap();

        assert_eq!(dict.count(), 2);
        assert_eq!(dict.find_index("GENDER"), Some(1));

        dict.set_label(0, "Monthly Salary in USD").unwrap();
        assert_eq!(dict.get(0).unwrap().label, "Monthly Salary in USD");

        dict.set_measure(1, Measure::Ordinal).unwrap();
        assert_eq!(dict.get(1).unwrap().measure, Measure::Ordinal);

        // Test duplicate name
        let dup =
            VariableMetadata::new("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
                .unwrap();
        assert!(matches!(
            dict.add(dup),
            Err(CoreError::DuplicateVariableName(_))
        ));
    }

    #[test]
    fn test_metadata_serialization_roundtrip() {
        let mut dict = VariableDictionary::new();
        let mut v = VariableMetadata::new(
            "gender",
            DataType::Numeric { decimals: 0 },
            Measure::Nominal,
        )
        .unwrap();
        v.values.insert("1", "Male");
        v.values.insert("2", "Female");
        dict.add(v).unwrap();

        let json = serde_json::to_string(&dict).unwrap();
        let deserialized: VariableDictionary = serde_json::from_str(&json).unwrap();

        assert_eq!(dict, deserialized);
        assert_eq!(
            deserialized.get(0).unwrap().values.get_label("1"),
            Some("Male")
        );
    }

    #[test]
    fn test_set_value_labels_and_missing_values() {
        let mut dict = VariableDictionary::new();
        let v = VariableMetadata::new(
            "status",
            DataType::Numeric { decimals: 0 },
            Measure::Nominal,
        )
        .unwrap();
        dict.add(v).unwrap();

        let mut labels = ValueLabels::new();
        labels.insert("1", "Active");
        labels.insert("2", "Inactive");
        dict.set_value_labels(0, labels.clone()).unwrap();
        assert_eq!(dict.get(0).unwrap().values.get_label("1"), Some("Active"));

        let missing = MissingValues {
            discrete: vec!["99".to_string()],
            range: Some((900.0, 999.0)),
        };
        dict.set_missing_values(0, missing.clone()).unwrap();
        assert_eq!(dict.get(0).unwrap().missing.discrete, vec!["99"]);
        assert_eq!(dict.get(0).unwrap().missing.range, Some((900.0, 999.0)));
    }
}
