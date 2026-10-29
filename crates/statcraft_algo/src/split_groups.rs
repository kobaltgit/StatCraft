use crate::AlgoError;
use statcraft_core::{DataSet, SplitSubgroup};

/// Extracts the active split groups from the dataset.
/// If split file is not active or no grouping variables are defined,
/// returns a single group containing all active rows.
pub fn extract_split_groups(dataset: &DataSet) -> Result<Vec<SplitSubgroup>, AlgoError> {
    Ok(dataset.split_groups())
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_core::{
        DataType, Measure, RawValue, SplitFileConfig, SplitFileMode, ValueLabels,
    };

    fn create_test_dataset() -> DataSet {
        let mut ds = DataSet::blank(6, 3).unwrap();
        // Col 0: gender (1, 1, 2, 2, 1, 2)
        // Col 1: dept ("HR", "IT", "HR", "IT", "HR", "IT")
        // Col 2: salary (1000, 2000, 3000, 4000, 1500, 4500)
        ds.dictionary.get_mut(0).unwrap().name = "gender".to_string();
        ds.dictionary.get_mut(1).unwrap().name = "dept".to_string();
        ds.dictionary.get_mut(1).unwrap().var_type = DataType::String { max_len: 10 };
        ds.dictionary.get_mut(1).unwrap().measure = Measure::Nominal;
        ds.dictionary.get_mut(2).unwrap().name = "salary".to_string();

        let mut labels = ValueLabels::new();
        labels.insert("1", "Male");
        labels.insert("2", "Female");
        ds.dictionary.get_mut(0).unwrap().values = labels;

        ds.table.columns[0].name = "gender".to_string();
        ds.table.columns[1].name = "dept".to_string();
        ds.table.columns[1].data_type = statcraft_core::RawDataType::Text;
        ds.table.columns[2].name = "salary".to_string();

        let genders = [1.0, 1.0, 2.0, 2.0, 1.0, 2.0];
        let depts = ["HR", "IT", "HR", "IT", "HR", "IT"];
        let salaries = [1000.0, 2000.0, 3000.0, 4000.0, 1500.0, 4500.0];

        for r in 0..6 {
            ds.set_cell(r, 0, RawValue::Number(genders[r])).unwrap();
            ds.set_cell(r, 1, RawValue::Text(depts[r].to_string()))
                .unwrap();
            ds.set_cell(r, 2, RawValue::Number(salaries[r])).unwrap();
        }

        ds
    }

    #[test]
    fn test_extract_split_groups_single_var() {
        let mut ds = create_test_dataset();
        ds.set_split_file(Some(SplitFileConfig::new(
            SplitFileMode::CompareGroups,
            vec!["gender".to_string()],
        )))
        .unwrap();

        let groups = extract_split_groups(&ds).expect("Should extract groups");
        assert_eq!(groups.len(), 2);

        // Group 1: gender = 1 [Male] (rows 0, 1, 4)
        assert_eq!(groups[0].keys[0].var_name, "gender");
        assert_eq!(groups[0].keys[0].raw_value, RawValue::Number(1.0));
        assert_eq!(groups[0].row_indices, vec![0, 1, 4]);
        assert!(groups[0].label.contains("Male"));

        // Group 2: gender = 2 [Female] (rows 2, 3, 5)
        assert_eq!(groups[1].keys[0].var_name, "gender");
        assert_eq!(groups[1].keys[0].raw_value, RawValue::Number(2.0));
        assert_eq!(groups[1].row_indices, vec![2, 3, 5]);
        assert!(groups[1].label.contains("Female"));
    }

    #[test]
    fn test_extract_split_groups_multi_var() {
        let mut ds = create_test_dataset();
        ds.set_split_file(Some(SplitFileConfig::new(
            SplitFileMode::OrganizeOutput,
            vec!["gender".to_string(), "dept".to_string()],
        )))
        .unwrap();

        let groups = extract_split_groups(&ds).expect("Should extract groups");
        // 4 combinations: (1, HR), (1, IT), (2, HR), (2, IT)
        assert_eq!(groups.len(), 4);
        assert_eq!(groups[0].row_indices, vec![0, 4]); // 1, HR
        assert_eq!(groups[1].row_indices, vec![1]); // 1, IT
        assert_eq!(groups[2].row_indices, vec![2]); // 2, HR
        assert_eq!(groups[3].row_indices, vec![3, 5]); // 2, IT
    }

    #[test]
    fn test_extract_split_groups_with_filter() {
        let mut ds = create_test_dataset();
        // Add filter var
        let filter_col = ds
            .add_variable(
                "filter_$",
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )
            .unwrap();
        // Include only rows 0, 1, 2 (exclude rows 3, 4, 5)
        ds.set_cell(0, filter_col, RawValue::Number(1.0)).unwrap();
        ds.set_cell(1, filter_col, RawValue::Number(1.0)).unwrap();
        ds.set_cell(2, filter_col, RawValue::Number(1.0)).unwrap();
        ds.set_cell(3, filter_col, RawValue::Number(0.0)).unwrap();
        ds.set_cell(4, filter_col, RawValue::Number(0.0)).unwrap();
        ds.set_cell(5, filter_col, RawValue::Number(0.0)).unwrap();
        ds.set_filter_variable(Some("filter_$".to_string()))
            .unwrap();

        ds.set_split_file(Some(SplitFileConfig::new(
            SplitFileMode::CompareGroups,
            vec!["gender".to_string()],
        )))
        .unwrap();

        let groups = extract_split_groups(&ds).expect("Should extract groups");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].row_indices, vec![0, 1]);
        assert_eq!(groups[1].row_indices, vec![2]);
    }

    #[test]
    fn test_extract_split_groups_with_missing_values() {
        let mut ds = DataSet::blank(3, 1).unwrap();
        ds.dictionary.get_mut(0).unwrap().name = "cat".to_string();
        ds.table.columns[0].name = "cat".to_string();

        ds.set_cell(0, 0, RawValue::Number(1.0)).unwrap();
        ds.set_cell(1, 0, RawValue::Null).unwrap();
        ds.set_cell(2, 0, RawValue::Number(1.0)).unwrap();

        ds.set_split_file(Some(SplitFileConfig::new(
            SplitFileMode::CompareGroups,
            vec!["cat".to_string()],
        )))
        .unwrap();

        let groups = extract_split_groups(&ds).expect("Should extract groups");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].row_indices, vec![0, 2]);
        assert_eq!(groups[1].row_indices, vec![1]);
        assert_eq!(groups[1].label, "cat = .");
    }
}
