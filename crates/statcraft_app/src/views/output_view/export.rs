use super::extract_item_tsv;
use super::model::{OutputItemKind, OutputViewerState};
use statcraft_reporting::{
    export_document_to_html, export_document_to_tsv, export_document_to_txt, CellAlignment,
    ReportDocument, ReportItem, ReportNotes, ReportSection, ReportTable,
};

pub fn output_viewer_to_report_document(state: &OutputViewerState) -> ReportDocument {
    let mut doc = ReportDocument::new("StatCraft Output Report");

    for proc in &state.procedures {
        let mut section = ReportSection::new(proc.title());

        for item in &proc.items {
            match &item.kind {
                OutputItemKind::Log(sps) => {
                    section.add_item(ReportItem::Log(sps.clone()));
                }
                OutputItemKind::Notes(notes) => {
                    section.add_item(ReportItem::Notes(ReportNotes {
                        dataset_name: notes.dataset_name.clone(),
                        timestamp: notes.timestamp.clone(),
                        active_cases: notes.active_cases,
                        active_vars: notes.active_vars,
                    }));
                }
                OutputItemKind::Warning(msg) => {
                    section.add_item(ReportItem::Warning(msg.clone()));
                }
                OutputItemKind::CustomText { title, content } => {
                    section.add_item(ReportItem::Text {
                        title: title.clone(),
                        content: content.clone(),
                    });
                }
                OutputItemKind::DatasetSummary { name, rows, cols } => {
                    let mut table = ReportTable::new(item.title());
                    table.headers = vec!["Property".to_string(), "Value".to_string()];
                    table.alignments = vec![CellAlignment::Left, CellAlignment::Left];
                    table.rows = vec![
                        vec!["Active Dataset".to_string(), name.clone()],
                        vec!["Row Count".to_string(), rows.to_string()],
                        vec!["Column Count".to_string(), cols.to_string()],
                    ];
                    section.add_item(ReportItem::Table(table));
                }
                _ => {
                    if let Some(tsv) = extract_item_tsv(&item.kind) {
                        let lines: Vec<&str> = tsv.lines().collect();
                        if !lines.is_empty() {
                            let mut headers = Vec::new();
                            let mut data_rows = Vec::new();
                            let mut footnotes = Vec::new();

                            let mut iter = lines.into_iter();
                            if let Some(first_line) = iter.next() {
                                headers = first_line.split('\t').map(String::from).collect();
                            }

                            for line in iter {
                                let trimmed = line.trim();
                                if trimmed.starts_with("Valid N")
                                    || trimmed.starts_with('*')
                                    || trimmed.starts_with("a.")
                                    || trimmed.starts_with("b.")
                                {
                                    footnotes.push(trimmed.to_string());
                                } else if !trimmed.is_empty() {
                                    let cells: Vec<String> =
                                        line.split('\t').map(String::from).collect();
                                    data_rows.push(cells);
                                }
                            }

                            let alignments = headers
                                .iter()
                                .map(|h| {
                                    let lower = h.to_lowercase();
                                    if lower.contains("variable")
                                        || lower.contains("model")
                                        || lower.contains("source")
                                        || lower.contains("name")
                                        || lower.contains("parameter")
                                    {
                                        CellAlignment::Left
                                    } else {
                                        CellAlignment::Right
                                    }
                                })
                                .collect();

                            let table = ReportTable {
                                title: item.title(),
                                headers,
                                rows: data_rows,
                                alignments,
                                footnotes,
                            };
                            section.add_item(ReportItem::Table(table));
                        }
                    }
                }
            }
        }

        doc.add_section(section);
    }

    doc
}

pub fn export_output_to_html(state: &OutputViewerState) -> String {
    let doc = output_viewer_to_report_document(state);
    export_document_to_html(&doc)
}

pub fn export_output_to_txt(state: &OutputViewerState) -> String {
    let doc = output_viewer_to_report_document(state);
    export_document_to_txt(&doc)
}

pub fn export_output_to_tsv(state: &OutputViewerState) -> String {
    let doc = output_viewer_to_report_document(state);
    export_document_to_tsv(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::output_view::model::{OutputItem, OutputNotes, OutputProcedure};

    #[test]
    fn test_export_output_state_to_html() {
        let mut state = OutputViewerState::new();
        let proc = OutputProcedure {
            id: 1,
            title_key: "output-log",
            custom_title: Some("Frequencies Analysis".to_string()),
            is_expanded: true,
            items: vec![
                OutputItem {
                    id: 2,
                    title_key: "output-notes",
                    custom_title: None,
                    kind: OutputItemKind::Notes(OutputNotes {
                        dataset_name: "Employee data".to_string(),
                        timestamp: "2026-10-29 12:00:00".to_string(),
                        active_cases: 474,
                        active_vars: 10,
                    }),
                },
                OutputItem {
                    id: 3,
                    title_key: "output-log",
                    custom_title: None,
                    kind: OutputItemKind::Log("FREQUENCIES VARIABLES=gender.".to_string()),
                },
            ],
        };
        state.procedures.push(proc);

        let html = export_output_to_html(&state);
        assert!(html.contains("Frequencies Analysis"));
        assert!(html.contains("Employee data"));
        assert!(html.contains("FREQUENCIES VARIABLES=gender."));
    }

    #[test]
    fn test_export_output_state_to_txt() {
        let mut state = OutputViewerState::new();
        state.add_log("DESCRIPTIVES VAR=salary.");

        let txt = export_output_to_txt(&state);
        assert!(txt.contains("LOG / SYNTAX:"));
        assert!(txt.contains("DESCRIPTIVES VAR=salary."));
    }

    #[test]
    fn test_export_empty_output_state() {
        let state = OutputViewerState::new();
        let html = export_output_to_html(&state);
        assert!(html.contains("StatCraft Output Report"));
    }
}
