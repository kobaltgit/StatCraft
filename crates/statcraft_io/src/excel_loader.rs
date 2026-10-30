use std::collections::HashSet;
use std::io::{Read, Seek};
use std::path::Path;

use calamine::{open_workbook_auto, open_workbook_auto_from_rs, Data, Range, Reader, Sheets};

use crate::csv_loader::{IoError, RawColumn, RawDataType, RawTable, RawValue};

/// Loads an Excel workbook from the specified file path.
///
/// Supports `.xlsx`, `.xlsm`, `.xls`, `.ods`.
/// If `sheet_name` is `None`, the first worksheet is loaded.
pub fn load_excel_from_path<P: AsRef<Path>>(
    path: P,
    sheet_name: Option<&str>,
) -> Result<RawTable, IoError> {
    let path_ref = path.as_ref();
    let mut workbook: Sheets<_> = open_workbook_auto(path_ref)
        .map_err(|e| IoError::Excel(format!("Failed to open '{}': {}", path_ref.display(), e)))?;

    load_worksheet_from_workbook(&mut workbook, sheet_name)
}

/// Loads an Excel workbook from any seekable reader.
pub fn load_excel_from_reader<R: Read + Seek + Clone>(
    reader: R,
    sheet_name: Option<&str>,
) -> Result<RawTable, IoError> {
    let mut workbook: Sheets<_> = open_workbook_auto_from_rs(reader)
        .map_err(|e| IoError::Excel(format!("Failed to parse workbook stream: {}", e)))?;

    load_worksheet_from_workbook(&mut workbook, sheet_name)
}

fn load_worksheet_from_workbook<RS: Read + Seek, R: Reader<RS>>(
    workbook: &mut R,
    sheet_name: Option<&str>,
) -> Result<RawTable, IoError> {
    let sheet_names = workbook.sheet_names();
    if sheet_names.is_empty() {
        return Err(IoError::EmptyFile(
            "Workbook contains no sheets".to_string(),
        ));
    }

    let target_sheet = match sheet_name {
        Some(name) => {
            if !sheet_names.iter().any(|s| s == name) {
                return Err(IoError::Excel(format!(
                    "Sheet '{}' not found in workbook. Available: {:?}",
                    name, sheet_names
                )));
            }
            name.to_string()
        }
        None => sheet_names[0].clone(),
    };

    let range = workbook
        .worksheet_range(&target_sheet)
        .map_err(|e| IoError::Excel(format!("Failed to read sheet '{}': {:?}", target_sheet, e)))?;

    load_excel_from_range(range)
}

/// Converts a calamine `Range<Data>` into a `RawTable`.
///
/// Row 0 is interpreted as headers.
/// If headers are empty or duplicate, standard SPSS sanitized names are generated.
pub fn load_excel_from_range(range: Range<Data>) -> Result<RawTable, IoError> {
    let width = range.width();
    let height = range.height();

    if width == 0 || height == 0 {
        return Ok(RawTable::empty());
    }

    // Extract headers from first row
    let mut raw_headers = Vec::with_capacity(width);
    for col_idx in 0..width {
        let cell = range.get((0, col_idx));
        let header_str = match cell {
            Some(Data::String(s)) => s.clone(),
            Some(Data::Int(i)) => i.to_string(),
            Some(Data::Float(f)) => {
                if f.fract() == 0.0 {
                    format!("{:.0}", f)
                } else {
                    f.to_string()
                }
            }
            Some(Data::Bool(b)) => b.to_string(),
            Some(Data::DateTime(dt)) => dt.to_string(),
            Some(Data::DateTimeIso(s)) | Some(Data::DurationIso(s)) => s.clone(),
            _ => String::new(),
        };
        raw_headers.push(header_str);
    }

    let sanitized_names = sanitize_and_dedup_headers(&raw_headers);

    // If only 1 row (headers), return 0 data rows
    if height == 1 {
        let columns = sanitized_names
            .into_iter()
            .map(|name| RawColumn {
                name,
                data_type: RawDataType::Numeric,
                values: Vec::new(),
            })
            .collect();
        return Ok(RawTable {
            columns,
            row_count: 0,
        });
    }

    let row_count = height - 1;
    let mut columns = Vec::with_capacity(width);

    for col_idx in 0..width {
        let col_name = sanitized_names
            .get(col_idx)
            .cloned()
            .unwrap_or_else(|| format!("VAR{:05}", col_idx + 1));

        let mut values = Vec::with_capacity(row_count);
        let mut all_numeric = true;

        for row_idx in 1..height {
            let cell = range.get((row_idx, col_idx));
            let val = match cell {
                Some(c) => cell_to_candidate(c),
                None => RawValue::Null,
            };

            if let RawValue::Text(_) = &val {
                all_numeric = false;
            }
            values.push(val);
        }

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
            name: col_name,
            data_type,
            values,
        });
    }

    Ok(RawTable { columns, row_count })
}

fn cell_to_candidate(cell: &Data) -> RawValue {
    match cell {
        Data::Int(i) => RawValue::Number(*i as f64),
        Data::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                RawValue::Null
            } else {
                RawValue::Number(*f)
            }
        }
        Data::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty()
                || trimmed == "."
                || trimmed.eq_ignore_ascii_case("na")
                || trimmed.eq_ignore_ascii_case("nan")
                || trimmed.eq_ignore_ascii_case("null")
            {
                RawValue::Null
            } else if let Ok(n) = trimmed.parse::<f64>() {
                RawValue::Number(n)
            } else {
                RawValue::Text(s.clone())
            }
        }
        Data::Bool(b) => RawValue::Number(if *b { 1.0 } else { 0.0 }),
        Data::DateTime(dt) => RawValue::Text(dt.to_string()),
        Data::DateTimeIso(s) | Data::DurationIso(s) => RawValue::Text(s.clone()),
        Data::Error(_) | Data::Empty => RawValue::Null,
    }
}

