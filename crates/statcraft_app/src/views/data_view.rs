use crate::theme::Tokens;
use crate::widgets::paint_measure_icon;
use egui::{pos2, vec2, Align2, FontId, Key, Rect, ScrollArea, Sense, Stroke, Ui};
use statcraft_core::DataSet;
use statcraft_i18n::t;

pub struct DataViewState {
    pub show_value_labels: bool,
    pub selected_cell: Option<(usize, usize)>,
    pub editing_cell: Option<(usize, usize)>,
    pub edit_buffer: String,
    pub focus_edit: bool,
}

impl Default for DataViewState {
    fn default() -> Self {
        Self {
            show_value_labels: true,
            selected_cell: None,
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

    let row_height = 22.0_f32;
    let col_width = 130.0_f32;
    let row_header_width = 50.0_f32;

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

    let mut action_insert_row = None;
    let mut action_delete_row = None;
    let mut commit_edit = None;
    let mut next_edit_target = None;

    // Outer horizontal scroll area so headers and data rows stay in sync horizontally
    ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 1. Pinned Column Headers Row
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                // Top-left corner (Row number column header)
                let (rect, _) =
                    ui.allocate_exact_size(vec2(row_header_width, row_height), Sense::hover());
                ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
                ui.painter()
                    .rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    t!("header-row"),
                    FontId::proportional(12.0),
                    Tokens::TEXT_PRIMARY,
                );

                // Variable column headers
                for col_idx in 0..total_cols {
                    let var_meta = ds.dictionary.get(col_idx);
                    let col_name = var_meta.map(|v| v.name.as_str()).unwrap_or("VAR");
                    let measure = var_meta
                        .map(|v| v.measure)
                        .unwrap_or(statcraft_core::Measure::Nominal);

                    let (rect, _) =
                        ui.allocate_exact_size(vec2(col_width, row_height), Sense::hover());
                    ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
                    ui.painter()
                        .rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));

                    // Vector measure icon
                    let icon_rect = Rect::from_min_size(
                        pos2(rect.min.x + 4.0_f32, rect.min.y + 3.0_f32),
                        vec2(16.0_f32, 16.0_f32),
                    );
                    paint_measure_icon(ui.painter(), icon_rect, measure);

