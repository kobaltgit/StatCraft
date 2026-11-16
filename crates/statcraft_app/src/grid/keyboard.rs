use super::selection::GridSelection;
use egui::{Event, Key, Ui};

/// Keyboard action events produced by spreadsheet grid navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardEvent {
    /// Move selection cursor and optionally extend range.
    Navigate {
        target_row: usize,
        target_col: usize,
        extend_selection: bool,
    },
    /// Initiate inline cell editing directly with a typed character.
    StartDirectTyping(char),
    /// Cancel active cell editing.
    CancelEditing,
    /// Commit active cell editing.
    CommitEditing,
    /// Clear content of selected cell or range.
    ClearSelected,
    /// Copy selected cells to clipboard.
    Copy,
    /// Paste clipboard content into grid.
    Paste,
    /// Select the entire table.
    SelectAll,
}

/// Processes keyboard events for the grid when it has focus and is not actively editing.
pub fn handle_grid_keyboard(
    ui: &Ui,
    selection: &GridSelection,
    is_editing: bool,
    total_rows: usize,
    total_cols: usize,
) -> Vec<KeyboardEvent> {
    let mut events = Vec::new();

    if total_rows == 0 || total_cols == 0 {
        return events;
    }

    let (cur_r, cur_c) = selection.active_cell;
    let shift = ui.input(|i| i.modifiers.shift);
    let ctrl = ui.input(|i| i.modifiers.command); // Ctrl on Win/Linux, Cmd on Mac

    // 1. When actively editing, handle Escape, Enter, Tab
    if is_editing {
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            events.push(KeyboardEvent::CancelEditing);
        }
        return events;
    }

    // 2. Global grid shortcuts: Ctrl+A, Ctrl+C, Ctrl+V, Delete, Backspace
    if ctrl && ui.input(|i| i.key_pressed(Key::A)) {
        events.push(KeyboardEvent::SelectAll);
        return events;
    }

    if ctrl && ui.input(|i| i.key_pressed(Key::C)) {
        events.push(KeyboardEvent::Copy);
        return events;
    }

    if ctrl && ui.input(|i| i.key_pressed(Key::V)) {
        events.push(KeyboardEvent::Paste);
        return events;
    }

    if ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
        events.push(KeyboardEvent::ClearSelected);
        return events;
    }

    // 3. Navigation shortcuts: Home / End
    if ui.input(|i| i.key_pressed(Key::Home)) {
        let (target_r, target_c) = if ctrl { (0, 0) } else { (cur_r, 0) };
        events.push(KeyboardEvent::Navigate {
            target_row: target_r,
            target_col: target_c,
            extend_selection: shift,
        });
        return events;
    }

    if ui.input(|i| i.key_pressed(Key::End)) {
        let max_r = total_rows.saturating_sub(1);
        let max_c = total_cols.saturating_sub(1);
        let (target_r, target_c) = if ctrl { (max_r, max_c) } else { (cur_r, max_c) };
        events.push(KeyboardEvent::Navigate {
            target_row: target_r,
            target_col: target_c,
            extend_selection: shift,
        });
        return events;
    }

    // 4. Directional arrows
    let mut d_row: isize = 0;
    let mut d_col: isize = 0;

    if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
        d_row -= 1;
    }
    if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
        d_row += 1;
    }
    if ui.input(|i| i.key_pressed(Key::ArrowLeft)) {
        d_col -= 1;
    }
    if ui.input(|i| i.key_pressed(Key::ArrowRight)) {
        d_col += 1;
    }

    let is_tab = ui.input(|i| i.key_pressed(Key::Tab));
    let is_enter = ui.input(|i| i.key_pressed(Key::Enter));

    if is_tab {
        if shift {
            if cur_c > 0 {
                d_col -= 1;
            } else if cur_r > 0 {
                d_row -= 1;
                d_col = total_cols.saturating_sub(1) as isize;
            }
        } else if cur_c + 1 < total_cols {
            d_col += 1;
        } else if cur_r + 1 < total_rows {
            d_row += 1;
            d_col = -(cur_c as isize);
        }
    }

    if is_enter {
        if shift {
            if cur_r > 0 {
                d_row -= 1;
            }
        } else if cur_r + 1 < total_rows {
            d_row += 1;
        }
    }

    if d_row != 0 || d_col != 0 {
        let new_r =
            (cur_r as isize + d_row).clamp(0, total_rows.saturating_sub(1) as isize) as usize;
        let new_c =
            (cur_c as isize + d_col).clamp(0, total_cols.saturating_sub(1) as isize) as usize;

        let extend = if is_tab || is_enter { false } else { shift };
        events.push(KeyboardEvent::Navigate {
            target_row: new_r,
            target_col: new_c,
            extend_selection: extend,
        });
        return events;
    }

    // 6. Direct Typing: detect typed alphanumeric/symbol characters to initiate editing
    if !ctrl {
        let text_input = ui.input(|i| {
            for event in &i.events {
                if let Event::Text(t) = event {
                    if let Some(ch) = t.chars().next() {
                        if !ch.is_control() {
                            return Some(ch);
                        }
                    }
                }
            }
            None
        });

        if let Some(ch) = text_input {
            events.push(KeyboardEvent::StartDirectTyping(ch));
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_events_smoke() {
        let sel = GridSelection::single(2, 3);
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let evs = handle_grid_keyboard(ui, &sel, false, 10, 10);
                assert!(evs.is_empty());
            });
        });
    }
}
