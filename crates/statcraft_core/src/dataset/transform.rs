use crate::variable::{
    sanitize_variable_name, CoreError, DataType, Measure, VariableDictionary, VariableMetadata,
};
use crate::DataSet;
use statcraft_io::{RawColumn, RawDataType, RawTable, RawValue};
use std::collections::HashSet;

pub fn reorder_variables(ds: &mut DataSet, permutation: &[usize]) -> Result<(), CoreError> {
    let total = ds.dictionary.count();
    if permutation.len() != total {
        return Err(CoreError::IndexOutOfBounds {
            index: permutation.len(),
            total,
        });
    }
    for &idx in permutation {
        if idx >= total {
            return Err(CoreError::IndexOutOfBounds { index: idx, total });
        }
    }
    ds.table
        .reorder_columns(permutation)
        .map_err(|_| CoreError::IndexOutOfBounds {
            index: permutation.len(),
            total,
        })?;
    ds.dictionary.reorder(permutation)?;
    ds.delta.undo_stack.clear();
    ds.delta.redo_stack.clear();
    Ok(())
}

pub fn restore_structure(ds: &mut DataSet, table: RawTable, dictionary: VariableDictionary) {
    ds.table = table;
    ds.dictionary = dictionary;
    ds.delta.undo_stack.clear();
    ds.delta.redo_stack.clear();
    ds.active_filter_var = None;
    ds.split_file = None;
    ds.weight_var = None;
}

pub fn transpose(
    ds: &mut DataSet,
    vars: Option<&[String]>,
    name_var: Option<&str>,
) -> Result<(RawTable, VariableDictionary), CoreError> {
    let name_var_idx = match name_var {
        Some(n) if !n.trim().is_empty() => {
            let idx = ds
                .dictionary
                .find_index(n)
                .ok_or_else(|| CoreError::VariableNotFound(n.to_string()))?;
            Some(idx)
        }
        _ => None,
    };

    let target_indices: Vec<usize> = match vars {
        Some(var_list) if !var_list.is_empty() => {
            let mut indices = Vec::with_capacity(var_list.len());
            for v in var_list {
                let idx = ds
                    .dictionary
                    .find_index(v)
                    .ok_or_else(|| CoreError::VariableNotFound(v.clone()))?;
                if Some(idx) != name_var_idx && !indices.contains(&idx) {
                    indices.push(idx);
                }
            }
            indices
        }
        _ => (0..ds.dictionary.count())
            .filter(|&idx| Some(idx) != name_var_idx)
            .collect(),
    };

    if target_indices.is_empty() {
        return Err(CoreError::VariableNotFound(
            "No variables selected for transposition".to_string(),
        ));
    }

    let num_cases = ds.row_count();
    let num_vars = target_indices.len();

    let is_all_numeric = target_indices.iter().all(|&idx| {
        ds.dictionary
            .get(idx)
            .map(|m| m.var_type.is_numeric())
            .unwrap_or(false)
    });

    let mut col_names = Vec::with_capacity(num_cases);
    let mut used_names = HashSet::new();
    used_names.insert("CASE_LBL".to_string().to_ascii_uppercase());

    for row in 0..num_cases {
        let base_name = if let Some(n_idx) = name_var_idx {
            let val_str = ds
                .table
                .columns
                .get(n_idx)
                .and_then(|c| c.values.get(row))
                .map(|v| match v {
                    RawValue::Text(t) => t.trim().to_string(),
                    RawValue::Number(n) => {
                        if n.fract() == 0.0 {
                            format!("{:.0}", n)
                        } else {
                            n.to_string()
                        }
                    }
                    RawValue::Null => format!("VAR{:03}", row + 1),
                })
                .unwrap_or_else(|| format!("VAR{:03}", row + 1));
            sanitize_variable_name(&val_str, row)
        } else {
            format!("VAR{:03}", row + 1)
        };

        let mut final_name = base_name.clone();
        let mut counter = 1;
        while used_names.contains(&final_name.to_ascii_uppercase()) {
            final_name = format!("{}_{}", base_name, counter);
            counter += 1;
        }
        used_names.insert(final_name.to_ascii_uppercase());
        col_names.push(final_name);
    }

    let case_lbl_values: Vec<RawValue> = target_indices
        .iter()
        .map(|&idx| {
            let var_name = ds
                .dictionary
                .get(idx)
                .map(|m| m.name.clone())
                .unwrap_or_default();
            RawValue::Text(var_name)
        })
        .collect();

    let mut new_columns = Vec::with_capacity(num_cases + 1);
    new_columns.push(RawColumn {
        name: "CASE_LBL".to_string(),
        data_type: RawDataType::Text,
        values: case_lbl_values,
    });

    let col_raw_type = if is_all_numeric {
        RawDataType::Numeric
    } else {
        RawDataType::Text
    };

    for (row_idx, col_name) in col_names.iter().enumerate() {
        let mut values = Vec::with_capacity(num_vars);
        for &var_idx in &target_indices {
            let val = ds
                .table
                .columns
                .get(var_idx)
                .and_then(|c| c.values.get(row_idx))
                .cloned()
                .unwrap_or(RawValue::Null);

            let converted_val = if is_all_numeric {
                val
            } else {
                match val {
                    RawValue::Text(t) => RawValue::Text(t),
                    RawValue::Number(n) => {
                        if n.fract() == 0.0 {
                            RawValue::Text(format!("{:.0}", n))
                        } else {
                            RawValue::Text(n.to_string())
                        }
                    }
                    RawValue::Null => RawValue::Null,
                }
            };
            values.push(converted_val);
        }

        new_columns.push(RawColumn {
            name: col_name.clone(),
            data_type: col_raw_type,
            values,
        });
    }

    let mut new_dict = VariableDictionary::new();
    let mut case_lbl_meta = VariableMetadata::new(
        "CASE_LBL",
        DataType::String { max_len: 255 },
        Measure::Nominal,
    )?;
    case_lbl_meta.label = "Case Label".to_string();
    new_dict.add(case_lbl_meta)?;

    let (var_type, measure) = if is_all_numeric {
        (DataType::Numeric { decimals: 2 }, Measure::Scale)
    } else {
        (DataType::String { max_len: 255 }, Measure::Nominal)
    };

    for col_name in &col_names {
        new_dict.add(VariableMetadata::new(col_name, var_type.clone(), measure)?)?;
    }

    let new_table = RawTable {
        columns: new_columns,
        row_count: num_vars,
    };

    let old_table = std::mem::replace(&mut ds.table, new_table);
    let old_dict = std::mem::replace(&mut ds.dictionary, new_dict);

    ds.delta.undo_stack.clear();
    ds.delta.redo_stack.clear();
    ds.active_filter_var = None;
    ds.split_file = None;
    ds.weight_var = None;

    Ok((old_table, old_dict))
}
