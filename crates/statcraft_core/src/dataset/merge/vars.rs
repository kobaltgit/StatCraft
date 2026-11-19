use crate::dataset::DataSet;
use crate::variable::{
    sanitize_variable_name, CoreError, DataType, Measure, VariableDictionary, VariableMetadata,
};
use statcraft_io::{RawColumn, RawDataType, RawTable, RawValue};
use std::collections::{HashMap, HashSet};

use super::types::{AddVarsConfig, MergeMethod};

/// Appends variables from a secondary dataset into the active dataset (`MATCH FILES`).
/// Returns the previous `(RawTable, VariableDictionary)` to support Undo.
pub fn add_variables(
    active_ds: &mut DataSet,
    second_table: &RawTable,
    second_dict: &VariableDictionary,
    config: &AddVarsConfig,
) -> Result<(RawTable, VariableDictionary), CoreError> {
    let old_table = active_ds.table.clone();
    let old_dict = active_ds.dictionary.clone();

    match config.method {
        MergeMethod::Sequential => {
            merge_sequential(active_ds, second_table, second_dict, config)?;
        }
        MergeMethod::OneToOneWithKeys => {
            merge_keyed_one_to_one(active_ds, second_table, second_dict, config)?;
        }
        MergeMethod::OneToManyTable => {
            merge_keyed_table_lookup(active_ds, second_table, second_dict, config)?;
        }
    }

    active_ds.delta.undo_stack.clear();
    active_ds.delta.redo_stack.clear();

    Ok((old_table, old_dict))
}

