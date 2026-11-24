use crate::theme::Tokens;
use egui::{pos2, vec2, Color32, Id, PopupCloseBehavior, Rect, Response, Sense, Stroke, Ui};
use statcraft_i18n::t;

/// The canonical 5x6 SPSS color matrix (30 shades).
pub const SPSS_COLOR_MATRIX: [[u8; 3]; 30] = [
    // Row 1: Dark tones
    [139, 0, 0],
    [0, 100, 0],
    [0, 0, 139],
    [0, 128, 128],
    [139, 0, 139],
    [64, 64, 64],
    // Row 2: Medium/Rich tones
    [204, 0, 0],
    [0, 153, 0],
    [0, 51, 204],
    [0, 153, 153],
    [153, 0, 153],
    [128, 128, 128],
    // Row 3: Standard vibrant tones
    [255, 0, 0],
    [0, 204, 0],
    [0, 102, 255],
    [255, 204, 0],
    [255, 0, 255],
    [192, 192, 192],
    // Row 4: Pastel/Soft tones
    [255, 153, 102],
    [153, 204, 0],
    [51, 153, 255],
    [255, 255, 153],
    [255, 153, 204],
    [204, 204, 204],
    // Row 5: Very light/Tint tones
    [255, 204, 204],
    [204, 255, 204],
    [204, 229, 255],
    [255, 255, 204],
    [229, 204, 255],
    [240, 240, 240],
];

/// Renders a single square color swatch.
fn render_swatch(ui: &mut Ui, color: Option<[u8; 3]>, is_selected: bool, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let painter = ui.painter();

    match color {
        Some([r, g, b]) => {
            let fill = Color32::from_rgb(r, g, b);
            painter.rect_filled(rect, 1.0_f32, fill);
        }
        None => {
            // White background with diagonal red slash
            painter.rect_filled(rect, 1.0_f32, Color32::WHITE);
            painter.line_segment(
                [
                    pos2(rect.min.x + 2.0_f32, rect.min.y + 2.0_f32),
                    pos2(rect.max.x - 2.0_f32, rect.max.y - 2.0_f32),
                ],
                Stroke::new(1.5_f32, Color32::from_rgb(220, 30, 30)),
            );
        }
    }

    let border_stroke = if is_selected {
        Stroke::new(2.0_f32, Tokens::accent())
    } else if response.hovered() {
        Stroke::new(1.5_f32, Tokens::text_primary())
    } else {
        Stroke::new(1.0_f32, Tokens::border())
    };
    painter.rect_stroke(rect, 1.0_f32, border_stroke);

    response
}

/// Renders the color palette popup contents.
pub fn render_palette_grid(ui: &mut Ui, selected: &mut Option<[u8; 3]>) {
    ui.spacing_mut().item_spacing = vec2(3.0_f32, 3.0_f32);

    // Row 1: Special swatches: White, Black, None
    ui.horizontal(|ui| {
        if render_swatch(
            ui,
            Some([255, 255, 255]),
            *selected == Some([255, 255, 255]),
            20.0_f32,
        )
        .on_hover_text(t!("dialog-format-color-white"))
        .clicked()
        {
            *selected = Some([255, 255, 255]);
            ui.close_menu();
        }

        if render_swatch(ui, Some([0, 0, 0]), *selected == Some([0, 0, 0]), 20.0_f32)
            .on_hover_text(t!("dialog-format-color-black"))
            .clicked()
        {
            *selected = Some([0, 0, 0]);
            ui.close_menu();
        }

        if render_swatch(ui, None, selected.is_none(), 20.0_f32)
            .on_hover_text(t!("dialog-format-color-none"))
            .clicked()
        {
            *selected = None;
            ui.close_menu();
        }
    });

    ui.add_space(2.0_f32);

    // 5 rows of 6 color swatches
    for row_idx in 0..5 {
        ui.horizontal(|ui| {
            for col_idx in 0..6 {
                let color = SPSS_COLOR_MATRIX[row_idx * 6 + col_idx];
                let is_sel = *selected == Some(color);
                let resp = render_swatch(ui, Some(color), is_sel, 20.0_f32);
                if resp.clicked() {
                    *selected = Some(color);
                    ui.close_menu();
                }
            }
        });
    }

    ui.add_space(4.0_f32);
    ui.separator();

    // "Other..." button
    ui.horizontal(|ui| {
        let mut custom_srgb = selected.unwrap_or([255, 255, 255]);
        if ui.color_edit_button_srgb(&mut custom_srgb).changed() {
            *selected = Some(custom_srgb);
        }
        ui.label(t!("dialog-format-color-custom"));
    });
}

