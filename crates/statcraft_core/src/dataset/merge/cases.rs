use crate::dataset::DataSet;
use crate::variable::{
    sanitize_variable_name, CoreError, DataType, Measure, VariableDictionary, VariableMetadata,
};
use statcraft_io::{RawColumn, RawDataType, RawTable, RawValue};
use std::collections::{HashMap, HashSet};

use super::types::AddCasesConfig;

/// Appends cases from a secondary dataset into the active dataset (`ADD FILES`).
/// Returns the previous `(RawTable, VariableDictionary)` to support Undo.
pub fn add_cases(
    active_ds: &mut DataSet,
    second_table: &RawTable,
    second_dict: &VariableDictionary,
    config: &AddCasesConfig,
) -> Result<(RawTable, VariableDictionary), CoreError> {
    let old_table = active_ds.table.clone();
    let old_dict = active_ds.dictionary.clone();

    let n_active = active_ds.table.row_count;
    let n_second = second_table.row_count;
    let n_total = n_active.saturating_add(n_second);

    // Build map of secondary column names (handling renames)
    let second_name_map: HashMap<String, usize> = second_table
        .columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let final_name = config
                .renames_second
                .get(&col.name)
                .cloned()
                .unwrap_or_else(|| col.name.clone());
            (final_name.to_lowercase(), idx)
        })
        .collect();

    // Determine target variable names
    let mut target_names: Vec<String> = Vec::new();
    let mut seen_names: HashSet<String> = HashSet::new();

    if !config.included_variables.is_empty() {
        for v in &config.included_variables {
            let lower = v.trim().to_lowercase();
            if !lower.is_empty() && seen_names.insert(lower) {
                target_names.push(v.trim().to_string());
            }
        }
    } else {
        // Active variables first
        for i in 0..active_ds.dictionary.count() {
            if let Some(meta) = active_ds.dictionary.get(i) {
                if seen_names.insert(meta.name.to_lowercase()) {
                    target_names.push(meta.name.clone());
                }
            }
        }
        // Then remaining secondary variables
        for (idx, col) in second_table.columns.iter().enumerate() {
            let renamed = config.renames_second.get(&col.name).unwrap_or(&col.name);
            let lower = renamed.to_lowercase();
            if seen_names.insert(lower) {
                if let Some(meta) = second_dict.get(idx) {
                    target_names.push(renamed.clone());
                    let _ = meta;
                } else {
                    target_names.push(renamed.clone());
                }
            }
        }
    }

    // Indicator variable (/IN=source01)
    let in_var_name = config
        .in_variable
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(ref in_name) = in_var_name {
        if seen_names.insert(in_name.to_lowercase()) {
            target_names.push(in_name.clone());
        }
    }

    // Build new VariableDictionary and Columns
    let mut new_dict = VariableDictionary::new();
    let mut new_columns: Vec<RawColumn> = Vec::with_capacity(target_names.len());

    for (var_idx, var_name) in target_names.iter().enumerate() {
        if in_var_name.as_ref() == Some(var_name) {
            let safe_name = sanitize_variable_name(var_name, var_idx + 1);
            let meta = VariableMetadata::new(
                &safe_name,
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )?;
            new_dict.add(meta)?;

            let mut values = Vec::with_capacity(n_total);
            for _ in 0..n_active {
                values.push(RawValue::Number(0.0));
            }
            for _ in 0..n_second {
                values.push(RawValue::Number(1.0));
            }
            new_columns.push(RawColumn {
                name: safe_name,
                data_type: RawDataType::Numeric,
                values,
            });
            continue;
        }

        let active_col_idx = active_ds.dictionary.find_index(var_name);
        let second_col_idx = second_name_map.get(&var_name.to_lowercase()).copied();

        // Determine metadata and data type
        let (meta, raw_type) = match (active_col_idx, second_col_idx) {
            (Some(a_idx), Some(s_idx)) => {
                let a_meta = active_ds
                    .dictionary
                    .get(a_idx)
                    .ok_or_else(|| CoreError::VariableNotFound(var_name.clone()))?;
                let s_meta = second_dict.get(s_idx);

                let combined_type = match (&a_meta.var_type, s_meta.map(|m| &m.var_type)) {
                    (DataType::String { max_len: l1 }, Some(DataType::String { max_len: l2 })) => {
                        DataType::String {
                            max_len: (*l1).max(*l2),
                        }
                    }
                    (t, _) => t.clone(),
                };

                let mut m = a_meta.clone();
                m.var_type = combined_type;
                let raw_type = if matches!(m.var_type, DataType::String { .. }) {
                    RawDataType::Text
                } else {
                    RawDataType::Numeric
                };
                (m, raw_type)
            }
            (Some(a_idx), None) => {
                let a_meta = active_ds
                    .dictionary
                    .get(a_idx)
                    .ok_or_else(|| CoreError::VariableNotFound(var_name.clone()))?;
                let raw_type = if matches!(a_meta.var_type, DataType::String { .. }) {
                    RawDataType::Text
                } else {
                    RawDataType::Numeric
                };
                (a_meta.clone(), raw_type)
            }
            (None, Some(s_idx)) => {
                let s_meta = second_dict.get(s_idx).cloned().unwrap_or_else(|| {
                    VariableMetadata::new(
                        var_name,
                        DataType::Numeric { decimals: 2 },
                        Measure::Scale,
                    )
                    .unwrap_or_else(|_| {
                        let safe = sanitize_variable_name(var_name, var_idx + 1);
                        VariableMetadata::new(
                            &safe,
                            DataType::Numeric { decimals: 2 },
                            Measure::Scale,
                        )
                        .unwrap_or_else(|_| VariableMetadata {
                            name: format!("VAR{:05}", var_idx + 1),
                            var_type: DataType::Numeric { decimals: 2 },
                            width: 8,
                            decimals: 2,
                            label: String::new(),
                            values: crate::variable::ValueLabels::new(),
                            missing: crate::variable::MissingValues::default(),
                            columns: 8,
                            align: crate::variable::Alignment::Right,
                            measure: Measure::Scale,
                            role: crate::variable::Role::Input,
                        })
                    })
                });
                let mut m = s_meta;
                m.name = var_name.clone();
                let raw_type = if matches!(m.var_type, DataType::String { .. }) {
                    RawDataType::Text
                } else {
                    RawDataType::Numeric
                };
                (m, raw_type)
            }
            (None, None) => {
                let meta = VariableMetadata::new(
                    var_name,
                    DataType::Numeric { decimals: 2 },
                    Measure::Scale,
                )?;
                (meta, RawDataType::Numeric)
            }
        };

        new_dict.add(meta)?;

        // Populate values
        let mut values = Vec::with_capacity(n_total);

        // Active dataset rows
        if let Some(a_idx) = active_col_idx {
            if let Some(col) = active_ds.table.columns.get(a_idx) {
                values.extend_from_slice(&col.values);
            } else {
                values.resize(n_active, RawValue::Null);
            }
        } else {
            values.resize(n_active, RawValue::Null);
        }

        // Secondary dataset rows
        if let Some(s_idx) = second_col_idx {
            if let Some(col) = second_table.columns.get(s_idx) {
                values.extend_from_slice(&col.values);
            } else {
                values.resize(n_total, RawValue::Null);
            }
        } else {
            values.resize(n_total, RawValue::Null);
        }

        new_columns.push(RawColumn {
            name: var_name.clone(),
            data_type: raw_type,
            values,
        });
    }

    let mut new_table = RawTable {
        columns: new_columns,
        row_count: n_total,
    };

    // Sort / interleave by keys if requested
    if let Some(ref keys) = config.interleave_by {
        if !keys.is_empty() && n_total > 1 {
            interleave_rows(&mut new_table, keys);
        }
    }

    active_ds.table = new_table;
    active_ds.dictionary = new_dict;
    active_ds.delta.undo_stack.clear();
    active_ds.delta.redo_stack.clear();

    Ok((old_table, old_dict))
}

