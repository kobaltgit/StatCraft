use crate::csv_loader::{IoError, RawTable};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub const STC_FORMAT_VERSION: u32 = 1;
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StcDatasetRecord {
    pub dataset_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_path: Option<String>,
    pub metadata_json: String,
    pub table: RawTable,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_filter_var: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight_var: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub split_file_json: Option<String>,
    /// Persistent dataset tab color hex (e.g. "#2E86AB")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StcContainer {
    pub format_version: u32,
    pub dataset_name: String,
    pub metadata_json: String,
    pub table: RawTable,
    pub active_filter_var: Option<String>,
    pub weight_var: Option<String>,
    pub split_file_json: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspace_datasets: Vec<StcDatasetRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_dataset_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_viewer_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub syntax_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_color: Option<String>,
}

impl StcContainer {
    pub fn new(
        dataset_name: impl Into<String>,
        metadata_json: impl Into<String>,
        table: RawTable,
        active_filter_var: Option<String>,
        weight_var: Option<String>,
        split_file_json: Option<String>,
    ) -> Self {
        Self {
            format_version: STC_FORMAT_VERSION,
            dataset_name: dataset_name.into(),
            metadata_json: metadata_json.into(),
            table,
            active_filter_var,
            weight_var,
            split_file_json,
            workspace_datasets: Vec::new(),
            active_dataset_index: None,
            output_viewer_json: None,
            syntax_text: None,
            tab_color: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_project_data(
        dataset_name: impl Into<String>,
        metadata_json: impl Into<String>,
        table: RawTable,
        active_filter_var: Option<String>,
        weight_var: Option<String>,
        split_file_json: Option<String>,
        output_viewer_json: Option<String>,
        syntax_text: Option<String>,
    ) -> Self {
        Self {
            format_version: STC_FORMAT_VERSION,
            dataset_name: dataset_name.into(),
            metadata_json: metadata_json.into(),
            table,
            active_filter_var,
            weight_var,
            split_file_json,
            workspace_datasets: Vec::new(),
            active_dataset_index: None,
            output_viewer_json,
            syntax_text,
            tab_color: None,
        }
    }

    pub fn with_workspace(
        workspace_datasets: Vec<StcDatasetRecord>,
        active_dataset_index: Option<usize>,
        output_viewer_json: Option<String>,
        syntax_text: Option<String>,
    ) -> Self {
        let (
            dataset_name,
            metadata_json,
            table,
            active_filter_var,
            weight_var,
            split_file_json,
            tab_color,
        ) = if let Some(first) = active_dataset_index
            .and_then(|idx| workspace_datasets.get(idx))
            .or_else(|| workspace_datasets.first())
        {
            (
                first.dataset_name.clone(),
                first.metadata_json.clone(),
                first.table.clone(),
                first.active_filter_var.clone(),
                first.weight_var.clone(),
                first.split_file_json.clone(),
                first.tab_color.clone(),
            )
        } else {
            (
                "DataSet1".to_string(),
                r#"{"variables":[]}"#.to_string(),
                RawTable {
                    columns: vec![],
                    row_count: 0,
                },
                None,
                None,
                None,
                None,
            )
        };

        Self {
            format_version: STC_FORMAT_VERSION,
            dataset_name,
            metadata_json,
            table,
            active_filter_var,
            weight_var,
            split_file_json,
            workspace_datasets,
            active_dataset_index,
            output_viewer_json,
            syntax_text,
            tab_color,
        }
    }

    pub fn extract_datasets(&self) -> Vec<StcDatasetRecord> {
        if !self.workspace_datasets.is_empty() {
            self.workspace_datasets.clone()
        } else {
            vec![StcDatasetRecord {
                dataset_name: self.dataset_name.clone(),
                original_path: None,
                metadata_json: self.metadata_json.clone(),
                table: self.table.clone(),
                active_filter_var: self.active_filter_var.clone(),
                weight_var: self.weight_var.clone(),
                split_file_json: self.split_file_json.clone(),
                tab_color: self.tab_color.clone(),
            }]
        }
    }
}

pub fn save_stc_container<W: Write>(writer: W, container: &StcContainer) -> Result<(), IoError> {
    let mut encoder = GzEncoder::new(writer, Compression::default());
    serde_json::to_writer(&mut encoder, container)
        .map_err(|e| IoError::Serialization(e.to_string()))?;
    encoder.finish().map_err(|e| IoError::Io(e.to_string()))?;
    Ok(())
}

pub fn load_stc_container<R: Read>(mut reader: R) -> Result<StcContainer, IoError> {
    let mut magic = [0u8; 2];
    let n = reader
        .read(&mut magic)
        .map_err(|e| IoError::Io(e.to_string()))?;
    if n == 0 {
        return Err(IoError::EmptyFile("File is empty".to_string()));
    }

    let chained = std::io::Cursor::new(&magic[..n]).chain(reader);

    if n >= 2 && magic == GZIP_MAGIC {
        let gz = GzDecoder::new(chained);
        serde_json::from_reader(gz).map_err(|e| IoError::Deserialization(e.to_string()))
    } else {
        serde_json::from_reader(chained).map_err(|e| IoError::Deserialization(e.to_string()))
    }
}

pub fn save_stc_container_to_path<P: AsRef<Path>>(
    path: P,
    container: &StcContainer,
) -> Result<(), IoError> {
    let file = File::create(path).map_err(|e| IoError::Io(e.to_string()))?;
    let writer = BufWriter::new(file);
    save_stc_container(writer, container)
}

pub fn load_stc_container_from_path<P: AsRef<Path>>(path: P) -> Result<StcContainer, IoError> {
    let file = File::open(path).map_err(|e| IoError::Io(e.to_string()))?;
    let reader = BufReader::new(file);
    load_stc_container(reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_loader::{RawColumn, RawDataType, RawValue};

    #[test]
    fn test_stc_container_roundtrip() {
        let table = RawTable {
            columns: vec![RawColumn {
                name: "score".to_string(),
                data_type: RawDataType::Numeric,
                values: vec![RawValue::Number(95.5), RawValue::Null],
            }],
            row_count: 2,
        };

        let container = StcContainer::new(
            "TestSet",
            r#"{"variables":[]}"#,
            table,
            Some("filter_var".to_string()),
            Some("weight_var".to_string()),
            None,
        );

        let mut buf = Vec::new();
        save_stc_container(&mut buf, &container).unwrap();
        assert!(!buf.is_empty());

        let loaded = load_stc_container(buf.as_slice()).unwrap();
        assert_eq!(loaded.format_version, STC_FORMAT_VERSION);
        assert_eq!(loaded.dataset_name, "TestSet");
        assert_eq!(loaded.metadata_json, r#"{"variables":[]}"#);
        assert_eq!(loaded.active_filter_var, Some("filter_var".to_string()));
        assert_eq!(loaded.weight_var, Some("weight_var".to_string()));
        assert_eq!(loaded.table.row_count, 2);
        assert_eq!(loaded.table.columns[0].name, "score");
        assert_eq!(loaded.output_viewer_json, None);
        assert_eq!(loaded.syntax_text, None);
    }

    #[test]
    fn test_stc_container_with_project_data_roundtrip() {
        let table = RawTable {
            columns: vec![RawColumn {
                name: "id".to_string(),
                data_type: RawDataType::Numeric,
                values: vec![RawValue::Number(1.0)],
            }],
            row_count: 1,
        };

        let container = StcContainer::with_project_data(
            "FullProject",
            r#"{"variables":[]}"#,
            table,
            None,
            None,
            None,
            Some(r#"{"procedures":[]}"#.to_string()),
            Some("DESCRIPTIVES VARIABLES=id.".to_string()),
        );

        let mut buf = Vec::new();
        save_stc_container(&mut buf, &container).unwrap();

        let loaded = load_stc_container(buf.as_slice()).unwrap();
        assert_eq!(loaded.dataset_name, "FullProject");
        assert_eq!(
            loaded.output_viewer_json.as_deref(),
            Some(r#"{"procedures":[]}"#)
        );
        assert_eq!(
            loaded.syntax_text.as_deref(),
            Some("DESCRIPTIVES VARIABLES=id.")
        );
    }

    #[test]
    fn test_stc_container_with_workspace_roundtrip() {
        let table1 = RawTable {
            columns: vec![RawColumn {
                name: "col1".to_string(),
                data_type: RawDataType::Numeric,
                values: vec![RawValue::Number(10.0)],
            }],
            row_count: 1,
        };
        let table2 = RawTable {
            columns: vec![RawColumn {
                name: "col2".to_string(),
                data_type: RawDataType::Text,
                values: vec![RawValue::Text("hello".to_string())],
            }],
            row_count: 1,
        };

        let ds1 = StcDatasetRecord {
            dataset_name: "DataSet1".to_string(),
            original_path: Some("file1.sav".to_string()),
            metadata_json: r#"{"variables":[]}"#.to_string(),
            table: table1,
            active_filter_var: None,
            weight_var: None,
            split_file_json: None,
            tab_color: Some("#10B981".to_string()),
        };
        let ds2 = StcDatasetRecord {
            dataset_name: "DataSet2".to_string(),
            original_path: Some("file2.csv".to_string()),
            metadata_json: r#"{"variables":[]}"#.to_string(),
            table: table2,
            active_filter_var: None,
            weight_var: None,
            split_file_json: None,
            tab_color: Some("#F59E0B".to_string()),
        };

        let container = StcContainer::with_workspace(
            vec![ds1.clone(), ds2.clone()],
            Some(1),
            Some(r#"{"history":[]}"#.to_string()),
            Some("FREQUENCIES col2.".to_string()),
        );

        let mut buf = Vec::new();
        save_stc_container(&mut buf, &container).unwrap();

        let loaded = load_stc_container(buf.as_slice()).unwrap();
        assert_eq!(loaded.workspace_datasets.len(), 2);
        assert_eq!(loaded.active_dataset_index, Some(1));
        assert_eq!(loaded.dataset_name, "DataSet2");
        assert_eq!(loaded.tab_color, Some("#F59E0B".to_string()));
        let extracted = loaded.extract_datasets();
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].dataset_name, "DataSet1");
        assert_eq!(extracted[0].tab_color, Some("#10B981".to_string()));
        assert_eq!(extracted[1].dataset_name, "DataSet2");
        assert_eq!(extracted[1].tab_color, Some("#F59E0B".to_string()));
    }

    #[test]
    fn test_stc_legacy_container_without_colors_backward_compat() {
        let legacy_json = r#"{
            "format_version": 1,
            "dataset_name": "LegacySet",
            "metadata_json": "{\"variables\":[]}",
            "table": {"columns": [], "row_count": 0},
            "active_filter_var": null,
            "weight_var": null,
            "split_file_json": null,
            "workspace_datasets": [
                {
                    "dataset_name": "LegacySet",
                    "metadata_json": "{\"variables\":[]}",
                    "table": {"columns": [], "row_count": 0}
                }
            ]
        }"#;

        let loaded: StcContainer = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(loaded.dataset_name, "LegacySet");
        assert_eq!(loaded.tab_color, None);
        let extracted = loaded.extract_datasets();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].tab_color, None);
    }
}
