use egui::{ScrollArea, TextEdit, TextStyle, Ui};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct SyntaxEditorState {
    pub text: String,
}

impl SyntaxEditorState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    pub fn reset_with_header(&mut self) {
        let ts = current_utc_timestamp();
        self.text = format!("* StatCraft Syntax Editor.\n* Created: {}.\n\n", ts);
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

pub fn render_syntax_editor(ui: &mut Ui, state: &mut SyntaxEditorState) {
    ui.vertical(|ui| {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    TextEdit::multiline(&mut state.text)
                        .font(TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY),
                );
            });
    });
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
    fn test_syntax_editor_reset_with_header() {
        let mut state = SyntaxEditorState::new();
        assert!(state.is_empty());
        state.reset_with_header();
        assert!(!state.is_empty());
        assert!(state.text.contains("* StatCraft Syntax Editor."));
        assert!(state.text.contains("* Created:"));
    }

    #[test]
    fn test_syntax_editor_clear() {
        let mut state = SyntaxEditorState::new();
        state.reset_with_header();
        state.clear();
        assert!(state.is_empty());
    }
}
