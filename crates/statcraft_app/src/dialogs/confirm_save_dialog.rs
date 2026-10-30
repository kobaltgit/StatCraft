use crate::dialogs::common::DialogShell;
use crate::theme::Tokens;
use egui::{vec2, Align, Button, Context, Layout, TextStyle};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmSaveAction {
    None,
    Save,
    DontSave,
    Cancel,
}

#[derive(Debug, Clone, Default)]
pub struct ConfirmSaveDialogState {
    pub is_open: bool,
}

pub fn render_confirm_save_dialog(
    ctx: &Context,
    state: &mut ConfirmSaveDialogState,
) -> ConfirmSaveAction {
    if !state.is_open {
        return ConfirmSaveAction::None;
    }

    let mut action = ConfirmSaveAction::None;
    let mut is_open = state.is_open;

    DialogShell::alert(t!("dialog-confirm-save-title")).show(ctx, &mut is_open, |ui| {
        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.label(t!("dialog-confirm-save-message"));
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(14.0);

            let font_id = TextStyle::Button.resolve(ui.style());
            let save_str = t!("dialog-confirm-save-btn-save");
            let dont_save_str = t!("dialog-confirm-save-btn-dont-save");
            let cancel_str = t!("dialog-confirm-save-btn-cancel");

            let labels = [&save_str, &dont_save_str, &cancel_str];
            let mut max_btn_w = 95.0_f32;
            for lbl in labels {
                let galley = ui.painter().layout_no_wrap(
                    lbl.to_string(),
                    font_id.clone(),
                    Tokens::TEXT_PRIMARY,
                );
                let w = galley.rect.width() + 24.0;
                if w > max_btn_w {
                    max_btn_w = w;
                }
            }
            let btn_size = vec2(max_btn_w, 26.0);

            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let cancel_btn = Button::new(cancel_str).min_size(btn_size).rounding(3.0);
                    if ui.add(cancel_btn).clicked() {
                        action = ConfirmSaveAction::Cancel;
                    }

                    let dont_save_btn = Button::new(dont_save_str).min_size(btn_size).rounding(3.0);
                    if ui.add(dont_save_btn).clicked() {
                        action = ConfirmSaveAction::DontSave;
                    }

                    let save_btn = Button::new(save_str).min_size(btn_size).rounding(3.0);
                    if ui.add(save_btn).clicked() {
                        action = ConfirmSaveAction::Save;
                    }
                });
            });
        });
    });

    if !is_open || action != ConfirmSaveAction::None {
        state.is_open = false;
    }

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_save_dialog_default_state() {
        let state = ConfirmSaveDialogState::default();
        assert!(!state.is_open);
    }

    #[test]
    fn test_confirm_save_dialog_i18n_shield_fit() {
        let ctx = Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let font_id = TextStyle::Button.resolve(ui.style());
                let labels = [
                    t!("dialog-confirm-save-btn-save"),
                    t!("dialog-confirm-save-btn-dont-save"),
                    t!("dialog-confirm-save-btn-cancel"),
                ];

                let mut max_btn_w = 95.0_f32;
                for lbl in &labels {
                    let galley = ui.painter().layout_no_wrap(
                        lbl.to_string(),
                        font_id.clone(),
                        Tokens::TEXT_PRIMARY,
                    );
                    let w = galley.rect.width() + 24.0;
                    if w > max_btn_w {
                        max_btn_w = w;
                    }
                }

                // 3 buttons + spacing must comfortably fit within Alert dialog content width (440 - 24 = 416 px)
                let total_width = (max_btn_w * 3.0) + (ui.spacing().item_spacing.x * 2.0);
                let content_width = 440.0 - 24.0;
                assert!(
                    total_width <= content_width,
                    "Buttons total width ({}) exceeds alert content width ({})",
                    total_width,
                    content_width
                );
            });
        });
    }
}
