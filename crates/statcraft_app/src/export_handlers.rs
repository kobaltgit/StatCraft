use crate::app::StatCraftApp;
use crate::status_message::StatusMessage;
use crate::views::output_view::{
    export_output_to_html, export_output_to_tsv, export_output_to_txt, OutputViewerState,
};
use rfd::FileDialog;
use statcraft_core::{export_dataset_to_csv, CsvExportOptions, DataSet};
use statcraft_i18n::t;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn determine_csv_options_for_path(path: &Path) -> CsvExportOptions {
    let mut options = CsvExportOptions::default();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let lower = ext.to_lowercase();
        if lower == "tsv" || lower == "txt" {
            options.delimiter = b'\t';
        }
    }
    options
}

pub fn export_dataset_file(
    dataset: &DataSet,
    path: &Path,
    options: &CsvExportOptions,
) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("Cannot create file: {}", e))?;
    let writer = BufWriter::new(file);
    export_dataset_to_csv(writer, dataset, options)
        .map_err(|e| format!("CSV export failed: {}", e))?;
    Ok(())
}

pub fn generate_export_output_content(
    output_state: &OutputViewerState,
    path: &Path,
) -> (String, &'static str) {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "html".to_string());

    match ext.as_str() {
        "txt" => (export_output_to_txt(output_state), "TEXT"),
        "tsv" => (export_output_to_tsv(output_state), "TAB"),
        _ => (export_output_to_html(output_state), "HTML"),
    }
}

pub fn export_output_file(output_state: &OutputViewerState, path: &Path) -> Result<String, String> {
    let (content, format_type) = generate_export_output_content(output_state, path);
    let mut file = File::create(path).map_err(|e| format!("Cannot create file: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Cannot write to file: {}", e))?;
    file.flush()
        .map_err(|e| format!("Cannot flush file: {}", e))?;
    Ok(format_type.to_string())
}

impl StatCraftApp {
    pub fn handle_export_data(&mut self) {
        let Some(dataset) = &self.dataset else {
            return;
        };

        let dialog = FileDialog::new()
            .set_title(t!("dialog-export-data-title"))
            .add_filter(t!("filter-csv-comma"), &["csv"])
            .add_filter(t!("filter-tsv"), &["tsv", "txt"])
            .add_filter(t!("filter-csv-semicolon"), &["csv"]);

        if let Some(path) = dialog.save_file() {
            let options = determine_csv_options_for_path(&path);
            let path_str = path.display().to_string();

            match export_dataset_file(dataset, &path, &options) {
                Ok(()) => {
                    self.status_message = StatusMessage::DataExported {
                        path: path_str.clone(),
                    };
                    let type_str = if options.delimiter == b'\t' {
                        "TAB"
                    } else if options.delimiter == b';' {
                        "CSV (SEMICOLON)"
                    } else {
                        "CSV"
                    };
                    let syntax = format!(
                        "SAVE TRANSLATE OUTFILE='{}'\n  /TYPE={}\n  /REPLACE\n  /FIELDNAMES.",
                        path_str, type_str
                    );
                    self.output_view_state.add_log(syntax);
                }
                Err(err) => {
                    self.status_message = StatusMessage::Error(err);
                }
            }
        }
    }

    pub fn handle_export_output(&mut self) {
        let dialog = FileDialog::new()
            .set_title(t!("dialog-export-output-title"))
            .add_filter(t!("filter-html"), &["html", "htm"])
            .add_filter(t!("filter-txt"), &["txt"])
            .add_filter(t!("filter-tsv-bundle"), &["tsv"]);

        if let Some(path) = dialog.save_file() {
            let path_str = path.display().to_string();

            match export_output_file(&self.output_view_state, &path) {
                Ok(fmt) => {
                    self.status_message = StatusMessage::OutputExported {
                        path: path_str.clone(),
                    };
                    let syntax = format!(
                        "OUTPUT EXPORT\n  /CONTENTS=ALL\n  /TYPE={}\n  /DESTINATION='{}'.",
                        fmt, path_str
                    );
                    self.output_view_state.add_log(syntax);
                }
                Err(err) => {
                    self.status_message = StatusMessage::Error(err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_determine_csv_options_extension() {
        let p_csv = PathBuf::from("my_data.csv");
        assert_eq!(determine_csv_options_for_path(&p_csv).delimiter, b',');

        let p_tsv = PathBuf::from("my_data.tsv");
        assert_eq!(determine_csv_options_for_path(&p_tsv).delimiter, b'\t');

        let p_txt = PathBuf::from("my_data.txt");
        assert_eq!(determine_csv_options_for_path(&p_txt).delimiter, b'\t');
    }

    #[test]
    fn test_generate_export_output_content_formats() {
        let mut state = OutputViewerState::new();
        state.add_log("TEST LOG.");

        let (html, fmt_html) =
            generate_export_output_content(&state, &PathBuf::from("report.html"));
        assert_eq!(fmt_html, "HTML");
        assert!(html.contains("<!DOCTYPE html>"));

        let (txt, fmt_txt) = generate_export_output_content(&state, &PathBuf::from("report.txt"));
        assert_eq!(fmt_txt, "TEXT");
        assert!(txt.contains("TEST LOG."));

        let (tsv, fmt_tsv) = generate_export_output_content(&state, &PathBuf::from("report.tsv"));
        assert_eq!(fmt_tsv, "TAB");
        assert!(tsv.is_empty() || tsv.contains("TEST"));
    }
}
