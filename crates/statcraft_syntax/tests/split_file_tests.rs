use statcraft_core::{DataSet, RawValue, SplitFileMode};
use statcraft_syntax::commands::SplitFileCommand;
use statcraft_syntax::CommandBus;

fn make_sample_dataset() -> DataSet {
    let mut ds = DataSet::blank(6, 3).unwrap();
    // Col 0: dept ("Sales", "Eng", "Sales", "Eng", "Sales", "Eng")
    // Col 1: gender (1, 1, 2, 2, 1, 2)
    // Col 2: salary (1000, 2000, 1500, 2500, 1200, 2700)
    ds.dictionary.get_mut(0).unwrap().name = "dept".to_string();
    ds.dictionary.get_mut(1).unwrap().name = "gender".to_string();
    ds.dictionary.get_mut(2).unwrap().name = "salary".to_string();

    ds.table.columns[0].name = "dept".to_string();
    ds.table.columns[0].data_type = statcraft_core::RawDataType::Text;
    ds.table.columns[1].name = "gender".to_string();
    ds.table.columns[2].name = "salary".to_string();

    let depts = ["Sales", "Eng", "Sales", "Eng", "Sales", "Eng"];
    let genders = [1.0, 1.0, 2.0, 2.0, 1.0, 2.0];
    let salaries = [1000.0, 2000.0, 1500.0, 2500.0, 1200.0, 2700.0];

    for r in 0..6 {
        ds.set_cell(r, 0, RawValue::Text(depts[r].to_string()))
            .unwrap();
        ds.set_cell(r, 1, RawValue::Number(genders[r])).unwrap();
        ds.set_cell(r, 2, RawValue::Number(salaries[r])).unwrap();
    }

    ds
}

#[test]
fn test_split_file_command_bus_integration() {
    let mut bus = CommandBus::with_defaults();
    let mut ds = Some(make_sample_dataset());

    let params = serde_json::json!({
        "mode": "separate",
        "variables": ["dept"],
        "sort": true
    });

    let res = bus
        .execute("data.split_file", &mut ds, params)
        .expect("Should execute");
    assert_eq!(res["status"], "ok");

    let current = ds.as_ref().unwrap();
    assert!(current.split_file().unwrap().is_active());
    assert_eq!(
        current.split_file().unwrap().mode,
        SplitFileMode::OrganizeOutput
    );

    // Verify dataset is sorted by dept: "Eng" rows come before "Sales" rows
    assert_eq!(
        current.get_cell(0, 0),
        Some(&RawValue::Text("Eng".to_string()))
    );
    assert_eq!(
        current.get_cell(1, 0),
        Some(&RawValue::Text("Eng".to_string()))
    );
    assert_eq!(
        current.get_cell(2, 0),
        Some(&RawValue::Text("Eng".to_string()))
    );
    assert_eq!(
        current.get_cell(3, 0),
        Some(&RawValue::Text("Sales".to_string()))
    );

    // Check split groups
    let groups = current.split_groups();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].row_indices, vec![0, 1, 2]);
    assert_eq!(groups[1].row_indices, vec![3, 4, 5]);

    // Test Undo
    bus.undo(&mut ds).expect("Undo should succeed");
    let restored = ds.as_ref().unwrap();
    assert_eq!(restored.split_file(), None);
    // Row 0 was "Sales" before sorting
    assert_eq!(
        restored.get_cell(0, 0),
        Some(&RawValue::Text("Sales".to_string()))
    );
}

#[test]
fn test_split_file_sps_syntax_generation() {
    let sps1 = SplitFileCommand::to_sps(SplitFileMode::Off, &[], false);
    assert!(sps1.contains("SPLIT FILE OFF."));

    let sps2 =
        SplitFileCommand::to_sps(SplitFileMode::CompareGroups, &["gender".to_string()], true);
    assert!(sps2.contains("SORT CASES BY gender (A)."));
    assert!(sps2.contains("SPLIT FILE LAYERED BY gender."));

    let sps3 = SplitFileCommand::to_sps(
        SplitFileMode::OrganizeOutput,
        &["gender".to_string(), "dept".to_string()],
        false,
    );
    assert!(!sps3.contains("SORT CASES"));
    assert!(sps3.contains("SPLIT FILE SEPARATE BY gender dept."));
}
