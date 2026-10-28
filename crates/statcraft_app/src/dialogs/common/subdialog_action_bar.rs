use crate::theme::Tokens;
use egui::{vec2, Button, TextStyle, Ui};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubdialogAction {
    None,
    Continue,
    Cancel,
    Help,
}

pub struct SubdialogActionBar {
    can_continue: bool,
    show_help: bool,
}

impl Default for SubdialogActionBar {
    fn default() -> Self {
        Self {
            can_continue: true,
            show_help: true,
        }
    }
}

impl SubdialogActionBar {
    pub fn new(can_continue: bool) -> Self {
        Self {
            can_continue,
            ..Default::default()
        }
    }

    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    pub fn show(self, ui: &mut Ui) -> SubdialogAction {
        let mut action = SubdialogAction::None;

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        let font_id = TextStyle::Button.resolve(ui.style());
        let labels = [
            t!("dialog-btn-continue"),
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
            let cont_btn = Button::new(t!("dialog-btn-continue"))
                .min_size(btn_size)
                .rounding(3.0);
            if ui.add_enabled(self.can_continue, cont_btn).clicked() {
                action = SubdialogAction::Continue;
            }

            let cancel_btn = Button::new(t!("dialog-btn-cancel"))
                .min_size(btn_size)
                .rounding(3.0);
            if ui.add(cancel_btn).clicked() {
                action = SubdialogAction::Cancel;
            }

            if self.show_help {
                let help_btn = Button::new(t!("dialog-btn-help"))
                    .min_size(btn_size)
                    .rounding(3.0);
                if ui.add(help_btn).clicked() {
                    action = SubdialogAction::Help;
                }
            }
        });

        action
    }
}
