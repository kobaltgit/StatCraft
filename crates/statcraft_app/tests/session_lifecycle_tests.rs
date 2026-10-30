use egui::Context;
use statcraft_app::app_session::PendingSessionAction;
use statcraft_app::dialogs::render_confirm_save_dialog;
use statcraft_app::recent_files::RecentFiles;
use statcraft_app::views::output_view::OutputItemKind;
use statcraft_app::{StatCraftApp, Tab};
use statcraft_core::{DataType, Measure};
use std::path::PathBuf;

#[test]
fn test_recent_files_queue_and_limits() {
    let mut rf = RecentFiles::new();
    assert!(rf.is_empty());
    assert_eq!(rf.len(), 0);

    // 1. Add unique paths
    for i in 1..=5 {
        rf.add(PathBuf::from(format!("test_data_{}.csv", i)));
    }
    assert_eq!(rf.len(), 5);
    assert_eq!(rf.paths[0], PathBuf::from("test_data_5.csv"));

    // 2. Deduplication moves path to front
    rf.add(PathBuf::from("test_data_2.csv"));
    assert_eq!(rf.len(), 5);
    assert_eq!(rf.paths[0], PathBuf::from("test_data_2.csv"));
    assert_eq!(rf.paths[1], PathBuf::from("test_data_5.csv"));

    // 3. Overflow beyond MAX_ENTRIES (10) truncates older entries
    for i in 6..=16 {
        rf.add(PathBuf::from(format!("overflow_{}.csv", i)));
    }
    assert_eq!(rf.len(), RecentFiles::MAX_ENTRIES);
    assert_eq!(rf.paths[0], PathBuf::from("overflow_16.csv"));
    assert_eq!(rf.paths[9], PathBuf::from("overflow_7.csv"));

    // 4. Clear empties list
    rf.clear();
    assert!(rf.is_empty());
    assert_eq!(rf.len(), 0);
}

