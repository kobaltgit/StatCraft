use crate::theme::Tokens;
use egui::{pos2, Area, Color32, Id, Key, Order, Painter, Pos2, Rect, ScrollArea, Stroke, Ui};
use statcraft_core::DataSet;
use statcraft_i18n::t;
use std::collections::{HashMap, HashSet};

/// Definition of an active filter applied to a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnFilter {
    pub allowed_values: HashSet<String>,
    pub include_blanks: bool,
}

/// Persistent state for column auto-filters.
#[derive(Debug, Clone, Default)]
pub struct GridFilterState {
    pub active_filters: HashMap<usize, ColumnFilter>,
    pub open_col: Option<usize>,
    pub open_pos: Option<Pos2>,
    pub search_query: String,
    pub pending_allowed: HashSet<String>,
    pub pending_include_blanks: bool,
    pub just_opened: bool,
}

impl GridFilterState {
    /// Whether column `col` currently has an active filter applied.
    #[inline]
    pub fn is_col_filtered(&self, col: usize) -> bool {
        self.active_filters.contains_key(&col)
    }

    /// Whether any column in the grid currently has an active filter applied.
    #[inline]
    pub fn has_any_filter(&self) -> bool {
        !self.active_filters.is_empty()
    }

    /// Checks if a row satisfies all currently active column filters.
    pub fn is_row_matched(&self, ds: &DataSet, row: usize) -> bool {
        if self.active_filters.is_empty() {
            return true;
        }

        for (&col, filter) in &self.active_filters {
            let val = ds.get_cell_display(row, col, false);
            if val.is_empty() || val == "." {
                if !filter.include_blanks {
                    return false;
                }
            } else if !filter.allowed_values.contains(&val) {
                return false;
            }
        }

        true
    }

    /// Opens the auto-filter popup for column `col`.
    pub fn open_for_col(&mut self, col: usize, ds: &DataSet, button_pos: Pos2) {
        let (all_values, has_blanks) = ds.unique_values(col, false);

        if let Some(existing) = self.active_filters.get(&col) {
            self.pending_allowed = existing.allowed_values.clone();
            self.pending_include_blanks = existing.include_blanks;
        } else {
            self.pending_allowed = all_values.into_iter().collect();
            self.pending_include_blanks = has_blanks;
        }

        self.open_col = Some(col);
        self.open_pos = Some(button_pos);
        self.search_query.clear();
        self.just_opened = true;
    }

    /// Closes the currently open auto-filter popup.
    pub fn close(&mut self) {
        self.open_col = None;
        self.open_pos = None;
        self.search_query.clear();
        self.pending_allowed.clear();
        self.just_opened = false;
    }
}

/// Action produced by interaction with the auto-filter popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterMenuAction {
    SortAsc(usize),
    SortDesc(usize),
    ApplyFilter {
        col: usize,
        allowed_values: HashSet<String>,
        include_blanks: bool,
    },
    ClearFilter(usize),
    Close,
}

/// Renders a vector filter icon button in the right part of a column header.
pub fn paint_filter_button(
    painter: &Painter,
    button_rect: Rect,
    is_filtered: bool,
    is_hovered: bool,
) {
    let p = painter;

    let bg_color = if is_filtered {
        Tokens::accent()
    } else if is_hovered {
        Tokens::bg_panel()
    } else {
        Tokens::bg_header()
    };

    p.rect_filled(button_rect, 2.0_f32, bg_color);
    p.rect_stroke(button_rect, 2.0_f32, Stroke::new(0.5_f32, Tokens::border()));

    let center = button_rect.center();

    if is_filtered {
        // Draw funnel vector icon
        let top_w = 4.5_f32;
        let neck_w = 1.5_f32;
        let top_y = center.y - 4.0_f32;
        let mid_y = center.y - 0.5_f32;
        let bot_y = center.y + 4.5_f32;

        let points = vec![
            pos2(center.x - top_w, top_y),
            pos2(center.x + top_w, top_y),
            pos2(center.x + neck_w, mid_y),
            pos2(center.x + neck_w, bot_y),
            pos2(center.x - neck_w, bot_y),
            pos2(center.x - neck_w, mid_y),
        ];
        p.add(egui::Shape::convex_polygon(
            points,
            Color32::WHITE,
            Stroke::NONE,
        ));
    } else {
        // Draw downward chevron triangle
        let hw = 3.5_f32;
        let y0 = center.y - 2.0_f32;
        let y1 = center.y + 2.5_f32;

        let stroke = Stroke::new(1.2_f32, Tokens::text_muted());
        p.line_segment([pos2(center.x - hw, y0), pos2(center.x, y1)], stroke);
        p.line_segment([pos2(center.x, y1), pos2(center.x + hw, y0)], stroke);
    }
}

