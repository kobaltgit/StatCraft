use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, Color32, FontId, Sense, Stroke, Ui};

pub fn adjust_widths(ui: &mut Ui, cols: &mut [(String, f32)]) {
    for (name, width) in cols {
        let text_w = ui
            .painter()
            .layout_no_wrap(
                name.clone(),
                FontId::proportional(12.0),
                Tokens::text_primary(),
            )
            .rect
            .width();
        *width = (*width).max(text_w + 16.0);
    }
}

pub fn render_header_row(ui: &mut Ui, cols: &[(String, f32)], row_height: f32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (col_name, width) in cols {
            let (rect, _) = ui.allocate_exact_size(vec2(*width, row_height), Sense::hover());
            ui.painter().rect_filled(rect, 0.0_f32, Tokens::bg_header());
            ui.painter()
                .rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::border()));
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                col_name,
                FontId::proportional(11.5),
                Tokens::text_primary(),
            );
        }
    });
}

pub fn render_data_row(
    ui: &mut Ui,
    cols: &[(String, f32)],
    vals: &[(String, Align2)],
    row_height: f32,
    bg_color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (c_idx, (text, align)) in vals.iter().enumerate() {
            let width = cols.get(c_idx).map(|c| c.1).unwrap_or(80.0_f32);
            let (rect, _) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
            ui.painter().rect_filled(rect, 0.0_f32, bg_color);
            ui.painter()
                .rect_stroke(rect, 0.0_f32, Stroke::new(0.5_f32, Tokens::border()));

            let pos = if *align == Align2::LEFT_CENTER {
                pos2(rect.min.x + 6.0_f32, rect.center().y)
            } else if *align == Align2::CENTER_CENTER {
                rect.center()
            } else {
                pos2(rect.max.x - 6.0_f32, rect.center().y)
            };

            let clip = rect.shrink2(vec2(4.0_f32, 0.0_f32));
            ui.painter().with_clip_rect(clip).text(
                pos,
                *align,
                text.as_str(),
                FontId::proportional(11.5),
                Tokens::text_primary(),
            );
        }
    });
}

pub fn format_opt(val: Option<f64>) -> String {
    val.map(|v| format!("{:.4}", v))
        .unwrap_or_else(|| "-".to_string())
}

pub fn format_opt_2dec(val: Option<f64>) -> String {
    val.map(|v| format!("{:.2}", v))
        .unwrap_or_else(|| "-".to_string())
}

pub fn format_sig(val: Option<f64>) -> String {
    match val {
        Some(v) if v < 0.001 => "< .001".to_string(),
        Some(v) => format!("{:.3}", v),
        None => "-".to_string(),
    }
}

