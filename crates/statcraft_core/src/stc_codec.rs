use crate::dataset::{DataSet, DeltaBuffer};
use crate::split::SplitFileConfig;
use crate::variable::VariableDictionary;
use statcraft_io::{
    export_raw_table_to_csv_path, load_stc_container, load_stc_container_from_path,
    save_stc_container, save_stc_container_to_path, CsvExportOptions, IoError, StcContainer,
};
use std::io::{Read, Write};
use std::path::Path;

pub fn project_to_stc(
    dataset: &DataSet,
    dataset_name: &str,
    output_viewer_json: Option<String>,
    syntax_text: Option<String>,
) -> Result<StcContainer, IoError> {
    let metadata_json = serde_json::to_string(&dataset.dictionary)
        .map_err(|e| IoError::Serialization(e.to_string()))?;

    let split_file_json = match &dataset.split_file {
        Some(sf) => {
            let json =
                serde_json::to_string(sf).map_err(|e| IoError::Serialization(e.to_string()))?;
            Some(json)
        }
        None => None,
    };

    Ok(StcContainer::with_project_data(
        dataset_name,
        metadata_json,
        dataset.table.clone(),
        dataset.active_filter_var.clone(),
        dataset.weight_var.clone(),
        split_file_json,
        output_viewer_json,
        syntax_text,
    ))
}

pub fn dataset_to_stc(dataset: &DataSet, dataset_name: &str) -> Result<StcContainer, IoError> {
    project_to_stc(dataset, dataset_name, None, None)
}

pub fn stc_to_dataset(container: StcContainer) -> Result<DataSet, IoError> {
    let dictionary: VariableDictionary = serde_json::from_str(&container.metadata_json)
        .map_err(|e| IoError::Deserialization(e.to_string()))?;

    let split_file: Option<SplitFileConfig> = match container.split_file_json.as_deref() {
        Some(json_str) => Some(
            serde_json::from_str(json_str).map_err(|e| IoError::Deserialization(e.to_string()))?,
        ),
        None => None,
    };

    Ok(DataSet {
        table: container.table,
        dictionary,
        delta: DeltaBuffer::new(),
        active_filter_var: container.active_filter_var,
        split_file,
        weight_var: container.weight_var,
    })
}

pub fn save_stc_to_writer<W: Write>(writer: W, dataset: &DataSet) -> Result<(), IoError> {
    let container = dataset_to_stc(dataset, "DataSet1")?;
    save_stc_container(writer, &container)
}

pub fn load_stc_from_reader<R: Read>(reader: R) -> Result<DataSet, IoError> {
    let container = load_stc_container(reader)?;
    stc_to_dataset(container)
}

pub fn save_project_stc_to_path<P: AsRef<Path>>(
    path: P,
    dataset: &DataSet,
    output_viewer_json: Option<String>,
    syntax_text: Option<String>,
) -> Result<(), IoError> {
    let dataset_name = path
        .as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("DataSet1");
    let container = project_to_stc(dataset, dataset_name, output_viewer_json, syntax_text)?;
    save_stc_container_to_path(path, &container)
}

pub fn load_project_stc_from_path<P: AsRef<Path>>(
    path: P,
) -> Result<(DataSet, Option<String>, Option<String>), IoError> {
    let container = load_stc_container_from_path(path)?;
    let output_json = container.output_viewer_json.clone();
    let syntax = container.syntax_text.clone();
    let dataset = stc_to_dataset(container)?;
    Ok((dataset, output_json, syntax))
}

pub fn save_stc_to_path<P: AsRef<Path>>(path: P, dataset: &DataSet) -> Result<(), IoError> {
    save_project_stc_to_path(path, dataset, None, None)
}

pub fn load_stc_from_path<P: AsRef<Path>>(path: P) -> Result<DataSet, IoError> {
    let (dataset, _output, _syntax) = load_project_stc_from_path(path)?;
    Ok(dataset)
}

