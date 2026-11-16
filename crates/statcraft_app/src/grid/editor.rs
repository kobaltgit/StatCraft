use super::geo::Geo;
use egui::{vec2, Key, Ui};

/// Navigation direction upon committing an inline cell edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMove {
    Down,
    Up,
    Right,
    Left,
}

/// Action produced by inline cell editor completion or cancellation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorAction {
    Commit {
        row: usize,
        col: usize,
        text: String,
        move_direction: Option<EditorMove>,
    },
    Cancel,
}

/// Persistent state for the floating inline cell editor.
#[derive(Debug, Clone, Default)]
pub struct GridEditorState {
    pub editing_cell: Option<(usize, usize)>,
    pub edit_buffer: String,
    pub focus_requested: bool,
}

impl GridEditorState {
    /// Starts editing cell `(row, col)` with the provided initial text.
    pub fn start_editing(&mut self, row: usize, col: usize, initial_text: String) {
        self.editing_cell = Some((row, col));
        self.edit_buffer = initial_text;
        self.focus_requested = true;
    }

    /// Stops editing and clears the editor buffer.
    pub fn stop(&mut self) {
        self.editing_cell = None;
        self.edit_buffer.clear();
        self.focus_requested = false;
    }

    /// Whether an inline edit is currently active.
    #[inline]
    pub fn is_editing(&self) -> bool {
        self.editing_cell.is_some()
    }
}

/// Renders the floating inline cell editor over the target cell.
pub fn render_cell_editor(
    ui: &mut Ui,
    geo: &Geo,
    state: &mut GridEditorState,
) -> Option<EditorAction> {
    let (row, col) = state.editing_cell?;
    let cell_rect = geo.cell_rect(row, col)?;

    // Don't render editor if cell is completely outside quad_center
    if !geo.quad_center.intersects(cell_rect) {
        return None;
    }

    let inner_rect = cell_rect.shrink2(vec2(2.0_f32, 1.0_f32));
    let mut action = None;

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
        let text_edit = egui::TextEdit::singleline(&mut state.edit_buffer)
            .frame(false)
            .clip_text(true);
        let text_res = ui.add_sized(inner_rect.size(), text_edit);

        if state.focus_requested {
            text_res.request_focus();
            state.focus_requested = false;
        }

        let enter_pressed = ui.input(|i| i.key_pressed(Key::Enter));
        let tab_pressed = ui.input(|i| i.key_pressed(Key::Tab));
        let esc_pressed = ui.input(|i| i.key_pressed(Key::Escape));
        let shift = ui.input(|i| i.modifiers.shift);

        if esc_pressed {
            action = Some(EditorAction::Cancel);
        } else if enter_pressed {
            let move_dir = if shift {
                EditorMove::Up
            } else {
                EditorMove::Down
            };
            action = Some(EditorAction::Commit {
                row,
                col,
                text: state.edit_buffer.clone(),
                move_direction: Some(move_dir),
            });
        } else if tab_pressed {
            let move_dir = if shift {
                EditorMove::Left
            } else {
                EditorMove::Right
            };
            action = Some(EditorAction::Commit {
                row,
                col,
                text: state.edit_buffer.clone(),
                move_direction: Some(move_dir),
            });
        } else if text_res.lost_focus() && !enter_pressed && !tab_pressed && !esc_pressed {
            action = Some(EditorAction::Commit {
                row,
                col,
                text: state.edit_buffer.clone(),
                move_direction: None,
            });
        }
    });

    if action.is_some() {
        state.stop();
    }

    action
}