pub fn sanitize_and_dedup_headers(raw_headers: &[String]) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::with_capacity(raw_headers.len());

    for (i, h) in raw_headers.iter().enumerate() {
        let trimmed = h.trim();
        let base = if trimmed.is_empty() {
            format!("VAR{:05}", i + 1)
        } else {
            let mut s: String = trimmed
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            while s.contains("__") {
                s = s.replace("__", "_");
            }
            let s = s.trim_matches('_').to_string();
            let mut s = if s.is_empty() {
                format!("VAR{:05}", i + 1)
            } else {
                s
            };
            if let Some(first) = s.chars().next() {
                if first.is_ascii_digit() {
                    s = format!("V_{}", s);
                }
            }
            s
        };

        let mut candidate = base.clone();
        let mut suffix = 1;
        while seen.contains(&candidate) {
            candidate = format!("{}_{}", base, suffix);
            suffix += 1;
        }
        seen.insert(candidate.clone());
        result.push(candidate);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_range() {
        let r: Range<Data> = Range::empty();
        let table = load_excel_from_range(r).unwrap();
        assert_eq!(table.row_count, 0);
        assert_eq!(table.col_count(), 0);
    }

    #[test]
    fn test_headers_only() {
        let mut r: Range<Data> = Range::new((0, 0), (0, 1));
        r.set_value((0, 0), Data::String("ID".to_string()));
        r.set_value((0, 1), Data::String("Salary".to_string()));

        let table = load_excel_from_range(r).unwrap();
        assert_eq!(table.row_count, 0);
        assert_eq!(table.col_count(), 2);
        assert_eq!(table.columns[0].name, "ID");
        assert_eq!(table.columns[1].name, "Salary");
    }

    #[test]
    fn test_header_sanitization_and_dedup() {
        let raw = vec![
            "   ".to_string(),
            "Total Price ($)".to_string(),
            "Total Price ($)".to_string(),
            "2024_Sales".to_string(),
            "valid_name".to_string(),
        ];
        let cleaned = sanitize_and_dedup_headers(&raw);
        assert_eq!(cleaned[0], "VAR00001");
        assert_eq!(cleaned[1], "Total_Price");
        assert_eq!(cleaned[2], "Total_Price_1");
        assert_eq!(cleaned[3], "V_2024_Sales");
        assert_eq!(cleaned[4], "valid_name");
    }

    #[test]
    fn test_mixed_types_and_missing() {
        let mut r: Range<Data> = Range::new((0, 0), (3, 2));
        // Headers
        r.set_value((0, 0), Data::String("num_col".to_string()));
        r.set_value((0, 1), Data::String("str_col".to_string()));
        r.set_value((0, 2), Data::String("bool_col".to_string()));

        // Row 1
        r.set_value((1, 0), Data::Int(10));
        r.set_value((1, 1), Data::String("Alice".to_string()));
        r.set_value((1, 2), Data::Bool(true));

        // Row 2
        r.set_value((2, 0), Data::Float(20.5));
        r.set_value((2, 1), Data::Empty);
        r.set_value((2, 2), Data::Bool(false));

        // Row 3
        r.set_value((3, 0), Data::String(".".to_string())); // Missing
        r.set_value((3, 1), Data::String("Bob".to_string()));
        r.set_value((3, 2), Data::Int(1));

        let table = load_excel_from_range(r).unwrap();
        assert_eq!(table.row_count, 3);
        assert_eq!(table.col_count(), 3);

        // Column 0: Numeric
        assert_eq!(table.columns[0].data_type, RawDataType::Numeric);
        assert_eq!(table.columns[0].values[0], RawValue::Number(10.0));
        assert_eq!(table.columns[0].values[1], RawValue::Number(20.5));
        assert_eq!(table.columns[0].values[2], RawValue::Null);

        // Column 1: Text
        assert_eq!(table.columns[1].data_type, RawDataType::Text);
        assert_eq!(
            table.columns[1].values[0],
            RawValue::Text("Alice".to_string())
        );
        assert_eq!(table.columns[1].values[1], RawValue::Null);
        assert_eq!(
            table.columns[1].values[2],
            RawValue::Text("Bob".to_string())
        );

        // Column 2: Numeric (bools convert to 1.0 / 0.0)
        assert_eq!(table.columns[2].data_type, RawDataType::Numeric);
        assert_eq!(table.columns[2].values[0], RawValue::Number(1.0));
        assert_eq!(table.columns[2].values[1], RawValue::Number(0.0));
        assert_eq!(table.columns[2].values[2], RawValue::Number(1.0));
    }

    #[test]
    fn test_column_with_text_and_number_casts_to_text() {
        let mut r: Range<Data> = Range::new((0, 0), (2, 0));
        r.set_value((0, 0), Data::String("code".to_string()));
        r.set_value((1, 0), Data::Int(100));
        r.set_value((2, 0), Data::String("ABC".to_string()));

        let table = load_excel_from_range(r).unwrap();
        assert_eq!(table.columns[0].data_type, RawDataType::Text);
        assert_eq!(
            table.columns[0].values[0],
            RawValue::Text("100".to_string())
        );
        assert_eq!(
            table.columns[0].values[1],
            RawValue::Text("ABC".to_string())
        );
    }
}
