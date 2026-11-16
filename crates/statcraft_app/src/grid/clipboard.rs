use super::selection::CellRange;
use statcraft_core::DataSet;

/// Serializes the selected rectangular cell range into a standard TSV string for the clipboard.
pub fn serialize_range_to_tsv(ds: &DataSet, range: &CellRange, show_value_labels: bool) -> String {
    let mut out = String::new();
    let total_rows = ds.row_count();
    let total_cols = ds.col_count();

    for r in range.min_row..=range.max_row {
        if r >= total_rows {
            break;
        }
        for (i, c) in (range.min_col..=range.max_col).enumerate() {
            if i > 0 {
                out.push('\t');
            }
            if c < total_cols {
                let cell_text = ds.get_cell_display(r, c, show_value_labels);
                if cell_text.contains('\t') || cell_text.contains('\n') || cell_text.contains('"') {
                    out.push('"');
                    out.push_str(&cell_text.replace('"', "\"\""));
                    out.push('"');
                } else {
                    out.push_str(&cell_text);
                }
            }
        }
        out.push('\n');
    }

    out
}

/// Parses a tabular string (TSV or CSV) from the system clipboard into a 2D matrix of strings.
pub fn parse_tsv_matrix(text: &str) -> Vec<Vec<String>> {
    let trimmed = text.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut matrix = Vec::new();
    for line in trimmed.lines() {
        let clean_line = line.trim_end_matches('\r');
        let mut row = Vec::new();

        // Detect tab separator, fallback to comma if no tabs present
        let sep = if clean_line.contains('\t') { '\t' } else { ',' };

        let mut in_quotes = false;
        let mut field = String::new();
        let mut chars = clean_line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '"' {
                if in_quotes && chars.peek() == Some(&'"') {
                    // Escaped quote: ""
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            } else if ch == sep && !in_quotes {
                row.push(field.trim().to_string());
                field = String::new();
            } else {
                field.push(ch);
            }
        }
        row.push(field.trim().to_string());
        matrix.push(row);
    }

    matrix
}

/// Pastes a 2D matrix of string values into the dataset starting at `(start_row, start_col)`.
/// Automatically appends new rows if the matrix extends past the current row count.
/// Returns the bounding range of modified cells: `(rows_affected, cols_affected)`.
pub fn paste_matrix_into_dataset(
    ds: &mut DataSet,
    start_row: usize,
    start_col: usize,
    matrix: &[Vec<String>],
) -> (usize, usize) {
    if matrix.is_empty() || ds.col_count() == 0 {
        return (0, 0);
    }

    let needed_rows = start_row + matrix.len();
    while ds.row_count() < needed_rows {
        ds.add_empty_row();
    }

    let mut max_cols_affected = 0;
    let total_cols = ds.col_count();

    for (r_idx, row_values) in matrix.iter().enumerate() {
        let target_r = start_row + r_idx;
        max_cols_affected = max_cols_affected.max(row_values.len());

        for (c_idx, val) in row_values.iter().enumerate() {
            let target_c = start_col + c_idx;
            if target_c < total_cols {
                let _ = ds.set_cell_parsed(target_r, target_c, val);
            }
        }
    }

    let cols_count = max_cols_affected.min(total_cols.saturating_sub(start_col));
    (matrix.len(), cols_count)
}