#[test]
fn test_new_dataset_session_lifecycle() {
    let mut app = StatCraftApp::new_headless();

    // Populate dataset with a variable
    if let Some(ds) = &mut app.dataset {
        ds.add_variable("income", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
    }
    app.current_file_path = Some(PathBuf::from("active.csv"));
    assert!(app.has_unsaved_changes());

    // Execute New Dataset
    app.execute_new_dataset();

    // Verification:
    assert_eq!(app.current_file_path, None);
    assert_eq!(app.dataset.as_ref().map(|d| d.col_count()).unwrap_or(0), 0);
    assert!(app.command_bus.history().is_empty());

    // Status message and log
    assert_eq!(
        app.status_message.text(),
        statcraft_i18n::t!("status-new-dataset-created")
    );
    let log_proc = app.output_view_state.procedures.iter().find(|p| {
        p.items
            .iter()
            .any(|item| matches!(&item.kind, OutputItemKind::Log(s) if s.contains("NEW DATASET.")))
    });
    assert!(
        log_proc.is_some(),
        "Expected Output Viewer to record 'NEW DATASET.' in log procedure"
    );

    // Active tab in dock exists and can be set
    let data_tab_pos = app.dock_state.find_tab(&Tab::DataView);
    assert!(data_tab_pos.is_some());
}

#[test]
fn test_new_syntax_session_lifecycle() {
    let mut app = StatCraftApp::new_headless();

    // Execute New Syntax
    app.syntax_editor_state.reset_with_header();
    if let Some(pos) = app.dock_state.find_tab(&Tab::SyntaxEditor) {
        app.dock_state.set_active_tab(pos);
    }
    app.status_message = statcraft_app::app::StatusMessage::NewSyntaxCreated;

    // Verification:
    assert!(!app.syntax_editor_state.is_empty());
    assert!(app
        .syntax_editor_state
        .text
        .contains("* StatCraft Syntax Editor."));
    assert!(app.syntax_editor_state.text.contains("* Created:"));
    assert_eq!(
        app.status_message.text(),
        statcraft_i18n::t!("status-new-syntax-created")
    );

    let syntax_tab_pos = app.dock_state.find_tab(&Tab::SyntaxEditor);
    assert!(syntax_tab_pos.is_some());
}

#[test]
fn test_new_project_session_lifecycle() {
    let mut app = StatCraftApp::new_headless();

    // Populate dataset, output view, and syntax editor
    if let Some(ds) = &mut app.dataset {
        ds.add_variable("var1", DataType::Numeric { decimals: 0 }, Measure::Nominal)
            .unwrap();
    }
    app.output_view_state.add_log("Existing Log");
    app.syntax_editor_state.text = "DESCRIPTIVES VARIABLES=ALL.".to_string();
    app.current_file_path = Some(PathBuf::from("proj.stc"));

    assert!(!app.output_view_state.is_empty());
    assert!(!app.syntax_editor_state.is_empty());

    // Execute New Project
    app.execute_new_project();

    // Verification:
    assert_eq!(app.current_file_path, None);
    assert_eq!(app.dataset.as_ref().map(|d| d.col_count()).unwrap_or(0), 0);
    assert!(app.output_view_state.is_empty());
    assert!(app.syntax_editor_state.is_empty());
    assert_eq!(
        app.status_message.text(),
        statcraft_i18n::t!("status-new-project-created")
    );
}

#[test]
fn test_close_project_session_lifecycle() {
    let mut app = StatCraftApp::new_headless();

    // Populate state
    if let Some(ds) = &mut app.dataset {
        ds.add_variable("var1", DataType::Numeric { decimals: 0 }, Measure::Nominal)
            .unwrap();
    }
    app.output_view_state.add_log("Existing Log");
    app.current_file_path = Some(PathBuf::from("proj.stc"));

    // Execute Close Project
    app.execute_close_project();

    // Verification:
    assert_eq!(app.current_file_path, None);
    assert_eq!(app.dataset.as_ref().map(|d| d.col_count()).unwrap_or(0), 0);
    assert!(app.output_view_state.is_empty());
    assert!(app.syntax_editor_state.is_empty());
    assert_eq!(
        app.status_message.text(),
        statcraft_i18n::t!("status-ready")
    );
}

#[test]
fn test_has_unsaved_changes_detection() {
    let mut app = StatCraftApp::new_headless();
    assert!(!app.has_unsaved_changes());

    if let Some(ds) = &mut app.dataset {
        ds.add_variable("x", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
    }
    assert!(app.has_unsaved_changes());

    // Save project to temp path
    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join("statcraft_dirty_marker_test.stc");
    app.save_to_path(save_path.clone());

    // After save, dirty marker is cleared
    assert!(!app.has_unsaved_changes());
    assert_eq!(
        app.status_message.color(),
        statcraft_app::theme::Tokens::MEASURE_NOMINAL
    );

    // Record a cell edit
    if let Some(ds) = &mut app.dataset {
        ds.set_cell_parsed(0, 0, "42.0").unwrap();
    }
    assert!(app.has_unsaved_changes());

    // Undo cell edit restores checkpoint state
    if let Some(ds) = &mut app.dataset {
        assert!(ds.undo());
    }
    assert!(!app.has_unsaved_changes());

    // Redo cell edit makes it dirty again
    if let Some(ds) = &mut app.dataset {
        assert!(ds.redo());
    }
    assert!(app.has_unsaved_changes());

    // Execute new dataset clears dirty state
    app.execute_new_dataset();
    assert!(!app.has_unsaved_changes());

    let _ = std::fs::remove_file(save_path);
}

#[test]
fn test_toolbar_save_action_execution() {
    let mut app = StatCraftApp::new_headless();
    if let Some(ds) = &mut app.dataset {
        ds.add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
    }
    let temp_dir = std::env::temp_dir();
    let save_path = temp_dir.join("statcraft_toolbar_save_test.stc");
    app.current_file_path = Some(save_path.clone());

    assert!(app.has_unsaved_changes());

    // Trigger save_project directly (as dispatched by ToolbarAction::SaveData)
    app.save_project();

    assert!(save_path.exists());
    assert!(!app.has_unsaved_changes());
    assert!(matches!(
        app.status_message,
        statcraft_app::StatusMessage::ProjectSaved { .. }
    ));
    assert_eq!(
        app.status_message.color(),
        statcraft_app::theme::Tokens::MEASURE_NOMINAL
    );

    let _ = std::fs::remove_file(save_path);
}

#[test]
fn test_headless_render_session_components() {
    let ctx = Context::default();
    let mut app = StatCraftApp::new_headless();

    // Headless run with empty recent files
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.render_recent_files_menu(ui);
        });
    });

    // Populate recent files and render again
    app.recent_files.add(PathBuf::from("salary_survey.csv"));
    app.recent_files.add(PathBuf::from("customer_churn.csv"));

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.render_recent_files_menu(ui);
        });
    });

    // Test Confirm Save modal rendering
    app.confirm_save_dialog_state.is_open = true;
    app.pending_session_action = Some(PendingSessionAction::NewDataset);

    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_confirm_save_dialog(ctx, &mut app.confirm_save_dialog_state);
        assert_eq!(action, statcraft_app::dialogs::ConfirmSaveAction::None);
    });
}

