use super::color_palette::render_color_picker;
use super::format_masks::{get_format_mask_label, FORMAT_MASK_KEYS};
use super::state::TableStyleState;
use crate::dialogs::common::{
    DialogShell, DialogSize, GroupBox, SubdialogAction, SubdialogActionBar,
};
use crate::theme::Tokens;
use egui::{pos2, vec2, Button, Color32, ComboBox, Context, FontId, RichText, Sense, Stroke, Ui};
use statcraft_i18n::t;
use statcraft_reporting::styling::{CellStyleFormat, CellTargetScope};

pub fn render_format_subdialog(ctx: &Context, state: &mut TableStyleState) {
    if state.editing_format_idx.is_none() {
        return;
    }

    let mut is_open = true;
    let shell = DialogShell::new(
        t!("dialog-table-style-format-title"),
        DialogSize::Custom(vec2(460.0, 420.0)),
    );

    shell.show(ctx, &mut is_open, |ui| {
        render_format_content(ui, state);
    });

    if !is_open {
        state.editing_format_idx = None;
    }
}

fn render_format_content(ui: &mut Ui, state: &mut TableStyleState) {
    ui.add_space(4.0);

    // 1. Live Sample Preview Box (compact 240x34 px centered box matching SPSS)
    GroupBox::new(t!("dialog-format-preview-header")).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            let bg = if state.fmt_use_defaults {
                Tokens::bg_panel()
            } else {
                match state.fmt_bg_color {
                    Some([r, g, b]) => Color32::from_rgb(r, g, b),
                    None => Tokens::bg_panel(),
                }
            };

            let fg = if state.fmt_use_defaults {
                Tokens::text_primary()
            } else {
                match state.fmt_text_color {
                    Some([r, g, b]) => Color32::from_rgb(r, g, b),
                    None => Tokens::text_primary(),
                }
            };

            let (box_rect, _) = ui.allocate_exact_size(vec2(240.0_f32, 34.0_f32), Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(box_rect, 2.0_f32, bg);
            painter.rect_stroke(box_rect, 2.0_f32, Stroke::new(1.0_f32, Tokens::border()));

            let text_str = t!("dialog-table-style-format-preview-text");
            let is_bold = state.fmt_bold && !state.fmt_use_defaults;
            let is_italic = state.fmt_italic && !state.fmt_use_defaults;
            let is_underline = state.fmt_underline && !state.fmt_use_defaults;

            let format = egui::text::TextFormat {
                font_id: FontId::proportional(15.0_f32),
                color: fg,
                italics: is_italic,
                underline: if is_underline {
                    Stroke::new(1.2_f32, fg)
                } else {
                    Stroke::NONE
                },
                ..Default::default()
            };
            let mut job = egui::text::LayoutJob::default();
            job.append(&text_str, 0.0_f32, format);
            let galley = ui.fonts(|f| f.layout_job(job));
            let text_pos = box_rect.center();
            let top_left = pos2(
                text_pos.x - galley.size().x * 0.5_f32,
                text_pos.y - galley.size().y * 0.5_f32,
            );

            painter.galley(top_left, galley.clone(), Color32::PLACEHOLDER);
            if is_bold {
                painter.galley(
                    top_left + vec2(0.7_f32, 0.0_f32),
                    galley.clone(),
                    Color32::PLACEHOLDER,
                );
                painter.galley(
                    top_left + vec2(0.35_f32, 0.35_f32),
                    galley,
                    Color32::PLACEHOLDER,
                );
            }
        });
    });

    ui.add_space(6.0);

    // 2. Radio: Defaults vs New formatting
    ui.radio_value(
        &mut state.fmt_use_defaults,
        true,
        t!("dialog-format-use-defaults"),
    );

    ui.add_space(4.0);

    ui.radio_value(
        &mut state.fmt_use_defaults,
        false,
        t!("dialog-format-apply-new"),
    );

    ui.add_space(6.0);

    // 3. New formatting options (active when !fmt_use_defaults)
    ui.add_enabled_ui(!state.fmt_use_defaults, |ui| {
        // Toolbar: [ B ] [ I ] [ U ] [ Text Color ] [ Bg Color ]
        ui.horizontal(|ui| {
            let b_btn = Button::new(RichText::new(t!("dialog-format-btn-bold")).size(13.0_f32))
                .selected(state.fmt_bold);
            if ui.add_sized([28.0_f32, 24.0_f32], b_btn).clicked() {
                state.fmt_bold = !state.fmt_bold;
                state.fmt_use_defaults = false;
            }

            let i_btn = Button::new(
                RichText::new(t!("dialog-format-btn-italic"))
                    .italics()
                    .size(13.0_f32),
            )
            .selected(state.fmt_italic);
            if ui.add_sized([28.0_f32, 24.0_f32], i_btn).clicked() {
                state.fmt_italic = !state.fmt_italic;
                state.fmt_use_defaults = false;
            }

            let u_btn = Button::new(
                RichText::new(t!("dialog-format-btn-underline"))
                    .underline()
                    .size(13.0_f32),
            )
            .selected(state.fmt_underline);
            if ui.add_sized([28.0_f32, 24.0_f32], u_btn).clicked() {
                state.fmt_underline = !state.fmt_underline;
                state.fmt_use_defaults = false;
            }

            ui.add_space(8.0_f32);

            // Canonical SPSS Text Color Palette Button & Menu
            render_color_picker(ui, "fmt_text_color", true, &mut state.fmt_text_color);

            ui.add_space(4.0);

            // Canonical SPSS Background Color Palette Button & Menu
            render_color_picker(ui, "fmt_bg_color", false, &mut state.fmt_bg_color);
        });

        ui.add_space(8.0);

        // Format dropdown with all 34 canonical SPSS formats
        ui.horizontal(|ui| {
            ui.add_sized(
                [140.0, 20.0],
                egui::Label::new(t!("dialog-format-label-format")),
            );
            ComboBox::from_id_salt("table_style_fmt_masks")
                .width(220.0)
                .selected_text(get_format_mask_label(state.fmt_format_idx))
                .show_ui(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;
                    for (idx, key) in FORMAT_MASK_KEYS.iter().enumerate() {
                        let label = t!(key);
                        if ui
                            .selectable_label(state.fmt_format_idx == idx, label)
                            .clicked()
                        {
                            state.fmt_format_idx = idx;
                        }
                    }
                });
        });

        ui.add_space(6.0);

        // Checkbox: Change number of decimals -> Decimals: [ 0 ]
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut state.fmt_decimals_enabled,
                t!("dialog-format-decimals-checkbox"),
            );
            ui.add_space(8.0);
            ui.add_enabled_ui(state.fmt_decimals_enabled, |ui| {
                ui.label(t!("dialog-format-decimals-label"));
                ui.add(egui::TextEdit::singleline(&mut state.fmt_decimals_val).desired_width(40.0));
            });
        });

        ui.add_space(6.0);

        // Apply to: [ Только ячейки / Весь столбец / Вся строка ]
        ui.horizontal(|ui| {
            ui.add_sized(
                [140.0, 20.0],
                egui::Label::new(t!("dialog-format-apply-to-label")),
            );
            let apply_options = [
                (0, t!("dialog-format-apply-cells-only")),
                (1, t!("dialog-format-apply-entire-column")),
                (2, t!("dialog-format-apply-entire-row")),
            ];
            let current_apply_label = apply_options
                .iter()
                .find(|(idx, _)| *idx == state.fmt_apply_to_idx)
                .map(|(_, l)| l.as_str())
                .unwrap_or("");

            ComboBox::from_id_salt("table_style_fmt_apply_to")
                .width(180.0)
                .selected_text(current_apply_label)
                .show_ui(ui, |ui| {
                    for (idx, label) in &apply_options {
                        if ui
                            .selectable_label(state.fmt_apply_to_idx == *idx, label)
                            .clicked()
                        {
                            state.fmt_apply_to_idx = *idx;
                        }
                    }
                });
        });

        ui.add_space(6.0);

        // Checkbox: Replacement value -> New value: [    ]
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut state.fmt_replace_enabled,
                t!("dialog-format-replace-checkbox"),
            );
            ui.add_space(8.0);
            ui.add_enabled_ui(state.fmt_replace_enabled, |ui| {
                ui.label(t!("dialog-format-replace-label"));
                ui.add(egui::TextEdit::singleline(&mut state.fmt_replace_val).desired_width(80.0));
            });
        });
    });

    ui.add_space(10.0);

    let action = SubdialogActionBar::default().show(ui);
    match action {
        SubdialogAction::Continue => {
            let fmt = if state.fmt_use_defaults {
                CellStyleFormat {
                    reset_defaults: true,
                    ..Default::default()
                }
            } else {
                let scope = match state.fmt_apply_to_idx {
                    1 => CellTargetScope::EntireColumn,
                    2 => CellTargetScope::EntireRow,
                    _ => CellTargetScope::CellsOnly,
                };
                let decimals = if state.fmt_decimals_enabled {
                    state.fmt_decimals_val.trim().parse::<usize>().ok()
                } else {
                    None
                };
                let replace =
                    if state.fmt_replace_enabled && !state.fmt_replace_val.trim().is_empty() {
                        Some(state.fmt_replace_val.trim().to_string())
                    } else {
                        None
                    };

                CellStyleFormat {
                    bold: state.fmt_bold,
                    italic: state.fmt_italic,
                    underline: state.fmt_underline,
                    text_color: state.fmt_text_color,
                    bg_color: state.fmt_bg_color,
                    format_mask_idx: state.fmt_format_idx,
                    decimals,
                    target_scope: scope,
                    replace_value: replace,
                    reset_defaults: false,
                }
            };

            if let Some(idx) = state.editing_format_idx {
                if idx < state.rules.len() {
                    state.rules[idx].format = fmt;
                }
            }
            state.editing_format_idx = None;
        }
        SubdialogAction::Cancel => {
            state.editing_format_idx = None;
        }
        _ => {}
    }
}