fn interleave_rows(table: &mut RawTable, keys: &[String]) {
    let key_indices: Vec<usize> = keys
        .iter()
        .filter_map(|k| {
            table
                .columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(k))
        })
        .collect();

    if key_indices.is_empty() {
        return;
    }

    let mut row_indices: Vec<usize> = (0..table.row_count).collect();
    row_indices.sort_by(|&r1, &r2| {
        for &k_idx in &key_indices {
            let col = &table.columns[k_idx];
            let v1 = col.values.get(r1).unwrap_or(&RawValue::Null);
            let v2 = col.values.get(r2).unwrap_or(&RawValue::Null);
            let cmp = compare_raw_values(v1, v2);
            if cmp != std::cmp::Ordering::Equal {
                return cmp;
            }
        }
        r1.cmp(&r2)
    });

    for col in &mut table.columns {
        let old_values = col.values.clone();
        for (new_row, &orig_row) in row_indices.iter().enumerate() {
            if let Some(val) = old_values.get(orig_row) {
                if new_row < col.values.len() {
                    col.values[new_row] = val.clone();
                }
            }
        }
    }
}

pub(crate) fn compare_raw_values(v1: &RawValue, v2: &RawValue) -> std::cmp::Ordering {
    match (v1, v2) {
        (RawValue::Null, RawValue::Null) => std::cmp::Ordering::Equal,
        (RawValue::Null, _) => std::cmp::Ordering::Greater,
        (_, RawValue::Null) => std::cmp::Ordering::Less,
        (RawValue::Number(n1), RawValue::Number(n2)) => {
            n1.partial_cmp(n2).unwrap_or(std::cmp::Ordering::Equal)
        }
        (RawValue::Text(t1), RawValue::Text(t2)) => t1.cmp(t2),
        (RawValue::Number(_), RawValue::Text(_)) => std::cmp::Ordering::Less,
        (RawValue::Text(_), RawValue::Number(_)) => std::cmp::Ordering::Greater,
    }
}
