use statcraft_app::theme::{color32_to_hsl, hex_to_color32};
use statcraft_core::stc_codec::{load_workspace_stc_from_path, save_workspace_stc_to_path};
use statcraft_core::{DataSet, WorkspaceManager};
use std::path::PathBuf;

#[test]
fn test_dataset_color_generation_contrast_and_persistence() {
    let temp_dir = std::env::temp_dir();
    let project_file = temp_dir.join("test_dynamic_colors_persistence.stc");

    let mut ws = WorkspaceManager::new();

    let ds1 = DataSet::blank(10, 0).unwrap();
    let ds2 = DataSet::blank(20, 0).unwrap();
    let ds3 = DataSet::blank(30, 0).unwrap();

    let id1 = ws.add_dataset(ds1, "merge_base.csv", Some(PathBuf::from("merge_base.csv")));
    let id2 = ws.add_dataset(
        ds2,
        "merge_add_cases.csv",
        Some(PathBuf::from("merge_add_cases.csv")),
    );
    let id3 = ws.add_dataset(
        ds3,
        "merge_add_vars.csv",
        Some(PathBuf::from("merge_add_vars.csv")),
    );

    let hex1 = ws.get_session(id1).unwrap().tab_color.clone().unwrap();
    let hex2 = ws.get_session(id2).unwrap().tab_color.clone().unwrap();
    let hex3 = ws.get_session(id3).unwrap().tab_color.clone().unwrap();

    // Verify all 3 hex colors are distinct
    assert_ne!(hex1, hex2);
    assert_ne!(hex2, hex3);
    assert_ne!(hex1, hex3);

    // Verify hue angular distance >= 45 degrees
    let c1 = hex_to_color32(&hex1).unwrap();
    let c2 = hex_to_color32(&hex2).unwrap();
    let c3 = hex_to_color32(&hex3).unwrap();

    let h1 = color32_to_hsl(c1).0;
    let h2 = color32_to_hsl(c2).0;
    let h3 = color32_to_hsl(c3).0;

    let hue_dist = |a: f32, b: f32| {
        let diff = (a - b).abs().rem_euclid(360.0);
        if diff > 180.0 {
            360.0 - diff
        } else {
            diff
        }
    };

    assert!(
        hue_dist(h1, h2) >= 45.0,
        "Angular distance between tab 1 ({h1}) and tab 2 ({h2}) must be >= 45 deg, got {}",
        hue_dist(h1, h2)
    );
    assert!(
        hue_dist(h2, h3) >= 45.0,
        "Angular distance between tab 2 ({h2}) and tab 3 ({h3}) must be >= 45 deg, got {}",
        hue_dist(h2, h3)
    );
    assert!(
        hue_dist(h1, h3) >= 45.0,
        "Angular distance between tab 1 ({h1}) and tab 3 ({h3}) must be >= 45 deg, got {}",
        hue_dist(h1, h3)
    );

    // Verify session_color_by_name matches
    assert_eq!(
        ws.session_color_by_name("merge_base.csv"),
        Some(hex1.clone())
    );
    assert_eq!(
        ws.session_color_by_name("merge_add_cases.csv"),
        Some(hex2.clone())
    );
    assert_eq!(
        ws.session_color_by_name("merge_add_vars.csv"),
        Some(hex3.clone())
    );

    // Save project to .stc container
    save_workspace_stc_to_path(&project_file, &ws, None, None).unwrap();

    // Reload workspace from .stc container
    let (loaded_ws, _, _) = load_workspace_stc_from_path(&project_file).unwrap();
    assert_eq!(loaded_ws.session_count(), 3);

    // Verify colors are preserved 100% byte-for-byte
    assert_eq!(
        loaded_ws.sessions()[0].tab_color,
        Some(hex1.clone()),
        "Tab color for session 0 must be preserved byte-for-byte"
    );
    assert_eq!(
        loaded_ws.sessions()[1].tab_color,
        Some(hex2.clone()),
        "Tab color for session 1 must be preserved byte-for-byte"
    );
    assert_eq!(
        loaded_ws.sessions()[2].tab_color,
        Some(hex3.clone()),
        "Tab color for session 2 must be preserved byte-for-byte"
    );

    // Verify session_color_by_name on reloaded workspace
    assert_eq!(
        loaded_ws.session_color_by_name("merge_base.csv"),
        Some(hex1)
    );
    assert_eq!(
        loaded_ws.session_color_by_name("merge_add_cases.csv"),
        Some(hex2)
    );
    assert_eq!(
        loaded_ws.session_color_by_name("merge_add_vars.csv"),
        Some(hex3)
    );

    let _ = std::fs::remove_file(&project_file);
}
