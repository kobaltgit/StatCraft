use crate::split::{SplitFileConfig, SplitFileMode, SplitKeyValue, SplitSubgroup};
use crate::variable::{
    Alignment, CoreError, DataType, Measure, VariableDictionary, VariableMetadata,
};
use statcraft_io::{RawDataType, RawTable, RawValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellCoord {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellEdit {
    pub coord: CellCoord,
    pub old_val: RawValue,
    pub new_val: RawValue,
}

#[derive(Debug, Clone, Default)]
pub struct DeltaBuffer {
    pub undo_stack: Vec<CellEdit>,
    pub redo_stack: Vec<CellEdit>,
}

impl DeltaBuffer {
    pub fn new() -> Self {
        Self {
            undo_stack: vec![],
            redo_stack: vec![],
        }
    }

    pub fn record_edit(&mut self, edit: CellEdit) {
        self.undo_stack.push(edit);
        self.redo_stack.clear();
    }
}

#[derive(Debug, Clone)]
pub struct DataSet {
    pub table: RawTable,
    pub dictionary: VariableDictionary,
    pub delta: DeltaBuffer,
    pub active_filter_var: Option<String>,
    pub split_file: Option<SplitFileConfig>,
    pub weight_var: Option<String>,
}

impl DataSet {
    pub fn empty() -> Self {
        Self {
            table: RawTable::empty(),
            dictionary: VariableDictionary::new(),
            delta: DeltaBuffer::new(),
            active_filter_var: None,
            split_file: None,
            weight_var: None,
        }
    }

    pub fn blank(rows: usize, cols: usize) -> Result<Self, CoreError> {
        let mut dictionary = VariableDictionary::new();
        let table = RawTable::with_dimensions(rows, cols);

        for i in 0..cols {
            let name = format!("VAR{:05}", i + 1);
            let var =
                VariableMetadata::new(&name, DataType::Numeric { decimals: 2 }, Measure::Scale)?;
            dictionary.add(var)?;
        }

        Ok(Self {
            table,
            dictionary,
            delta: DeltaBuffer::new(),
            active_filter_var: None,
            split_file: None,
            weight_var: None,
        })
    }

    pub fn from_raw_table(table: RawTable) -> Result<Self, CoreError> {
        let mut dictionary = VariableDictionary::new();

        for col in &table.columns {
            let (var_type, measure) = match col.data_type {
                RawDataType::Numeric => (DataType::Numeric { decimals: 2 }, Measure::Scale),
                RawDataType::Text => (DataType::String { max_len: 255 }, Measure::Nominal),
            };

            let var = VariableMetadata::new(&col.name, var_type, measure)?;
            dictionary.add(var)?;
        }

        Ok(Self {
            table,
            dictionary,
            delta: DeltaBuffer::new(),
            active_filter_var: None,
            split_file: None,
            weight_var: None,
        })
    }

    pub fn row_count(&self) -> usize {
        self.table.row_count
    }

    pub fn col_count(&self) -> usize {
        self.table.col_count()
    }

    pub fn set_filter_variable(&mut self, var_name: Option<String>) -> Result<(), CoreError> {
        if let Some(ref name) = var_name {
            if self.dictionary.find_index(name).is_none() {
                return Err(CoreError::VariableNotFound(name.clone()));
            }
        }
        self.active_filter_var = var_name;
        Ok(())
    }

    pub fn active_filter_var(&self) -> Option<&str> {
        self.active_filter_var.as_deref()
    }

    pub fn set_weight_variable(&mut self, var_name: Option<String>) -> Result<(), CoreError> {
        if let Some(ref name) = var_name {
            let idx = self
                .dictionary
                .find_index(name)
                .ok_or_else(|| CoreError::VariableNotFound(name.clone()))?;
            let meta = self
                .dictionary
                .get(idx)
                .ok_or_else(|| CoreError::VariableNotFound(name.clone()))?;
            if !meta.var_type.is_numeric() {
                return Err(CoreError::TypeMismatch(format!(
                    "Weight variable '{name}' must be numeric"
                )));
            }
        }
        self.weight_var = var_name;
        Ok(())
    }

    pub fn weight_var(&self) -> Option<&str> {
        self.weight_var.as_deref()
    }

    pub fn is_weighted(&self) -> bool {
        self.weight_var.is_some()
    }

    pub fn row_weight(&self, row_idx: usize) -> f64 {
        if !self.is_row_active(row_idx) {
            return 0.0;
        }
        let var_name = match &self.weight_var {
            Some(name) => name,
            None => return 1.0,
        };
        let col_idx = match self.dictionary.find_index(var_name) {
            Some(idx) => idx,
            None => return 1.0,
        };
        match self.get_cell(row_idx, col_idx) {
            Some(RawValue::Number(n)) if !n.is_nan() && *n > 0.0 => *n,
            _ => 0.0,
        }
    }

    pub fn set_split_file(&mut self, config: Option<SplitFileConfig>) -> Result<(), CoreError> {
        if let Some(ref cfg) = config {
            if cfg.mode != SplitFileMode::Off {
                for var_name in &cfg.variables {
                    if self.dictionary.find_index(var_name).is_none() {
                        return Err(CoreError::VariableNotFound(var_name.clone()));
                    }
                }
            }
        }
        self.split_file = config;
        Ok(())
    }

    pub fn split_file(&self) -> Option<&SplitFileConfig> {
        self.split_file.as_ref()
    }

    pub fn split_groups(&self) -> Vec<SplitSubgroup> {
        let active_rows = self.active_row_indices();
        if active_rows.is_empty() {
            return Vec::new();
        }

        let config = match &self.split_file {
            Some(cfg) if cfg.is_active() => cfg,
            _ => {
                return vec![SplitSubgroup {
                    keys: Vec::new(),
                    label: String::new(),
                    row_indices: active_rows,
                }];
            }
        };

        let mut col_indices = Vec::with_capacity(config.variables.len());
        for var_name in &config.variables {
            if let Some(idx) = self.dictionary.find_index(var_name) {
                col_indices.push((var_name.clone(), idx));
            }
        }

        if col_indices.is_empty() {
            return vec![SplitSubgroup {
                keys: Vec::new(),
                label: String::new(),
                row_indices: active_rows,
            }];
        }

        let mut groups: Vec<SplitSubgroup> = Vec::new();

        for r in active_rows {
            let mut row_keys = Vec::with_capacity(col_indices.len());
            let mut label_parts = Vec::with_capacity(col_indices.len());

            for (var_name, col_idx) in &col_indices {
                let raw_val = self
                    .get_cell(r, *col_idx)
                    .cloned()
                    .unwrap_or(RawValue::Null);
                let disp_val = self.get_cell_display(r, *col_idx, true);
                let raw_str = self.get_cell_display(r, *col_idx, false);

                let label_part = if !disp_val.is_empty() && disp_val != raw_str && raw_str != "." {
                    format!("{var_name} = {raw_str} [{disp_val}]")
                } else if disp_val.is_empty() || disp_val == "." {
                    format!("{var_name} = .")
                } else {
                    format!("{var_name} = {disp_val}")
                };

                label_parts.push(label_part);
                row_keys.push(SplitKeyValue {
                    var_name: var_name.clone(),
                    raw_value: raw_val,
                    display_value: disp_val,
                });
            }

            let full_label = label_parts.join(", ");

            if let Some(last) = groups.last_mut() {
                if last.keys == row_keys {
                    last.row_indices.push(r);
                    continue;
                }
            }

            if let Some(existing) = groups.iter_mut().find(|g| g.keys == row_keys) {
                existing.row_indices.push(r);
            } else {
                groups.push(SplitSubgroup {
                    keys: row_keys,
                    label: full_label,
                    row_indices: vec![r],
                });
            }
        }

        groups
    }

    pub fn is_row_active(&self, row_idx: usize) -> bool {
        if row_idx >= self.row_count() {
            return false;
        }
        let filter_var = match &self.active_filter_var {
            Some(v) => v,
            None => return true,
        };
        let col_idx = match self.dictionary.find_index(filter_var) {
            Some(idx) => idx,
            None => return true,
        };
        match self.get_cell(row_idx, col_idx) {
            Some(RawValue::Number(n)) => !n.is_nan() && *n != 0.0,
            Some(RawValue::Text(s)) => {
                let trimmed = s.trim();
                trimmed == "1" || trimmed.eq_ignore_ascii_case("true")
            }
            Some(RawValue::Null) | None => false,
        }
    }

    pub fn is_row_filtered(&self, row_idx: usize) -> bool {
        self.is_row_active(row_idx)
    }

    pub fn active_row_indices(&self) -> Vec<usize> {
        if self.active_filter_var.is_none() {
            return (0..self.row_count()).collect();
        }
        (0..self.row_count())
            .filter(|&r| self.is_row_active(r))
            .collect()
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&RawValue> {
        self.table.get_value(row, col)
    }

    pub fn get_cell_display(&self, row: usize, col: usize, show_labels: bool) -> String {
        let raw = match self.get_cell(row, col) {
            Some(v) => v,
            None => return String::new(),
        };

        let var_meta = self.dictionary.get(col);

        if show_labels {
            if let Some(meta) = var_meta {
                if !meta.values.is_empty() {
                    let code_str = match raw {
                        RawValue::Number(n) => {
                            if n.fract() == 0.0 {
                                format!("{:.0}", n)
                            } else {
                                n.to_string()
                            }
                        }
                        RawValue::Text(s) => s.clone(),
                        RawValue::Null => return ".".to_string(),
                    };
                    if let Some(label) = meta.values.get_label(&code_str) {
                        return label.to_string();
                    }
                }
            }
        }

        match raw {
            RawValue::Null => ".".to_string(),
            RawValue::Text(s) => s.clone(),
            RawValue::Number(n) => {
                if let Some(meta) = var_meta {
                    match &meta.var_type {
                        DataType::Numeric { decimals } => {
                            format!("{:.precision$}", n, precision = *decimals as usize)
                        }
                        DataType::Comma { decimals } => {
                            format_number_grouped(*n, *decimals, ',', '.')
                        }
                        DataType::Dot { decimals } => {
                            format_number_grouped(*n, *decimals, '.', ',')
                        }
                        DataType::Scientific { decimals } => {
                            format!("{:.precision$E}", n, precision = *decimals as usize)
                        }
                        DataType::Dollar { decimals } => {
                            format!("${}", format_number_grouped(*n, *decimals, ',', '.'))
                        }
                        DataType::CustomCurrency {
                            format_code,
                            decimals,
                        } => {
                            format!(
                                "{} {}",
                                format_code,
                                format_number_grouped(*n, *decimals, ',', '.')
                            )
                        }
                        DataType::RestrictedNumeric { width } => {
                            let int_val = if *n < 0.0 { 0 } else { *n as u64 };
                            format!("{:0width$}", int_val, width = *width)
                        }
                        DataType::Date { .. } | DataType::String { .. } => {
                            if n.fract() == 0.0 {
                                format!("{:.0}", n)
                            } else {
                                n.to_string()
                            }
                        }
                    }
                } else if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    format!("{:.4}", n)
                }
            }
        }
    }

    pub fn set_cell(&mut self, row: usize, col: usize, new_val: RawValue) -> Result<(), CoreError> {
        if col >= self.table.columns.len() || row >= self.table.row_count {
            return Err(CoreError::CellOutOfBounds { row, col });
        }

        let old_val = self.table.columns[col].values[row].clone();
        if old_val == new_val {
            return Ok(());
        }

        let edit = CellEdit {
            coord: CellCoord { row, col },
            old_val: old_val.clone(),
            new_val: new_val.clone(),
        };

        self.table.columns[col].values[row] = new_val;
        self.delta.record_edit(edit);
        Ok(())
    }

    pub fn set_cell_parsed(&mut self, row: usize, col: usize, text: &str) -> Result<(), CoreError> {
        let trimmed = text.trim();
        let val = if trimmed.is_empty() || trimmed == "." {
            RawValue::Null
        } else if let Some(meta) = self.dictionary.get(col) {
            if meta.var_type.is_numeric() {
                if let Some(parsed) = parse_number_loose(trimmed) {
                    RawValue::Number(parsed)
                } else {
                    return Err(CoreError::TypeMismatch(text.to_string()));
                }
            } else {
                RawValue::Text(trimmed.to_string())
            }
        } else if let Some(parsed) = parse_number_loose(trimmed) {
            RawValue::Number(parsed)
        } else {
            RawValue::Text(trimmed.to_string())
        };

        self.set_cell(row, col, val)
    }

    pub fn add_variable(
        &mut self,
        name: &str,
        var_type: DataType,
        measure: Measure,
    ) -> Result<usize, CoreError> {
        let meta = VariableMetadata::new(name, var_type.clone(), measure)?;
        let col_type = if var_type.is_numeric() {
            RawDataType::Numeric
        } else {
            RawDataType::Text
        };
        self.table.add_column(&meta.name, col_type);
        self.dictionary.add(meta)
    }

    pub fn remove_variable(&mut self, col_idx: usize) -> Result<(), CoreError> {
        let total = self.dictionary.count();
        if col_idx >= total {
            return Err(CoreError::IndexOutOfBounds {
                index: col_idx,
                total,
            });
        }
        if let Some(ref filter_var) = self.active_filter_var {
            if let Some(removed_meta) = self.dictionary.get(col_idx) {
                if removed_meta.name == *filter_var {
                    self.active_filter_var = None;
                }
            }
        }
        if let Some(removed_meta) = self.dictionary.get(col_idx) {
            let is_weight = self
                .weight_var
                .as_ref()
                .is_some_and(|w| w.eq_ignore_ascii_case(&removed_meta.name));
            if is_weight {
                self.weight_var = None;
            }
        }
        if let Some(ref mut split_cfg) = self.split_file {
            if let Some(removed_meta) = self.dictionary.get(col_idx) {
                split_cfg.variables.retain(|v| v != &removed_meta.name);
                if split_cfg.variables.is_empty() {
                    self.split_file = None;
                }
            }
        }
        self.dictionary.remove(col_idx)?;
        self.table.remove_column(col_idx);
        Ok(())
    }

    pub fn rename_variable(&mut self, col_idx: usize, new_name: &str) -> Result<(), CoreError> {
        if let Some(ref filter_var) = self.active_filter_var {
            if let Some(old_meta) = self.dictionary.get(col_idx) {
                if old_meta.name == *filter_var {
                    self.active_filter_var = Some(new_name.to_string());
                }
            }
        }
        if let Some(old_meta) = self.dictionary.get(col_idx) {
            let is_weight = self
                .weight_var
                .as_ref()
                .is_some_and(|w| w.eq_ignore_ascii_case(&old_meta.name));
            if is_weight {
                self.weight_var = Some(new_name.to_string());
            }
        }
        if let Some(ref mut split_cfg) = self.split_file {
            if let Some(old_meta) = self.dictionary.get(col_idx) {
                for var in &mut split_cfg.variables {
                    if *var == old_meta.name {
                        *var = new_name.to_string();
                    }
                }
            }
        }
        self.dictionary.rename(col_idx, new_name)?;
        self.table.rename_column(col_idx, new_name);
        Ok(())
    }

    pub fn set_variable_type(
        &mut self,
        col_idx: usize,
        var_type: DataType,
        width: usize,
        decimals: u8,
    ) -> Result<(), CoreError> {
        self.dictionary.set_type(col_idx, var_type, width, decimals)
    }

    pub fn set_variable_decimals(&mut self, col_idx: usize, decimals: u8) -> Result<(), CoreError> {
        self.dictionary.set_decimals(col_idx, decimals)
    }

    pub fn set_variable_width(&mut self, col_idx: usize, width: usize) -> Result<(), CoreError> {
        self.dictionary.set_width(col_idx, width)
    }

    pub fn set_variable_align(
        &mut self,
        col_idx: usize,
        align: Alignment,
    ) -> Result<(), CoreError> {
        self.dictionary.set_align(col_idx, align)
    }

    pub fn add_empty_row(&mut self) {
        self.table.add_row();
    }

    pub fn remove_row(&mut self, row_idx: usize) -> Result<(), CoreError> {
        if row_idx >= self.table.row_count {
            return Err(CoreError::CellOutOfBounds {
                row: row_idx,
                col: 0,
            });
        }
        self.table.remove_row(row_idx);
        Ok(())
    }

    pub fn reorder_cases(&mut self, permutation: &[usize]) -> Result<(), CoreError> {
        if permutation.len() != self.row_count() {
            return Err(CoreError::IndexOutOfBounds {
                index: permutation.len(),
                total: self.row_count(),
            });
        }
        for &idx in permutation {
            if idx >= self.row_count() {
                return Err(CoreError::IndexOutOfBounds {
                    index: idx,
                    total: self.row_count(),
                });
            }
        }
        self.table
            .reorder_rows(permutation)
            .map_err(|_| CoreError::IndexOutOfBounds {
                index: permutation.len(),
                total: self.row_count(),
            })?;
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        if let Some(edit) = self.delta.undo_stack.pop() {
            let row = edit.coord.row;
            let col = edit.coord.col;
            if col < self.table.columns.len() && row < self.table.row_count {
                self.table.columns[col].values[row] = edit.old_val.clone();
                self.delta.redo_stack.push(edit);
                return true;
            }
        }
        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some(edit) = self.delta.redo_stack.pop() {
            let row = edit.coord.row;
            let col = edit.coord.col;
            if col < self.table.columns.len() && row < self.table.row_count {
                self.table.columns[col].values[row] = edit.new_val.clone();
                self.delta.undo_stack.push(edit);
                return true;
            }
        }
        false
    }
}

pub fn format_number_grouped(n: f64, decimals: u8, group_sep: char, dec_sep: char) -> String {
    let sign = if n < 0.0 { "-" } else { "" };
    let formatted = format!("{:.precision$}", n.abs(), precision = decimals as usize);
    let mut parts = formatted.split('.');
    let int_part = parts.next().unwrap_or("0");
    let fract_part = parts.next();

    let mut grouped_int = String::with_capacity(int_part.len() + int_part.len() / 3);
    let bytes = int_part.as_bytes();
    let len = bytes.len();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            grouped_int.push(group_sep);
        }
        grouped_int.push(b as char);
    }

    if let Some(fract) = fract_part {
        if decimals > 0 {
            format!("{}{}{}{}", sign, grouped_int, dec_sep, fract)
        } else {
            format!("{}{}", sign, grouped_int)
        }
    } else {
        format!("{}{}", sign, grouped_int)
    }
}

