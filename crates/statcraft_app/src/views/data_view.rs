use crate::grid::{
    render_grid, Geo, GridConfig, GridEvent, GridInteractState, GridRenderParams, LineIndex,
};
use egui::{vec2, Key, Ui, Vec2};
use statcraft_core::DataSet;
use statcraft_i18n::t;
use std::collections::HashMap;

/// Persistent UI state for the DataView spreadsheet grid.
#[derive(Debug, Clone)]
pub struct DataViewState {
    pub show_value_labels: bool,
    pub selected_cell: Option<(usize, usize)>,
    pub editing_cell: Option<(usize, usize)>,
    pub edit_buffer: String,
    pub focus_edit: bool,
    pub scroll_to_row: Option<usize>,
    pub scroll_to_col: Option<usize>,
    pub scroll: Vec2,
    pub col_widths: HashMap<usize, f32>,
    pub interact_state: GridInteractState,
}

impl Default for DataViewState {
    fn default() -> Self {
        Self {
            show_value_labels: true,
            selected_cell: None,
            editing_cell: None,
            edit_buffer: String::new(),
            focus_edit: false,
            scroll_to_row: None,
            scroll_to_col: None,
            scroll: Vec2::ZERO,
            col_widths: HashMap::new(),
            interact_state: GridInteractState::default(),
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

    let (geo, events) = render_grid(
        ui,
        GridRenderParams {
            config: &config,
            rows: rows_index,
            cols: cols_index,
            scroll: &mut state.scroll,
            interact_state: &mut state.interact_state,
            selected_cell: state.selected_cell,
            editing_cell: state.editing_cell,
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

    // Render active inline TextEdit on top of editing cell
    let mut commit_edit = None;
    let mut next_edit_target = None;

    if let Some((r, c)) = state.editing_cell {
        if let Some(cell_rect) = geo.cell_rect(r, c) {
            let inner_rect = cell_rect.shrink2(vec2(2.0_f32, 1.0_f32));
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                let text_edit = egui::TextEdit::singleline(&mut state.edit_buffer)
                    .frame(false)
                    .clip_text(true);
                let text_res = ui.add_sized(inner_rect.size(), text_edit);

                if state.focus_edit {
                    text_res.request_focus();
                    state.focus_edit = false;
                }

                let enter_pressed = ui.input(|i| i.key_pressed(Key::Enter));
                let tab_pressed = ui.input(|i| i.key_pressed(Key::Tab));
                let shift = ui.input(|i| i.modifiers.shift);
                let esc_pressed = ui.input(|i| i.key_pressed(Key::Escape));

                if enter_pressed {
                    commit_edit = Some((r, c, state.edit_buffer.clone()));
                    state.editing_cell = None;
                    next_edit_target = Some((r + 1, c));
                } else if tab_pressed {
                    commit_edit = Some((r, c, state.edit_buffer.clone()));
                    state.editing_cell = None;
                    if shift {
                        if c > 0 {
                            next_edit_target = Some((r, c - 1));
                        }
                    } else if c + 1 < total_cols {
                        next_edit_target = Some((r, c + 1));
                    } else {
                        next_edit_target = Some((r + 1, 0));
                    }
                } else if esc_pressed {
                    state.editing_cell = None;
                } else if text_res.lost_focus() && !enter_pressed && !tab_pressed && !esc_pressed {
                    commit_edit = Some((r, c, state.edit_buffer.clone()));
                    state.editing_cell = None;
                }
            });
        }
    }

    // Process user interaction events
    for event in events {
        match event {
            GridEvent::SelectCell { row, col } => {
                if let Some((cur_r, cur_c)) = state.editing_cell {
                    commit_edit = Some((cur_r, cur_c, state.edit_buffer.clone()));
                    state.editing_cell = None;
                }
                state.selected_cell = Some((row, col));
            }
            GridEvent::DoubleClickCell { row, col } => {
                if let Some((cur_r, cur_c)) = state.editing_cell {
                    commit_edit = Some((cur_r, cur_c, state.edit_buffer.clone()));
                }
                state.selected_cell = Some((row, col));
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
                state.edit_buffer = buffer;
                state.editing_cell = Some((row, col));
                state.focus_edit = true;
            }
            GridEvent::ResizeCol { col, new_width } => {
                state.col_widths.insert(col, new_width);
                let char_cols = ((new_width - 30.0) / 10.0).round().max(1.0) as usize;
                if let Some(var) = ds.dictionary.get_mut(col) {
                    var.columns = char_cols;
                }
            }
            GridEvent::AutoFitCol { col } => {
                let mut max_w = 40.0_f32;
                if let Some(var) = ds.dictionary.get(col) {
                    max_w = max_w.max(var.name.len() as f32 * 8.5 + 36.0);
                }
                let check_rows = total_rows.min(100);
                for r in 0..check_rows {
                    let val_str = ds.get_cell_display(r, col, state.show_value_labels);
                    max_w = max_w.max(val_str.len() as f32 * 7.5 + 16.0);
                }
                let fitted_w = max_w.clamp(60.0, 500.0);
                state.col_widths.insert(col, fitted_w);
                let char_cols = ((fitted_w - 30.0) / 10.0).round().max(1.0) as usize;
                if let Some(var) = ds.dictionary.get_mut(col) {
                    var.columns = char_cols;
                }
            }
            GridEvent::SelectRow { row } => {
                state.selected_cell = Some((row, 0));
            }
            GridEvent::SelectCol { col } => {
                state.selected_cell = Some((0, col));
            }
            GridEvent::SelectAll => {
                state.selected_cell = Some((0, 0));
            }
            GridEvent::ContextMenuRow { row } => {
                state.selected_cell = Some((row, 0));
            }
            GridEvent::ContextMenuCol { col } => {
                state.selected_cell = Some((0, col));
            }
            GridEvent::ContextMenuCell { row, col } => {
                state.selected_cell = Some((row, col));
            }
            GridEvent::ScrollChanged { new_scroll } => {
                state.scroll = new_scroll;
            }
        }
    }

    if let Some((r, c, text)) = commit_edit {
        while r >= ds.row_count() {
            ds.add_empty_row();
        }
        let _ = ds.set_cell_parsed(r, c, &text);
    }

    if let Some((target_r, target_c)) = next_edit_target {
        state.selected_cell = Some((target_r, target_c));
        let cur_val = if target_r < ds.row_count() {
            ds.get_cell_display(target_r, target_c, false)
        } else {
            String::new()
        };
        state.edit_buffer = if cur_val == "." {
            String::new()
        } else {
            cur_val
        };
        state.editing_cell = Some((target_r, target_c));
        state.focus_edit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_view_state_defaults() {
        let state = DataViewState::default();
        assert!(state.show_value_labels);
        assert_eq!(state.selected_cell, None);
        assert_eq!(state.scroll, Vec2::ZERO);
        assert!(state.col_widths.is_empty());
    }
}
