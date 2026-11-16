use crate::grid::{
    apply_auto_fill, parse_tsv_matrix, paste_matrix_into_dataset, render_grid,
    serialize_range_to_tsv, EditorAction, EditorMove, Geo, GridConfig, GridEditorState, GridEvent,
    GridInteractState, GridRenderParams, GridSelection, KeyboardEvent, LineIndex,
};
use egui::{Color32, FontId, Ui, Vec2};
use statcraft_core::DataSet;
use statcraft_i18n::t;
use std::collections::HashMap;

/// Persistent UI state for the DataView spreadsheet grid.
#[derive(Debug, Clone)]
pub struct DataViewState {
    pub show_value_labels: bool,
    pub selection: GridSelection,
    pub editor_state: GridEditorState,
    pub scroll_to_row: Option<usize>,
    pub scroll_to_col: Option<usize>,
    pub scroll: Vec2,
    pub col_widths: HashMap<usize, f32>,
    pub interact_state: GridInteractState,

    // Backward-compatibility mirrors
    pub selected_cell: Option<(usize, usize)>,
    pub editing_cell: Option<(usize, usize)>,
    pub edit_buffer: String,
    pub focus_edit: bool,
}

impl Default for DataViewState {
    fn default() -> Self {
        Self {
            show_value_labels: true,
            selection: GridSelection::single(0, 0),
            editor_state: GridEditorState::default(),
            scroll_to_row: None,
            scroll_to_col: None,
            scroll: Vec2::ZERO,
            col_widths: HashMap::new(),
            interact_state: GridInteractState::default(),
            selected_cell: Some((0, 0)),
            editing_cell: None,
            edit_buffer: String::new(),
            focus_edit: false,
        }
    }
}

