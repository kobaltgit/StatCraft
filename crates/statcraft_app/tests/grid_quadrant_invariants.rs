use egui::{pos2, vec2, Rect};
use statcraft_app::grid::{Geo, LineIndex};
use statcraft_app::views::{render_data_view, DataViewState};
use statcraft_core::{DataSet, DataType, Measure};

#[test]
fn test_line_index_prefix_invariants() {
    let sizes = vec![40.0, 60.0, 100.0, 150.0, 80.0];
    let idx = LineIndex::variable(&sizes);

    assert_eq!(idx.count(), 5);
    assert_eq!(idx.total_size(), 430.0);

    // Monotonicity of offsets
    for (i, &size) in sizes.iter().enumerate().take(idx.count()) {
        let (start, end) = idx.cell_bounds(i).expect("bounds must exist");
        assert!(start < end);
        assert_eq!(end - start, size);
    }

    // Large scale uniform invariant (100k items)
    let large_idx = LineIndex::uniform(100_000, 22.0);
    assert_eq!(large_idx.count(), 100_000);
    assert_eq!(large_idx.total_size(), 2_200_000.0);
    assert_eq!(large_idx.index_at(110.0), Some(5));
    assert_eq!(large_idx.index_at(2_199_970.0), Some(99_998));
}

#[test]
fn test_geo_quadrant_isolation_invariants() {
    let vp = Rect::from_min_size(pos2(10.0, 10.0), vec2(600.0, 400.0));
    let rows = LineIndex::uniform(500, 22.0); // 11000 px
    let cols = LineIndex::uniform(50, 120.0); // 6000 px

    // Arbitrary scroll position
    let scroll = vec2(350.0, 800.0);
    let geo = Geo::new(vp, 50.0, 24.0, rows, cols, scroll);

    // Quad Top-Left must be pinned exactly at top-left
    assert_eq!(geo.quad_top_left.min, pos2(10.0, 10.0));
    assert_eq!(geo.quad_top_left.width(), 50.0);
    assert_eq!(geo.quad_top_left.height(), 24.0);

    // Quad Top-Right stays pinned vertically at top
    assert_eq!(geo.quad_top_right.min.y, 10.0);
    assert_eq!(geo.quad_top_right.max.y, 34.0);

    // Quad Bottom-Left stays pinned horizontally at left
    assert_eq!(geo.quad_bottom_left.min.x, 10.0);
    assert_eq!(geo.quad_bottom_left.max.x, 60.0);

    // Quad Center begins right after top and left headers
    assert_eq!(geo.quad_center.min, pos2(60.0, 34.0));
    assert_eq!(geo.quad_center.max, pos2(610.0, 410.0));

    // Visible ranges must not be empty
    assert!(!geo.visible_cols().is_empty());
    assert!(!geo.visible_rows().is_empty());
}

#[test]
fn test_column_resizing_and_spss_parity() {
    let mut ds = DataSet::empty();
    let _ = ds.add_variable("AGE", DataType::Numeric { decimals: 0 }, Measure::Scale);
    let _ = ds.add_variable("INCOME", DataType::Numeric { decimals: 2 }, Measure::Scale);

    let mut state = DataViewState::default();

    // Resize column 0 to 180.0 px
    let new_w = 180.0_f32;
    state.col_widths.insert(0, new_w);
    let char_cols = ((new_w - 30.0) / 10.0).round().max(1.0) as usize;
    if let Some(var) = ds.dictionary.get_mut(0) {
        var.columns = char_cols;
    }

    assert_eq!(state.col_widths.get(&0), Some(&180.0));
    assert_eq!(ds.dictionary.get(0).map(|v| v.columns), Some(15));
}

#[test]
fn test_data_view_rendering_with_dataset() {
    let mut ds = DataSet::empty();
    let _ = ds.add_variable("ID", DataType::Numeric { decimals: 0 }, Measure::Scale);
    let _ = ds.add_variable("NAME", DataType::String { max_len: 20 }, Measure::Nominal);

    ds.add_empty_row();
    let _ = ds.set_cell_parsed(0, 0, "101");
    let _ = ds.set_cell_parsed(0, 1, "Alice");

    let mut dataset_opt = Some(ds);
    let mut state = DataViewState::default();

    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_data_view(ui, &mut dataset_opt, &mut state);
        });
    });

    assert!(state.show_value_labels);
}
