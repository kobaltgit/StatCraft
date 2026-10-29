use super::model::{OutputItem, OutputNotes};
use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, Sense, Stroke, Ui};
use statcraft_i18n::t;

pub fn render_log_block(ui: &mut Ui, sps: &str) {
    let block_w = (ui.available_width() - 16.0).max(200.0);
    let lines_count = sps.lines().count().max(1);
    let height = (lines_count as f32 * 16.0 + 12.0).max(24.0);

    let (rect, _) = ui.allocate_exact_size(vec2(block_w, height), Sense::hover());
    ui.painter().rect_filled(rect, 2.0_f32, Tokens::BG_ROW_EVEN);
    ui.painter()
        .rect_stroke(rect, 2.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));

    let mut y = rect.min.y + 6.0;
    for line in sps.lines() {
        ui.painter().text(
            pos2(rect.min.x + 8.0, y),
            Align2::LEFT_TOP,
            line,
            FontId::monospace(11.0),
            Tokens::TEXT_PRIMARY,
        );
        y += 16.0;
    }
}

pub fn render_notes_block(ui: &mut Ui, notes: &OutputNotes) {
    let row_h = 20.0_f32;
    let label_w = 160.0_f32;
    let val_w = 260.0_f32;

    let rows: [(&str, String); 4] = [
        ("Output Created", notes.timestamp.clone()),
        ("Active Dataset", notes.dataset_name.clone()),
        ("N of Active Cases", notes.active_cases.to_string()),
        ("N of Variables", notes.active_vars.to_string()),
    ];

    ui.vertical(|ui| {
        for (label, val) in &rows {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                let (r1, _) = ui.allocate_exact_size(vec2(label_w, row_h), Sense::hover());
                ui.painter().rect_filled(r1, 0.0_f32, Tokens::BG_HEADER);
                ui.painter()
                    .rect_stroke(r1, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));
                ui.painter().text(
                    pos2(r1.min.x + 6.0, r1.center().y),
                    Align2::LEFT_CENTER,
                    *label,
                    FontId::proportional(11.0),
                    Tokens::TEXT_MUTED,
                );

                let (r2, _) = ui.allocate_exact_size(vec2(val_w, row_h), Sense::hover());
                ui.painter().rect_filled(r2, 0.0_f32, Tokens::BG_ROW_EVEN);
                ui.painter()
                    .rect_stroke(r2, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));
                ui.painter().text(
                    pos2(r2.min.x + 6.0, r2.center().y),
                    Align2::LEFT_CENTER,
                    val,
                    FontId::proportional(11.0),
                    Tokens::TEXT_PRIMARY,
                );
            });
        }
    });
}

pub fn render_item_context_menu(
    ui: &mut Ui,
    item: &OutputItem,
    maybe_tsv: &Option<String>,
    is_copied: bool,
    coords: (usize, usize),
    item_copied: &mut Option<usize>,
    item_to_delete: &mut Option<(usize, usize)>,
) {
    if let Some(tsv_data) = maybe_tsv {
        let label = if is_copied {
            t!("output-menu-copied")
        } else {
            t!("output-menu-copy-tsv")
        };
        if ui.button(label).clicked() {
            ui.ctx().output_mut(|o| o.copied_text = tsv_data.clone());
            *item_copied = Some(item.id);
            ui.close_menu();
        }
    }
    if ui.button(t!("output-menu-copy-text")).clicked() {
        ui.ctx().output_mut(|o| o.copied_text = item.title());
        ui.close_menu();
    }
    ui.separator();
    if ui.button(t!("output-menu-delete")).clicked() {
        *item_to_delete = Some(coords);
        ui.close_menu();
    }
}