#[test]
fn test_save_and_load_stc_project() {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("statcraft_session_test.stc");

    let mut app = StatCraftApp::new_headless();
    if let Some(ds) = &mut app.dataset {
        ds.add_variable("income", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
        ds.add_variable("dept", DataType::String { max_len: 20 }, Measure::Nominal)
            .unwrap();
    }

    // Save project
    app.save_to_path(file_path.clone());
    assert!(file_path.exists());
    assert_eq!(app.current_file_path, Some(file_path.clone()));
    assert_eq!(app.recent_files.paths[0], file_path);
    assert!(matches!(
        app.status_message,
        statcraft_app::StatusMessage::ProjectSaved { .. }
    ));

    // Clear app and load project
    let mut new_app = StatCraftApp::new_headless();
    new_app.load_file_from_path(file_path.clone());

    assert_eq!(new_app.current_file_path, Some(file_path.clone()));
    assert_eq!(new_app.recent_files.paths[0], file_path);
    assert!(matches!(
        new_app.status_message,
        statcraft_app::StatusMessage::ProjectOpened { .. }
    ));

    let loaded_ds = new_app.dataset.as_ref().unwrap();
    assert_eq!(loaded_ds.col_count(), 2);
    assert_eq!(loaded_ds.dictionary.get(0).unwrap().name, "income");
    assert_eq!(loaded_ds.dictionary.get(1).unwrap().name, "dept");

    let _ = std::fs::remove_file(file_path);
}

#[test]
fn test_save_and_load_full_project_with_output_and_syntax() {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("statcraft_full_session_test.stc");

    let mut app = StatCraftApp::new_headless();
    if let Some(ds) = &mut app.dataset {
        ds.add_variable("income", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
        ds.add_variable("dept", DataType::String { max_len: 20 }, Measure::Nominal)
            .unwrap();
    }

    // Add procedures, logs and tables to Output Viewer
    app.output_view_state
        .add_log("DESCRIPTIVES VARIABLES=income.");
    let item_id = app.output_view_state.alloc_id();
    let proc_id = app.output_view_state.alloc_id();
    app.output_view_state
        .procedures
        .push(statcraft_app::views::output_view::OutputProcedure {
            id: proc_id,
            title_key: "output-descriptives-title",
            custom_title: Some("Employee Income Descriptives".to_string()),
            is_expanded: true,
            items: vec![statcraft_app::views::output_view::OutputItem {
                id: item_id,
                title_key: "output-descriptives-table",
                custom_title: None,
                kind: statcraft_app::views::output_view::OutputItemKind::DescriptivesTable(vec![
                    statcraft_algo::DescriptivesResult {
                        variable: "income".to_string(),
                        n_valid: 100,
                        n_missing: 0,
                        mean: Some(54321.0),
                        se_mean: Some(123.45),
                        std_dev: Some(1234.5),
                        variance: Some(1524000.0),
                        min: Some(20000.0),
                        max: Some(90000.0),
                        range: Some(70000.0),
                        sum: Some(5432100.0),
                        skewness: None,
                        se_skewness: None,
                        kurtosis: None,
                        se_kurtosis: None,
                    },
                ]),
            }],
        });

    // Add script to Syntax Editor
    app.syntax_editor_state.text =
        "* Run descriptives on income\nDESCRIPTIVES VARIABLES=income.\n".to_string();

    // Save project
    app.save_to_path(file_path.clone());
    assert!(file_path.exists());

    // Reset app state via execute_new_project()
    app.execute_new_project();
    assert_eq!(app.current_file_path, None);
    assert_eq!(app.dataset.as_ref().map(|d| d.col_count()).unwrap_or(0), 0);
    assert!(app.output_view_state.is_empty());
    assert!(app.syntax_editor_state.is_empty());

    // Load saved project
    app.load_file_from_path(file_path.clone());
    assert_eq!(app.current_file_path, Some(file_path.clone()));

    // Verify 100% restoration:
    let loaded_ds = app.dataset.as_ref().unwrap();
    assert_eq!(loaded_ds.col_count(), 2);
    assert_eq!(loaded_ds.dictionary.get(0).unwrap().name, "income");
    assert_eq!(loaded_ds.dictionary.get(1).unwrap().name, "dept");

    // Output viewer procedures restored (+ 1 new GET FILE log appended on load)
    assert_eq!(app.output_view_state.procedures.len(), 3);
    assert_eq!(
        app.output_view_state.procedures[1].custom_title.as_deref(),
        Some("Employee Income Descriptives")
    );
    match &app.output_view_state.procedures[1].items[0].kind {
        statcraft_app::views::output_view::OutputItemKind::DescriptivesTable(res) => {
            assert_eq!(res.len(), 1);
            assert_eq!(res[0].variable, "income");
            assert_eq!(res[0].mean, Some(54321.0));
        }
        _ => panic!("Expected DescriptivesTable"),
    }
    let last_proc = app.output_view_state.procedures.last().unwrap();
    assert!(last_proc.items.iter().any(|item| {
        matches!(&item.kind, statcraft_app::views::output_view::OutputItemKind::Log(msg) if msg.contains("GET FILE="))
    }));

    // Syntax restored
    assert_eq!(
        app.syntax_editor_state.text,
        "* Run descriptives on income\nDESCRIPTIVES VARIABLES=income.\n"
    );

    let _ = std::fs::remove_file(file_path);
}

#[test]
fn test_save_data_as_csv() {
    let temp_dir = std::env::temp_dir();
    let csv_path = temp_dir.join("statcraft_session_test.csv");

    let mut app = StatCraftApp::new_headless();
    if let Some(ds) = &mut app.dataset {
        ds.add_variable("age", DataType::Numeric { decimals: 0 }, Measure::Scale)
            .unwrap();
    }

    app.save_to_path(csv_path.clone());
    assert!(csv_path.exists());
    assert!(matches!(
        app.status_message,
        statcraft_app::StatusMessage::DataSaved { .. }
    ));

    let _ = std::fs::remove_file(csv_path);
}

#[test]
fn test_load_spss_sav_session_lifecycle() {
    let temp_dir = std::env::temp_dir();
    let sav_path = temp_dir.join("statcraft_session_test.sav");

    // Build minimal valid synthetic SPSS .sav
    let mut buf = vec![0u8; 176];
    buf[0..4].copy_from_slice(b"$FL2");
    buf[64..68].copy_from_slice(&2i32.to_le_bytes()); // layout code = 2
    buf[68..72].copy_from_slice(&1i32.to_le_bytes()); // nominal case size = 1
    buf[72..76].copy_from_slice(&1i32.to_le_bytes()); // compression = 1
    buf[76..80].copy_from_slice(&0i32.to_le_bytes()); // weight index = 0
    buf[80..84].copy_from_slice(&1i32.to_le_bytes()); // case count = 1
    buf[84..92].copy_from_slice(&100.0f64.to_le_bytes()); // bias = 100.0

    // Record 2: Variable SALARY
    buf.extend_from_slice(&2i32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes()); // numeric
    buf.extend_from_slice(&1i32.to_le_bytes()); // has label
    buf.extend_from_slice(&0i32.to_le_bytes()); // no missing
    buf.extend_from_slice(&0x00050802u32.to_le_bytes()); // print format F8.2
    buf.extend_from_slice(&0x00050802u32.to_le_bytes()); // write format
    buf.extend_from_slice(b"SALARY  "); // 8 bytes name
    let label = "Current Salary";
    buf.extend_from_slice(&(label.len() as i32).to_le_bytes());
    buf.extend_from_slice(label.as_bytes());
    let pad = (4 - (label.len() % 4)) % 4;
    buf.resize(buf.len() + pad, 0);

    // Record 999: Data
    buf.extend_from_slice(&999i32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes()); // filler

    // 1 case: salary = 50 (code = 150)
    let mut opcodes = [0u8; 8];
    opcodes[0] = 150; // 150 - 100 = 50
    opcodes[1] = 252; // EOF
    buf.extend_from_slice(&opcodes);

    std::fs::write(&sav_path, &buf).unwrap();

    let mut app = StatCraftApp::new_headless();
    app.load_file_from_path(sav_path.clone());

    assert_eq!(app.current_file_path, Some(sav_path.clone()));
    assert_eq!(app.recent_files.paths[0], sav_path);
    assert!(matches!(
        app.status_message,
        statcraft_app::StatusMessage::DatasetImported { .. }
    ));

    let ds = app.dataset.as_ref().unwrap();
    assert_eq!(ds.row_count(), 1);
    assert_eq!(ds.col_count(), 1);
    assert_eq!(ds.dictionary.get(0).unwrap().name, "SALARY");
    assert_eq!(ds.dictionary.get(0).unwrap().label, "Current Salary");

    let has_get_file_log = app.output_view_state.procedures.iter().any(|proc| {
        proc.items.iter().any(|item| {
            if let OutputItemKind::Log(text) = &item.kind {
                text.contains("GET FILE=") && text.contains("statcraft_session_test.sav")
            } else {
                false
            }
        })
    });
    assert!(has_get_file_log, "Output log must contain GET FILE='...'.");

    let _ = std::fs::remove_file(sav_path);
}
