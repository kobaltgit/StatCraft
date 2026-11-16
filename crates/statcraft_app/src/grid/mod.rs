pub mod geo;
pub mod interact;
pub mod line_index;
pub mod quadrant;

pub use geo::Geo;
pub use interact::{handle_grid_interactions, GridDrag, GridEvent, GridInteractState};
pub use line_index::LineIndex;
pub use quadrant::QuadrantPaint;

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
    pub interact_state: &'a mut GridInteractState,
    pub selected_cell: Option<(usize, usize)>,
    pub editing_cell: Option<(usize, usize)>,
}

/// Unified entrypoint for rendering a 4-quadrant virtualized grid with frozen headers.
pub fn render_grid<ColFn, RowFn, CellFn>(
    ui: &mut Ui,
    params: GridRenderParams<'_>,
    col_info: ColFn,
    row_info: RowFn,
    cell_info: CellFn,
) -> (Geo, Vec<GridEvent>)
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
        params.selected_cell,
        params.editing_cell,
        cell_info,
    );

    // 2. Process interactions (clicks, double clicks, resizing, scrolling)
    let events = handle_grid_interactions(ui, &geo, params.interact_state);

    // Update scroll if changed
    for ev in &events {
        if let GridEvent::ScrollChanged { new_scroll } = ev {
            *params.scroll = *new_scroll;
        }
    }

    // Allocate the viewport rect so parent layouts wrap properly
    ui.allocate_rect(viewport_rect, Sense::hover());

    (geo, events)
}