pub fn save_csv_to_path<P: AsRef<Path>>(path: P, dataset: &DataSet) -> Result<(), IoError> {
    let options = CsvExportOptions::default();
    export_raw_table_to_csv_path(path, &dataset.table, &options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split::SplitFileMode;
    use crate::variable::{
        Alignment, DataType, Measure, MissingValues, Role, ValueLabels, VariableMetadata,
    };
    use statcraft_io::{RawColumn, RawDataType, RawTable, RawValue};

    #[test]
    fn test_roundtrip_all_9_datatypes_and_metadata() {
        let mut dictionary = VariableDictionary::new();

        let types = vec![
            ("var_num", DataType::Numeric { decimals: 2 }),
            ("var_comma", DataType::Comma { decimals: 0 }),
            ("var_dot", DataType::Dot { decimals: 3 }),
            ("var_sci", DataType::Scientific { decimals: 4 }),
            (
                "var_date",
                DataType::Date {
                    format: "YYYY-MM-DD".to_string(),
                },
            ),
            ("var_dollar", DataType::Dollar { decimals: 2 }),
            (
                "var_curr",
                DataType::CustomCurrency {
                    format_code: "EUR".to_string(),
                    decimals: 2,
                },
            ),
            ("var_str", DataType::String { max_len: 50 }),
            ("var_restr", DataType::RestrictedNumeric { width: 6 }),
        ];

        let mut columns = Vec::new();

        for (i, (name, dt)) in types.into_iter().enumerate() {
            let measure = match i % 3 {
                0 => Measure::Scale,
                1 => Measure::Ordinal,
                _ => Measure::Nominal,
            };

            let mut meta = VariableMetadata::new(name, dt.clone(), measure).unwrap();
            meta.label = format!("Label for {}", name);
            meta.width = 10 + i;
            meta.columns = 12 + i;
            meta.align = match i % 3 {
                0 => Alignment::Left,
                1 => Alignment::Center,
                _ => Alignment::Right,
            };
            meta.role = if i % 2 == 0 {
                Role::Input
            } else {
                Role::Target
            };

            if i == 0 {
                let mut vl = ValueLabels::new();
                vl.insert("1", "Low");
                vl.insert("2", "High");
                meta.values = vl;
            }

            if i == 1 {
                meta.missing = MissingValues {
                    discrete: vec!["99".to_string(), "999".to_string()],
                    range: Some((100.0, 200.0)),
                };
            }

            dictionary.add(meta).unwrap();

            let raw_type = if dt.is_numeric() {
                RawDataType::Numeric
            } else {
                RawDataType::Text
            };

            let vals = if dt.is_numeric() {
                vec![RawValue::Number(i as f64 * 10.5), RawValue::Null]
            } else {
                vec![RawValue::Text(format!("val_{}", i)), RawValue::Null]
            };

            columns.push(RawColumn {
                name: name.to_string(),
                data_type: raw_type,
                values: vals,
            });
        }

        let table = RawTable {
            columns,
            row_count: 2,
        };

        let split_config =
            SplitFileConfig::new(SplitFileMode::CompareGroups, vec!["var_num".to_string()]);

        let original_dataset = DataSet {
            table,
            dictionary,
            delta: DeltaBuffer::new(),
            active_filter_var: Some("var_num".to_string()),
            split_file: Some(split_config),
            weight_var: Some("var_comma".to_string()),
        };

        // Roundtrip via memory writer/reader
        let mut buffer = Vec::new();
        save_stc_to_writer(&mut buffer, &original_dataset).unwrap();
        assert!(!buffer.is_empty());

        let restored = load_stc_from_reader(buffer.as_slice()).unwrap();

        assert_eq!(restored.row_count(), original_dataset.row_count());
        assert_eq!(restored.col_count(), original_dataset.col_count());
        assert_eq!(
            restored.active_filter_var,
            original_dataset.active_filter_var
        );
        assert_eq!(restored.weight_var, original_dataset.weight_var);
        assert_eq!(restored.split_file, original_dataset.split_file);
        assert_eq!(restored.dictionary, original_dataset.dictionary);
        assert_eq!(restored.table, original_dataset.table);
    }

    #[test]
    fn test_save_load_project_stc_with_output_and_syntax() {
        let temp_dir = std::env::temp_dir();
        let project_path = temp_dir.join("statcraft_test_project.stc");

        let dataset = DataSet::blank(10, 0).unwrap();
        let output_json = Some(r#"{"procedures":[{"id":1}]}"#.to_string());
        let syntax_text = Some("FREQUENCIES VARIABLES=ALL.".to_string());

        save_project_stc_to_path(
            &project_path,
            &dataset,
            output_json.clone(),
            syntax_text.clone(),
        )
        .unwrap();

        let (loaded_ds, loaded_out, loaded_syn) =
            load_project_stc_from_path(&project_path).unwrap();

        assert_eq!(loaded_ds.row_count(), 10);
        assert_eq!(loaded_ds.col_count(), 0);
        assert_eq!(loaded_out, output_json);
        assert_eq!(loaded_syn, syntax_text);

        let _ = std::fs::remove_file(&project_path);
    }
}