pub fn render_table_cell(
    ui: &mut Ui,
    rect: egui::Rect,
    text: &str,
    align: Align2,
    default_bg_color: egui::Color32,
    style: Option<&statcraft_reporting::styling::CellStyleFormat>,
) {
    let bg = if let Some(s) = style {
        if let Some([r, g, b]) = s.bg_color {
            egui::Color32::from_rgb(r, g, b)
        } else {
            default_bg_color
        }
    } else {
        default_bg_color
    };

    ui.painter().rect_filled(rect, 0.0_f32, bg);
    ui.painter()
        .rect_stroke(rect, 0.0_f32, Stroke::new(0.5_f32, Tokens::border()));

    let fg = if let Some(s) = style {
        if let Some([r, g, b]) = s.text_color {
            egui::Color32::from_rgb(r, g, b)
        } else {
            Tokens::text_primary()
        }
    } else {
        Tokens::text_primary()
    };

    let (is_bold, is_italic, is_underline) = if let Some(s) = style {
        (s.bold, s.italic, s.underline)
    } else {
        (false, false, false)
    };

    let font_id = FontId::proportional(11.5_f32);
    let format = egui::text::TextFormat {
        font_id,
        color: fg,
        italics: is_italic,
        underline: if is_underline {
            Stroke::new(1.0_f32, fg)
        } else {
            Stroke::NONE
        },
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::default();
    job.append(text, 0.0_f32, format);
    let galley = ui.fonts(|f| f.layout_job(job));

    let top_left = if align == Align2::LEFT_CENTER {
        pos2(
            rect.min.x + 6.0_f32,
            rect.center().y - galley.size().y * 0.5_f32,
        )
    } else if align == Align2::CENTER_CENTER {
        pos2(
            rect.center().x - galley.size().x * 0.5_f32,
            rect.center().y - galley.size().y * 0.5_f32,
        )
    } else {
        pos2(
            rect.max.x - 6.0_f32 - galley.size().x,
            rect.center().y - galley.size().y * 0.5_f32,
        )
    };

    let clip = rect.shrink2(vec2(4.0_f32, 0.0_f32));
    ui.painter()
        .with_clip_rect(clip)
        .galley(top_left, galley.clone(), Color32::PLACEHOLDER);

    if is_bold {
        ui.painter().with_clip_rect(clip).galley(
            top_left + vec2(0.6_f32, 0.0_f32),
            galley.clone(),
            Color32::PLACEHOLDER,
        );
        ui.painter().with_clip_rect(clip).galley(
            top_left + vec2(0.3_f32, 0.3_f32),
            galley,
            Color32::PLACEHOLDER,
        );
    }
}

pub fn render_data_row_styled(
    ui: &mut Ui,
    cols: &[(String, f32)],
    vals: &[(String, Align2)],
    row_height: f32,
    default_bg_color: egui::Color32,
    r_idx: usize,
    styles: Option<
        &std::collections::HashMap<(usize, usize), statcraft_reporting::styling::CellStyleFormat>,
    >,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (c_idx, (text, align)) in vals.iter().enumerate() {
            let width = cols.get(c_idx).map(|c| c.1).unwrap_or(80.0_f32);
            let (rect, _) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
            let cell_style = styles.and_then(|m| m.get(&(r_idx, c_idx)));
            render_table_cell(
                ui,
                rect,
                text.as_str(),
                *align,
                default_bg_color,
                cell_style,
            );
        }
    });
}

pub fn render_report_table(ui: &mut Ui, table: &statcraft_reporting::model::ReportTable) {
    if !table.title.is_empty() {
        ui.strong(&table.title);
        ui.add_space(6.0_f32);
    }

    let mut cols: Vec<(String, f32)> = table
        .headers
        .iter()
        .map(|h| (h.clone(), 90.0_f32))
        .collect();
    adjust_widths(ui, &mut cols);

    let row_height = 22.0_f32;
    egui::ScrollArea::horizontal()
        .id_salt(format!("scroll_rep_tab_{}", table.title))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);

                for (r_idx, row) in table.rows.iter().enumerate() {
                    let default_bg = if r_idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };

                    let row_vals: Vec<(String, Align2)> = row
                        .iter()
                        .enumerate()
                        .map(|(c_idx, text)| {
                            let align = table.alignments.get(c_idx).copied().unwrap_or({
                                if c_idx == 0 {
                                    statcraft_reporting::model::CellAlignment::Left
                                } else {
                                    statcraft_reporting::model::CellAlignment::Right
                                }
                            });
                            let egui_align = match align {
                                statcraft_reporting::model::CellAlignment::Left => {
                                    Align2::LEFT_CENTER
                                }
                                statcraft_reporting::model::CellAlignment::Center => {
                                    Align2::CENTER_CENTER
                                }
                                statcraft_reporting::model::CellAlignment::Right => {
                                    Align2::RIGHT_CENTER
                                }
                            };
                            (text.clone(), egui_align)
                        })
                        .collect();

                    render_data_row_styled(
                        ui,
                        &cols,
                        &row_vals,
                        row_height,
                        default_bg,
                        r_idx,
                        Some(&table.cell_styles),
                    );
                }

                for footnote in &table.footnotes {
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(footnote)
                            .italics()
                            .size(11.0)
                            .color(Tokens::text_muted()),
                    );
                }
            });
        });
}