/// Extends data series downwards from `source` range into `target` range (Auto-Fill Series).
pub fn apply_auto_fill(ds: &mut DataSet, source: &CellRange, target: &CellRange) {
    if ds.col_count() == 0 || source == target {
        return;
    }

    // Ensure enough rows
    while ds.row_count() <= target.max_row {
        ds.add_empty_row();
    }

    if target.max_row > source.max_row {
        let max_col = source.max_col.min(ds.col_count().saturating_sub(1));
        for c in source.min_col..=max_col {
            let mut source_values: Vec<String> = Vec::new();
            for r in source.min_row..=source.max_row {
                source_values.push(ds.get_cell_display(r, c, false));
            }
            if source_values.is_empty() {
                continue;
            }

            let numeric_vals: Vec<Option<f64>> = source_values
                .iter()
                .map(|s| s.parse::<f64>().ok())
                .collect();
            let all_numeric = numeric_vals.iter().all(|v| v.is_some());

            if all_numeric && source_values.len() >= 2 {
                let v0 = numeric_vals[0].unwrap_or(0.0);
                let v1 = numeric_vals[1].unwrap_or(0.0);
                let step = v1 - v0;
                let last_val = numeric_vals.last().copied().flatten().unwrap_or(0.0);
                let mut cur = last_val;

                for r in (source.max_row + 1)..=target.max_row {
                    cur += step;
                    let text = if cur.fract() == 0.0 {
                        format!("{:.0}", cur)
                    } else {
                        format!("{:.2}", cur)
                    };
                    let _ = ds.set_cell_parsed(r, c, &text);
                }
            } else if all_numeric && source_values.len() == 1 {
                let val_str = &source_values[0];
                for r in (source.max_row + 1)..=target.max_row {
                    let _ = ds.set_cell_parsed(r, c, val_str);
                }
            } else {
                let len = source_values.len();
                for (idx, r) in ((source.max_row + 1)..=target.max_row).enumerate() {
                    let val = &source_values[idx % len];
                    let _ = ds.set_cell_parsed(r, c, val);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::{DataType, Measure};

    #[test]
    fn test_tsv_parse_basic() {
        let raw = "10\t20\t30\n40\t50\t60";
        let matrix = parse_tsv_matrix(raw);
        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix[0], vec!["10", "20", "30"]);
        assert_eq!(matrix[1], vec!["40", "50", "60"]);
    }

    #[test]
    fn test_tsv_parse_with_quotes() {
        let raw = "1\t\"Hello, World\"\t\"She said \"\"Yes\"\"\"\n2\tSingle\tThree";
        let matrix = parse_tsv_matrix(raw);
        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix[0][1], "Hello, World");
        assert_eq!(matrix[0][2], "She said \"Yes\"");
        assert_eq!(matrix[1][0], "2");
    }

    #[test]
    fn test_serialize_and_paste_roundtrip() {
        let mut ds = DataSet::empty();
        let _ = ds.add_variable("A", DataType::Numeric { decimals: 0 }, Measure::Scale);
        let _ = ds.add_variable("B", DataType::Numeric { decimals: 0 }, Measure::Scale);

        ds.add_empty_row();
        ds.add_empty_row();
        let _ = ds.set_cell_parsed(0, 0, "100");
        let _ = ds.set_cell_parsed(0, 1, "200");
        let _ = ds.set_cell_parsed(1, 0, "300");
        let _ = ds.set_cell_parsed(1, 1, "400");

        let range = CellRange::from_corners(0, 0, 1, 1);
        let tsv = serialize_range_to_tsv(&ds, &range, false);
        assert_eq!(tsv.trim(), "100\t200\n300\t400");

        let parsed = parse_tsv_matrix(&tsv);
        assert_eq!(parsed.len(), 2);

        // Paste starting at row 2
        let (rows, cols) = paste_matrix_into_dataset(&mut ds, 2, 0, &parsed);
        assert_eq!(rows, 2);
        assert_eq!(cols, 2);
        assert_eq!(ds.row_count(), 4);
        assert_eq!(ds.get_cell_display(2, 0, false), "100");
        assert_eq!(ds.get_cell_display(3, 1, false), "400");
    }

    #[test]
    fn test_auto_fill_numeric_series() {
        let mut ds = DataSet::empty();
        let _ = ds.add_variable("NUM", DataType::Numeric { decimals: 0 }, Measure::Scale);
        ds.add_empty_row();
        ds.add_empty_row();
        let _ = ds.set_cell_parsed(0, 0, "1");
        let _ = ds.set_cell_parsed(1, 0, "2");

        let source = CellRange::from_corners(0, 0, 1, 0);
        let target = CellRange::from_corners(0, 0, 4, 0);

        apply_auto_fill(&mut ds, &source, &target);
        assert_eq!(ds.row_count(), 5);
        assert_eq!(ds.get_cell_display(2, 0, false), "3");
        assert_eq!(ds.get_cell_display(3, 0, false), "4");
        assert_eq!(ds.get_cell_display(4, 0, false), "5");
    }
}
