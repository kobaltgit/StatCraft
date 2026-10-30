use crate::dataset::DataSet;
use crate::variable::CoreError;
use statcraft_io::{write_csv_row, CsvExportOptions, RawValue};
use std::io::Write;

pub fn export_dataset_to_csv<W: Write>(
    mut writer: W,
    dataset: &DataSet,
    options: &CsvExportOptions,
) -> Result<(), CoreError> {
    let col_count = dataset.table.col_count();
    if col_count == 0 {
        return Ok(());
    }

    if options.include_header {
        let headers: Vec<&str> = dataset
            .table
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        write_csv_row(&mut writer, &headers, options.delimiter)
            .map_err(|e| CoreError::TypeMismatch(format!("I/O write header error: {}", e)))?;
    }

    let mut row_buf = Vec::with_capacity(col_count);
    for row_idx in 0..dataset.table.row_count {
        row_buf.clear();
        for col_idx in 0..col_count {
            let meta = dataset.dictionary.get(col_idx);
            let val = dataset.table.get_value(row_idx, col_idx);

            let cell_text = match val {
                Some(RawValue::Number(n)) => {
                    let mut matched_label = None;
                    if options.export_labels {
                        if let Some(m) = meta {
                            let code = if n.fract() == 0.0 {
                                format!("{:.0}", n)
                            } else {
                                n.to_string()
                            };
                            if let Some(label) = m.values.get_label(&code) {
                                matched_label = Some(label.to_string());
                            }
                        }
                    }

                    if let Some(label) = matched_label {
                        label
                    } else {
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
                }
                Some(RawValue::Text(s)) => {
                    let mut matched_label = None;
                    if options.export_labels {
                        if let Some(m) = meta {
                            if let Some(label) = m.values.get_label(s) {
                                matched_label = Some(label.to_string());
                            }
                        }
                    }
                    matched_label.unwrap_or_else(|| s.clone())
                }
                Some(RawValue::Null) | None => options.sysmis_repr.clone(),
            };

            row_buf.push(cell_text);
        }

        let row_refs: Vec<&str> = row_buf.iter().map(|s| s.as_str()).collect();
        write_csv_row(&mut writer, &row_refs, options.delimiter)
            .map_err(|e| CoreError::TypeMismatch(format!("I/O write row error: {}", e)))?;
    }

    writer
        .flush()
        .map_err(|e| CoreError::TypeMismatch(format!("I/O flush error: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_dataset_labels_and_values() {
        let mut ds = DataSet::blank(2, 2).unwrap();
        // Setup var 0 (gender)
        ds.dictionary.rename(0, "gender").unwrap();
        ds.table.columns[0].name = "gender".to_string();
        let mut labels = crate::variable::ValueLabels::new();
        labels.insert("1", "Male");
        labels.insert("2", "Female");
        ds.dictionary.set_value_labels(0, labels).unwrap();

        // Setup var 1 (salary)
        ds.dictionary.rename(1, "salary").unwrap();
        ds.table.columns[1].name = "salary".to_string();

        // Populate table
        ds.table.columns[0].values[0] = RawValue::Number(1.0);
        ds.table.columns[1].values[0] = RawValue::Number(3500.50);

        ds.table.columns[0].values[1] = RawValue::Number(2.0);
        ds.table.columns[1].values[1] = RawValue::Number(4200.0);

        // 1. Export with labels
        let mut out_labels = Vec::new();
        let options_labels = CsvExportOptions {
            delimiter: b',',
            export_labels: true,
            include_header: true,
            decimal_separator: b'.',
            sysmis_repr: String::new(),
        };
        export_dataset_to_csv(&mut out_labels, &ds, &options_labels).unwrap();
        let csv_labels = String::from_utf8(out_labels).unwrap();
        let lines: Vec<&str> = csv_labels.lines().collect();

        assert_eq!(lines[0], "gender,salary");
        assert_eq!(lines[1], "Male,3500.5000");
        assert_eq!(lines[2], "Female,4200");

        // 2. Export numeric codes without labels
        let mut out_codes = Vec::new();
        let options_codes = CsvExportOptions {
            delimiter: b',',
            export_labels: false,
            include_header: true,
            decimal_separator: b'.',
            sysmis_repr: String::new(),
        };
        export_dataset_to_csv(&mut out_codes, &ds, &options_codes).unwrap();
        let csv_codes = String::from_utf8(out_codes).unwrap();
        let lines_codes: Vec<&str> = csv_codes.lines().collect();

        assert_eq!(lines_codes[0], "gender,salary");
        assert_eq!(lines_codes[1], "1,3500.5000");
        assert_eq!(lines_codes[2], "2,4200");
    }

    #[test]
    fn test_export_dataset_tsv() {
        let mut ds = DataSet::blank(1, 1).unwrap();
        ds.table.columns[0].name = "v1".to_string();
        ds.table.columns[0].values[0] = RawValue::Text("tab\there".to_string());

        let mut out = Vec::new();
        let options = CsvExportOptions {
            delimiter: b'\t',
            export_labels: false,
            include_header: true,
            decimal_separator: b'.',
            sysmis_repr: String::new(),
        };
        export_dataset_to_csv(&mut out, &ds, &options).unwrap();
        let tsv_str = String::from_utf8(out).unwrap();
        let lines: Vec<&str> = tsv_str.lines().collect();
        assert_eq!(lines, vec!["v1", "\"tab\there\""]);
    }

    #[test]
    fn test_export_empty_dataset() {
        let ds = DataSet::empty();
        let mut out = Vec::new();
        let options = CsvExportOptions::default();
        export_dataset_to_csv(&mut out, &ds, &options).unwrap();
        assert!(out.is_empty());
    }
}
