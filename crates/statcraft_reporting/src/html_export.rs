use crate::model::{CellAlignment, ReportDocument, ReportItem, ReportNotes, ReportTable};

pub fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub fn export_document_to_html(doc: &ReportDocument) -> String {
    let mut html = String::with_capacity(4096);
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str(&format!(
        "<title>{}</title>\n",
        escape_html(if doc.title.is_empty() {
            "StatCraft Output Report"
        } else {
            &doc.title
        })
    ));
    html.push_str("<style>\n");
    html.push_str(
        r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    color: #1e293b;
    background-color: #f8fafc;
    margin: 0;
    padding: 24px;
}
.report-container {
    max-width: 1080px;
    margin: 0 auto;
    background: #ffffff;
    border: 1px solid #e2e8f0;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
    padding: 32px;
}
.report-header {
    border-bottom: 2px solid #2b4380;
    padding-bottom: 12px;
    margin-bottom: 28px;
}
.report-header h1 {
    margin: 0;
    color: #2b4380;
    font-size: 24px;
    font-weight: 700;
}
.section {
    margin-bottom: 36px;
}
.section-title {
    font-size: 18px;
    font-weight: 700;
    color: #1e3a8a;
    border-bottom: 1px solid #cbd5e1;
    padding-bottom: 6px;
    margin-top: 24px;
    margin-bottom: 16px;
}
.syntax-log {
    background-color: #f1f5f9;
    border-left: 4px solid #3b82f6;
    color: #0f172a;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 12px;
    line-height: 1.5;
    padding: 10px 14px;
    margin: 12px 0;
    white-space: pre-wrap;
    word-break: break-word;
    border-radius: 2px;
}
.warning-block {
    background-color: #fffbeb;
    border-left: 4px solid #f59e0b;
    color: #92400e;
    font-size: 13px;
    padding: 10px 14px;
    margin: 12px 0;
    border-radius: 2px;
}
.notes-table {
    border-collapse: collapse;
    width: auto;
    min-width: 420px;
    margin: 14px 0;
    font-size: 12px;
}
.notes-table th {
    background-color: #f1f5f9;
    color: #475569;
    font-weight: 600;
    text-align: left;
    padding: 6px 14px;
    border: 1px solid #cbd5e1;
    width: 160px;
}
.notes-table td {
    padding: 6px 14px;
    border: 1px solid #cbd5e1;
    color: #1e293b;
}
.spss-table-container {
    margin: 18px 0;
    overflow-x: auto;
}
.table-caption {
    font-size: 13px;
    font-weight: 700;
    color: #1e293b;
    margin-bottom: 6px;
}
.spss-table {
    border-collapse: collapse;
    width: 100%;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    background: #ffffff;
}
.spss-table thead th {
    border-top: 2px solid #000000;
    border-bottom: 1px solid #000000;
    background-color: #f8fafc;
    color: #0f172a;
    font-weight: 600;
    padding: 6px 12px;
    vertical-align: bottom;
}
.spss-table tbody tr {
    border-bottom: 1px solid #e2e8f0;
}
.spss-table tbody tr:nth-child(even) {
    background-color: #fbfcfd;
}
.spss-table tbody tr:last-child {
    border-bottom: 2px solid #000000;
}
.spss-table td {
    padding: 5px 12px;
    color: #334155;
}
.align-left { text-align: left; }
.align-center { text-align: center; }
.align-right { text-align: right; }
.footnotes {
    margin-top: 6px;
    font-size: 11px;
    color: #64748b;
    font-style: italic;
    line-height: 1.4;
}
"#,
    );
    html.push_str("</style>\n</head>\n<body>\n");
    html.push_str("<div class=\"report-container\">\n");
    html.push_str("<div class=\"report-header\">\n");
    html.push_str(&format!(
        "<h1>{}</h1>\n",
        escape_html(if doc.title.is_empty() {
            "StatCraft Output Report"
        } else {
            &doc.title
        })
    ));
    html.push_str("</div>\n");

    for section in &doc.sections {
        html.push_str("<div class=\"section\">\n");
        html.push_str(&format!(
            "<h2 class=\"section-title\">{}</h2>\n",
            escape_html(&section.title)
        ));

        for item in &section.items {
            match item {
                ReportItem::Log(sps) => {
                    html.push_str("<pre class=\"syntax-log\">");
                    html.push_str(&escape_html(sps));
                    html.push_str("</pre>\n");
                }
                ReportItem::Notes(notes) => {
                    render_html_notes(&mut html, notes);
                }
                ReportItem::Table(table) => {
                    render_html_table(&mut html, table);
                }
                ReportItem::Text { title, content } => {
                    html.push_str(&format!(
                        "<div class=\"custom-text\"><h3>{}</h3><p>{}</p></div>\n",
                        escape_html(title),
                        escape_html(content)
                    ));
                }
                ReportItem::Warning(msg) => {
                    html.push_str("<div class=\"warning-block\">");
                    html.push_str(&escape_html(msg));
                    html.push_str("</div>\n");
                }
            }
        }

        html.push_str("</div>\n");
    }

    html.push_str("</div>\n</body>\n</html>\n");
    html
}

