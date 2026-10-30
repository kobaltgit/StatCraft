use super::csv_loader::{IoError, RawTable, RawValue};
use std::io::Write;

#[derive(Clone, Debug, PartialEq)]
pub struct CsvExportOptions {
    pub delimiter: u8,
    pub export_labels: bool,
    pub include_header: bool,
    pub decimal_separator: u8,
    pub sysmis_repr: String,
}

impl Default for CsvExportOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            export_labels: false,
            include_header: true,
            decimal_separator: b'.',
            sysmis_repr: String::new(),
        }
    }
}

pub fn escape_csv_field(val: &str, delimiter: u8) -> String {
    let delim_char = delimiter as char;
    let needs_quotes =
        val.contains(delim_char) || val.contains('"') || val.contains('\n') || val.contains('\r');

    if needs_quotes {
        let mut escaped = String::with_capacity(val.len() + 8);
        escaped.push('"');
        for ch in val.chars() {
            if ch == '"' {
                escaped.push('"');
                escaped.push('"');
            } else {
                escaped.push(ch);
            }
        }
        escaped.push('"');
        escaped
    } else {
        val.to_string()
    }
}

pub fn write_csv_row<W: Write>(
    writer: &mut W,
    fields: &[&str],
    delimiter: u8,
) -> Result<(), std::io::Error> {
    let delim_char = delimiter as char;
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(writer, "{}", delim_char)?;
        }
        let escaped = escape_csv_field(field, delimiter);
        write!(writer, "{}", escaped)?;
    }
    writeln!(writer)?;
    Ok(())
}

pub fn export_raw_table_to_csv<W: Write>(
    mut writer: W,
    table: &RawTable,
    options: &CsvExportOptions,
) -> Result<(), IoError> {
    let col_count = table.col_count();
    if col_count == 0 {
        return Ok(());
    }

    if options.include_header {
        let headers: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
        write_csv_row(&mut writer, &headers, options.delimiter)
            .map_err(|e| IoError::Io(e.to_string()))?;
    }

    let mut row_buf = Vec::with_capacity(col_count);
    for row_idx in 0..table.row_count {
        row_buf.clear();
        for col_idx in 0..col_count {
            let val_str = match table.get_value(row_idx, col_idx) {
                Some(RawValue::Number(n)) => {
                    let formatted = if n.fract() == 0.0 {
                        format!("{:.0}", n)
                    } else {
                        format!("{:.4}", n)
                    };
                    if options.decimal_separator == b',' {
                        formatted.replace('.', ",")
                    } else {
                        formatted
                    }
                }
                Some(RawValue::Text(s)) => s.clone(),
                Some(RawValue::Null) | None => options.sysmis_repr.clone(),
            };
            row_buf.push(val_str);
        }

        let row_refs: Vec<&str> = row_buf.iter().map(|s| s.as_str()).collect();
        write_csv_row(&mut writer, &row_refs, options.delimiter)
            .map_err(|e| IoError::Io(e.to_string()))?;
    }

    writer.flush().map_err(|e| IoError::Io(e.to_string()))?;
    Ok(())
}

pub fn export_raw_table_to_csv_path<P: AsRef<std::path::Path>>(
    path: P,
    table: &RawTable,
    options: &CsvExportOptions,
) -> Result<(), IoError> {
    let file = std::fs::File::create(path.as_ref())
        .map_err(|e| IoError::Io(format!("Cannot create file: {}", e)))?;
    let writer = std::io::BufWriter::new(file);
    export_raw_table_to_csv(writer, table, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_loader::{RawColumn, RawDataType};

    #[test]
    fn test_escape_csv_field() {
        assert_eq!(escape_csv_field("simple", b','), "simple");
        assert_eq!(escape_csv_field("has,comma", b','), "\"has,comma\"");
        assert_eq!(escape_csv_field("has,comma", b';'), "has,comma");
        assert_eq!(escape_csv_field("has;semi", b';'), "\"has;semi\"");
        assert_eq!(escape_csv_field("has\ttab", b'\t'), "\"has\ttab\"");
        assert_eq!(
            escape_csv_field("say \"hello\"", b','),
            "\"say \"\"hello\"\"\""
        );
        assert_eq!(escape_csv_field("multi\nline", b','), "\"multi\nline\"");
    }

    #[test]
    fn test_export_raw_table_comma() {
        let col1 = RawColumn {
            name: "id".to_string(),
            data_type: RawDataType::Numeric,
            values: vec![RawValue::Number(1.0), RawValue::Number(2.0)],
        };
        let col2 = RawColumn {
            name: "city, state".to_string(),
            data_type: RawDataType::Text,
            values: vec![RawValue::Text("New York, NY".to_string()), RawValue::Null],
        };
        let table = RawTable {
            columns: vec![col1, col2],
            row_count: 2,
        };

        let mut output = Vec::new();
        let options = CsvExportOptions {
            delimiter: b',',
            export_labels: false,
            include_header: true,
            decimal_separator: b'.',
            sysmis_repr: "NA".to_string(),
        };

        export_raw_table_to_csv(&mut output, &table, &options).unwrap();
        let csv_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv_str.lines().collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "id,\"city, state\"");
        assert_eq!(lines[1], "1,\"New York, NY\"");
        assert_eq!(lines[2], "2,NA");
    }

    #[test]
    fn test_export_raw_table_semicolon_and_decimal_comma() {
        let col1 = RawColumn {
            name: "score".to_string(),
            data_type: RawDataType::Numeric,
            values: vec![RawValue::Number(3.25), RawValue::Number(42.0)],
        };
        let table = RawTable {
            columns: vec![col1],
            row_count: 2,
        };

        let mut output = Vec::new();
        let options = CsvExportOptions {
            delimiter: b';',
            export_labels: false,
            include_header: true,
            decimal_separator: b',',
            sysmis_repr: String::new(),
        };

        export_raw_table_to_csv(&mut output, &table, &options).unwrap();
        let csv_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv_str.lines().collect();

        assert_eq!(lines[0], "score");
        assert_eq!(lines[1], "3,2500");
        assert_eq!(lines[2], "42");
    }

    #[test]
    fn test_export_empty_raw_table() {
        let table = RawTable::empty();
        let mut output = Vec::new();
        let options = CsvExportOptions::default();
        export_raw_table_to_csv(&mut output, &table, &options).unwrap();
        assert!(output.is_empty());
    }
}
