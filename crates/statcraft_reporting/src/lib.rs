//! Reporting module for StatCraft (Pivot tables, export).

pub mod html_export;
pub mod model;
pub mod tsv_export;
pub mod txt_export;

pub use html_export::{escape_html, export_document_to_html};
pub use model::{
    CellAlignment, ReportDocument, ReportItem, ReportNotes, ReportSection, ReportTable,
};
pub use tsv_export::export_document_to_tsv;
pub use txt_export::export_document_to_txt;
