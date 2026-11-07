use egui::{ScrollArea, TextEdit, TextStyle, Ui};
use statcraft_i18n::t;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxEditorAction {
    None,
    RunAll,
    RunSelection(String),
    Clear,
    Open,
    Save,
}

#[derive(Debug, Clone, Default)]
pub struct SyntaxEditorState {
    pub text: String,
    pub selected_text: Option<String>,
    pub cursor_char_idx: usize,
    pub current_file_path: Option<PathBuf>,
}

impl SyntaxEditorState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            selected_text: None,
            cursor_char_idx: 0,
            current_file_path: None,
        }
    }

    pub fn reset_with_header(&mut self) {
        let ts = current_utc_timestamp();
        self.text = format!("* StatCraft Syntax Editor.\n* Created: {}.\n\n", ts);
        self.selected_text = None;
        self.cursor_char_idx = self.text.len();
        self.current_file_path = None;
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.selected_text = None;
        self.cursor_char_idx = 0;
        self.current_file_path = None;
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn current_line_or_all(&self) -> String {
        let chars: Vec<char> = self.text.chars().collect();
        if chars.is_empty() {
            return String::new();
        }
        let idx = self.cursor_char_idx.min(chars.len());
        let mut line_start = idx;
        while line_start > 0 && chars[line_start - 1] != '\n' {
            line_start -= 1;
        }
        let mut line_end = idx;
        while line_end < chars.len() && chars[line_end] != '\n' {
            line_end += 1;
        }
        let line: String = chars[line_start..line_end].iter().collect();
        if line.trim().is_empty() {
            self.text.clone()
        } else {
            line
        }
    }
}

pub fn render_syntax_editor(ui: &mut Ui, state: &mut SyntaxEditorState) -> SyntaxEditorAction {
    let mut action = SyntaxEditorAction::None;

    ui.vertical(|ui| {
        // Toolbar
        ui.horizontal(|ui| {
            if ui.button(t!("syntax-open")).clicked() {
                action = SyntaxEditorAction::Open;
            }
            if ui.button(t!("syntax-save")).clicked() {
                action = SyntaxEditorAction::Save;
            }
            ui.separator();
            if ui.button(t!("syntax-run-all")).clicked() {
                action = SyntaxEditorAction::RunAll;
            }
            if ui.button(t!("syntax-run-selection")).clicked() {
                let to_run = state
                    .selected_text
                    .clone()
                    .unwrap_or_else(|| state.current_line_or_all());
                action = SyntaxEditorAction::RunSelection(to_run);
            }
            if ui.button(t!("syntax-clear")).clicked() {
                state.clear();
                action = SyntaxEditorAction::Clear;
            }
            if let Some(path) = &state.current_file_path {
                ui.separator();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("syntax.sps");
                ui.label(
                    egui::RichText::new(name)
                        .italics()
                        .color(crate::theme::Tokens::text_muted()),
                );
            }
        });

        ui.separator();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let output = TextEdit::multiline(&mut state.text)
                    .font(TextStyle::Monospace)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .show(ui);

                if let Some(cursor_range) = output.cursor_range {
                    let c1 = cursor_range.primary.ccursor.index;
                    let c2 = cursor_range.secondary.ccursor.index;
                    let min = c1.min(c2);
                    let max = c1.max(c2);
                    state.cursor_char_idx = c1;

                    if min < max {
                        let chars: Vec<char> = state.text.chars().collect();
                        if max <= chars.len() {
                            let sel: String = chars[min..max].iter().collect();
                            state.selected_text = Some(sel);
                        }
                    } else {
                        state.selected_text = None;
                    }
                }
            });
    });

    action
}

fn current_utc_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let minutes = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    let mut year = 1970;
    let mut day_count = days as i64;
    loop {
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };
        if day_count < days_in_year {
            break;
        }
        day_count -= days_in_year;
        year += 1;
    }
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &md in &month_days {
        if day_count < md {
            break;
        }
        day_count -= md;
        month += 1;
    }
    let day = day_count + 1;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year, month, day, hours, minutes, seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_editor_state_reset_and_clear() {
        let mut state = SyntaxEditorState::new();
        assert!(state.is_empty());
        assert!(state.current_file_path.is_none());
        state.reset_with_header();
        assert!(!state.is_empty());
        assert!(state.text.contains("StatCraft Syntax Editor"));
        state.current_file_path = Some(PathBuf::from("test.sps"));
        state.clear();
        assert!(state.is_empty());
        assert!(state.current_file_path.is_none());
    }

    #[test]
    fn test_current_line_or_all() {
        let mut state = SyntaxEditorState::new();
        state.text = "DESCRIPTIVES VARIABLES=salary.\nFREQUENCIES VARIABLES=gender.\n".to_string();
        state.cursor_char_idx = 5;
        let line = state.current_line_or_all();
        assert_eq!(line, "DESCRIPTIVES VARIABLES=salary.");
    }
}