/// Renders the floating auto-filter dropdown popup menu.
pub fn render_filter_popup(
    ui: &mut Ui,
    state: &mut GridFilterState,
    ds: &DataSet,
) -> Option<FilterMenuAction> {
    let col = state.open_col?;
    let pos = state.open_pos?;

    let (unique_values, has_blanks) = ds.unique_values(col, false);
    let mut action = None;

    let popup_id = Id::new("grid_autofilter_popup");
    let area = Area::new(popup_id).order(Order::Foreground).fixed_pos(pos);

    let area_res = area.show(ui.ctx(), |ui| {
        let frame = egui::Frame::menu(ui.style())
            .fill(Tokens::bg_panel())
            .stroke(Stroke::new(1.0_f32, Tokens::border()))
            .inner_margin(egui::Margin::symmetric(8.0, 8.0));

        frame.show(ui, |ui| {
            ui.set_max_width(220.0_f32);

            // 1. Quick Sort Actions
            if ui.button(t!("grid-filter-sort-asc")).clicked() {
                action = Some(FilterMenuAction::SortAsc(col));
            }
            if ui.button(t!("grid-filter-sort-desc")).clicked() {
                action = Some(FilterMenuAction::SortDesc(col));
            }

            ui.separator();

            // 2. Search Box
            ui.label(t!("grid-filter-search"));
            ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text("...")
                    .desired_width(200.0_f32),
            );

            ui.add_space(4.0_f32);

            // 3. Filtered unique values list
            let query = state.search_query.to_lowercase();
            let matching_values: Vec<&String> = unique_values
                .iter()
                .filter(|v| query.is_empty() || v.to_lowercase().contains(&query))
                .collect();

            // Select All Checkbox
            let all_selected = matching_values
                .iter()
                .all(|v| state.pending_allowed.contains(*v))
                && (!has_blanks || state.pending_include_blanks);

            let mut select_all_state = all_selected;
            if ui
                .checkbox(&mut select_all_state, t!("grid-filter-select-all"))
                .changed()
            {
                if select_all_state {
                    for v in &matching_values {
                        state.pending_allowed.insert((*v).clone());
                    }
                    if has_blanks {
                        state.pending_include_blanks = true;
                    }
                } else {
                    for v in &matching_values {
                        state.pending_allowed.remove(*v);
                    }
                    if has_blanks {
                        state.pending_include_blanks = false;
                    }
                }
            }

            ui.separator();

            // 4. Scrollable Checkbox List
            ScrollArea::vertical().max_height(140.0_f32).show(ui, |ui| {
                if has_blanks && (query.is_empty() || "(blanks)".contains(&query)) {
                    ui.checkbox(&mut state.pending_include_blanks, t!("grid-filter-blanks"));
                }

                for val in matching_values {
                    let mut is_checked = state.pending_allowed.contains(val);
                    if ui.checkbox(&mut is_checked, val).changed() {
                        if is_checked {
                            state.pending_allowed.insert(val.clone());
                        } else {
                            state.pending_allowed.remove(val);
                        }
                    }
                }
            });

            ui.separator();

            // 5. Bottom Action Buttons: Clear & Apply
            ui.horizontal(|ui| {
                if state.is_col_filtered(col) && ui.button(t!("grid-filter-clear")).clicked() {
                    action = Some(FilterMenuAction::ClearFilter(col));
                }

                if ui.button(t!("grid-filter-apply")).clicked() {
                    action = Some(FilterMenuAction::ApplyFilter {
                        col,
                        allowed_values: state.pending_allowed.clone(),
                        include_blanks: state.pending_include_blanks,
                    });
                }
            });
        });
    });

    let popup_rect = area_res.response.rect;

    if ui.input(|i| i.key_pressed(Key::Escape)) {
        action = Some(FilterMenuAction::Close);
    } else if state.just_opened {
        state.just_opened = false;
    } else if ui.input(|i| i.pointer.any_pressed() || i.pointer.any_click()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()))
        {
            if !popup_rect.contains(pos) {
                action = Some(FilterMenuAction::Close);
            }
        }
    }

    if action.is_some() {
        state.close();
    }

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_filter_state_matching() {
        let mut ds = DataSet::blank(4, 2).unwrap();
        ds.set_cell(0, 0, statcraft_core::RawValue::Text("M".into()))
            .unwrap();
        ds.set_cell(1, 0, statcraft_core::RawValue::Text("F".into()))
            .unwrap();
        ds.set_cell(2, 0, statcraft_core::RawValue::Text("M".into()))
            .unwrap();
        ds.set_cell(3, 0, statcraft_core::RawValue::Null).unwrap();

        let mut state = GridFilterState::default();
        assert!(state.is_row_matched(&ds, 0));
        assert!(state.is_row_matched(&ds, 1));

        let mut allowed = HashSet::new();
        allowed.insert("M".to_string());
        state.active_filters.insert(
            0,
            ColumnFilter {
                allowed_values: allowed,
                include_blanks: false,
            },
        );

        assert!(state.is_col_filtered(0));
        assert!(!state.is_col_filtered(1));

        assert!(state.is_row_matched(&ds, 0));
        assert!(!state.is_row_matched(&ds, 1));
        assert!(state.is_row_matched(&ds, 2));
        assert!(!state.is_row_matched(&ds, 3));
    }
}
