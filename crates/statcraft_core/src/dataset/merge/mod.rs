pub mod cases;
pub mod source;
pub mod types;
pub mod vars;

pub use cases::add_cases;
pub use source::load_dataset_from_file;
pub use types::{AddCasesConfig, AddVarsConfig, MergeMethod, VariablePairInfo, VariablePairStatus};
pub use vars::add_variables;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::DataSet;
    use statcraft_io::RawValue;

    fn make_test_dataset_a() -> DataSet {
        let mut ds = DataSet::blank(3, 2).expect("blank ds");
        ds.dictionary.get_mut(0).expect("var1").name = "id".to_string();
        ds.table.columns[0].name = "id".to_string();
        ds.table.columns[0].values = vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(3.0),
        ];

        ds.dictionary.get_mut(1).expect("var2").name = "age".to_string();
        ds.table.columns[1].name = "age".to_string();
        ds.table.columns[1].values = vec![
            RawValue::Number(25.0),
            RawValue::Number(30.0),
            RawValue::Number(35.0),
        ];
        ds
    }

    fn make_test_dataset_b() -> DataSet {
        let mut ds = DataSet::blank(2, 2).expect("blank ds");
        ds.dictionary.get_mut(0).expect("var1").name = "id".to_string();
        ds.table.columns[0].name = "id".to_string();
        ds.table.columns[0].values = vec![RawValue::Number(4.0), RawValue::Number(5.0)];

        ds.dictionary.get_mut(1).expect("var2").name = "age".to_string();
        ds.table.columns[1].name = "age".to_string();
        ds.table.columns[1].values = vec![RawValue::Number(40.0), RawValue::Number(45.0)];
        ds
    }

    #[test]
    fn test_add_cases_basic() {
        let mut ds_a = make_test_dataset_a();
        let ds_b = make_test_dataset_b();

        let config = AddCasesConfig {
            in_variable: Some("source".to_string()),
            ..Default::default()
        };

        let res = add_cases(&mut ds_a, &ds_b.table, &ds_b.dictionary, &config);
        assert!(res.is_ok());
        assert_eq!(ds_a.row_count(), 5);
        assert_eq!(ds_a.col_count(), 3); // id, age, source

        // Check source column
        let src_col = ds_a
            .table
            .columns
            .iter()
            .find(|c| c.name == "source")
            .expect("source col");
        assert_eq!(
            src_col.values,
            vec![
                RawValue::Number(0.0),
                RawValue::Number(0.0),
                RawValue::Number(0.0),
                RawValue::Number(1.0),
                RawValue::Number(1.0),
            ]
        );
    }

    #[test]
    fn test_add_variables_sequential() {
        let mut ds_a = make_test_dataset_a();
        let mut ds_b = DataSet::blank(3, 1).expect("blank");
        ds_b.dictionary.get_mut(0).expect("var").name = "salary".to_string();
        ds_b.table.columns[0].name = "salary".to_string();
        ds_b.table.columns[0].values = vec![
            RawValue::Number(100.0),
            RawValue::Number(200.0),
            RawValue::Number(300.0),
        ];

        let config = AddVarsConfig {
            method: MergeMethod::Sequential,
            ..Default::default()
        };

        let res = add_variables(&mut ds_a, &ds_b.table, &ds_b.dictionary, &config);
        assert!(res.is_ok());
        assert_eq!(ds_a.row_count(), 3);
        assert_eq!(ds_a.col_count(), 3);
        assert!(ds_a.dictionary.find_by_name("salary").is_some());
    }

    #[test]
    fn test_add_variables_keyed_one_to_one() {
        let mut ds_a = make_test_dataset_a();
        let mut ds_b = DataSet::blank(2, 2).expect("blank");
        ds_b.dictionary.get_mut(0).expect("var1").name = "id".to_string();
        ds_b.table.columns[0].name = "id".to_string();
        ds_b.table.columns[0].values = vec![RawValue::Number(2.0), RawValue::Number(3.0)];

        ds_b.dictionary.get_mut(1).expect("var2").name = "income".to_string();
        ds_b.table.columns[1].name = "income".to_string();
        ds_b.table.columns[1].values = vec![RawValue::Number(5000.0), RawValue::Number(6000.0)];

        let config = AddVarsConfig {
            method: MergeMethod::OneToOneWithKeys,
            keys: vec!["id".to_string()],
            ..Default::default()
        };

        let res = add_variables(&mut ds_a, &ds_b.table, &ds_b.dictionary, &config);
        assert!(res.is_ok());
        assert_eq!(ds_a.row_count(), 3);
        assert_eq!(ds_a.col_count(), 3); // id, age, income

        let inc_col = ds_a
            .table
            .columns
            .iter()
            .find(|c| c.name == "income")
            .expect("income col");
        assert_eq!(inc_col.values[0], RawValue::Null); // id 1 had no income
        assert_eq!(inc_col.values[1], RawValue::Number(5000.0)); // id 2
        assert_eq!(inc_col.values[2], RawValue::Number(6000.0)); // id 3
    }
}