pub fn render_data_view(ui: &mut Ui, dataset: &mut Option<DataSet>, state: &mut DataViewState) {
    let ds = match dataset {
        Some(d) => d,
        None => {
            ui.centered_and_justified(|ui| {
                ui.label(t!("output-no-analysis"));
            });
            return;
        }
    };

    let total_rows = ds.row_count();
    let total_cols = ds.col_count();

    // Sync external selected_cell modifications (e.g. from GoTo dialog)
    if let Some((r, c)) = state.selected_cell {
        if (r, c) != state.selection.active_cell {
            state.selection = GridSelection::single(r, c);
        }
    }

    // Toolbar for toggling value labels and displaying dataset stats
    ui.horizontal(|ui| {
        let label_text = format!("1/A  {}", t!("toolbar-toggle-labels"));
        if ui
            .selectable_label(state.show_value_labels, label_text)
            .clicked()
        {
            state.show_value_labels = !state.show_value_labels;
        }
        ui.separator();
        ui.label(t!("status-rows-count", count = total_rows));
        ui.label(t!("status-cols-count", count = total_cols));
    });

    ui.separator();

    if total_cols == 0 {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(t!("output-no-analysis"));
            ui.add_space(8.0);
            if ui.button(t!("menu-insert-variable")).clicked() {
                let _ = ds.add_variable(
                    "VAR00001",
                    statcraft_core::DataType::Numeric { decimals: 2 },
                    statcraft_core::Measure::Scale,
                );
            }
        });
        return;
    }

    let row_height = 22.0_f32;
    let header_width = 50.0_f32;
    let header_height = 24.0_f32;

    // 1 ghost row at bottom for appending new cases
    let display_rows = total_rows + 1;
    let rows_index = LineIndex::uniform(display_rows, row_height);

    // Build column index from custom widths or VariableMetadata.columns (SPSS parity)
    let default_col_w = 120.0_f32;
    let mut overrides = HashMap::new();
    for col_idx in 0..total_cols {
        if let Some(&w) = state.col_widths.get(&col_idx) {
            overrides.insert(col_idx, w);
        } else if let Some(var) = ds.dictionary.get(col_idx) {
            let w = (var.columns as f32 * 10.0 + 30.0).clamp(50.0, 600.0);
            overrides.insert(col_idx, w);
        }
    }
    let cols_index = LineIndex::from_default_and_overrides(total_cols, default_col_w, &overrides);

    // Handle programmatic scroll-to requests (e.g. from GoTo dialog)
    if state.scroll_to_row.is_some() || state.scroll_to_col.is_some() {
        let target_r = state.scroll_to_row.take().unwrap_or(0);
        let target_c = state.scroll_to_col.take().unwrap_or(0);
        let temp_geo = Geo::new(
            ui.available_rect_before_wrap(),
            header_width,
            header_height,
            rows_index.clone(),
            cols_index.clone(),
            state.scroll,
        );
        state.scroll = temp_geo.scroll_to_cell(target_r, target_c);
    }

    let config = GridConfig {
        header_width,
        header_height,
        corner_label: t!("header-row"),
        min_col_width: 40.0,
        max_col_width: 800.0,
    };

    let result = render_grid(
        ui,
        GridRenderParams {
            config: &config,
            rows: rows_index,
            cols: cols_index,
            scroll: &mut state.scroll,
            selection: &mut state.selection,
            editor_state: &mut state.editor_state,
            interact_state: &mut state.interact_state,
        },
        |col| {
            let var_meta = ds.dictionary.get(col);
            let name = var_meta
                .map(|v| v.name.clone())
                .unwrap_or_else(|| "VAR".to_string());
            let measure = var_meta
                .map(|v| v.measure)
                .unwrap_or(statcraft_core::Measure::Nominal);
            (name, measure)
        },
        |row| {
            let label = if row < total_rows {
                format!("{}", row + 1)
            } else {
                "*".to_string()
            };
            let is_active = if row < total_rows {
                ds.is_row_active(row)
            } else {
                true
            };
            (label, is_active)
        },
        |row, col| {
            if row < total_rows {
                ds.get_cell_display(row, col, state.show_value_labels)
            } else {
                String::new()
            }
        },
    );

    // 1. Handle Inline Editor Commit / Cancel
    if let Some(action) = result.editor_action {
        match action {
            EditorAction::Commit {
                row,
                col,
                text,
                move_direction,
            } => {
                while row >= ds.row_count() {
                    ds.add_empty_row();
                }
                let _ = ds.set_cell_parsed(row, col, &text);

                if let Some(dir) = move_direction {
                    let mut next_r = row;
                    let mut next_c = col;
                    match dir {
                        EditorMove::Down => {
                            next_r = (row + 1).min(display_rows.saturating_sub(1));
                        }
                        EditorMove::Up => {
                            next_r = row.saturating_sub(1);
                        }
                        EditorMove::Right => {
                            if col + 1 < total_cols {
                                next_c = col + 1;
                            } else {
                                next_r = (row + 1).min(display_rows.saturating_sub(1));
                                next_c = 0;
                            }
                        }
                        EditorMove::Left => {
                            if col > 0 {
                                next_c = col - 1;
                            } else if row > 0 {
                                next_r = row - 1;
                                next_c = total_cols.saturating_sub(1);
                            }
                        }
                    }
                    state.selection = GridSelection::single(next_r, next_c);
                    state.scroll = result.geo.scroll_to_cell(next_r, next_c);
                }
            }
            EditorAction::Cancel => {}
        }
    }

    // 2. Handle Keyboard Events (Direct Typing, Copy, Paste, Clear)
    for ev in result.kb_events {
        match ev {
            KeyboardEvent::StartDirectTyping(ch) => {
                let (r, c) = state.selection.active_cell;
                state.editor_state.start_editing(r, c, ch.to_string());
            }
            KeyboardEvent::ClearSelected => {
                for r in state.selection.range.min_row..=state.selection.range.max_row {
                    for c in state.selection.range.min_col..=state.selection.range.max_col {
                        if r < ds.row_count() && c < ds.col_count() {
                            let _ = ds.set_cell_parsed(r, c, "");
                        }
                    }
                }
            }
            KeyboardEvent::Copy => {
                let tsv =
                    serialize_range_to_tsv(ds, &state.selection.range, state.show_value_labels);
                ui.output_mut(|o| o.copied_text = tsv);
            }
            KeyboardEvent::Paste => {
                let paste_text = ui.input(|i| {
                    for event in &i.events {
                        if let egui::Event::Paste(text) = event {
                            return Some(text.clone());
                        }
                    }
                    None
                });

                if let Some(text) = paste_text {
                    let matrix = parse_tsv_matrix(&text);
                    let (start_r, start_c) = state.selection.active_cell;
                    let (rows_p, cols_p) = paste_matrix_into_dataset(ds, start_r, start_c, &matrix);
                    if rows_p > 0 && cols_p > 0 {
                        state.selection = GridSelection::from_anchor_and_active(
                            (start_r, start_c),
                            (start_r + rows_p - 1, start_c + cols_p - 1),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    // 3. Handle Mouse Interaction Events
    for event in result.mouse_events {
        match event {
            GridEvent::DoubleClickCell { row, col } => {
                let cur_val = if row < total_rows {
                    ds.get_cell_display(row, col, false)
                } else {
                    String::new()
                };
                let buffer = if cur_val == "." {
                    String::new()
                } else {
                    cur_val
                };
                state.editor_state.start_editing(row, col, buffer);
            }
            GridEvent::ResizeCol { col, new_width } => {
                state.col_widths.insert(col, new_width);
                let char_cols = ((new_width - 30.0) / 10.0).round().max(1.0) as usize;
                if let Some(var) = ds.dictionary.get_mut(col) {
                    var.columns = char_cols;
                }
            }
            GridEvent::AutoFitCol { col } => {
                let mut max_w = 50.0_f32;
                if let Some(var) = ds.dictionary.get(col) {
                    let text_w = ui.fonts(|f| {
                        f.layout_no_wrap(
                            var.name.clone(),
                            FontId::proportional(12.0),
                            Color32::WHITE,
                        )
                        .size()
                        .x
                    });
                    max_w = max_w.max(text_w + 40.0);
                }
                let check_rows = total_rows.min(100);
                for r in 0..check_rows {
                    let val_str = ds.get_cell_display(r, col, state.show_value_labels);
                    if !val_str.is_empty() {
                        let text_w = ui.fonts(|f| {
                            f.layout_no_wrap(val_str, FontId::proportional(12.0), Color32::WHITE)
                                .size()
                                .x
                        });
                        max_w = max_w.max(text_w + 24.0);
                    }
                }
                let fitted_w = max_w.clamp(60.0, 600.0);
                state.col_widths.insert(col, fitted_w);
                let char_cols = ((fitted_w - 30.0) / 10.0).round().max(1.0) as usize;
                if let Some(var) = ds.dictionary.get_mut(col) {
                    var.columns = char_cols;
                }
                ui.ctx().request_repaint();
            }
            GridEvent::AutoFillSeries {
                source_range,
                target_range,
            } => {
                apply_auto_fill(ds, &source_range, &target_range);
                state.selection = GridSelection::from_anchor_and_active(
                    (target_range.min_row, target_range.min_col),
                    (target_range.max_row, target_range.max_col),
                );
                ui.ctx().request_repaint();
            }
            _ => {}
        }
    }

    // Mirror to backward-compatible fields
    state.selected_cell = Some(state.selection.active_cell);
    state.editing_cell = state.editor_state.editing_cell;
    state.edit_buffer = state.editor_state.edit_buffer.clone();
    state.focus_edit = state.editor_state.focus_requested;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_view_state_defaults() {
        let state = DataViewState::default();
        assert!(state.show_value_labels);
        assert_eq!(state.selection.active_cell, (0, 0));
        assert_eq!(state.scroll, Vec2::ZERO);
        assert!(state.col_widths.is_empty());
    }
}
