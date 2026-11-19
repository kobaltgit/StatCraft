use super::filter_menu::GridFilterState;
use super::selection::CellRange;
use crate::theme::Tokens;
use egui::{Area, Frame, Id, Key, Order, Pos2, Stroke, Ui};
use statcraft_core::DataSet;
use statcraft_i18n::t;

/// Target for which the context menu was triggered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMenuTarget {
    CellRange {
        range: CellRange,
        active_cell: (usize, usize),
    },
    RowHeader {
        target_row: usize,
        selected_rows: Vec<usize>,
    },
    ColHeader {
        target_col: usize,
        selected_cols: Vec<usize>,
    },
}

/// Persistent state for the right-click context menu.
#[derive(Debug, Clone, Default)]
pub struct GridContextMenuState {
    pub target: Option<ContextMenuTarget>,
    pub open_pos: Option<Pos2>,
    pub just_opened: bool,
    /// Backward-compatible field holding active cell if a cell or cell range was clicked.
    pub open_cell: Option<(usize, usize)>,
}

impl GridContextMenuState {
    /// Opens the context menu for a single cell.
    pub fn open(&mut self, row: usize, col: usize, pos: Pos2) {
        self.open_cell_range(CellRange::single(row, col), (row, col), pos);
    }

    /// Opens the context menu for a rectangular cell range.
    pub fn open_cell_range(&mut self, range: CellRange, active_cell: (usize, usize), pos: Pos2) {
        self.target = Some(ContextMenuTarget::CellRange { range, active_cell });
        self.open_pos = Some(pos);
        self.open_cell = Some(active_cell);
        self.just_opened = true;
    }

    /// Opens the context menu for row header(s).
    pub fn open_row_header(&mut self, target_row: usize, selected_rows: Vec<usize>, pos: Pos2) {
        self.target = Some(ContextMenuTarget::RowHeader {
            target_row,
            selected_rows,
        });
        self.open_pos = Some(pos);
        self.open_cell = None;
        self.just_opened = true;
    }

    /// Opens the context menu for column header(s).
    pub fn open_col_header(&mut self, target_col: usize, selected_cols: Vec<usize>, pos: Pos2) {
        self.target = Some(ContextMenuTarget::ColHeader {
            target_col,
            selected_cols,
        });
        self.open_pos = Some(pos);
        self.open_cell = None;
        self.just_opened = true;
    }

    /// Closes the context menu.
    pub fn close(&mut self) {
        self.target = None;
        self.open_pos = None;
        self.open_cell = None;
        self.just_opened = false;
    }

    /// Whether the context menu is currently open.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.target.is_some()
    }
}

/// Action produced by clicking an item in the context menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMenuAction {
    FilterByCellValue { row: usize, col: usize },
    ClearColumnFilter { col: usize },
    SortAsc(usize),
    SortDesc(usize),
    Copy,
    CopyRange,
    Paste,
    ClearContents,
    InsertCaseAbove(usize),
    DeleteCases(Vec<usize>),
    ClearCases(Vec<usize>),
    InsertVariable(usize),
    AutoFitCol(usize),
    Close,
}

