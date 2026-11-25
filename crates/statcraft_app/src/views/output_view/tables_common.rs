pub use super::tables_bootstrap::{
    bootstrap_column_headers, bootstrap_specs_to_tsv, bootstrap_table_to_tsv,
    format_bootstrap_cells, format_ci_level, render_bootstrap_specifications_table,
    render_bootstrap_table,
};
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

pub fn adjust_widths_with_rows(ui: &mut Ui, cols: &mut [(String, f32)], rows: &[Vec<String>]) {
    adjust_widths(ui, cols);
    for row in rows {
        for (idx, cell) in row.iter().enumerate() {
            if let Some((_, width)) = cols.get_mut(idx) {
                let text_w = ui
                    .painter()
                    .layout_no_wrap(
                        cell.clone(),
                        FontId::proportional(11.5),
                        Tokens::text_primary(),
                    )
                    .rect
                    .width();
                *width = (*width).max((text_w + 16.0).min(360.0));
            }
        }
    }
}

pub fn format_fixed_4(val: f64) -> String {
    let clean = if val.abs() < 5e-5 { 0.0 } else { val };
    format!("{:.4}", clean)
}

pub fn resolve_var_label(dataset: Option<&statcraft_core::DataSet>, name: &str) -> Option<String> {
    dataset
        .and_then(|ds| ds.dictionary.find_by_name(name))
        .map(|m| m.label.trim().to_string())
        .filter(|l| !l.is_empty())
}

pub fn compute_xy_bounds(pts: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for &(x, y) in pts {
        if x.is_finite() && y.is_finite() {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }
    }
    if !x_min.is_finite() || (x_max - x_min).abs() < 1e-12 {
        x_min = 0.0;
        x_max = 1.0;
    }
    if !y_min.is_finite() || (y_max - y_min).abs() < 1e-12 {
        y_min -= 1.0;
        y_max += 1.0;
    }
    (x_min, x_max, y_min, y_max)
}

pub fn pad_axis_range(lo: f64, hi: f64, frac: f64) -> (f64, f64) {
    let span = (hi - lo).abs();
    let pad = if span < 1e-12 { 1.0 } else { span * frac };
    (lo - pad, hi + pad)
}

#[allow(clippy::too_many_arguments)]
pub fn render_xy_scatter_plot(
    ui: &mut Ui,
    pts: &[(f64, f64)],
    x_raw: (f64, f64),
    y_raw: (f64, f64),
    reg_line: Option<((f64, f64), (f64, f64))>,
    include_identity: bool,
    x_label: &str,
    y_label: &str,
    line_color: Color32,
) {
    let x_range = pad_axis_range(x_raw.0, x_raw.1, 0.06);
    let y_range = pad_axis_range(y_raw.0, y_raw.1, 0.10);

    let total_size = vec2(480.0, 270.0);
    let (resp, painter) = ui.allocate_painter(total_size, egui::Sense::hover());
    let outer = resp.rect;

    painter.rect_filled(outer, 4.0_f32, Tokens::bg_panel());
    painter.rect_stroke(outer, 4.0_f32, Stroke::new(1.0_f32, Tokens::border()));

    let pad_l = 64.0;
    let pad_r = 16.0;
    let pad_t = 22.0;
    let pad_b = 36.0;
    let plot_rect = egui::Rect::from_min_max(
        pos2(outer.min.x + pad_l, outer.min.y + pad_t),
        pos2(outer.max.x - pad_r, outer.max.y - pad_b),
    );

    let dx = (x_range.1 - x_range.0).max(1e-12);
    let dy = (y_range.1 - y_range.0).max(1e-12);
    let to_screen = |x: f64, y: f64| -> egui::Pos2 {
        let nx = ((x - x_range.0) / dx) as f32;
        let ny = ((y - y_range.0) / dy) as f32;
        pos2(
            plot_rect.min.x + nx * plot_rect.width(),
            plot_rect.max.y - ny * plot_rect.height(),
        )
    };

    painter.line_segment(
        [
            pos2(plot_rect.min.x, plot_rect.max.y),
            pos2(plot_rect.max.x, plot_rect.max.y),
        ],
        Stroke::new(1.0_f32, Tokens::border()),
    );
    painter.line_segment(
        [
            pos2(plot_rect.min.x, plot_rect.min.y),
            pos2(plot_rect.min.x, plot_rect.max.y),
        ],
        Stroke::new(1.0_f32, Tokens::border()),
    );

    let clipped = painter.with_clip_rect(plot_rect);
    if include_identity {
        let s0 = x_range.0.max(y_range.0);
        let s1 = x_range.1.min(y_range.1);
        if s0 < s1 {
            clipped.add(egui::Shape::dashed_line(
                &[to_screen(s0, s0), to_screen(s1, s1)],
                Stroke::new(1.5_f32, Tokens::text_muted()),
                6.0,
                4.0,
            ));
        }
    }
    for &(x, y) in pts {
        if x.is_finite() && y.is_finite() {
            clipped.circle_filled(to_screen(x, y), 3.0_f32, Tokens::text_primary());
        }
    }
    if let Some(((x0, y0), (x1, y1))) = reg_line {
        clipped.line_segment(
            [to_screen(x0, y0), to_screen(x1, y1)],
            Stroke::new(2.0_f32, line_color),
        );
    }

    let font = FontId::proportional(10.0);
    let y_mid = 0.5 * (y_range.0 + y_range.1);
    for (y_val, y_px) in [
        (y_range.0, plot_rect.max.y),
        (y_mid, plot_rect.center().y),
        (y_range.1, plot_rect.min.y),
    ] {
        painter.text(
            pos2(plot_rect.min.x - 5.0, y_px),
            Align2::RIGHT_CENTER,
            format!("{:.2}", y_val),
            font.clone(),
            Tokens::text_muted(),
        );
    }
    painter.text(
        pos2(plot_rect.min.x, outer.min.y + 10.0),
        Align2::LEFT_CENTER,
        y_label,
        font.clone(),
        Tokens::text_muted(),
    );
    painter.text(
        pos2(plot_rect.center().x, outer.max.y - 10.0),
        Align2::CENTER_CENTER,
        x_label,
        font.clone(),
        Tokens::text_muted(),
    );
    painter.text(
        pos2(plot_rect.min.x, plot_rect.max.y + 8.0),
        Align2::LEFT_CENTER,
        format!("{:.2}", x_range.0),
        font.clone(),
        Tokens::text_muted(),
    );
    painter.text(
        pos2(plot_rect.max.x, plot_rect.max.y + 8.0),
        Align2::RIGHT_CENTER,
        format!("{:.2}", x_range.1),
        font,
        Tokens::text_muted(),
    );
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
