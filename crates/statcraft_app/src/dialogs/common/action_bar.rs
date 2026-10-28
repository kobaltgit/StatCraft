use crate::theme::Tokens;
use egui::{vec2, Button, TextStyle, Ui};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionBarAction {
    None,
    Ok,
    Paste,
    Reset,
    Cancel,
    Help,
}

pub struct DialogActionBar {
    can_run: bool,
    show_paste: bool,
    show_reset: bool,
    show_cancel: bool,
    show_help: bool,
}

impl Default for DialogActionBar {
    fn default() -> Self {
        Self {
            can_run: true,
            show_paste: true,
            show_reset: true,
            show_cancel: true,
            show_help: true,
        }
    }
}

impl DialogActionBar {
    pub fn new(can_run: bool) -> Self {
        Self {
            can_run,
            ..Default::default()
        }
    }

    pub fn show_paste(mut self, show: bool) -> Self {
        self.show_paste = show;
        self
    }

    pub fn show_reset(mut self, show: bool) -> Self {
        self.show_reset = show;
        self
    }

    pub fn show_cancel(mut self, show: bool) -> Self {
        self.show_cancel = show;
        self
    }

    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    pub fn show(self, ui: &mut Ui) -> ActionBarAction {
        let mut action = ActionBarAction::None;

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        // i18n Shield: calculate uniform button width
        let font_id = TextStyle::Button.resolve(ui.style());
        let labels = [
            t!("dialog-btn-ok"),
            t!("dialog-btn-paste"),
            t!("dialog-btn-reset"),
            t!("dialog-btn-cancel"),
            t!("dialog-btn-help"),
        ];

        let mut max_width = 80.0_f32;
        for label in &labels {
            let galley = ui.painter().layout_no_wrap(
                label.to_string(),
                font_id.clone(),
                Tokens::TEXT_PRIMARY,
            );
            let w = galley.rect.width() + 20.0;
            if w > max_width {
                max_width = w;
            }
        }

        let btn_size = vec2(max_width, 26.0);

        ui.horizontal(|ui| {
            // 1. OK Button
            let ok_btn = Button::new(t!("dialog-btn-ok"))
                .min_size(btn_size)
                .rounding(3.0);
            if ui.add_enabled(self.can_run, ok_btn).clicked() {
                action = ActionBarAction::Ok;
            }

            // 2. Paste Button
            if self.show_paste {
                let paste_btn = Button::new(t!("dialog-btn-paste"))
                    .min_size(btn_size)
                    .rounding(3.0);
                if ui.add_enabled(self.can_run, paste_btn).clicked() {
                    action = ActionBarAction::Paste;
                }
            }

            // 3. Reset Button
            if self.show_reset {
                let reset_btn = Button::new(t!("dialog-btn-reset"))
                    .min_size(btn_size)
                    .rounding(3.0);
                if ui.add(reset_btn).clicked() {
                    action = ActionBarAction::Reset;
                }
            }

            // 4. Cancel Button
            if self.show_cancel {
                let cancel_btn = Button::new(t!("dialog-btn-cancel"))
                    .min_size(btn_size)
                    .rounding(3.0);
                if ui.add(cancel_btn).clicked() {
                    action = ActionBarAction::Cancel;
                }
            }

            // 5. Help Button
            if self.show_help {
                let help_btn = Button::new(t!("dialog-btn-help"))
                    .min_size(btn_size)
                    .rounding(3.0);
                if ui.add(help_btn).clicked() {
                    action = ActionBarAction::Help;
                }
            }
        });

        action
    }
}