fn merge_sequential(
    active_ds: &mut DataSet,
    second_table: &RawTable,
    second_dict: &VariableDictionary,
    config: &AddVarsConfig,
) -> Result<(), CoreError> {
    let n_active = active_ds.table.row_count;
    let n_second = second_table.row_count;
    let n_total = n_active.max(n_second);

    // Prepare active columns (pad if second is longer)
    let mut new_columns = active_ds.table.columns.clone();
    for col in &mut new_columns {
        if col.values.len() < n_total {
            col.values.resize(n_total, RawValue::Null);
        }
    }

    let mut new_dict = active_ds.dictionary.clone();
    let drop_set: HashSet<String> = config
        .drop_second
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    // Process secondary columns
    for (s_idx, col) in second_table.columns.iter().enumerate() {
        if drop_set.contains(&col.name.to_lowercase()) {
            continue;
        }

        let target_name = config
            .renames_second
            .get(&col.name)
            .cloned()
            .unwrap_or_else(|| col.name.clone());

        // Prevent duplicate variable name in dictionary
        let safe_name = if new_dict.find_by_name(&target_name).is_some() {
            resolve_unique_name(&new_dict, &target_name)
        } else {
            target_name
        };

        let mut values = col.values.clone();
        if values.len() < n_total {
            values.resize(n_total, RawValue::Null);
        }

        let s_meta = second_dict.get(s_idx).cloned().unwrap_or_else(|| {
            VariableMetadata::new(
                &safe_name,
                DataType::Numeric { decimals: 2 },
                Measure::Scale,
            )
            .unwrap_or_else(|_| VariableMetadata {
                name: safe_name.clone(),
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
        });

        let mut meta = s_meta;
        meta.name = safe_name.clone();
        new_dict.add(meta)?;

        new_columns.push(RawColumn {
            name: safe_name,
            data_type: col.data_type,
            values,
        });
    }

    add_indicator_columns(
        &mut new_columns,
        &mut new_dict,
        config,
        n_total,
        |row| row < n_active,
        |row| row < n_second,
    )?;

    active_ds.table = RawTable {
        columns: new_columns,
        row_count: n_total,
    };
    active_ds.dictionary = new_dict;

    Ok(())
}

fn merge_keyed_one_to_one(
    active_ds: &mut DataSet,
    second_table: &RawTable,
    second_dict: &VariableDictionary,
    config: &AddVarsConfig,
) -> Result<(), CoreError> {
    if config.keys.is_empty() {
        return Err(CoreError::MergeError(
            "Keyed merge requires at least one key variable".to_string(),
        ));
    }

    let active_key_indices: Vec<usize> = find_indices(&active_ds.table, &config.keys)?;
    let second_key_indices: Vec<usize> = find_indices(second_table, &config.keys)?;

    // Map secondary rows by key vector (FIFO queue for duplicates)
    let mut second_map: HashMap<Vec<KeyValWrapper>, Vec<usize>> = HashMap::new();
    for row in 0..second_table.row_count {
        let key_vals = extract_keys(second_table, row, &second_key_indices);
        second_map.entry(key_vals).or_default().push(row);
    }

    // Determine row pairings: (Option<active_row>, Option<second_row>)
    let mut pairings: Vec<(Option<usize>, Option<usize>)> = Vec::new();
    let mut second_matched: HashSet<usize> = HashSet::new();

    for a_row in 0..active_ds.table.row_count {
        let key_vals = extract_keys(&active_ds.table, a_row, &active_key_indices);
        if let Some(list) = second_map.get_mut(&key_vals) {
            if let Some(s_row) = list
                .iter()
                .find(|&&idx| !second_matched.contains(&idx))
                .copied()
            {
                second_matched.insert(s_row);
                pairings.push((Some(a_row), Some(s_row)));
                continue;
            }
        }
        pairings.push((Some(a_row), None));
    }

    // Add unmatched secondary rows (outer join)
    for s_row in 0..second_table.row_count {
        if !second_matched.contains(&s_row) {
            pairings.push((None, Some(s_row)));
        }
    }

    build_matched_dataset(active_ds, second_table, second_dict, config, &pairings)
}

fn merge_keyed_table_lookup(
    active_ds: &mut DataSet,
    second_table: &RawTable,
    second_dict: &VariableDictionary,
    config: &AddVarsConfig,
) -> Result<(), CoreError> {
    if config.keys.is_empty() {
        return Err(CoreError::MergeError(
            "Table lookup merge requires at least one key variable".to_string(),
        ));
    }

    let active_key_indices: Vec<usize> = find_indices(&active_ds.table, &config.keys)?;
    let second_key_indices: Vec<usize> = find_indices(second_table, &config.keys)?;

    // In SPSS /TABLE, table keys must be unique. We map unique secondary row.
    let mut table_map: HashMap<Vec<KeyValWrapper>, usize> = HashMap::new();
    for row in 0..second_table.row_count {
        let key_vals = extract_keys(second_table, row, &second_key_indices);
        table_map.entry(key_vals).or_insert(row);
    }

    let pairings: Vec<(Option<usize>, Option<usize>)> = (0..active_ds.table.row_count)
        .map(|a_row| {
            let key_vals = extract_keys(&active_ds.table, a_row, &active_key_indices);
            let s_row = table_map.get(&key_vals).copied();
            (Some(a_row), s_row)
        })
        .collect();

    build_matched_dataset(active_ds, second_table, second_dict, config, &pairings)
}

fn build_matched_dataset(
    active_ds: &mut DataSet,
    second_table: &RawTable,
    second_dict: &VariableDictionary,
    config: &AddVarsConfig,
    pairings: &[(Option<usize>, Option<usize>)],
) -> Result<(), CoreError> {
    let n_total = pairings.len();
    let mut new_columns = Vec::new();
    let mut new_dict = VariableDictionary::new();

    // 1. Copy active columns
    for (a_idx, col) in active_ds.table.columns.iter().enumerate() {
        let meta = active_ds
            .dictionary
            .get(a_idx)
            .ok_or_else(|| CoreError::IndexOutOfBounds {
                index: a_idx,
                total: active_ds.table.col_count(),
            })?;
        new_dict.add(meta.clone())?;

        let mut values = Vec::with_capacity(n_total);
        for &(a_row, s_row) in pairings {
            if let Some(r) = a_row {
                values.push(col.values.get(r).cloned().unwrap_or(RawValue::Null));
            } else if let Some(sr) = s_row {
                // If it's a key variable, take key value from secondary
                if config
                    .keys
                    .iter()
                    .any(|k| k.eq_ignore_ascii_case(&col.name))
                {
                    if let Some(s_pos) = second_table
                        .columns
                        .iter()
                        .position(|c| c.name.eq_ignore_ascii_case(&col.name))
                    {
                        values.push(
                            second_table.columns[s_pos]
                                .values
                                .get(sr)
                                .cloned()
                                .unwrap_or(RawValue::Null),
                        );
                        continue;
                    }
                }
                values.push(RawValue::Null);
            } else {
                values.push(RawValue::Null);
            }
        }

        new_columns.push(RawColumn {
            name: col.name.clone(),
            data_type: col.data_type,
            values,
        });
    }

    // 2. Append non-key secondary columns
    let drop_set: HashSet<String> = config
        .drop_second
        .iter()
        .map(|s| s.to_lowercase())
        .collect();
    let key_set: HashSet<String> = config.keys.iter().map(|s| s.to_lowercase()).collect();

    for (s_idx, col) in second_table.columns.iter().enumerate() {
        if key_set.contains(&col.name.to_lowercase()) || drop_set.contains(&col.name.to_lowercase())
        {
            continue;
        }

        let target_name = config
            .renames_second
            .get(&col.name)
            .cloned()
            .unwrap_or_else(|| col.name.clone());

        let safe_name = if new_dict.find_by_name(&target_name).is_some() {
            resolve_unique_name(&new_dict, &target_name)
        } else {
            target_name
        };

        let s_meta = second_dict.get(s_idx).cloned().unwrap_or_else(|| {
            VariableMetadata::new(
                &safe_name,
                DataType::Numeric { decimals: 2 },
                Measure::Scale,
            )
            .unwrap_or_else(|_| VariableMetadata {
                name: safe_name.clone(),
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
        });

        let mut meta = s_meta;
        meta.name = safe_name.clone();
        new_dict.add(meta)?;

        let mut values = Vec::with_capacity(n_total);
        for &(_, s_row) in pairings {
            if let Some(sr) = s_row {
                values.push(col.values.get(sr).cloned().unwrap_or(RawValue::Null));
            } else {
                values.push(RawValue::Null);
            }
        }

        new_columns.push(RawColumn {
            name: safe_name,
            data_type: col.data_type,
            values,
        });
    }

    add_indicator_columns(
        &mut new_columns,
        &mut new_dict,
        config,
        n_total,
        |row| pairings[row].0.is_some(),
        |row| pairings[row].1.is_some(),
    )?;

    active_ds.table = RawTable {
        columns: new_columns,
        row_count: n_total,
    };
    active_ds.dictionary = new_dict;

    Ok(())
}

fn add_indicator_columns<F1, F2>(
    columns: &mut Vec<RawColumn>,
    dict: &mut VariableDictionary,
    config: &AddVarsConfig,
    n_total: usize,
    has_active: F1,
    has_second: F2,
) -> Result<(), CoreError>
where
    F1: Fn(usize) -> bool,
    F2: Fn(usize) -> bool,
{
    if let Some(ref in_act) = config.in_active_variable {
        let safe = sanitize_variable_name(in_act, dict.count() + 1);
        let meta =
            VariableMetadata::new(&safe, DataType::Numeric { decimals: 0 }, Measure::Nominal)?;
        dict.add(meta)?;
        let mut values = Vec::with_capacity(n_total);
        for r in 0..n_total {
            values.push(RawValue::Number(if has_active(r) { 1.0 } else { 0.0 }));
        }
        columns.push(RawColumn {
            name: safe,
            data_type: RawDataType::Numeric,
            values,
        });
    }

    if let Some(ref in_sec) = config.in_second_variable {
        let safe = sanitize_variable_name(in_sec, dict.count() + 1);
        let meta =
            VariableMetadata::new(&safe, DataType::Numeric { decimals: 0 }, Measure::Nominal)?;
        dict.add(meta)?;
        let mut values = Vec::with_capacity(n_total);
        for r in 0..n_total {
            values.push(RawValue::Number(if has_second(r) { 1.0 } else { 0.0 }));
        }
        columns.push(RawColumn {
            name: safe,
            data_type: RawDataType::Numeric,
            values,
        });
    }

    Ok(())
}

fn find_indices(table: &RawTable, keys: &[String]) -> Result<Vec<usize>, CoreError> {
    let mut indices = Vec::with_capacity(keys.len());
    for k in keys {
        let idx = table
            .columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(k))
            .ok_or_else(|| CoreError::VariableNotFound(k.clone()))?;
        indices.push(idx);
    }
    Ok(indices)
}

fn extract_keys(table: &RawTable, row: usize, indices: &[usize]) -> Vec<KeyValWrapper> {
    indices
        .iter()
        .map(|&i| {
            let val = table.columns[i]
                .values
                .get(row)
                .cloned()
                .unwrap_or(RawValue::Null);
            KeyValWrapper(val)
        })
        .collect()
}

fn resolve_unique_name(dict: &VariableDictionary, base_name: &str) -> String {
    let mut counter = 1;
    loop {
        let candidate = format!("{}_{}", base_name, counter);
        if dict.find_by_name(&candidate).is_none() {
            return candidate;
        }
        counter += 1;
    }
}

#[derive(Debug, Clone)]
struct KeyValWrapper(RawValue);

impl PartialEq for KeyValWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (RawValue::Null, RawValue::Null) => true,
            (RawValue::Number(a), RawValue::Number(b)) => a.to_bits() == b.to_bits(),
            (RawValue::Text(a), RawValue::Text(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for KeyValWrapper {}

impl std::hash::Hash for KeyValWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.0 {
            RawValue::Null => 0u8.hash(state),
            RawValue::Number(n) => {
                1u8.hash(state);
                n.to_bits().hash(state);
            }
            RawValue::Text(t) => {
                2u8.hash(state);
                t.hash(state);
            }
        }
    }
}
