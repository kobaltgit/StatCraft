use crate::theme::Tokens;
use crate::widgets::paint_vector_arrow;
use egui::{vec2, Response, Sense, Stroke, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    ToTarget,
    ToSource,
}

pub struct TransferButton {
    direction: TransferDirection,
    enabled: bool,
}

impl TransferButton {
    pub fn new(direction: TransferDirection, enabled: bool) -> Self {
        Self { direction, enabled }
    }

    pub fn to_target(enabled: bool) -> Self {
        Self::new(TransferDirection::ToTarget, enabled)
    }

    pub fn to_source(enabled: bool) -> Self {
        Self::new(TransferDirection::ToSource, enabled)
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let size = vec2(28.0, 28.0);
        let (rect, response) = ui.allocate_exact_size(
            size,
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );

        let bg = if self.enabled && response.hovered() {
            Tokens::ACCENT
        } else {
            Tokens::BG_HEADER
        };

        let stroke = Stroke::new(
            1.0_f32,
            if self.enabled {
                Tokens::BORDER
            } else {
                Tokens::TEXT_MUTED
            },
        );

        ui.painter().rect_filled(rect, 3.0, bg);
        ui.painter().rect_stroke(rect, 3.0, stroke);

        let arrow_color = if self.enabled {
            Tokens::TEXT_PRIMARY
        } else {
            Tokens::TEXT_MUTED
        };

        let right = match self.direction {
            TransferDirection::ToTarget => true,
            TransferDirection::ToSource => false,
        };

        paint_vector_arrow(ui.painter(), rect, right, arrow_color);
        response
    }
}
