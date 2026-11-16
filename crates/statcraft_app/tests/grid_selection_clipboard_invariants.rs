use egui::{pos2, vec2, Rect};
use statcraft_app::grid::{
    apply_auto_fill, parse_tsv_matrix, paste_matrix_into_dataset, serialize_range_to_tsv,
    CellRange, EditorAction, EditorMove, Geo, GridEditorState, GridSelection, LineIndex,
};
use statcraft_core::{DataSet, DataType, Measure};

#[test]
fn test_marquee_selection_bounds_and_normalization() {
    let range_inverted = CellRange::from_corners(5, 4, 1, 2);

    assert_eq!(range_inverted.min_row, 1);
    assert_eq!(range_inverted.max_row, 5);
    assert_eq!(range_inverted.min_col, 2);
    assert_eq!(range_inverted.max_col, 4);

    assert_eq!(range_inverted.row_count(), 5);
    assert_eq!(range_inverted.col_count(), 3);
    assert!(range_inverted.contains_cell(3, 3));
    assert!(!range_inverted.contains_cell(0, 3));
    assert!(!range_inverted.contains_cell(3, 5));

    let sel = GridSelection::from_anchor_and_active((1, 2), (3, 4));
    let vp = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
    let rows = LineIndex::uniform(100, 24.0);
    let cols = LineIndex::uniform(50, 80.0);
    let geo = Geo::new(vp, 50.0, 24.0, rows, cols, vec2(0.0, 0.0));

    let screen_rect = sel.bounding_rect(&geo).expect("must be in bounds");
    assert!(screen_rect.width() > 0.0);
    assert!(screen_rect.height() > 0.0);

    let handle_rect = sel.fill_handle_rect(&geo).expect("must have fill handle");
    assert_eq!(handle_rect.size(), vec2(6.0, 6.0));
    assert_eq!(handle_rect.center(), screen_rect.max);
}

#[test]
fn test_tsv_clipboard_copy_paste_invariants() {
    let mut ds = DataSet::empty();
    ds.add_variable("var_num", DataType::Numeric { decimals: 0 }, Measure::Scale)
        .expect("var 1");
    ds.add_variable(
        "var_str",
        DataType::String { max_len: 20 },
        Measure::Nominal,
    )
    .expect("var 2");

    for i in 0..5 {
        ds.add_empty_row();
        let _ = ds.set_cell_parsed(i, 0, &(i * 10).to_string());
        let _ = ds.set_cell_parsed(i, 1, &format!("text_{i}"));
    }

    // 1. Serialize range (0..2, 0..1)
    let range = CellRange::from_corners(0, 0, 2, 1);
    let tsv = serialize_range_to_tsv(&ds, &range, false);
    assert!(tsv.contains("0\ttext_0\n"));
    assert!(tsv.contains("10\ttext_1\n"));
    assert!(tsv.contains("20\ttext_2"));

    // 2. Parse TSV matrix
    let matrix = parse_tsv_matrix(&tsv);
    assert_eq!(matrix.len(), 3);
    assert_eq!(matrix[0], vec!["0", "text_0"]);
    assert_eq!(matrix[1], vec!["10", "text_1"]);
    assert_eq!(matrix[2], vec!["20", "text_2"]);

    // 3. Paste matrix into offset (2, 0)
    let (rows_pasted, cols_pasted) = paste_matrix_into_dataset(&mut ds, 2, 0, &matrix);
    assert_eq!(rows_pasted, 3);
    assert_eq!(cols_pasted, 2);

    // Verify cell at (2, 0) has 0 and at (3, 0) has 10
    let v20 = ds.get_cell_display(2, 0, false);
    assert_eq!(v20, "0");
    let v30 = ds.get_cell_display(3, 0, false);
    assert_eq!(v30, "10");
}

#[test]
fn test_auto_fill_series_and_repetition() {
    let mut ds = DataSet::empty();
    ds.add_variable("seq", DataType::Numeric { decimals: 0 }, Measure::Scale)
        .expect("var");

    for _ in 0..8 {
        ds.add_empty_row();
    }

    // Seed rows 0..2 with 10, 20, 30
    let _ = ds.set_cell_parsed(0, 0, "10");
    let _ = ds.set_cell_parsed(1, 0, "20");
    let _ = ds.set_cell_parsed(2, 0, "30");

    // Auto-fill from 0..2 down through row 5
    let source_range = CellRange::from_corners(0, 0, 2, 0);
    let target_range = CellRange::from_corners(0, 0, 5, 0);

    apply_auto_fill(&mut ds, &source_range, &target_range);

    assert_eq!(ds.get_cell_display(3, 0, false), "40");
    assert_eq!(ds.get_cell_display(4, 0, false), "50");
    assert_eq!(ds.get_cell_display(5, 0, false), "60");
}

#[test]
fn test_direct_typing_and_editor_invariants() {
    let mut editor = GridEditorState::default();
    assert!(!editor.is_editing());

    // Start editing cell (3, 2) with initial character '9'
    editor.start_editing(3, 2, "9".to_string());
    assert!(editor.is_editing());
    assert_eq!(editor.editing_cell, Some((3, 2)));
    assert_eq!(editor.edit_buffer, "9");

    // Commit buffer and move down (Enter)
    editor.edit_buffer = "987.65".to_string();
    let action = EditorAction::Commit {
        row: 3,
        col: 2,
        text: editor.edit_buffer.clone(),
        move_direction: Some(EditorMove::Down),
    };

    match action {
        EditorAction::Commit {
            row,
            col,
            text,
            move_direction,
        } => {
            assert_eq!(row, 3);
            assert_eq!(col, 2);
            assert_eq!(text, "987.65");
            assert_eq!(move_direction, Some(EditorMove::Down));
            editor.stop();
            assert!(!editor.is_editing());
        }
        _ => panic!("Expected Commit"),
    }
}