                    // Column name with clipping
                    let text_clip_rect =
                        Rect::from_min_max(pos2(rect.min.x + 22.0_f32, rect.min.y), rect.max);
                    ui.painter().with_clip_rect(text_clip_rect).text(
                        pos2(rect.min.x + 24.0_f32, rect.center().y),
                        Align2::LEFT_CENTER,
                        col_name,
                        FontId::proportional(12.0),
                        Tokens::TEXT_PRIMARY,
                    );
                }
            });

            // 2. Data Rows (+ 1 ghost row at bottom for appending new cases)
            let display_rows = total_rows + 1;
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, row_height, display_rows, |ui, row_range| {
                    for row_idx in row_range {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                            // Row index header cell with Context Menu
                            let (rect, res) = ui.allocate_exact_size(
                                vec2(row_header_width, row_height),
                                Sense::click(),
                            );
                            ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
                            ui.painter().rect_stroke(
                                rect,
                                0.0_f32,
                                Stroke::new(0.5_f32, Tokens::BORDER),
                            );

                            let row_label = if row_idx < total_rows {
                                format!("{}", row_idx + 1)
                            } else {
                                "*".to_string()
                            };

                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                row_label,
                                FontId::proportional(11.0),
                                Tokens::TEXT_MUTED,
                            );

                            if row_idx < total_rows {
                                res.context_menu(|ui| {
                                    if ui.button(t!("menu-insert-case")).clicked() {
                                        action_insert_row = Some(row_idx);
                                        ui.close_menu();
                                    }
                                    if ui.button(t!("menu-delete-case")).clicked() {
                                        action_delete_row = Some(row_idx);
                                        ui.close_menu();
                                    }
                                });
                            }

                            // Data cells for this row
                            for col_idx in 0..total_cols {
                                let is_selected = state.selected_cell == Some((row_idx, col_idx));
                                let is_editing = state.editing_cell == Some((row_idx, col_idx));

                                let bg_color = if is_selected {
                                    Tokens::BG_SELECTED
                                } else if row_idx % 2 == 0 {
                                    Tokens::BG_ROW_EVEN
                                } else {
                                    Tokens::BG_ROW_ODD
                                };

                                let (rect, response) = ui.allocate_exact_size(
                                    vec2(col_width, row_height),
                                    Sense::click(),
                                );

                                if response.clicked() && !is_editing {
                                    if let Some((cur_r, cur_c)) = state.editing_cell {
                                        commit_edit =
                                            Some((cur_r, cur_c, state.edit_buffer.clone()));
                                        state.editing_cell = None;
                                    }
                                    state.selected_cell = Some((row_idx, col_idx));
                                }

                                if response.double_clicked() && !is_editing {
                                    let current_val = if row_idx < total_rows {
                                        ds.get_cell_display(row_idx, col_idx, false)
                                    } else {
                                        String::new()
                                    };
                                    let buffer = if current_val == "." {
                                        String::new()
                                    } else {
                                        current_val
                                    };
                                    state.edit_buffer = buffer;
                                    state.editing_cell = Some((row_idx, col_idx));
                                    state.focus_edit = true;
                                }

                                ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                ui.painter().rect_stroke(
                                    rect,
                                    0.0_f32,
                                    Stroke::new(0.5_f32, Tokens::BORDER),
                                );

                                if is_editing {
                                    let inner_rect = rect.shrink2(vec2(2.0_f32, 1.0_f32));
                                    ui.allocate_new_ui(
                                        egui::UiBuilder::new().max_rect(inner_rect),
                                        |ui| {
                                            let text_edit =
                                                egui::TextEdit::singleline(&mut state.edit_buffer)
                                                    .frame(false)
                                                    .clip_text(true);
                                            let text_res =
                                                ui.add_sized(inner_rect.size(), text_edit);

                                            if state.focus_edit {
                                                text_res.request_focus();
                                                state.focus_edit = false;
                                            }

                                            let enter_pressed =
                                                ui.input(|i| i.key_pressed(Key::Enter));
                                            let tab_pressed = ui.input(|i| i.key_pressed(Key::Tab));
                                            let shift = ui.input(|i| i.modifiers.shift);
                                            let esc_pressed =
                                                ui.input(|i| i.key_pressed(Key::Escape));

                                            if enter_pressed {
                                                commit_edit = Some((
                                                    row_idx,
                                                    col_idx,
                                                    state.edit_buffer.clone(),
                                                ));
                                                state.editing_cell = None;
                                                next_edit_target = Some((row_idx + 1, col_idx));
                                            } else if tab_pressed {
                                                commit_edit = Some((
                                                    row_idx,
                                                    col_idx,
                                                    state.edit_buffer.clone(),
                                                ));
                                                state.editing_cell = None;
                                                if shift {
                                                    if col_idx > 0 {
                                                        next_edit_target =
                                                            Some((row_idx, col_idx - 1));
                                                    }
                                                } else if col_idx + 1 < total_cols {
                                                    next_edit_target = Some((row_idx, col_idx + 1));
                                                } else {
                                                    next_edit_target = Some((row_idx + 1, 0));
                                                }
                                            } else if esc_pressed {
                                                state.editing_cell = None;
                                            } else if text_res.lost_focus()
                                                && !enter_pressed
                                                && !tab_pressed
                                                && !esc_pressed
                                            {
                                                commit_edit = Some((
                                                    row_idx,
                                                    col_idx,
                                                    state.edit_buffer.clone(),
                                                ));
                                                state.editing_cell = None;
                                            }
                                        },
                                    );
                                } else {
                                    let display_val = if row_idx < total_rows {
                                        ds.get_cell_display(
                                            row_idx,
                                            col_idx,
                                            state.show_value_labels,
                                        )
                                    } else {
                                        String::new()
                                    };

                                    let text_clip_rect = rect.shrink2(vec2(4.0_f32, 1.0_f32));
                                    ui.painter().with_clip_rect(text_clip_rect).text(
                                        pos2(rect.min.x + 6.0_f32, rect.center().y),
                                        Align2::LEFT_CENTER,
                                        display_val,
                                        FontId::proportional(12.0),
                                        Tokens::TEXT_PRIMARY,
                                    );
                                }
                            }
                        });
                    }
                });
        });

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

    if action_insert_row.is_some() {
        ds.add_empty_row();
    }
    if let Some(r) = action_delete_row {
        let _ = ds.remove_row(r);
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
    }
}
