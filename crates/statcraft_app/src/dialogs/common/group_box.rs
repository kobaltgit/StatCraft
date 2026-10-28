use crate::theme::Tokens;
use egui::{Frame, Margin, Stroke, Ui};

pub struct GroupBox {
    title: String,
}

impl GroupBox {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
        }
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        ui.add_space(4.0);
        let frame = Frame::group(ui.style())
            .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
            .fill(Tokens::BG_PANEL)
            .inner_margin(Margin::same(8.0));

        let res = frame.show(ui, |ui| {
            ui.label(egui::RichText::new(&self.title).strong().size(11.0));
            ui.add_space(4.0);
            add_contents(ui)
        });

        ui.add_space(4.0);
        res.inner
    }
}