/// Renders the floating context menu popup.
pub fn render_context_menu(
    ui: &mut Ui,
    state: &mut GridContextMenuState,
    _ds: &DataSet,
    filter_state: &GridFilterState,
) -> Option<ContextMenuAction> {
    let target = state.target.as_ref()?.clone();
    let pos = state.open_pos?;

    let mut action = None;
    let popup_id = Id::new("grid_context_menu_popup");

    let area_res = Area::new(popup_id)
        .order(Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            let frame = Frame::menu(ui.style())
                .fill(Tokens::bg_panel())
                .stroke(Stroke::new(1.0_f32, Tokens::border()))
                .inner_margin(egui::Margin::symmetric(6.0, 6.0));

            frame.show(ui, |ui| {
                ui.set_min_width(180.0_f32);

                match target {
                    ContextMenuTarget::CellRange { range, active_cell } => {
                        let (row, col) = active_cell;
                        if range.is_single_cell() {
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
                            if ui.button(t!("grid-menu-clear-contents")).clicked() {
                                action = Some(ContextMenuAction::ClearContents);
                            }
                        } else {
                            let cell_cnt = range.cell_count();
                            let copy_label = t!("grid-menu-copy-range", count = cell_cnt);
                            if ui.button(format!("{copy_label} (Ctrl+C)")).clicked() {
                                action = Some(ContextMenuAction::CopyRange);
                            }
                            if ui
                                .button(format!("{} (Ctrl+V)", t!("menu-edit-paste")))
                                .clicked()
                            {
                                action = Some(ContextMenuAction::Paste);
                            }
                            if ui.button(t!("grid-menu-clear-contents")).clicked() {
                                action = Some(ContextMenuAction::ClearContents);
                            }
                        }
                    }
                    ContextMenuTarget::RowHeader {
                        target_row,
                        selected_rows,
                    } => {
                        if ui.button(t!("grid-menu-insert-case")).clicked() {
                            action = Some(ContextMenuAction::InsertCaseAbove(target_row));
                        }
                        let rows_to_del = if selected_rows.is_empty() {
                            vec![target_row]
                        } else {
                            selected_rows.clone()
                        };
                        let del_label = t!("grid-menu-delete-cases", count = rows_to_del.len());
                        if ui.button(del_label).clicked() {
                            action = Some(ContextMenuAction::DeleteCases(rows_to_del.clone()));
                        }
                        if ui.button(t!("grid-menu-clear-cases")).clicked() {
                            action = Some(ContextMenuAction::ClearCases(rows_to_del));
                        }
                    }
                    ContextMenuTarget::ColHeader { target_col, .. } => {
                        if ui.button(t!("grid-filter-sort-asc")).clicked() {
                            action = Some(ContextMenuAction::SortAsc(target_col));
                        }
                        if ui.button(t!("grid-filter-sort-desc")).clicked() {
                            action = Some(ContextMenuAction::SortDesc(target_col));
                        }
                        ui.separator();
                        if ui.button(t!("grid-menu-insert-variable")).clicked() {
                            action = Some(ContextMenuAction::InsertVariable(target_col));
                        }
                        if ui.button(t!("grid-menu-autofit-col")).clicked() {
                            action = Some(ContextMenuAction::AutoFitCol(target_col));
                        }
                    }
                }
            });
        });

    let popup_rect = area_res.response.rect;

    if ui.input(|i| i.key_pressed(Key::Escape)) {
        action = Some(ContextMenuAction::Close);
    } else if state.just_opened {
        state.just_opened = false;
    } else if ui.input(|i| i.pointer.any_pressed() || i.pointer.any_click()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()))
        {
            if !popup_rect.contains(pos) {
                action = Some(ContextMenuAction::Close);
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
    fn test_context_menu_state_lifecycle() {
        let mut state = GridContextMenuState::default();
        assert!(!state.is_open());

        state.open(5, 3, Pos2::new(150.0, 200.0));
        assert!(state.is_open());
        assert_eq!(state.open_cell, Some((5, 3)));
        assert_eq!(state.open_pos, Some(Pos2::new(150.0, 200.0)));
        assert!(matches!(
            state.target,
            Some(ContextMenuTarget::CellRange { .. })
        ));

        state.close();
        assert!(!state.is_open());

        // Test row header opening
        state.open_row_header(2, vec![2, 3], Pos2::new(40.0, 100.0));
        assert!(state.is_open());
        assert_eq!(
            state.target,
            Some(ContextMenuTarget::RowHeader {
                target_row: 2,
                selected_rows: vec![2, 3],
            })
        );
        state.close();
        assert!(!state.is_open());

        // Test col header opening
        state.open_col_header(1, vec![1], Pos2::new(120.0, 20.0));
        assert!(state.is_open());
        assert_eq!(
            state.target,
            Some(ContextMenuTarget::ColHeader {
                target_col: 1,
                selected_cols: vec![1],
            })
        );
    }
}
