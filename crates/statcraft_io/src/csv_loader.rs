use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum IoError {
    #[error("File is empty: {0}")]
    EmptyFile(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("CSV parsing error: {0}")]
    Csv(String),

    #[error("Excel parsing error: {0}")]
    Excel(String),

    #[error("Malformed row at line {line}: expected {expected} columns, found {found}")]
    MalformedRow {
        line: usize,
        expected: usize,
        found: usize,
    },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Malformed SPSS file: {0}")]
    MalformedSpss(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

impl From<std::io::Error> for IoError {
    fn from(e: std::io::Error) -> Self {
        IoError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for IoError {
    fn from(e: serde_json::Error) -> Self {
        IoError::Deserialization(e.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawDataType {
    Numeric,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RawValue {
    Number(f64),
    Text(String),
    Null,
}

impl RawValue {
    pub fn as_display_string(&self) -> String {
        match self {
            RawValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    format!("{:.4}", n)
                }
            }
            RawValue::Text(s) => s.clone(),
            RawValue::Null => ".".to_string(),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, RawValue::Null)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawColumn {
    pub name: String,
    pub data_type: RawDataType,
    pub values: Vec<RawValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawTable {
    pub columns: Vec<RawColumn>,
    pub row_count: usize,
}

impl RawTable {
    pub fn empty() -> Self {
        Self {
            columns: vec![],
            row_count: 0,
        }
    }

    pub fn with_dimensions(row_count: usize, col_count: usize) -> Self {
        let mut columns = Vec::with_capacity(col_count);
        for i in 0..col_count {
            let name = format!("VAR{:05}", i + 1);
            columns.push(RawColumn {
                name,
                data_type: RawDataType::Numeric,
                values: vec![RawValue::Null; row_count],
            });
        }
        Self { columns, row_count }
    }

    pub fn col_count(&self) -> usize {
        self.columns.len()
    }

    pub fn get_value(&self, row: usize, col: usize) -> Option<&RawValue> {
        if col >= self.columns.len() {
            return None;
        }
        self.columns[col].values.get(row)
    }

    pub fn add_column(&mut self, name: &str, data_type: RawDataType) -> usize {
        let idx = self.columns.len();
        self.columns.push(RawColumn {
            name: name.to_string(),
            data_type,
            values: vec![RawValue::Null; self.row_count],
        });
        idx
    }

    pub fn remove_column(&mut self, col_idx: usize) -> Option<RawColumn> {
        if col_idx < self.columns.len() {
            Some(self.columns.remove(col_idx))
        } else {
            None
        }
    }

    pub fn rename_column(&mut self, col_idx: usize, new_name: &str) -> bool {
        if let Some(col) = self.columns.get_mut(col_idx) {
            col.name = new_name.to_string();
            true
        } else {
            false
        }
    }

    pub fn add_row(&mut self) {
        for col in &mut self.columns {
            col.values.push(RawValue::Null);
        }
        self.row_count += 1;
    }

    pub fn remove_row(&mut self, row_idx: usize) -> bool {
        if row_idx >= self.row_count {
            return false;
        }
        for col in &mut self.columns {
            if row_idx < col.values.len() {
                col.values.remove(row_idx);
            }
        }
        self.row_count -= 1;
        true
    }

    pub fn set_value(&mut self, row: usize, col: usize, val: RawValue) -> bool {
        if col < self.columns.len() && row < self.row_count && row < self.columns[col].values.len()
        {
            self.columns[col].values[row] = val;
            return true;
        }
        false
    }

    pub fn reorder_rows(&mut self, permutation: &[usize]) -> Result<(), IoError> {
        if permutation.len() != self.row_count {
            return Err(IoError::MalformedRow {
                line: 0,
                expected: self.row_count,
                found: permutation.len(),
            });
        }
        for col in &mut self.columns {
            let old_values = std::mem::take(&mut col.values);
            let mut new_values = Vec::with_capacity(permutation.len());
            for &idx in permutation {
                if idx < old_values.len() {
                    new_values.push(old_values[idx].clone());
                } else {
                    new_values.push(RawValue::Null);
                }
            }
            col.values = new_values;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CsvOptions {
    pub delimiter: u8,
    pub has_headers: bool,
    pub quote: u8,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            has_headers: true,
            quote: b'"',
        }
    }
}

pub fn detect_delimiter(sample: &str) -> u8 {
    let comma_count = sample.matches(',').count();
    let semicolon_count = sample.matches(';').count();
    let tab_count = sample.matches('\t').count();

    if semicolon_count > comma_count && semicolon_count > tab_count {
        b';'
    } else if tab_count > comma_count && tab_count > semicolon_count {
        b'\t'
    } else {
        b','
    }
}

pub fn load_csv_from_reader<R: Read>(
    reader: R,
    mut options: CsvOptions,
) -> Result<RawTable, IoError> {
    let mut buf_reader = BufReader::new(reader);

    // Read full string to auto-detect delimiter if default comma had no matches
    let mut content = String::new();
    buf_reader
        .read_to_string(&mut content)
        .map_err(|e| IoError::Io(e.to_string()))?;

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(IoError::EmptyFile("Input stream is empty".to_string()));
    }

    // Auto-adjust delimiter if default comma isn't prominent
    let detected = detect_delimiter(trimmed);
    options.delimiter = detected;

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(options.delimiter)
        .has_headers(options.has_headers)
        .quote(options.quote)
        .flexible(true)
        .from_reader(trimmed.as_bytes());

    let headers: Vec<String> = if options.has_headers {
        let hdrs = rdr.headers().map_err(|e| IoError::Csv(e.to_string()))?;
        hdrs.iter()
            .enumerate()
            .map(|(i, h)| {
                let trimmed = h.trim();
                if trimmed.is_empty() {
                    format!("VAR{:04}", i + 1)
                } else {
                    trimmed.to_string()
                }
            })
            .collect()
    } else {
        vec![]
    };

    let mut raw_records: Vec<Vec<String>> = vec![];
    for result in rdr.records() {
        let record = result.map_err(|e| IoError::Csv(e.to_string()))?;
        raw_records.push(record.iter().map(|s| s.trim().to_string()).collect());
    }

    let row_count = raw_records.len();
    if row_count == 0 {
        if headers.is_empty() {
            return Err(IoError::EmptyFile(
                "No data records found in CSV".to_string(),
            ));
        }
        // Headers only
        let columns = headers
            .into_iter()
            .map(|name| RawColumn {
                name,
                data_type: RawDataType::Text,
                values: vec![],
            })
            .collect();
        return Ok(RawTable {
            columns,
            row_count: 0,
        });
    }

    let col_count = if !headers.is_empty() {
        headers.len()
    } else {
        raw_records[0].len()
    };

    let col_names = if !headers.is_empty() {
        headers
    } else {
        (1..=col_count).map(|i| format!("VAR{:04}", i)).collect()
    };

    // Determine type and values for each column
    let mut columns: Vec<RawColumn> = Vec::with_capacity(col_count);

    for col_idx in 0..col_count {
        let name = col_names
            .get(col_idx)
            .cloned()
            .unwrap_or_else(|| format!("VAR{:04}", col_idx + 1));

        let mut values = Vec::with_capacity(row_count);
        let mut all_numeric = true;

        for row in raw_records.iter() {
            let str_val = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
            if str_val.is_empty() || str_val == "." || str_val.eq_ignore_ascii_case("na") {
                values.push(RawValue::Null);
            } else {
                // Try parse numeric (handle both dot and comma if delimiter was semicolon)
                let num_candidate =
                    if options.delimiter == b';' && str_val.contains(',') && !str_val.contains('.')
                    {
                        str_val.replace(',', ".").parse::<f64>()
                    } else {
                        str_val.parse::<f64>()
                    };

                match num_candidate {
                    Ok(n) => values.push(RawValue::Number(n)),
                    Err(_) => {
                        all_numeric = false;
                        values.push(RawValue::Text(str_val.to_string()));
                    }
                }
            }
        }

        // If not all numeric, re-cast any numbers to text
        let data_type = if all_numeric {
            RawDataType::Numeric
        } else {
            for v in values.iter_mut() {
                if let RawValue::Number(n) = v {
                    *v = RawValue::Text(if n.fract() == 0.0 {
                        format!("{:.0}", n)
                    } else {
                        n.to_string()
                    });
                }
            }
            RawDataType::Text
        };

        columns.push(RawColumn {
            name,
            data_type,
            values,
        });
    }

    Ok(RawTable { columns, row_count })
}

pub fn load_csv_from_path(path: &Path, options: CsvOptions) -> Result<RawTable, IoError> {
    let file = File::open(path)
        .map_err(|e| IoError::Io(format!("Cannot open {}: {}", path.display(), e)))?;
    load_csv_from_reader(file, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parse_numeric_columns() {
        let data = "id,age,salary\n1,25,50000.5\n2,30,65000.0\n3,42,82000.75\n";
        let table = load_csv_from_reader(data.as_bytes(), CsvOptions::default()).unwrap();

        assert_eq!(table.row_count, 3);
        assert_eq!(table.col_count(), 3);

        assert_eq!(table.columns[0].name, "id");
        assert_eq!(table.columns[0].data_type, RawDataType::Numeric);
        assert_eq!(table.get_value(0, 0), Some(&RawValue::Number(1.0)));

        assert_eq!(table.columns[2].name, "salary");
        assert_eq!(table.columns[2].data_type, RawDataType::Numeric);
        assert_eq!(table.get_value(2, 2), Some(&RawValue::Number(82000.75)));
    }

    #[test]
    fn test_csv_parse_string_columns() {
        let data = "name,department\nAlice,Engineering\nBob,Marketing\nCharlie,Design\n";
        let table = load_csv_from_reader(data.as_bytes(), CsvOptions::default()).unwrap();

        assert_eq!(table.row_count, 3);
        assert_eq!(table.col_count(), 2);
        assert_eq!(table.columns[0].data_type, RawDataType::Text);
        assert_eq!(
            table.get_value(1, 0),
            Some(&RawValue::Text("Bob".to_string()))
        );
    }

    #[test]
    fn test_csv_semicolon_delimiter() {
        let data = "id;cost;group\n101;12,50;A\n102;45,90;B\n";
        let table = load_csv_from_reader(data.as_bytes(), CsvOptions::default()).unwrap();

        assert_eq!(table.row_count, 2);
        assert_eq!(table.col_count(), 3);
        assert_eq!(table.columns[1].data_type, RawDataType::Numeric);
        assert_eq!(table.get_value(0, 1), Some(&RawValue::Number(12.50)));
    }

    #[test]
    fn test_csv_empty_file_error() {
        let empty_data = "";
        let result = load_csv_from_reader(empty_data.as_bytes(), CsvOptions::default());
        assert!(matches!(result, Err(IoError::EmptyFile(_))));
    }

    #[test]
    fn test_csv_missing_values() {
        let data = "id,score\n1,100\n2,.\n3,NA\n4,\n";
        let table = load_csv_from_reader(data.as_bytes(), CsvOptions::default()).unwrap();

        assert_eq!(table.row_count, 4);
        assert_eq!(table.get_value(0, 1), Some(&RawValue::Number(100.0)));
        assert_eq!(table.get_value(1, 1), Some(&RawValue::Null));
        assert_eq!(table.get_value(2, 1), Some(&RawValue::Null));
        assert_eq!(table.get_value(3, 1), Some(&RawValue::Null));
    }
}
