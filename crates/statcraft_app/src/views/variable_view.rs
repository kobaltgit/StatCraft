use crate::theme::Tokens;
use crate::widgets::paint_measure_icon;
use egui::{pos2, vec2, Align2, FontId, Rect, ScrollArea, Sense, Stroke, Ui};
use statcraft_core::variable::{Alignment, Role};
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableViewAction {
    OpenValueLabels(usize),
    OpenMissingValues(usize),
    OpenVariableType(usize),
    None,
}

#[derive(Default)]
pub struct VariableViewState {
    pub new_var_name: String,
    pub editing_name: Option<(usize, String)>,
}

pub fn render_variable_view(
    ui: &mut Ui,
    dataset: &mut Option<DataSet>,
    state: &mut VariableViewState,
) -> VariableViewAction {
    let mut view_action = VariableViewAction::None;
    let ds = match dataset {
        Some(d) => d,
        None => {
            ui.centered_and_justified(|ui| {
                ui.label(t!("output-no-analysis"));
            });
            return VariableViewAction::None;
        }
    };

    let var_count = ds.dictionary.count();

    let row_height = 24.0_f32;
    let font_header = FontId::proportional(12.0);
    let measure_header_width = |title: &str, min_w: f32| -> f32 {
        let measured = ui
            .painter()
            .layout_no_wrap(title.to_string(), font_header.clone(), Tokens::TEXT_PRIMARY)
            .rect
            .width();
        min_w.max(measured + 18.0_f32)
    };

    let headers = [
        (
            t!("header-row"),
            measure_header_width(&t!("header-row"), 45.0_f32),
        ),
        (
            t!("header-name"),
            measure_header_width(&t!("header-name"), 120.0_f32),
        ),
        (
            t!("header-type"),
            measure_header_width(&t!("header-type"), 110.0_f32),
        ),
        (
            t!("header-width"),
            measure_header_width(&t!("header-width"), 65.0_f32),
        ),
        (
            t!("header-decimals"),
            measure_header_width(&t!("header-decimals"), 75.0_f32),
        ),
        (
            t!("header-label"),
            measure_header_width(&t!("header-label"), 180.0_f32),
        ),
        (
            t!("header-values"),
            measure_header_width(&t!("header-values"), 110.0_f32),
        ),
        (
            t!("header-missing"),
            measure_header_width(&t!("header-missing"), 110.0_f32),
        ),
        (
            t!("header-align"),
            measure_header_width(&t!("header-align"), 85.0_f32),
        ),
        (
            t!("header-measure"),
            measure_header_width(&t!("header-measure"), 120.0_f32),
        ),
        (
            t!("header-role"),
            measure_header_width(&t!("header-role"), 85.0_f32),
        ),
    ];

    let mut action_delete_var = None;
    let mut action_insert_var = None;
    let mut rename_action = None;

    ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 1. Pinned Header Row
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                for (idx, (title, width)) in headers.iter().enumerate() {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(*width, row_height), Sense::hover());
                    ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
                    ui.painter()
                        .rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));

                    let align = if idx == 0 || idx == 3 || idx == 4 {
                        Align2::CENTER_CENTER
                    } else {
                        Align2::LEFT_CENTER
                    };
                    let text_pos = if align == Align2::CENTER_CENTER {
                        rect.center()
                    } else {
                        pos2(rect.min.x + 6.0_f32, rect.center().y)
                    };

                    let text_clip = rect.shrink2(vec2(2.0_f32, 0.0_f32));
                    ui.painter().with_clip_rect(text_clip).text(
                        text_pos,
                        align,
                        title.as_str(),
                        FontId::proportional(12.0),
                        Tokens::TEXT_PRIMARY,
                    );
                }
            });

            // 2. Variable Rows + 1 Ghost Row for Adding
            let total_rows = var_count + 1;
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, row_height, total_rows, |ui, row_range| {
                    for i in row_range {
                        let bg_color = if i % 2 == 0 {
                            Tokens::BG_ROW_EVEN
                        } else {
                            Tokens::BG_ROW_ODD
                        };

                        if i < var_count {
                            // Existing Variable Row
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                                let mut new_label = None;
                                let mut new_measure = None;
                                let mut new_align = None;
                                let mut new_role = None;
                                let mut width_delta = 0_isize;
                                let mut decimals_delta = 0_isize;

                                if let Some(var) = ds.dictionary.get(i) {
                                    // 0. Row Index Cell with Context Menu
                                    let (rect, res) = ui.allocate_exact_size(
                                        vec2(headers[0].1, row_height),
                                        Sense::click(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );
                                    ui.painter().text(
                                        rect.center(),
                                        Align2::CENTER_CENTER,
                                        format!("{}", i + 1),
                                        FontId::proportional(11.0),
                                        Tokens::TEXT_MUTED,
                                    );

                                    res.context_menu(|ui| {
                                        if ui.button(t!("menu-insert-variable")).clicked() {
                                            action_insert_var = Some(i);
                                            ui.close_menu();
                                        }
                                        if ui.button(t!("menu-delete-variable")).clicked() {
                                            action_delete_var = Some(i);
                                            ui.close_menu();
                                        }
                                    });

                                    // 1. Name Cell (In-place editable singleline TextEdit)
                                    let (rect, _) = ui.allocate_exact_size(
                                        vec2(headers[1].1, row_height),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );

                                    let inner_rect = rect.shrink2(vec2(2.0_f32, 1.0_f32));
                                    ui.allocate_new_ui(
                                        egui::UiBuilder::new().max_rect(inner_rect),
                                        |ui| {
                                            let mut name_text =
                                                if let Some((edit_idx, ref cur_text)) =
                                                    state.editing_name
                                                {
                                                    if edit_idx == i {
                                                        cur_text.clone()
                                                    } else {
                                                        var.name.clone()
                                                    }
                                                } else {
                                                    var.name.clone()
                                                };

                                            let text_edit =
                                                egui::TextEdit::singleline(&mut name_text)
                                                    .frame(false)
                                                    .clip_text(true);
                                            let text_res =
                                                ui.add_sized(inner_rect.size(), text_edit);

                                            if text_res.changed() {
                                                state.editing_name = Some((i, name_text.clone()));
                                            }

                                            if text_res.lost_focus() {
                                                if let Some((edit_idx, ref committed_name)) =
                                                    state.editing_name
                                                {
                                                    if edit_idx == i
                                                        && !committed_name.trim().is_empty()
                                                        && committed_name.trim() != var.name
                                                    {
                                                        rename_action = Some((
                                                            i,
                                                            committed_name.trim().to_string(),
                                                        ));
                                                    }
                                                }
                                                state.editing_name = None;
                                            }
                                        },
                                    );

                                    // 2. Type Cell + [...] button to open VariableTypeDialog
                                    let (rect, res) = ui.allocate_exact_size(
                                        vec2(headers[2].1, row_height),
                                        Sense::click(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );
                                    let type_str = t!(var.var_type.as_key());
                                    let clip = Rect::from_min_max(
                                        rect.min,
                                        pos2(rect.max.x - 22.0, rect.max.y),
                                    );
                                    ui.painter().with_clip_rect(clip).text(
                                        pos2(rect.min.x + 6.0_f32, rect.center().y),
                                        Align2::LEFT_CENTER,
                                        type_str,
                                        FontId::proportional(12.0),
                                        Tokens::TEXT_PRIMARY,
                                    );

                                    let btn_type_rect = Rect::from_min_max(
                                        pos2(rect.max.x - 20.0_f32, rect.min.y + 2.0_f32),
                                        pos2(rect.max.x - 2.0_f32, rect.max.y - 2.0_f32),
                                    );
                                    let btn_type_res =
                                        ui.allocate_rect(btn_type_rect, Sense::click());
                                    let btn_bg = if btn_type_res.hovered() {
                                        Tokens::ACCENT
                                    } else {
                                        Tokens::BG_HEADER
                                    };
                                    ui.painter().rect_filled(btn_type_rect, 2.0_f32, btn_bg);
                                    ui.painter().rect_stroke(
                                        btn_type_rect,
                                        2.0_f32,
                                        Stroke::new(1.0_f32, Tokens::BORDER),
                                    );
                                    ui.painter().text(
                                        btn_type_rect.center(),
                                        Align2::CENTER_CENTER,
                                        "...",
                                        FontId::proportional(10.0),
                                        Tokens::TEXT_PRIMARY,
                                    );

                                    if btn_type_res.clicked() || res.double_clicked() {
                                        view_action = VariableViewAction::OpenVariableType(i);
                                    }

                                    // 3. Width Cell with ▲ / ▼ spinner
                                    let (rect, _) = ui.allocate_exact_size(
                                        vec2(headers[3].1, row_height),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );

                                    ui.painter().text(
                                        pos2(rect.center().x - 6.0, rect.center().y),
                                        Align2::CENTER_CENTER,
                                        format!("{}", var.width),
                                        FontId::proportional(12.0),
                                        Tokens::TEXT_PRIMARY,
                                    );

                                    let up_rect = Rect::from_min_max(
                                        pos2(rect.max.x - 14.0, rect.min.y + 2.0),
                                        pos2(rect.max.x - 2.0, rect.center().y),
                                    );
                                    let dn_rect = Rect::from_min_max(
                                        pos2(rect.max.x - 14.0, rect.center().y),
                                        pos2(rect.max.x - 2.0, rect.max.y - 2.0),
                                    );

                                    let up_res = ui.allocate_rect(up_rect, Sense::click());
                                    let dn_res = ui.allocate_rect(dn_rect, Sense::click());
                                    if up_res.clicked() {
                                        width_delta = 1;
                                    }
                                    if dn_res.clicked() && var.width > 1 {
                                        width_delta = -1;
                                    }

                                    ui.painter().text(
                                        up_rect.center(),
                                        Align2::CENTER_CENTER,
                                        "^",
                                        FontId::proportional(9.0),
                                        if up_res.hovered() {
                                            Tokens::ACCENT
                                        } else {
                                            Tokens::TEXT_MUTED
                                        },
                                    );
                                    ui.painter().text(
                                        dn_rect.center(),
                                        Align2::CENTER_CENTER,
                                        "v",
                                        FontId::proportional(9.0),
                                        if dn_res.hovered() {
                                            Tokens::ACCENT
                                        } else {
                                            Tokens::TEXT_MUTED
                                        },
                                    );

                                    // 4. Decimals Cell with ▲ / ▼ spinner
                                    let (rect, _) = ui.allocate_exact_size(
                                        vec2(headers[4].1, row_height),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );

                                    let allows_dec = var.var_type.allows_decimals();
                                    let dec_text = if allows_dec {
                                        format!("{}", var.decimals)
                                    } else {
                                        "-".to_string()
                                    };

                                    ui.painter().text(
                                        pos2(rect.center().x - 6.0, rect.center().y),
                                        Align2::CENTER_CENTER,
                                        dec_text,
                                        FontId::proportional(12.0),
                                        if allows_dec {
                                            Tokens::TEXT_PRIMARY
                                        } else {
                                            Tokens::TEXT_MUTED
                                        },
                                    );

                                    if allows_dec {
                                        let up_rect = Rect::from_min_max(
                                            pos2(rect.max.x - 14.0, rect.min.y + 2.0),
                                            pos2(rect.max.x - 2.0, rect.center().y),
                                        );
                                        let dn_rect = Rect::from_min_max(
                                            pos2(rect.max.x - 14.0, rect.center().y),
                                            pos2(rect.max.x - 2.0, rect.max.y - 2.0),
                                        );

                                        let up_res = ui.allocate_rect(up_rect, Sense::click());
                                        let dn_res = ui.allocate_rect(dn_rect, Sense::click());
                                        if up_res.clicked() && var.decimals < 16 {
                                            decimals_delta = 1;
                                        }
                                        if dn_res.clicked() && var.decimals > 0 {
                                            decimals_delta = -1;
                                        }

                                        ui.painter().text(
                                            up_rect.center(),
                                            Align2::CENTER_CENTER,
                                            "^",
                                            FontId::proportional(9.0),
                                            if up_res.hovered() {
                                                Tokens::ACCENT
                                            } else {
                                                Tokens::TEXT_MUTED
                                            },
                                        );
                                        ui.painter().text(
                                            dn_rect.center(),
                                            Align2::CENTER_CENTER,
                                            "v",
                                            FontId::proportional(9.0),
                                            if dn_res.hovered() {
                                                Tokens::ACCENT
                                            } else {
                                                Tokens::TEXT_MUTED
                                            },
                                        );
                                    }

                                    // 5. Label Cell (Editable text field with borderless frame)
                                    let (rect, _) = ui.allocate_exact_size(
                                        vec2(headers[5].1, row_height),
                                        Sense::click(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );
                                    let inner_rect = rect.shrink2(vec2(2.0_f32, 2.0_f32));
                                    ui.allocate_new_ui(
                                        egui::UiBuilder::new().max_rect(inner_rect),
                                        |ui| {
                                            let mut text = var.label.clone();
                                            let res = ui.add_sized(
                                                vec2(inner_rect.width(), inner_rect.height()),
                                                egui::TextEdit::singleline(&mut text)
                                                    .frame(false)
                                                    .clip_text(true),
                                            );
                                            if res.changed() {
                                                new_label = Some(text);
                                            }
                                        },
                                    );

                                    // 6. Values Count Summary & [...] button
                                    let (rect, _) = ui.allocate_exact_size(
                                        vec2(headers[6].1, row_height),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );
                                    let values_summary = if var.values.is_empty() {
                                        t!("values-none")
                                    } else {
                                        t!("values-count", count = var.values.mapping.len())
                                    };
                                    let clip = Rect::from_min_max(
                                        rect.min,
                                        pos2(rect.max.x - 22.0_f32, rect.max.y),
                                    );
                                    ui.painter().with_clip_rect(clip).text(
                                        pos2(rect.min.x + 6.0_f32, rect.center().y),
                                        Align2::LEFT_CENTER,
                                        values_summary,
                                        FontId::proportional(11.0),
                                        Tokens::TEXT_PRIMARY,
                                    );

                                    let btn_val_rect = Rect::from_min_max(
                                        pos2(rect.max.x - 20.0_f32, rect.min.y + 2.0_f32),
                                        pos2(rect.max.x - 2.0_f32, rect.max.y - 2.0_f32),
                                    );
                                    let btn_val_res =
                                        ui.allocate_rect(btn_val_rect, Sense::click());
                                    let btn_val_bg = if btn_val_res.hovered() {
                                        Tokens::ACCENT
                                    } else {
                                        Tokens::BG_HEADER
                                    };
                                    ui.painter().rect_filled(btn_val_rect, 2.0_f32, btn_val_bg);
                                    ui.painter().rect_stroke(
                                        btn_val_rect,
                                        2.0_f32,
                                        Stroke::new(1.0_f32, Tokens::BORDER),
                                    );
                                    ui.painter().text(
                                        btn_val_rect.center(),
                                        Align2::CENTER_CENTER,
                                        "...",
                                        FontId::proportional(10.0),
                                        Tokens::TEXT_PRIMARY,
                                    );
                                    if btn_val_res.clicked() {
                                        view_action = VariableViewAction::OpenValueLabels(i);
                                    }

                                    // 7. Missing Summary & [...] button
                                    let (rect, _) = ui.allocate_exact_size(
                                        vec2(headers[7].1, row_height),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );
                                    let missing_summary =
                                        if let Some((low, high)) = var.missing.range {
                                            t!("missing-range-summary", low = low, high = high)
                                        } else if !var.missing.discrete.is_empty() {
                                            t!(
                                                "missing-discrete-count",
                                                count = var.missing.discrete.len()
                                            )
                                        } else {
                                            t!("missing-none")
                                        };
                                    let clip = Rect::from_min_max(
                                        rect.min,
                                        pos2(rect.max.x - 22.0_f32, rect.max.y),
                                    );
                                    ui.painter().with_clip_rect(clip).text(
                                        pos2(rect.min.x + 6.0_f32, rect.center().y),
                                        Align2::LEFT_CENTER,
                                        missing_summary,
                                        FontId::proportional(11.0),
                                        Tokens::TEXT_PRIMARY,
                                    );

                                    let btn_miss_rect = Rect::from_min_max(
                                        pos2(rect.max.x - 20.0_f32, rect.min.y + 2.0_f32),
                                        pos2(rect.max.x - 2.0_f32, rect.max.y - 2.0_f32),
                                    );
                                    let btn_miss_res =
                                        ui.allocate_rect(btn_miss_rect, Sense::click());
                                    let btn_miss_bg = if btn_miss_res.hovered() {
                                        Tokens::ACCENT
                                    } else {
                                        Tokens::BG_HEADER
                                    };
                                    ui.painter()
                                        .rect_filled(btn_miss_rect, 2.0_f32, btn_miss_bg);
                                    ui.painter().rect_stroke(
                                        btn_miss_rect,
                                        2.0_f32,
                                        Stroke::new(1.0_f32, Tokens::BORDER),
                                    );
                                    ui.painter().text(
                                        btn_miss_rect.center(),
                                        Align2::CENTER_CENTER,
                                        "...",
                                        FontId::proportional(10.0),
                                        Tokens::TEXT_PRIMARY,
                                    );
                                    if btn_miss_res.clicked() {
                                        view_action = VariableViewAction::OpenMissingValues(i);
                                    }

                                    // 8. Align Cell (Click to cycle Left -> Right -> Center)
                                    let (rect, res) = ui.allocate_exact_size(
                                        vec2(headers[8].1, row_height),
                                        Sense::click(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );

                                    let align_label = match var.align {
                                        Alignment::Left => t!("align-left"),
                                        Alignment::Right => t!("align-right"),
                                        Alignment::Center => t!("align-center"),
                                    };
                                    ui.painter().text(
                                        pos2(rect.min.x + 6.0, rect.center().y),
                                        Align2::LEFT_CENTER,
                                        align_label,
                                        FontId::proportional(11.0),
                                        Tokens::TEXT_PRIMARY,
                                    );

                                    if res.clicked() {
                                        let next = match var.align {
                                            Alignment::Left => Alignment::Right,
                                            Alignment::Right => Alignment::Center,
                                            Alignment::Center => Alignment::Left,
                                        };
                                        new_align = Some(next);
                                    }

                                    // 9. Measure Cell (Vector icon + click to cycle Scale -> Ordinal -> Nominal)
                                    let (rect, res) = ui.allocate_exact_size(
                                        vec2(headers[9].1, row_height),
                                        Sense::click(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );

                                    let icon_rect = Rect::from_min_size(
                                        pos2(rect.min.x + 6.0_f32, rect.min.y + 4.0_f32),
                                        vec2(16.0_f32, 16.0_f32),
                                    );
                                    paint_measure_icon(ui.painter(), icon_rect, var.measure);

                                    let measure_text = t!(var.measure.as_key());
                                    ui.painter().text(
                                        pos2(rect.min.x + 28.0_f32, rect.center().y),
                                        Align2::LEFT_CENTER,
                                        measure_text,
                                        FontId::proportional(11.0),
                                        Tokens::TEXT_PRIMARY,
                                    );

                                    if res.clicked() {
                                        let next = match var.measure {
                                            Measure::Scale => Measure::Ordinal,
                                            Measure::Ordinal => Measure::Nominal,
                                            Measure::Nominal => Measure::Scale,
                                        };
                                        new_measure = Some(next);
                                    }

                                    // 10. Role Cell (Click to cycle Input -> Target -> Both -> None)
                                    let (rect, res) = ui.allocate_exact_size(
                                        vec2(headers[10].1, row_height),
                                        Sense::click(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );

                                    let role_text = match var.role {
                                        Role::Input => t!("role-input"),
                                        Role::Target => t!("role-target"),
                                        Role::Both => t!("role-both"),
                                        Role::None => t!("role-none"),
                                        Role::Partition => t!("role-partition"),
                                        Role::Split => t!("role-split"),
                                    };
                                    ui.painter().text(
                                        pos2(rect.min.x + 6.0, rect.center().y),
                                        Align2::LEFT_CENTER,
                                        role_text,
                                        FontId::proportional(11.0),
                                        Tokens::TEXT_PRIMARY,
                                    );

                                    if res.clicked() {
                                        let next = match var.role {
                                            Role::Input => Role::Target,
                                            Role::Target => Role::Both,
                                            Role::Both => Role::None,
                                            _ => Role::Input,
                                        };
                                        new_role = Some(next);
                                    }
                                }

                                if let Some(label) = new_label {
                                    let _ = ds.dictionary.set_label(i, &label);
                                }
                                if let Some(measure) = new_measure {
                                    let _ = ds.dictionary.set_measure(i, measure);
                                }
                                if let Some(align) = new_align {
                                    let _ = ds.dictionary.set_align(i, align);
                                }
                                if let Some(role) = new_role {
                                    let _ = ds.dictionary.set_role(i, role);
                                }
                                if width_delta != 0 {
                                    if let Some(var) = ds.dictionary.get(i) {
                                        let new_w =
                                            (var.width as isize + width_delta).max(1) as usize;
                                        let _ = ds.dictionary.set_width(i, new_w);
                                    }
                                }
                                if decimals_delta != 0 {
                                    if let Some(var) = ds.dictionary.get(i) {
                                        let new_d = (var.decimals as isize + decimals_delta)
                                            .clamp(0, 16)
                                            as u8;
                                        let _ = ds.dictionary.set_decimals(i, new_d);
                                    }
                                }
                            });
                        } else {
                            // Row i == var_count: The Ghost Row for adding new variables!
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                                // 0. Star '*' header
                                let (rect, _) = ui.allocate_exact_size(
                                    vec2(headers[0].1, row_height),
                                    Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_HEADER);
                                ui.painter().rect_stroke(
                                    rect,
                                    0.0_f32,
                                    Stroke::new(0.5_f32, Tokens::BORDER),
                                );
                                ui.painter().text(
                                    rect.center(),
                                    Align2::CENTER_CENTER,
                                    "*",
                                    FontId::proportional(12.0),
                                    Tokens::TEXT_MUTED,
                                );

                                // 1. Name input placeholder
                                let (rect, _) = ui.allocate_exact_size(
                                    vec2(headers[1].1, row_height),
                                    Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                ui.painter().rect_stroke(
                                    rect,
                                    0.0_f32,
                                    Stroke::new(0.5_f32, Tokens::BORDER),
                                );

                                let inner_rect = rect.shrink2(vec2(2.0_f32, 1.0_f32));
                                ui.allocate_new_ui(
                                    egui::UiBuilder::new().max_rect(inner_rect),
                                    |ui| {
                                        let text_edit =
                                            egui::TextEdit::singleline(&mut state.new_var_name)
                                                .hint_text(t!("variable-new-placeholder"))
                                                .frame(false)
                                                .clip_text(true);
                                        let res = ui.add_sized(inner_rect.size(), text_edit);

                                        if res.lost_focus() && !state.new_var_name.trim().is_empty()
                                        {
                                            let name = state.new_var_name.trim().to_string();
                                            state.new_var_name.clear();
                                            let _ = ds.add_variable(
                                                &name,
                                                statcraft_core::DataType::Numeric { decimals: 2 },
                                                Measure::Scale,
                                            );
                                        }
                                    },
                                );

                                // Empty trailing cells for ghost row
                                for (_, width) in headers.iter().skip(2) {
                                    let (rect, _) = ui.allocate_exact_size(
                                        vec2(*width, row_height),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 0.0_f32, bg_color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0_f32,
                                        Stroke::new(0.5_f32, Tokens::BORDER),
                                    );
                                }
                            });
                        }
                    }
                });
        });

    if let Some((idx, new_name)) = rename_action {
        let _ = ds.rename_variable(idx, &new_name);
    }
    if let Some(idx) = action_delete_var {
        let _ = ds.remove_variable(idx);
    }
    if let Some(_idx) = action_insert_var {
        let name = format!("VAR{:05}", ds.dictionary.count() + 1);
        let _ = ds.add_variable(
            &name,
            statcraft_core::DataType::Numeric { decimals: 2 },
            Measure::Scale,
        );
    }

    view_action
}
