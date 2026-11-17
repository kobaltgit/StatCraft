use crate::sort_cases::SortDirection;
use serde::{Deserialize, Serialize};
use statcraft_core::VariableDictionary;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableSortAttribute {
    Name,
    Type,
    Label,
    Measure,
    Role,
    Alignment,
    Width,
    Decimals,
}

pub fn compute_variable_sort_permutation(
    dictionary: &VariableDictionary,
    attribute: VariableSortAttribute,
    direction: SortDirection,
) -> Vec<usize> {
    let total = dictionary.count();
    let mut indices: Vec<usize> = (0..total).collect();

    indices.sort_by(|&a, &b| {
        let var_a = match dictionary.get(a) {
            Some(v) => v,
            None => return Ordering::Equal,
        };
        let var_b = match dictionary.get(b) {
            Some(v) => v,
            None => return Ordering::Equal,
        };

        let ord = match attribute {
            VariableSortAttribute::Name => {
                var_a.name.to_lowercase().cmp(&var_b.name.to_lowercase())
            }
            VariableSortAttribute::Type => {
                let type_a = var_a.var_type.as_key();
                let type_b = var_b.var_type.as_key();
                type_a.cmp(type_b)
            }
            VariableSortAttribute::Label => {
                var_a.label.to_lowercase().cmp(&var_b.label.to_lowercase())
            }
            VariableSortAttribute::Measure => {
                let m_a = match var_a.measure {
                    statcraft_core::Measure::Scale => 0,
                    statcraft_core::Measure::Ordinal => 1,
                    statcraft_core::Measure::Nominal => 2,
                };
                let m_b = match var_b.measure {
                    statcraft_core::Measure::Scale => 0,
                    statcraft_core::Measure::Ordinal => 1,
                    statcraft_core::Measure::Nominal => 2,
                };
                m_a.cmp(&m_b)
            }
            VariableSortAttribute::Role => {
                let r_a = match var_a.role {
                    statcraft_core::Role::Input => 0,
                    statcraft_core::Role::Target => 1,
                    statcraft_core::Role::Both => 2,
                    statcraft_core::Role::None => 3,
                    statcraft_core::Role::Partition => 4,
                    statcraft_core::Role::Split => 5,
                };
                let r_b = match var_b.role {
                    statcraft_core::Role::Input => 0,
                    statcraft_core::Role::Target => 1,
                    statcraft_core::Role::Both => 2,
                    statcraft_core::Role::None => 3,
                    statcraft_core::Role::Partition => 4,
                    statcraft_core::Role::Split => 5,
                };
                r_a.cmp(&r_b)
            }
            VariableSortAttribute::Alignment => {
                let al_a = match var_a.align {
                    statcraft_core::Alignment::Left => 0,
                    statcraft_core::Alignment::Right => 1,
                    statcraft_core::Alignment::Center => 2,
                };
                let al_b = match var_b.align {
                    statcraft_core::Alignment::Left => 0,
                    statcraft_core::Alignment::Right => 1,
                    statcraft_core::Alignment::Center => 2,
                };
                al_a.cmp(&al_b)
            }
            VariableSortAttribute::Width => var_a.width.cmp(&var_b.width),
            VariableSortAttribute::Decimals => var_a.decimals.cmp(&var_b.decimals),
        };

        let primary = match direction {
            SortDirection::Ascending => ord,
            SortDirection::Descending => ord.reverse(),
        };

        primary.then_with(|| a.cmp(&b))
    });

    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::{DataType, Measure, VariableMetadata};

    #[test]
    fn test_sort_variables_by_name() {
        let mut dict = VariableDictionary::new();
        dict.add(
            VariableMetadata::new("zeta", DataType::Numeric { decimals: 2 }, Measure::Scale)
                .unwrap(),
        )
        .unwrap();
        dict.add(
            VariableMetadata::new("alpha", DataType::Numeric { decimals: 2 }, Measure::Scale)
                .unwrap(),
        )
        .unwrap();
        dict.add(
            VariableMetadata::new("Beta", DataType::Numeric { decimals: 2 }, Measure::Scale)
                .unwrap(),
        )
        .unwrap();

        let asc = compute_variable_sort_permutation(
            &dict,
            VariableSortAttribute::Name,
            SortDirection::Ascending,
        );
        assert_eq!(asc, vec![1, 2, 0]); // alpha, Beta, zeta

        let desc = compute_variable_sort_permutation(
            &dict,
            VariableSortAttribute::Name,
            SortDirection::Descending,
        );
        assert_eq!(desc, vec![0, 2, 1]); // zeta, Beta, alpha
    }

    #[test]
    fn test_sort_variables_by_decimals() {
        let mut dict = VariableDictionary::new();
        dict.add(
            VariableMetadata::new("v3", DataType::Numeric { decimals: 3 }, Measure::Scale).unwrap(),
        )
        .unwrap();
        dict.add(
            VariableMetadata::new("v1", DataType::Numeric { decimals: 1 }, Measure::Scale).unwrap(),
        )
        .unwrap();
        dict.add(
            VariableMetadata::new("v2", DataType::Numeric { decimals: 2 }, Measure::Scale).unwrap(),
        )
        .unwrap();

        let asc = compute_variable_sort_permutation(
            &dict,
            VariableSortAttribute::Decimals,
            SortDirection::Ascending,
        );
        assert_eq!(asc, vec![1, 2, 0]);
    }
}
