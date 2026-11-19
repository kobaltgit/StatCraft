use crate::app::{StatCraftApp, Tab};
use crate::app_session::PendingSessionAction;

impl StatCraftApp {
    /// Handles global keyboard shortcuts for the StatCraft desktop application.
    pub fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Window close interception when unsaved changes exist
        if ctx.input(|i| i.viewport().close_requested()) && self.has_unsaved_changes() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.request_session_action(PendingSessionAction::CloseProject);
        }

        // Ctrl+Shift+Tab -> Previous dataset tab
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::Tab,
            ))
        }) {
            self.switch_to_prev_dataset();
        }

        // Ctrl+Tab -> Next dataset tab
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Tab,
            ))
        }) {
            self.switch_to_next_dataset();
        }

        // Ctrl+W -> Close active dataset tab
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::W,
            ))
        }) {
            if let Some(id) = self.workspace.active_session_id() {
                self.request_close_dataset(id);
            }
        }

        // Project and File actions
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::N,
            ))
        }) {
            self.request_session_action(PendingSessionAction::NewProject);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::N,
            ))
        }) {
            self.request_session_action(PendingSessionAction::NewDataset);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::O,
            ))
        }) {
            self.open_data_dialog();
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::S,
            ))
        }) {
            self.save_project();
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::S,
            ))
        }) {
            self.save_project_as();
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::O,
            ))
        }) {
            self.open_project_dialog();
        }

        // Search and navigation
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::F,
            ))
        }) {
            self.open_find_dialog();
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::H,
            ))
        }) {
            self.open_replace_dialog();
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::G,
            ))
        }) {
            self.open_goto_case_dialog();
        }

        // Clipboard
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::C,
            ))
        }) {
            self.copy_with_variable_names(ctx);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::C,
            ))
        }) {
            self.copy_selected_cell(ctx);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::X,
            ))
        }) {
            self.cut_selected_cell(ctx);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::V,
            ))
        }) {
            self.paste_selected_cell(ctx);
        }

        // Tab Navigation Alt+1..4
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::ALT,
                egui::Key::Num1,
            ))
        }) {
            self.ensure_tab_open(Tab::DataView);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::ALT,
                egui::Key::Num2,
            ))
        }) {
            self.ensure_tab_open(Tab::VariableView);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::ALT,
                egui::Key::Num3,
            ))
        }) {
            self.ensure_tab_open(Tab::OutputViewer);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::ALT,
                egui::Key::Num4,
            ))
        }) {
            self.ensure_tab_open(Tab::SyntaxEditor);
        }
    }
}
