use crate::model::{ReportDocument, ReportItem};

pub fn export_document_to_tsv(doc: &ReportDocument) -> String {
    let mut out = String::with_capacity(4096);

    for section in &doc.sections {
        for item in &section.items {
            if let ReportItem::Table(table) = item {
                if !out.is_empty() {
                    out.push_str("\n\n");
                }

                if !table.title.is_empty() {
                    out.push_str(&table.title);
                    out.push('\n');
                }

                if !table.headers.is_empty() {
                    out.push_str(&table.headers.join("\t"));
                    out.push('\n');
                }

                for row in &table.rows {
                    out.push_str(&row.join("\t"));
                    out.push('\n');
                }

                for note in &table.footnotes {
                    out.push_str(note);
                    out.push('\n');
                }
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ReportSection, ReportTable};

    #[test]
    fn test_export_document_to_tsv() {
        let mut doc = ReportDocument::new("Analysis");
        let mut sec = ReportSection::new("Descriptives");

        let mut table1 = ReportTable::new("Table 1");
        table1.headers = vec!["ColA".to_string(), "ColB".to_string()];
        table1.rows = vec![vec!["1".to_string(), "2".to_string()]];
        sec.add_item(ReportItem::Table(table1));

        let mut table2 = ReportTable::new("Table 2");
        table2.headers = vec!["ColX".to_string(), "ColY".to_string()];
        table2.rows = vec![vec!["3".to_string(), "4".to_string()]];
        sec.add_item(ReportItem::Table(table2));

        doc.add_section(sec);

        let tsv = export_document_to_tsv(&doc);
        assert!(tsv.contains("Table 1\nColA\tColB\n1\t2"));
        assert!(tsv.contains("Table 2\nColX\tColY\n3\t4"));
    }
}
