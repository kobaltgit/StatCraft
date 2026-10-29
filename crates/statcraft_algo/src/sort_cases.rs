use crate::AlgoError;
use serde::{Deserialize, Serialize};
use statcraft_core::{DataSet, RawValue};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortKeySpec {
    pub column_name: String,
    pub direction: SortDirection,
}

impl SortKeySpec {
    pub fn new(column_name: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column_name: column_name.into(),
            direction,
        }
    }

    pub fn ascending(column_name: impl Into<String>) -> Self {
        Self::new(column_name, SortDirection::Ascending)
    }

    pub fn descending(column_name: impl Into<String>) -> Self {
        Self::new(column_name, SortDirection::Descending)
    }
}

fn is_missing_value(val: Option<&RawValue>) -> bool {
    match val {
        None | Some(RawValue::Null) => true,
        Some(RawValue::Number(n)) => n.is_nan(),
        Some(RawValue::Text(s)) => s.is_empty(),
    }
}

pub fn compare_raw_values(a: Option<&RawValue>, b: Option<&RawValue>) -> Ordering {
    let a_miss = is_missing_value(a);
    let b_miss = is_missing_value(b);

    if a_miss && b_miss {
        return Ordering::Equal;
    }
    if a_miss {
        return Ordering::Less;
    }
    if b_miss {
        return Ordering::Greater;
    }

    match (a, b) {
        (Some(RawValue::Number(n1)), Some(RawValue::Number(n2))) => match n1.partial_cmp(n2) {
            Some(ord) => ord,
            None => Ordering::Equal,
        },
        (Some(RawValue::Text(s1)), Some(RawValue::Text(s2))) => s1.cmp(s2),
        (Some(RawValue::Number(n1)), Some(RawValue::Text(s2))) => {
            if let Ok(n2) = s2.parse::<f64>() {
                match n1.partial_cmp(&n2) {
                    Some(ord) => ord,
                    None => Ordering::Less,
                }
            } else {
                Ordering::Less
            }
        }
        (Some(RawValue::Text(s1)), Some(RawValue::Number(n2))) => {
            if let Ok(n1) = s1.parse::<f64>() {
                match n1.partial_cmp(n2) {
                    Some(ord) => ord,
                    None => Ordering::Greater,
                }
            } else {
                Ordering::Greater
            }
        }
        _ => Ordering::Equal,
    }
}

pub fn compute_sort_permutation(
    dataset: &DataSet,
    keys: &[SortKeySpec],
) -> Result<Vec<usize>, AlgoError> {
    if keys.is_empty() {
        return Err(AlgoError::InvalidInput(
            "At least one sort key must be specified".to_string(),
        ));
    }

    let mut key_columns = Vec::with_capacity(keys.len());
    for key in keys {
        let col_idx = dataset
            .dictionary
            .find_index(&key.column_name)
            .ok_or_else(|| {
                AlgoError::InvalidInput(format!("Variable '{}' not found", key.column_name))
            })?;
        key_columns.push((col_idx, key.direction));
    }

    let row_count = dataset.row_count();
    let mut indices: Vec<usize> = (0..row_count).collect();

    indices.sort_by(|&row_a, &row_b| {
        for &(col_idx, direction) in &key_columns {
            let val_a = dataset.get_cell(row_a, col_idx);
            let val_b = dataset.get_cell(row_b, col_idx);
            let ord = compare_raw_values(val_a, val_b);
            let directed_ord = match direction {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            };
            if directed_ord != Ordering::Equal {
                return directed_ord;
            }
        }
        Ordering::Equal
    });

    Ok(indices)
}

