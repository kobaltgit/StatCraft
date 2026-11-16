use super::geo::Geo;
use super::quadrant::QuadrantPaint;
use crate::theme::Tokens;
use egui::{pos2, vec2, CursorIcon, PointerButton, Pos2, Rect, Ui, Vec2};

/// Ongoing drag interactions within the grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridDrag {
    /// Resizing column `col`.
    ResizeCol {
        col: usize,
        start_mouse_x: f32,
        orig_width: f32,
    },
    /// Dragging vertical scrollbar thumb.
    ScrollThumbV {
        start_mouse_y: f32,
        orig_scroll_y: f32,
    },
    /// Dragging horizontal scrollbar thumb.
    ScrollThumbH {
        start_mouse_x: f32,
        orig_scroll_x: f32,
    },
}

/// Persistent interaction state across frames.
#[derive(Debug, Clone, Default)]
pub struct GridInteractState {
    pub drag: Option<GridDrag>,
    pub hovered_boundary: Option<(usize, f32)>,
    pub hovered_col: Option<usize>,
    pub hovered_row: Option<usize>,
    pub hovered_cell: Option<(usize, usize)>,
}

/// Action events emitted by user input on the grid.
#[derive(Debug, Clone, PartialEq)]
pub enum GridEvent {
    SelectCell { row: usize, col: usize },
    DoubleClickCell { row: usize, col: usize },
    ContextMenuCell { row: usize, col: usize },
    ContextMenuRow { row: usize },
    ContextMenuCol { col: usize },
    ResizeCol { col: usize, new_width: f32 },
    AutoFitCol { col: usize },
    SelectRow { row: usize },
    SelectCol { col: usize },
    SelectAll,
    ScrollChanged { new_scroll: Vec2 },
}

/// Handles all user input (hover, drag, click, scroll, resize) on the 4-quadrant grid.
pub fn handle_grid_interactions(
    ui: &mut Ui,
    geo: &Geo,
    state: &mut GridInteractState,
) -> Vec<GridEvent> {
    let mut events = Vec::new();
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());

    // 1. Wheel Scrolling
    handle_wheel_scroll(ui, geo, &mut events);

    // 2. Active Drag handling
    if let Some(drag) = state.drag {
        handle_active_drag(ui, geo, drag, state, &mut events);
        return events;
    }

    // If pointer is outside viewport, reset hover state and exit
    let pos = match pointer_pos {
        Some(p) if geo.viewport_rect.contains(p) => p,
        _ => {
            state.hovered_boundary = None;
            state.hovered_col = None;
            state.hovered_row = None;
            state.hovered_cell = None;
            return events;
        }
    };

    // 3. Scrollbar thumbs hover/drag
    if handle_scrollbar_interaction(ui, geo, pos, state, &mut events) {
        return events;
    }

    // 4. Column boundary hit-test (Resize Cursor & Drag Initiation)
    let boundary_tolerance = 4.0_f32;
    if let Some((col_idx, screen_x)) = geo.col_boundary_near_pos(pos, boundary_tolerance) {
        state.hovered_boundary = Some((col_idx, screen_x));
        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);

        let double_clicked = ui.input(|i| i.pointer.button_double_clicked(PointerButton::Primary));
        if double_clicked {
            events.push(GridEvent::AutoFitCol { col: col_idx });
            return events;
        }

        let primary_pressed = ui.input(|i| i.pointer.button_pressed(PointerButton::Primary));
        if primary_pressed {
            let orig_w = geo.cols.get_size(col_idx).unwrap_or(100.0);
            state.drag = Some(GridDrag::ResizeCol {
                col: col_idx,
                start_mouse_x: pos.x,
                orig_width: orig_w,
            });
            return events;
        }
    } else {
        state.hovered_boundary = None;
    }

    // 5. Quadrant 1: Top-Left Corner
    if geo.quad_top_left.contains(pos) {
        if ui.input(|i| i.pointer.button_clicked(PointerButton::Primary)) {
            events.push(GridEvent::SelectAll);
        }
        return events;
    }

    // 6. Quadrant 2: Column Headers
    if geo.quad_top_right.contains(pos) {
        if let Some(col) = geo.col_header_at_pos(pos) {
            state.hovered_col = Some(col);
            if ui.input(|i| i.pointer.button_clicked(PointerButton::Secondary)) {
                events.push(GridEvent::ContextMenuCol { col });
            } else if ui.input(|i| i.pointer.button_clicked(PointerButton::Primary)) {
                events.push(GridEvent::SelectCol { col });
            }
        }
        return events;
    }

    // 7. Quadrant 3: Row Headers
    if geo.quad_bottom_left.contains(pos) {
        if let Some(row) = geo.row_header_at_pos(pos) {
            state.hovered_row = Some(row);
            if ui.input(|i| i.pointer.button_clicked(PointerButton::Secondary)) {
                events.push(GridEvent::ContextMenuRow { row });
            } else if ui.input(|i| i.pointer.button_clicked(PointerButton::Primary)) {
                events.push(GridEvent::SelectRow { row });
            }
        }
        return events;
    }

    // 8. Quadrant 4: Data Cells
    if geo.quad_center.contains(pos) {
        if let Some((row, col)) = geo.cell_at_pos(pos) {
            state.hovered_cell = Some((row, col));

            if ui.input(|i| i.pointer.button_double_clicked(PointerButton::Primary)) {
                events.push(GridEvent::DoubleClickCell { row, col });
            } else if ui.input(|i| i.pointer.button_clicked(PointerButton::Secondary)) {
                events.push(GridEvent::ContextMenuCell { row, col });
            } else if ui.input(|i| i.pointer.button_clicked(PointerButton::Primary)) {
                events.push(GridEvent::SelectCell { row, col });
            }
        }
    }

    events
}

