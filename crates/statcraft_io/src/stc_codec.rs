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
pub struct StcContainer {
    pub format_version: u32,
    pub dataset_name: String,
    pub metadata_json: String,
    pub table: RawTable,
    pub active_filter_var: Option<String>,
    pub weight_var: Option<String>,
    pub split_file_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_viewer_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub syntax_text: Option<String>,
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
            output_viewer_json: None,
            syntax_text: None,
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
            output_viewer_json,
            syntax_text,
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
}
