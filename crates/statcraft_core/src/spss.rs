use crate::dataset::{DataSet, DeltaBuffer};
use crate::variable::{
    Alignment, CoreError, DataType, Measure, MissingValues, Role, ValueLabels, VariableDictionary,
    VariableMetadata,
};
use statcraft_io::{RawDataType, SpssData};
use std::io::{Read, Seek};
use std::path::Path;

impl DataSet {
    pub fn from_spss_data(data: SpssData) -> Result<Self, CoreError> {
        let mut dictionary = VariableDictionary::new();

        for var_info in &data.variables {
            let var_type = match var_info.var_type {
                RawDataType::Numeric => {
                    let fmt_type = (var_info.print_format >> 16) & 0xFF;
                    match fmt_type {
                        6 => DataType::Comma {
                            decimals: var_info.decimals,
                        },
                        7 => DataType::Dot {
                            decimals: var_info.decimals,
                        },
                        8 => DataType::Dollar {
                            decimals: var_info.decimals,
                        },
                        20..=30 | 38..=39 => DataType::Date {
                            format: "DATE".to_string(),
                        },
                        _ => DataType::Numeric {
                            decimals: var_info.decimals,
                        },
                    }
                }
                RawDataType::Text => DataType::String {
                    max_len: var_info.string_len.max(1),
                },
            };

            let measure = match var_info.measure {
                1 => Measure::Nominal,
                2 => Measure::Ordinal,
                3 => Measure::Scale,
                _ => {
                    if var_info.var_type == RawDataType::Numeric {
                        Measure::Scale
                    } else {
                        Measure::Nominal
                    }
                }
            };

            let align = match var_info.alignment {
                0 => Alignment::Left,
                1 => Alignment::Right,
                2 => Alignment::Center,
                _ => {
                    if var_info.var_type == RawDataType::Text {
                        Alignment::Left
                    } else {
                        Alignment::Right
                    }
                }
            };

            let mut var_meta = VariableMetadata::new(&var_info.name, var_type, measure)?;
            var_meta.label = var_info.label.clone();
            var_meta.width = var_info.width.max(1);
            var_meta.decimals = var_info.decimals;
            var_meta.columns = var_info.display_width.max(1);
            var_meta.align = align;
            var_meta.role = Role::Input;
            var_meta.values = ValueLabels {
                mapping: var_info.value_labels.clone(),
            };
            var_meta.missing = MissingValues {
                discrete: var_info.missing_discrete.clone(),
                range: var_info.missing_range,
            };

            dictionary.add(var_meta)?;
        }

        Ok(Self {
            table: data.raw_table,
            dictionary,
            delta: DeltaBuffer::new(),
            active_filter_var: None,
            split_file: None,
            weight_var: data.weight_var_name,
        })
    }

    pub fn load_spss_from_path<P: AsRef<Path>>(path: P) -> Result<Self, CoreError> {
        let spss_data = statcraft_io::load_spss_from_path(path)?;
        Self::from_spss_data(spss_data)
    }

    pub fn load_spss_from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self, CoreError> {
        let spss_data = statcraft_io::load_spss_from_reader(reader)?;
        Self::from_spss_data(spss_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_io::{RawColumn, RawTable, RawValue, SpssVariableInfo};
    use std::collections::BTreeMap;

    #[test]
    fn test_from_spss_data_integration() {
        let mut labels = BTreeMap::new();
        labels.insert("1".to_string(), "Male".to_string());
        labels.insert("2".to_string(), "Female".to_string());

        let var1 = SpssVariableInfo {
            name: "gender".to_string(),
            original_name: "VAR00001".to_string(),
            var_type: RawDataType::Numeric,
            string_len: 0,
            print_format: 0x00050800,
            write_format: 0x00050800,
            decimals: 0,
            width: 8,
            label: "Gender of respondent".to_string(),
            measure: 1, // Nominal
            alignment: 1,
            display_width: 8,
            value_labels: labels,
            missing_discrete: vec!["9".to_string()],
            missing_range: None,
        };

        let raw_table = RawTable {
            columns: vec![RawColumn {
                name: "gender".to_string(),
                data_type: RawDataType::Numeric,
                values: vec![RawValue::Number(1.0), RawValue::Number(2.0)],
            }],
            row_count: 2,
        };

        let spss_data = SpssData {
            raw_table,
            variables: vec![var1],
            weight_var_name: None,
        };

        let ds = DataSet::from_spss_data(spss_data).unwrap();
        assert_eq!(ds.row_count(), 2);
        assert_eq!(ds.col_count(), 1);
        let meta = ds.dictionary.get(0).unwrap();
        assert_eq!(meta.name, "gender");
        assert_eq!(meta.label, "Gender of respondent");
        assert_eq!(meta.measure, Measure::Nominal);
        assert_eq!(meta.values.get_label("1"), Some("Male"));
        assert!(meta.missing.is_missing(&RawValue::Number(9.0)));
    }
}
