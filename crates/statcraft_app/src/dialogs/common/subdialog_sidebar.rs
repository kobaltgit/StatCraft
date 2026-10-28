use egui::{vec2, Button, Response, Ui};

pub struct SubdialogSidebar;

impl SubdialogSidebar {
    pub const BUTTON_WIDTH: f32 = 90.0;
    pub const BUTTON_HEIGHT: f32 = 24.0;
    pub const BUTTON_GAP: f32 = 6.0;

    pub fn button(ui: &mut Ui, label: impl AsRef<str>, enabled: bool) -> Response {
        let btn = Button::new(label.as_ref())
            .min_size(vec2(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT))
            .rounding(3.0);
        let resp = ui.add_enabled(enabled, btn);
        ui.add_space(Self::BUTTON_GAP);
        resp
    }
}
