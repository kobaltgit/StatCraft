pub mod clipboard;
pub mod editor;
pub mod geo;
pub mod interact;
pub mod keyboard;
pub mod line_index;
pub mod quadrant;
pub mod selection;

pub use clipboard::{
    apply_auto_fill, parse_tsv_matrix, paste_matrix_into_dataset, serialize_range_to_tsv,
};
pub use editor::{render_cell_editor, EditorAction, EditorMove, GridEditorState};
pub use geo::Geo;
pub use interact::{handle_grid_interactions, GridDrag, GridEvent, GridInteractState};
pub use keyboard::{handle_grid_keyboard, KeyboardEvent};
pub use line_index::LineIndex;
pub use quadrant::QuadrantPaint;
pub use selection::{paint_selection, CellRange, GridSelection};

use egui::{Sense, Ui, Vec2};
use statcraft_core::Measure;

/// Configuration options for 4-quadrant grid rendering.
#[derive(Debug, Clone)]
pub struct GridConfig {
    pub header_width: f32,
    pub header_height: f32,
    pub corner_label: String,
    pub min_col_width: f32,
    pub max_col_width: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            header_width: 50.0,
            header_height: 24.0,
            corner_label: String::new(),
            min_col_width: 40.0,
            max_col_width: 800.0,
        }
    }
}

/// Context parameters passed to `render_grid`.
pub struct GridRenderParams<'a> {
    pub config: &'a GridConfig,
    pub rows: LineIndex,
    pub cols: LineIndex,
    pub scroll: &'a mut Vec2,
    pub selection: &'a mut GridSelection,
    pub editor_state: &'a mut GridEditorState,
    pub interact_state: &'a mut GridInteractState,
}

/// Resulting actions and events emitted from one frame of grid rendering.
pub struct GridFrameResult {
    pub geo: Geo,
    pub mouse_events: Vec<GridEvent>,
    pub kb_events: Vec<KeyboardEvent>,
    pub editor_action: Option<EditorAction>,
}

/// Unified entrypoint for rendering a 4-quadrant virtualized grid with frozen headers,
/// range selection, inline cell editing, and keyboard navigation.
pub fn render_grid<ColFn, RowFn, CellFn>(
    ui: &mut Ui,
    params: GridRenderParams<'_>,
    col_info: ColFn,
    row_info: RowFn,
    cell_info: CellFn,
) -> GridFrameResult
where
    ColFn: FnMut(usize) -> (String, Measure),
    RowFn: FnMut(usize) -> (String, bool),
    CellFn: FnMut(usize, usize) -> String,
{
    let viewport_rect = ui.available_rect_before_wrap();
    let geo = Geo::new(
        viewport_rect,
        params.config.header_width,
        params.config.header_height,
        params.rows,
        params.cols,
        *params.scroll,
    );

    // 1. Render all 4 quadrants
    QuadrantPaint::paint_corner(ui.painter(), &geo, &params.config.corner_label);
    QuadrantPaint::paint_col_headers(
        ui.painter(),
        &geo,
        params.interact_state.hovered_col,
        col_info,
    );
    QuadrantPaint::paint_row_headers(
        ui.painter(),
        &geo,
        params.interact_state.hovered_row,
        row_info,
    );
    QuadrantPaint::paint_data_cells(
        ui.painter(),
        &geo,
        Some(params.selection),
        params.editor_state.editing_cell,
        cell_info,
    );

    // 2. Render range selection overlay and fill handle marker
    paint_selection(ui.painter(), &geo, params.selection);

    // 3. Render active floating inline cell editor
    let editor_action = render_cell_editor(ui, &geo, params.editor_state);

    // 4. Process keyboard navigation and shortcuts
    let kb_events = handle_grid_keyboard(
        ui,
        params.selection,
        params.editor_state.is_editing(),
        geo.rows.count(),
        geo.cols.count(),
    );

    for ev in &kb_events {
        match ev {
            KeyboardEvent::Navigate {
                target_row,
                target_col,
                extend_selection,
            } => {
                if *extend_selection {
                    params.selection.extend_to(*target_row, *target_col);
                } else {
                    *params.selection = GridSelection::single(*target_row, *target_col);
                }
                *params.scroll = geo.scroll_to_cell(*target_row, *target_col);
            }
            KeyboardEvent::SelectAll => {
                *params.selection = GridSelection::select_all(geo.rows.count(), geo.cols.count());
            }
            _ => {}
        }
    }

    // 5. Process mouse interactions (clicks, double clicks, resizing, scrolling, range drag)
    let mouse_events = handle_grid_interactions(ui, &geo, params.selection, params.interact_state);

    for ev in &mouse_events {
        match ev {
            GridEvent::SelectCell { row, col } => {
                *params.selection = GridSelection::single(*row, *col);
            }
            GridEvent::SelectRange { anchor, active } => {
                *params.selection = GridSelection::from_anchor_and_active(*anchor, *active);
            }
            GridEvent::SelectRow { row } => {
                *params.selection = GridSelection::select_row(*row, geo.cols.count());
            }
            GridEvent::SelectCol { col } => {
                *params.selection = GridSelection::select_col(*col, geo.rows.count());
            }
            GridEvent::SelectAll => {
                *params.selection = GridSelection::select_all(geo.rows.count(), geo.cols.count());
            }
            GridEvent::ScrollChanged { new_scroll } => {
                *params.scroll = *new_scroll;
            }
            _ => {}
        }
    }

    // Allocate the viewport rect so parent layouts wrap properly
    ui.allocate_rect(viewport_rect, Sense::hover());

    GridFrameResult {
        geo,
        mouse_events,
        kb_events,
        editor_action,
    }
}