pub fn invert_permutation(perm: &[usize]) -> Vec<usize> {
    let mut inv = vec![0; perm.len()];
    for (new_idx, &old_idx) in perm.iter().enumerate() {
        if old_idx < inv.len() {
            inv[old_idx] = new_idx;
        }
    }
    inv
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::{DataType, Measure, RawDataType};

    fn make_test_dataset() -> DataSet {
        let mut ds = DataSet::blank(5, 3).unwrap();
        // Rename variables: col 0 = "dept", col 1 = "salary", col 2 = "name"
        ds.dictionary.get_mut(0).unwrap().name = "dept".to_string();
        ds.dictionary.get_mut(1).unwrap().name = "salary".to_string();
        ds.dictionary.get_mut(2).unwrap().name = "name".to_string();
        ds.dictionary.get_mut(2).unwrap().var_type = DataType::String { max_len: 30 };
        ds.dictionary.get_mut(2).unwrap().measure = Measure::Nominal;
        ds.table.columns[0].name = "dept".to_string();
        ds.table.columns[1].name = "salary".to_string();
        ds.table.columns[2].name = "name".to_string();
        ds.table.columns[2].data_type = RawDataType::Text;

        // Row 0: dept=2, salary=3000, name="Alice"
        ds.set_cell(0, 0, RawValue::Number(2.0)).unwrap();
        ds.set_cell(0, 1, RawValue::Number(3000.0)).unwrap();
        ds.set_cell(0, 2, RawValue::Text("Alice".to_string()))
            .unwrap();

        // Row 1: dept=1, salary=5000, name="Bob"
        ds.set_cell(1, 0, RawValue::Number(1.0)).unwrap();
        ds.set_cell(1, 1, RawValue::Number(5000.0)).unwrap();
        ds.set_cell(1, 2, RawValue::Text("Bob".to_string()))
            .unwrap();

        // Row 2: dept=2, salary=4000, name="Charlie"
        ds.set_cell(2, 0, RawValue::Number(2.0)).unwrap();
        ds.set_cell(2, 1, RawValue::Number(4000.0)).unwrap();
        ds.set_cell(2, 2, RawValue::Text("Charlie".to_string()))
            .unwrap();

        // Row 3: dept=1, salary=2000, name="David"
        ds.set_cell(3, 0, RawValue::Number(1.0)).unwrap();
        ds.set_cell(3, 1, RawValue::Number(2000.0)).unwrap();
        ds.set_cell(3, 2, RawValue::Text("David".to_string()))
            .unwrap();

        // Row 4: dept=Null, salary=Null, name=""
        ds.set_cell(4, 0, RawValue::Null).unwrap();
        ds.set_cell(4, 1, RawValue::Null).unwrap();
        ds.set_cell(4, 2, RawValue::Null).unwrap();

        ds
    }

    #[test]
    fn test_sort_single_column_ascending() {
        let ds = make_test_dataset();
        let keys = vec![SortKeySpec::ascending("salary")];
        let perm = compute_sort_permutation(&ds, &keys).unwrap();

        // Null is treated as lowest -> Row 4 first, then 2000 (Row 3), 3000 (Row 0), 4000 (Row 2), 5000 (Row 1)
        assert_eq!(perm, vec![4, 3, 0, 2, 1]);
    }

    #[test]
    fn test_sort_single_column_descending() {
        let ds = make_test_dataset();
        let keys = vec![SortKeySpec::descending("salary")];
        let perm = compute_sort_permutation(&ds, &keys).unwrap();

        // Highest first: 5000 (Row 1), 4000 (Row 2), 3000 (Row 0), 2000 (Row 3), Null (Row 4)
        assert_eq!(perm, vec![1, 2, 0, 3, 4]);
    }

    #[test]
    fn test_sort_multi_column() {
        let ds = make_test_dataset();
        // SORT CASES BY dept (A) salary (D)
        let keys = vec![
            SortKeySpec::ascending("dept"),
            SortKeySpec::descending("salary"),
        ];
        let perm = compute_sort_permutation(&ds, &keys).unwrap();

        // dept Null (Row 4) first (since dept is ascending)
        // dept 1: salary 5000 (Row 1) then salary 2000 (Row 3)
        // dept 2: salary 4000 (Row 2) then salary 3000 (Row 0)
        assert_eq!(perm, vec![4, 1, 3, 2, 0]);
    }

    #[test]
    fn test_sort_text_column() {
        let ds = make_test_dataset();
        let keys = vec![SortKeySpec::ascending("name")];
        let perm = compute_sort_permutation(&ds, &keys).unwrap();

        // Null/empty (Row 4), Alice (0), Bob (1), Charlie (2), David (3)
        assert_eq!(perm, vec![4, 0, 1, 2, 3]);
    }

    #[test]
    fn test_invert_permutation() {
        let perm = vec![4, 1, 3, 2, 0];
        let inv = invert_permutation(&perm);
        assert_eq!(inv, vec![4, 1, 3, 2, 0]);

        let perm2 = vec![2, 0, 1];
        let inv2 = invert_permutation(&perm2);
        assert_eq!(inv2, vec![1, 2, 0]);

        // Reordering by perm2 and then inv2 restores original
        let original = vec!["A", "B", "C"];
        let reordered: Vec<_> = perm2.iter().map(|&i| original[i]).collect();
        assert_eq!(reordered, vec!["C", "A", "B"]);
        let restored: Vec<_> = inv2.iter().map(|&i| reordered[i]).collect();
        assert_eq!(restored, original);
    }

    #[test]
    fn test_sort_empty_keys_err() {
        let ds = make_test_dataset();
        let res = compute_sort_permutation(&ds, &[]);
        assert!(res.is_err());
    }

    #[test]
    fn test_sort_missing_var_err() {
        let ds = make_test_dataset();
        let res = compute_sort_permutation(&ds, &[SortKeySpec::ascending("non_existent")]);
        assert!(res.is_err());
    }
}