fn handle_wheel_scroll(ui: &mut Ui, geo: &Geo, events: &mut Vec<GridEvent>) {
    let scroll_delta = ui.input(|i| i.raw_scroll_delta);
    if scroll_delta == Vec2::ZERO {
        return;
    }

    let is_hovered = ui.input(|i| {
        i.pointer
            .hover_pos()
            .map(|p| geo.viewport_rect.contains(p))
            .unwrap_or(false)
    });
    if !is_hovered {
        return;
    }

    let max_s = geo.max_scroll();
    let shift_held = ui.input(|i| i.modifiers.shift);

    let mut dx = -scroll_delta.x;
    let mut dy = -scroll_delta.y;

    if shift_held && dx == 0.0 && dy != 0.0 {
        dx = dy;
        dy = 0.0;
    }

    let new_x = (geo.scroll.x + dx).clamp(0.0, max_s.x);
    let new_y = (geo.scroll.y + dy).clamp(0.0, max_s.y);

    let new_scroll = vec2(new_x, new_y);
    if (new_scroll - geo.scroll).length() > 0.001 {
        events.push(GridEvent::ScrollChanged { new_scroll });
    }
}

fn handle_active_drag(
    ui: &mut Ui,
    geo: &Geo,
    drag: GridDrag,
    state: &mut GridInteractState,
    events: &mut Vec<GridEvent>,
) {
    let pointer_pos = ui
        .input(|i| i.pointer.hover_pos())
        .unwrap_or(geo.viewport_rect.center());
    let primary_down = ui.input(|i| i.pointer.primary_down());

    match drag {
        GridDrag::ResizeCol {
            col,
            start_mouse_x,
            orig_width,
        } => {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
            let dx = pointer_pos.x - start_mouse_x;
            let new_width = (orig_width + dx).clamp(40.0, 800.0);

            // Render live resize guideline across whole grid
            let col_start = geo.cols.get_offset(col).unwrap_or(0.0);
            let screen_guideline_x = geo.quad_center.min.x + (col_start + new_width - geo.scroll.x);
            QuadrantPaint::paint_resize_guide(ui.painter(), geo, screen_guideline_x);

            if !primary_down {
                events.push(GridEvent::ResizeCol { col, new_width });
                state.drag = None;
            }
        }
        GridDrag::ScrollThumbV {
            start_mouse_y,
            orig_scroll_y,
        } => {
            let max_s = geo.max_scroll();
            let track_h = (geo.quad_center.height() - 12.0).max(1.0);
            let dy = pointer_pos.y - start_mouse_y;
            let scroll_ratio = max_s.y / track_h;
            let new_y = (orig_scroll_y + dy * scroll_ratio).clamp(0.0, max_s.y);

            events.push(GridEvent::ScrollChanged {
                new_scroll: vec2(geo.scroll.x, new_y),
            });

            if !primary_down {
                state.drag = None;
            }
        }
        GridDrag::ScrollThumbH {
            start_mouse_x,
            orig_scroll_x,
        } => {
            let max_s = geo.max_scroll();
            let track_w = (geo.quad_center.width() - 12.0).max(1.0);
            let dx = pointer_pos.x - start_mouse_x;
            let scroll_ratio = max_s.x / track_w;
            let new_x = (orig_scroll_x + dx * scroll_ratio).clamp(0.0, max_s.x);

            events.push(GridEvent::ScrollChanged {
                new_scroll: vec2(new_x, geo.scroll.y),
            });

            if !primary_down {
                state.drag = None;
            }
        }
    }
}