/// Renders a color picker button with a drop-down palette.
pub fn render_color_picker(
    ui: &mut Ui,
    id_salt: &str,
    is_text_color: bool,
    selected: &mut Option<[u8; 3]>,
) {
    let popup_id = Id::new(id_salt);
    let btn_size = vec2(36.0_f32, 24.0_f32);
    let (rect, response) = ui.allocate_exact_size(btn_size, Sense::click());

    if response.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
    }

    let painter = ui.painter();
    let bg_color = if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        Tokens::bg_panel()
    };
    painter.rect_filled(rect, 2.0_f32, bg_color);
    painter.rect_stroke(rect, 2.0_f32, Stroke::new(1.0_f32, Tokens::border()));

    if is_text_color {
        // Text 'A' with color bar underneath
        let text_color = match selected {
            Some([r, g, b]) => Color32::from_rgb(*r, *g, *b),
            None => Tokens::text_primary(),
        };
        let font_id = egui::FontId::proportional(13.0_f32);
        painter.text(
            pos2(rect.min.x + 10.0_f32, rect.center().y - 2.0_f32),
            egui::Align2::CENTER_CENTER,
            "A",
            font_id,
            Tokens::text_primary(),
        );
        // Colored bar under 'A'
        let bar_rect = Rect::from_min_max(
            pos2(rect.min.x + 5.0_f32, rect.max.y - 5.0_f32),
            pos2(rect.min.x + 15.0_f32, rect.max.y - 3.0_f32),
        );
        painter.rect_filled(bar_rect, 0.5_f32, text_color);
    } else {
        // Background color swatch rectangle
        let swatch_rect = Rect::from_min_max(
            pos2(rect.min.x + 4.0_f32, rect.min.y + 5.0_f32),
            pos2(rect.min.x + 18.0_f32, rect.max.y - 5.0_f32),
        );
        match selected {
            Some([r, g, b]) => {
                painter.rect_filled(swatch_rect, 1.0_f32, Color32::from_rgb(*r, *g, *b));
            }
            None => {
                painter.rect_filled(swatch_rect, 1.0_f32, Color32::WHITE);
                painter.line_segment(
                    [swatch_rect.left_top(), swatch_rect.right_bottom()],
                    Stroke::new(1.0_f32, Color32::from_rgb(220, 30, 30)),
                );
            }
        }
        painter.rect_stroke(swatch_rect, 1.0_f32, Stroke::new(1.0_f32, Tokens::border()));
    }

    // Small down arrow 'v'
    let arrow_pos = pos2(rect.max.x - 9.0_f32, rect.center().y);
    painter.line_segment(
        [
            pos2(arrow_pos.x - 3.0_f32, arrow_pos.y - 2.0_f32),
            arrow_pos,
        ],
        Stroke::new(1.2_f32, Tokens::text_muted()),
    );
    painter.line_segment(
        [
            arrow_pos,
            pos2(arrow_pos.x + 3.0_f32, arrow_pos.y - 2.0_f32),
        ],
        Stroke::new(1.2_f32, Tokens::text_muted()),
    );

    let tooltip = if is_text_color {
        t!("dialog-format-color-text-tooltip")
    } else {
        t!("dialog-format-color-bg-tooltip")
    };
    let response = response.on_hover_text(tooltip);

    egui::popup_below_widget(
        ui,
        popup_id,
        &response,
        PopupCloseBehavior::CloseOnClick,
        |ui| {
            render_palette_grid(ui, selected);
        },
    );
}