fn render_html_notes(html: &mut String, notes: &ReportNotes) {
    html.push_str("<table class=\"notes-table\">\n<tbody>\n");
    let rows: [(&str, &str); 4] = [
        ("Output Created", &notes.timestamp),
        ("Active Dataset", &notes.dataset_name),
        ("N of Active Cases", &notes.active_cases.to_string()),
        ("N of Variables", &notes.active_vars.to_string()),
    ];
    for (label, val) in &rows {
        html.push_str("<tr>");
        html.push_str(&format!("<th>{}</th>", escape_html(label)));
        html.push_str(&format!("<td>{}</td>", escape_html(val)));
        html.push_str("</tr>\n");
    }
    html.push_str("</tbody>\n</table>\n");
}

fn render_html_table(html: &mut String, table: &ReportTable) {
    html.push_str("<div class=\"spss-table-container\">\n");
    if !table.title.is_empty() {
        html.push_str(&format!(
            "<div class=\"table-caption\">{}</div>\n",
            escape_html(&table.title)
        ));
    }
    html.push_str("<table class=\"spss-table\">\n");

    if !table.headers.is_empty() {
        html.push_str("<thead>\n<tr>\n");
        for (i, header) in table.headers.iter().enumerate() {
            let align_class = table
                .alignments
                .get(i)
                .map(|a| match a {
                    CellAlignment::Left => "align-left",
                    CellAlignment::Center => "align-center",
                    CellAlignment::Right => "align-right",
                })
                .unwrap_or("align-left");
            html.push_str(&format!(
                "<th class=\"{}\">{}</th>\n",
                align_class,
                escape_html(header)
            ));
        }
        html.push_str("</tr>\n</thead>\n");
    }

    html.push_str("<tbody>\n");
    for row in &table.rows {
        html.push_str("<tr>\n");
        for (col_idx, cell) in row.iter().enumerate() {
            let align_class = table
                .alignments
                .get(col_idx)
                .map(|a| match a {
                    CellAlignment::Left => "align-left",
                    CellAlignment::Center => "align-center",
                    CellAlignment::Right => "align-right",
                })
                .unwrap_or("align-left");
            html.push_str(&format!(
                "<td class=\"{}\">{}</td>\n",
                align_class,
                escape_html(cell)
            ));
        }
        html.push_str("</tr>\n");
    }
    html.push_str("</tbody>\n");

    if !table.footnotes.is_empty() {
        html.push_str("<tfoot>\n<tr>\n");
        let colspan = table.headers.len().max(1);
        html.push_str(&format!(
            "<td colspan=\"{}\"><div class=\"footnotes\">\n",
            colspan
        ));
        for note in &table.footnotes {
            html.push_str(&format!("<div>{}</div>\n", escape_html(note)));
        }
        html.push_str("</div></td>\n</tr>\n</tfoot>\n");
    }

    html.push_str("</table>\n</div>\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ReportSection;

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<script>alert(\"xss & 'test'\")</script>"),
            "&lt;script&gt;alert(&quot;xss &amp; &#39;test&#39;&quot;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_export_document_to_html_contains_table_structure() {
        let mut doc = ReportDocument::new("Analysis 1");
        let mut sec = ReportSection::new("Descriptives");
        sec.add_item(ReportItem::Log("DESCRIPTIVES VAR=salary.".to_string()));

        let mut table = ReportTable::new("Descriptive Statistics");
        table.headers = vec!["Variable".to_string(), "N".to_string(), "Mean".to_string()];
        table.alignments = vec![
            CellAlignment::Left,
            CellAlignment::Right,
            CellAlignment::Right,
        ];
        table.rows = vec![vec![
            "salary".to_string(),
            "474".to_string(),
            "34419.57".to_string(),
        ]];
        table.footnotes = vec!["Valid N (listwise) = 474".to_string()];
        sec.add_item(ReportItem::Table(table));

        doc.add_section(sec);

        let html = export_document_to_html(&doc);
        assert!(html.contains("<table class=\"spss-table\">"));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<tfoot>"));
        assert!(html.contains("34419.57"));
        assert!(html.contains("Valid N (listwise) = 474"));
        assert!(html.contains("<pre class=\"syntax-log\">DESCRIPTIVES VAR=salary.</pre>"));
    }

    #[test]
    fn test_export_empty_document_to_html() {
        let doc = ReportDocument::default();
        let html = export_document_to_html(&doc);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("StatCraft Output Report"));
    }
}