fn handle_scrollbar_interaction(
    ui: &mut Ui,
    geo: &Geo,
    pos: Pos2,
    state: &mut GridInteractState,
    _events: &mut Vec<GridEvent>,
) -> bool {
    let max_s = geo.max_scroll();
    let bar_size = 8.0_f32;

    // Vertical scrollbar (right side of quad_center)
    if max_s.y > 0.0 {
        let track_rect = Rect::from_min_max(
            pos2(geo.quad_center.max.x - bar_size, geo.quad_center.min.y),
            geo.quad_center.max,
        );

        let track_h = track_rect.height();
        let thumb_h =
            (track_h * (geo.quad_center.height() / geo.rows.total_size())).clamp(20.0, track_h);
        let thumb_travel = track_h - thumb_h;
        let thumb_y = track_rect.min.y + (geo.scroll.y / max_s.y) * thumb_travel;
        let thumb_rect = Rect::from_min_size(
            pos2(track_rect.min.x + 1.0, thumb_y),
            vec2(bar_size - 2.0, thumb_h),
        );

        // Draw scrollbar
        ui.painter()
            .rect_filled(thumb_rect, 3.0_f32, Tokens::border());

        if thumb_rect.contains(pos)
            && ui.input(|i| i.pointer.button_pressed(PointerButton::Primary))
        {
            state.drag = Some(GridDrag::ScrollThumbV {
                start_mouse_y: pos.y,
                orig_scroll_y: geo.scroll.y,
            });
            return true;
        }
    }

    // Horizontal scrollbar (bottom side of quad_center)
    if max_s.x > 0.0 {
        let track_rect = Rect::from_min_max(
            pos2(geo.quad_center.min.x, geo.quad_center.max.y - bar_size),
            geo.quad_center.max,
        );

        let track_w = track_rect.width();
        let thumb_w =
            (track_w * (geo.quad_center.width() / geo.cols.total_size())).clamp(20.0, track_w);
        let thumb_travel = track_w - thumb_w;
        let thumb_x = track_rect.min.x + (geo.scroll.x / max_s.x) * thumb_travel;
        let thumb_rect = Rect::from_min_size(
            pos2(thumb_x, track_rect.min.y + 1.0),
            vec2(thumb_w, bar_size - 2.0),
        );

        // Draw scrollbar
        ui.painter()
            .rect_filled(thumb_rect, 3.0_f32, Tokens::border());

        if thumb_rect.contains(pos)
            && ui.input(|i| i.pointer.button_pressed(PointerButton::Primary))
        {
            state.drag = Some(GridDrag::ScrollThumbH {
                start_mouse_x: pos.x,
                orig_scroll_x: geo.scroll.x,
            });
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::line_index::LineIndex;

    #[test]
    fn test_grid_interact_state_default() {
        let state = GridInteractState::default();
        assert_eq!(state.drag, None);
        assert_eq!(state.hovered_boundary, None);
        assert_eq!(state.hovered_col, None);
        assert_eq!(state.hovered_row, None);
        assert_eq!(state.hovered_cell, None);
    }

    #[test]
    fn test_handle_grid_interactions_smoke() {
        let vp = Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 400.0));
        let rows = LineIndex::uniform(20, 20.0);
        let cols = LineIndex::uniform(10, 100.0);
        let geo = Geo::new(vp, 50.0, 25.0, rows, cols, vec2(0.0, 0.0));

        let mut state = GridInteractState::default();

        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let events = handle_grid_interactions(ui, &geo, &mut state);
                assert!(events.is_empty());
            });
        });
    }
}
