use super::filter_menu::GridFilterState;
use crate::theme::Tokens;
use egui::{Area, Frame, Id, Key, Order, Pos2, Stroke, Ui};
use statcraft_core::DataSet;
use statcraft_i18n::t;

/// Persistent state for the cell right-click context menu.
#[derive(Debug, Clone, Default)]
pub struct GridContextMenuState {
    pub open_cell: Option<(usize, usize)>,
    pub open_pos: Option<Pos2>,
}

impl GridContextMenuState {
    /// Opens the context menu at position `pos` for cell `(row, col)`.
    pub fn open(&mut self, row: usize, col: usize, pos: Pos2) {
        self.open_cell = Some((row, col));
        self.open_pos = Some(pos);
    }

    /// Closes the context menu.
    pub fn close(&mut self) {
        self.open_cell = None;
        self.open_pos = None;
    }

    /// Whether the context menu is currently open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.open_cell.is_some()
    }
}

/// Action produced by clicking an item in the cell context menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMenuAction {
    FilterByCellValue { row: usize, col: usize },
    ClearColumnFilter { col: usize },
    SortAsc(usize),
    SortDesc(usize),
    Copy,
    Paste,
    Close,
}

/// Renders the floating cell context menu popup.
pub fn render_context_menu(
    ui: &mut Ui,
    state: &mut GridContextMenuState,
    _ds: &DataSet,
    filter_state: &GridFilterState,
) -> Option<ContextMenuAction> {
    let (row, col) = state.open_cell?;
    let pos = state.open_pos?;

    let mut action = None;
    let popup_id = Id::new("grid_context_menu_popup");

    Area::new(popup_id)
        .order(Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            let frame = Frame::menu(ui.style())
                .fill(Tokens::bg_panel())
                .stroke(Stroke::new(1.0_f32, Tokens::border()))
                .inner_margin(egui::Margin::symmetric(6.0, 6.0));

            frame.show(ui, |ui| {
                ui.set_min_width(180.0_f32);

                // 1. Filter by Cell Value
                if ui.button(t!("grid-menu-filter-value")).clicked() {
                    action = Some(ContextMenuAction::FilterByCellValue { row, col });
                }

                if filter_state.is_col_filtered(col)
                    && ui.button(t!("grid-menu-clear-filter")).clicked()
                {
                    action = Some(ContextMenuAction::ClearColumnFilter { col });
                }

                ui.separator();

                // 2. Sort Actions
                if ui.button(t!("grid-filter-sort-asc")).clicked() {
                    action = Some(ContextMenuAction::SortAsc(col));
                }
                if ui.button(t!("grid-filter-sort-desc")).clicked() {
                    action = Some(ContextMenuAction::SortDesc(col));
                }

                ui.separator();

                // 3. Clipboard Actions
                if ui
                    .button(format!("{} (Ctrl+C)", t!("menu-edit-copy")))
                    .clicked()
                {
                    action = Some(ContextMenuAction::Copy);
                }
                if ui
                    .button(format!("{} (Ctrl+V)", t!("menu-edit-paste")))
                    .clicked()
                {
                    action = Some(ContextMenuAction::Paste);
                }
            });
        });

    if ui.input(|i| i.key_pressed(Key::Escape)) {
        action = Some(ContextMenuAction::Close);
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
    fn test_context_menu_state_lifecycle() {
        let mut state = GridContextMenuState::default();
        assert!(!state.is_open());

        state.open(5, 3, Pos2::new(150.0, 200.0));
        assert!(state.is_open());
        assert_eq!(state.open_cell, Some((5, 3)));
        assert_eq!(state.open_pos, Some(Pos2::new(150.0, 200.0)));

        state.close();
        assert!(!state.is_open());
    }
}