pub fn parse_number_loose(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "." {
        return None;
    }
    let mut cleaned = String::with_capacity(trimmed.len());
    let mut has_comma = false;
    let mut has_dot = false;
    for c in trimmed.chars() {
        if c.is_ascii_digit() || c == '-' || c == '+' || c == 'e' || c == 'E' {
            cleaned.push(c);
        } else if c == '.' {
            cleaned.push('.');
            has_dot = true;
        } else if c == ',' {
            has_comma = true;
            cleaned.push(',');
        }
    }
    if has_comma && has_dot {
        if let (Some(dot_pos), Some(comma_pos)) = (cleaned.rfind('.'), cleaned.rfind(',')) {
            if dot_pos > comma_pos {
                cleaned = cleaned.replace(',', "");
            } else {
                cleaned = cleaned.replace('.', "").replace(',', ".");
            }
        }
    } else if has_comma && !has_dot {
        cleaned = cleaned.replace(',', ".");
    }
    cleaned.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_io::{RawColumn, RawDataType, RawTable, RawValue};

    fn make_test_table() -> RawTable {
        let col1 = RawColumn {
            name: "id".to_string(),
            data_type: RawDataType::Numeric,
            values: vec![RawValue::Number(1.0), RawValue::Number(2.0)],
        };
        let col2 = RawColumn {
            name: "gender".to_string(),
            data_type: RawDataType::Numeric,
            values: vec![RawValue::Number(1.0), RawValue::Number(2.0)],
        };
        RawTable {
            columns: vec![col1, col2],
            row_count: 2,
        }
    }

    #[test]
    fn test_dataset_dimensions() {
        let table = make_test_table();
        let ds = DataSet::from_raw_table(table).unwrap();

        assert_eq!(ds.row_count(), 2);
        assert_eq!(ds.col_count(), 2);
        assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(1.0)));
        assert_eq!(ds.get_cell(5, 5), None);
    }

    #[test]
    fn test_dataset_get_cell_bounds_check() {
        let mut ds = DataSet::empty();
        assert_eq!(ds.get_cell(0, 0), None);
        assert!(matches!(
            ds.set_cell(0, 0, RawValue::Number(42.0)),
            Err(CoreError::CellOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_dataset_cell_edit_and_undo_redo() {
        let table = make_test_table();
        let mut ds = DataSet::from_raw_table(table).unwrap();

        // Initial value is 1.0
        assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(1.0)));

        // Edit cell to 99.0
        ds.set_cell(0, 0, RawValue::Number(99.0)).unwrap();
        assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(99.0)));

        // Undo
        assert!(ds.undo());
        assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(1.0)));

        // Redo
        assert!(ds.redo());
        assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(99.0)));
    }

    #[test]
    fn test_dataset_format_display_value() {
        let table = make_test_table();
        let mut ds = DataSet::from_raw_table(table).unwrap();

        // Without labels: 1.00 (since default numeric has decimals: 2)
        assert_eq!(ds.get_cell_display(0, 1, false), "1.00");

        // Add ValueLabel 1 => "Male"
        ds.dictionary.get_mut(1).unwrap().values.insert("1", "Male");

        // With labels: Male
        assert_eq!(ds.get_cell_display(0, 1, true), "Male");
    }

    #[test]
    fn test_dataset_blank_and_add_remove_variable() {
        let mut ds = DataSet::blank(10, 2).unwrap();
        assert_eq!(ds.row_count(), 10);
        assert_eq!(ds.col_count(), 2);
        assert_eq!(ds.dictionary.get(0).unwrap().name, "VAR00001");
        assert_eq!(ds.dictionary.get(1).unwrap().name, "VAR00002");

        // Add variable
        let idx = ds
            .add_variable("income", DataType::Dollar { decimals: 2 }, Measure::Scale)
            .unwrap();
        assert_eq!(idx, 2);
        assert_eq!(ds.col_count(), 3);
        assert_eq!(ds.get_cell(0, 2), Some(&RawValue::Null));

        // Set cell parsed
        ds.set_cell_parsed(0, 2, "$1,250.50").unwrap();
        assert_eq!(ds.get_cell(0, 2), Some(&RawValue::Number(1250.5)));
        assert_eq!(ds.get_cell_display(0, 2, false), "$1,250.50");

        // Rename variable
        ds.rename_variable(2, "monthly_income").unwrap();
        assert_eq!(ds.dictionary.get(2).unwrap().name, "monthly_income");

        // Remove variable
        ds.remove_variable(0).unwrap();
        assert_eq!(ds.col_count(), 2);
        assert_eq!(ds.dictionary.get(0).unwrap().name, "VAR00002");
    }

    #[test]
    fn test_dataset_rows_manipulation() {
        let mut ds = DataSet::blank(2, 1).unwrap();
        assert_eq!(ds.row_count(), 2);

        ds.add_empty_row();
        assert_eq!(ds.row_count(), 3);
        assert_eq!(ds.get_cell(2, 0), Some(&RawValue::Null));

        ds.remove_row(1).unwrap();
        assert_eq!(ds.row_count(), 2);
    }

    #[test]
    fn test_dataset_reorder_cases() {
        let mut ds = DataSet::blank(3, 1).unwrap();
        ds.set_cell(0, 0, RawValue::Number(10.0)).unwrap();
        ds.set_cell(1, 0, RawValue::Number(20.0)).unwrap();
        ds.set_cell(2, 0, RawValue::Number(30.0)).unwrap();

        ds.reorder_cases(&[2, 0, 1]).unwrap();
        assert_eq!(ds.get_cell(0, 0), Some(&RawValue::Number(30.0)));
        assert_eq!(ds.get_cell(1, 0), Some(&RawValue::Number(10.0)));
        assert_eq!(ds.get_cell(2, 0), Some(&RawValue::Number(20.0)));

        assert!(ds.reorder_cases(&[0, 1]).is_err());
        assert!(ds.reorder_cases(&[0, 1, 99]).is_err());
    }

    #[test]
    fn test_formatting_variations() {
        assert_eq!(
            format_number_grouped(1234567.89, 2, ',', '.'),
            "1,234,567.89"
        );
        assert_eq!(
            format_number_grouped(1234567.89, 2, '.', ','),
            "1.234.567,89"
        );
        assert_eq!(format_number_grouped(-9876.5, 1, ',', '.'), "-9,876.5");
        assert_eq!(parse_number_loose("1,234.56"), Some(1234.56));
        assert_eq!(parse_number_loose("1.234,56"), Some(1234.56));
        assert_eq!(parse_number_loose("$ 999.00"), Some(999.0));
    }

    #[test]
    fn test_dataset_filtering() {
        let mut ds = DataSet::blank(4, 2).unwrap();
        // col 0: VAR00001, col 1: VAR00002 (we use as filter)
        ds.set_cell(0, 1, RawValue::Number(1.0)).unwrap();
        ds.set_cell(1, 1, RawValue::Number(0.0)).unwrap();
        ds.set_cell(2, 1, RawValue::Number(1.0)).unwrap();
        ds.set_cell(3, 1, RawValue::Null).unwrap();

        assert_eq!(ds.active_row_indices(), vec![0, 1, 2, 3]);
        assert!(ds.is_row_active(1));

        ds.set_filter_variable(Some("VAR00002".to_string()))
            .unwrap();
        assert_eq!(ds.active_filter_var(), Some("VAR00002"));
        assert!(ds.is_row_active(0));
        assert!(!ds.is_row_active(1));
        assert!(ds.is_row_active(2));
        assert!(!ds.is_row_active(3));
        assert_eq!(ds.active_row_indices(), vec![0, 2]);

        // Non-existent variable
        assert!(ds.set_filter_variable(Some("NON_EXISTENT".into())).is_err());

        // Clear filter
        ds.set_filter_variable(None).unwrap();
        assert_eq!(ds.active_filter_var(), None);
        assert_eq!(ds.active_row_indices(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_dataset_split_file() {
        let mut ds = DataSet::blank(4, 2).unwrap();
        // Col 0: gender (1, 1, 2, 2)
        // Col 1: salary (100, 200, 300, 400)
        ds.dictionary.get_mut(0).unwrap().name = "gender".to_string();
        ds.dictionary.get_mut(1).unwrap().name = "salary".to_string();
        ds.table.columns[0].name = "gender".to_string();
        ds.table.columns[1].name = "salary".to_string();

        ds.set_cell(0, 0, RawValue::Number(1.0)).unwrap();
        ds.set_cell(1, 0, RawValue::Number(1.0)).unwrap();
        ds.set_cell(2, 0, RawValue::Number(2.0)).unwrap();
        ds.set_cell(3, 0, RawValue::Number(2.0)).unwrap();

        // Default: 1 group containing all rows
        let groups = ds.split_groups();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].row_indices, vec![0, 1, 2, 3]);

        // Enable split file
        ds.set_split_file(Some(SplitFileConfig::new(
            SplitFileMode::CompareGroups,
            vec!["gender".to_string()],
        )))
        .unwrap();

        assert!(ds.split_file().unwrap().is_active());
        let groups = ds.split_groups();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].row_indices, vec![0, 1]);
        assert_eq!(groups[1].row_indices, vec![2, 3]);
        assert_eq!(groups[0].keys[0].var_name, "gender");
        assert_eq!(groups[0].keys[0].raw_value, RawValue::Number(1.0));

        // Rename variable: split_file should be updated
        ds.rename_variable(0, "sex").unwrap();
        assert_eq!(ds.split_file().unwrap().variables, vec!["sex".to_string()]);

        // Delete variable: split_file should be cleared
        ds.remove_variable(0).unwrap();
        assert_eq!(ds.split_file(), None);
    }

    #[test]
    fn test_dataset_weighting() {
        let mut ds = DataSet::blank(5, 1).unwrap();
        ds.add_variable(
            "str_var",
            DataType::String { max_len: 20 },
            Measure::Nominal,
        )
        .unwrap();
        ds.add_variable(
            "filter_col",
            DataType::Numeric { decimals: 0 },
            Measure::Scale,
        )
        .unwrap();

        assert!(!ds.is_weighted());
        assert_eq!(ds.weight_var(), None);
        assert_eq!(ds.row_weight(0), 1.0);

        let err_missing = ds.set_weight_variable(Some("missing".to_string()));
        assert!(matches!(err_missing, Err(CoreError::VariableNotFound(_))));

        let err_str = ds.set_weight_variable(Some("str_var".to_string()));
        assert!(matches!(err_str, Err(CoreError::TypeMismatch(_))));

        ds.set_weight_variable(Some("VAR00001".to_string()))
            .unwrap();
        assert!(ds.is_weighted());
        assert_eq!(ds.weight_var(), Some("VAR00001"));

        ds.set_cell(0, 0, RawValue::Number(2.5)).unwrap();
        ds.set_cell(1, 0, RawValue::Number(0.0)).unwrap();
        ds.set_cell(2, 0, RawValue::Number(-3.0)).unwrap();
        ds.set_cell(3, 0, RawValue::Number(f64::NAN)).unwrap();
        ds.set_cell(4, 0, RawValue::Null).unwrap();

        assert_eq!(ds.row_weight(0), 2.5);
        assert_eq!(ds.row_weight(1), 0.0);
        assert_eq!(ds.row_weight(2), 0.0);
        assert_eq!(ds.row_weight(3), 0.0);
        assert_eq!(ds.row_weight(4), 0.0);

        ds.set_cell(0, 2, RawValue::Number(1.0)).unwrap();
        ds.set_cell(1, 2, RawValue::Number(1.0)).unwrap();
        ds.set_cell(2, 2, RawValue::Number(0.0)).unwrap();
        ds.set_filter_variable(Some("filter_col".to_string()))
            .unwrap();

        assert_eq!(ds.row_weight(0), 2.5);
        assert_eq!(ds.row_weight(1), 0.0);
        ds.set_cell(2, 0, RawValue::Number(10.0)).unwrap();
        assert_eq!(ds.row_weight(2), 0.0); // Inactive row -> 0.0

        ds.set_filter_variable(None).unwrap();
        assert_eq!(ds.row_weight(2), 10.0);

        ds.rename_variable(0, "w_cases").unwrap();
        assert_eq!(ds.weight_var(), Some("w_cases"));
        assert_eq!(ds.row_weight(0), 2.5);

        ds.remove_variable(0).unwrap();
        assert!(!ds.is_weighted());
        assert_eq!(ds.weight_var(), None);
        assert_eq!(ds.row_weight(0), 1.0);
    }
}
