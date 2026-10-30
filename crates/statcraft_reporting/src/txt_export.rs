use crate::model::{CellAlignment, ReportDocument, ReportItem, ReportNotes, ReportTable};

pub fn export_document_to_txt(doc: &ReportDocument) -> String {
    let mut out = String::with_capacity(4096);
    let title = if doc.title.is_empty() {
        "StatCraft Output Report"
    } else {
        &doc.title
    };

    out.push_str(
        "================================================================================\n",
    );
    out.push_str(&format!("  {}\n", title));
    out.push_str(
        "================================================================================\n\n",
    );

    for section in &doc.sections {
        out.push_str(&format!("* {}\n", section.title));
        out.push_str(&format!(
            "{}\n\n",
            "-".repeat(section.title.chars().count() + 2)
        ));

        for item in &section.items {
            match item {
                ReportItem::Log(sps) => {
                    out.push_str("LOG / SYNTAX:\n");
                    for line in sps.lines() {
                        out.push_str(&format!("  > {}\n", line));
                    }
                    out.push('\n');
                }
                ReportItem::Notes(notes) => {
                    render_txt_notes(&mut out, notes);
                    out.push('\n');
                }
                ReportItem::Table(table) => {
                    render_txt_table(&mut out, table);
                    out.push('\n');
                }
                ReportItem::Text { title, content } => {
                    out.push_str(&format!("{}:\n{}\n\n", title, content));
                }
                ReportItem::Warning(msg) => {
                    out.push_str(&format!("WARNING: {}\n\n", msg));
                }
            }
        }
        out.push('\n');
    }

    out
}

fn render_txt_notes(out: &mut String, notes: &ReportNotes) {
    let rows: [(&str, &str); 4] = [
        ("Output Created", &notes.timestamp),
        ("Active Dataset", &notes.dataset_name),
        ("N of Active Cases", &notes.active_cases.to_string()),
        ("N of Variables", &notes.active_vars.to_string()),
    ];

    let mut table = ReportTable::new("Notes");
    table.headers = vec!["Note".to_string(), "Value".to_string()];
    table.rows = rows
        .iter()
        .map(|(k, v)| vec![k.to_string(), v.to_string()])
        .collect();
    table.alignments = vec![CellAlignment::Left, CellAlignment::Left];
    render_txt_table(out, &table);
}

fn render_txt_table(out: &mut String, table: &ReportTable) {
    if !table.title.is_empty() {
        out.push_str(&format!("[{}]\n", table.title));
    }

    let num_cols = table
        .headers
        .len()
        .max(table.rows.iter().map(|r| r.len()).max().unwrap_or(0));

    if num_cols == 0 {
        return;
    }

    let mut col_widths = vec![0usize; num_cols];
    for (i, h) in table.headers.iter().enumerate() {
        col_widths[i] = col_widths[i].max(h.chars().count());
    }
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(cell.chars().count());
            }
        }
    }

    for w in &mut col_widths {
        *w = (*w).max(3) + 2;
    }

    let make_border = |sep: char| -> String {
        let mut b = String::new();
        b.push('+');
        for w in &col_widths {
            b.push_str(&sep.to_string().repeat(*w));
            b.push('+');
        }
        b.push('\n');
        b
    };

    let border_line = make_border('-');
    let double_border = make_border('=');

    out.push_str(&double_border);

    if !table.headers.is_empty() {
        out.push('|');
        for (i, h) in table.headers.iter().enumerate() {
            let width = col_widths.get(i).copied().unwrap_or(10);
            let align = table
                .alignments
                .get(i)
                .copied()
                .unwrap_or(CellAlignment::Left);
            out.push_str(&format_cell(h, width, align));
            out.push('|');
        }
        out.push('\n');
        out.push_str(&double_border);
    }

    for row in &table.rows {
        out.push('|');
        for i in 0..num_cols {
            let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
            let width = col_widths.get(i).copied().unwrap_or(10);
            let align = table
                .alignments
                .get(i)
                .copied()
                .unwrap_or(CellAlignment::Left);
            out.push_str(&format_cell(cell, width, align));
            out.push('|');
        }
        out.push('\n');
    }

    out.push_str(&border_line);

    for note in &table.footnotes {
        out.push_str(&format!("  * {}\n", note));
    }
}

fn format_cell(text: &str, width: usize, align: CellAlignment) -> String {
    let char_count = text.chars().count();
    let total_pad = width.saturating_sub(char_count);

    match align {
        CellAlignment::Left => {
            format!(" {}{} ", text, " ".repeat(total_pad.saturating_sub(2)))
        }
        CellAlignment::Right => {
            format!(" {}{} ", " ".repeat(total_pad.saturating_sub(2)), text)
        }
        CellAlignment::Center => {
            let pad_left = total_pad / 2;
            let pad_right = total_pad - pad_left;
            format!("{}{}{}", " ".repeat(pad_left), text, " ".repeat(pad_right))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ReportSection;

    #[test]
    fn test_export_document_to_txt() {
        let mut doc = ReportDocument::new("Test Report");
        let mut sec = ReportSection::new("Descriptives");
        sec.add_item(ReportItem::Log("DESCRIPTIVES VAR=age.".to_string()));

        let mut table = ReportTable::new("Age Summary");
        table.headers = vec!["Variable".to_string(), "Mean".to_string()];
        table.rows = vec![vec!["age".to_string(), "28.5".to_string()]];
        sec.add_item(ReportItem::Table(table));
        doc.add_section(sec);

        let txt = export_document_to_txt(&doc);
        assert!(txt.contains("Test Report"));
        assert!(txt.contains("LOG / SYNTAX:"));
        assert!(txt.contains("DESCRIPTIVES VAR=age."));
        assert!(txt.contains("[Age Summary]"));
        assert!(txt.contains("| Variable"));
        assert!(txt.contains("| age"));
    }
}
