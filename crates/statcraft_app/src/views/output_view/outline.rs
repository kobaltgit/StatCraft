use super::extract_item_tsv;
use super::model::{OutputItemKind, OutputViewerState, SelectedTarget};
use crate::theme::Tokens;
use crate::widgets::output_icons::{
    paint_icon_collapse_all, paint_icon_expand_all, paint_icon_trash, paint_output_folder_icon,
    paint_output_log_icon, paint_output_notes_icon, paint_output_root_icon,
    paint_output_table_icon, paint_output_warning_icon,
};
use crate::widgets::vector_arrow::paint_vector_arrow;
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_i18n::t;

pub fn render_outline_pane(ui: &mut Ui, state: &mut OutputViewerState) {
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0_f32, Tokens::BG_PANEL);
    ui.painter()
        .rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::BORDER));

    ui.vertical(|ui| {
        // Outline Toolbar at top
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
            ui.add_space(4.0_f32);

            // Clear All Button
            let btn_size = vec2(20.0, 20.0);
            let (trash_rect, trash_resp) = ui.allocate_exact_size(btn_size, Sense::click());
            if trash_resp.hovered() {
                ui.painter()
                    .rect_filled(trash_rect, 2.0_f32, Tokens::BG_ROW_EVEN);
            }
            paint_icon_trash(ui.painter(), trash_rect, Tokens::TEXT_PRIMARY);
            let trash_resp = trash_resp.on_hover_text(t!("output-btn-clear-all"));
            if trash_resp.clicked() {
                state.clear();
            }

            // Expand All Button
            let (exp_rect, exp_resp) = ui.allocate_exact_size(btn_size, Sense::click());
            if exp_resp.hovered() {
                ui.painter()
                    .rect_filled(exp_rect, 2.0_f32, Tokens::BG_ROW_EVEN);
            }
            paint_icon_expand_all(ui.painter(), exp_rect, Tokens::TEXT_PRIMARY);
            let exp_resp = exp_resp.on_hover_text(t!("output-btn-expand-all"));
            if exp_resp.clicked() {
                state.expand_all();
            }

            // Collapse All Button
            let (col_rect, col_resp) = ui.allocate_exact_size(btn_size, Sense::click());
            if col_resp.hovered() {
                ui.painter()
                    .rect_filled(col_rect, 2.0_f32, Tokens::BG_ROW_EVEN);
            }
            paint_icon_collapse_all(ui.painter(), col_rect, Tokens::TEXT_PRIMARY);
            let col_resp = col_resp.on_hover_text(t!("output-btn-collapse-all"));
            if col_resp.clicked() {
                state.collapse_all();
            }
        });

        ui.add_space(2.0_f32);
        ui.separator();

        ScrollArea::vertical()
            .id_salt("output_outline_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(4.0_f32);

                // Root Document Item
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                    let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                    paint_output_root_icon(ui.painter(), icon_rect, Tokens::ACCENT);
                    ui.label(t!("output-outline-root"));
                });

                ui.add_space(4.0_f32);

                // Procedures Tree
                let mut clicked_target: Option<SelectedTarget> = None;
                let mut toggle_proc: Option<usize> = None;
                let mut delete_item: Option<(usize, usize)> = None;
                let mut delete_proc: Option<usize> = None;
                let mut copied_item: Option<usize> = None;

                for (p_idx, proc) in state.procedures.iter().enumerate() {
                    let is_proc_selected = match state.selected_target {
                        Some(SelectedTarget::Procedure(idx)) => idx == p_idx,
                        _ => false,
                    };

                    // Procedure Node Row
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                        ui.add_space(8.0_f32);

                        // Arrow Toggle
                        let (arrow_rect, arrow_resp) =
                            ui.allocate_exact_size(vec2(12.0, 16.0), Sense::click());
                        paint_vector_arrow(
                            ui.painter(),
                            arrow_rect,
                            proc.is_expanded,
                            Tokens::TEXT_MUTED,
                        );
                        if arrow_resp.clicked() {
                            toggle_proc = Some(p_idx);
                        }

                        // Folder Icon
                        let (folder_rect, _) =
                            ui.allocate_exact_size(vec2(14.0, 16.0), Sense::hover());
                        let folder_color = if is_proc_selected {
                            Tokens::ACCENT
                        } else {
                            Tokens::TEXT_PRIMARY
                        };
                        paint_output_folder_icon(
                            ui.painter(),
                            folder_rect,
                            proc.is_expanded,
                            folder_color,
                        );

                        // Procedure Title
                        let item_w = (ui.available_width() - 8.0).max(40.0);
                        let (text_rect, text_resp) =
                            ui.allocate_exact_size(vec2(item_w, 18.0), Sense::click());

                        if is_proc_selected {
                            ui.painter()
                                .rect_filled(text_rect, 2.0_f32, Tokens::BG_ROW_ODD);
                            ui.painter().rect_stroke(
                                text_rect,
                                2.0_f32,
                                Stroke::new(1.0_f32, Tokens::ACCENT),
                            );
                        } else if text_resp.hovered() {
                            ui.painter()
                                .rect_filled(text_rect, 2.0_f32, Tokens::BG_ROW_EVEN);
                        }

                        let text_color = if is_proc_selected {
                            Tokens::ACCENT
                        } else {
                            Tokens::TEXT_PRIMARY
                        };

                        let title = proc.title();
                        let display_title = truncate_text_to_width(
                            ui,
                            &title,
                            &FontId::proportional(12.0),
                            (text_rect.width() - 4.0).max(10.0),
                        );

                        ui.painter().with_clip_rect(text_rect).text(
                            pos2(text_rect.min.x + 2.0, text_rect.center().y),
                            Align2::LEFT_CENTER,
                            display_title,
                            FontId::proportional(12.0),
                            text_color,
                        );

                        let text_resp = text_resp.on_hover_text(title);

                        if text_resp.clicked() || text_resp.secondary_clicked() {
                            clicked_target = Some(SelectedTarget::Procedure(p_idx));
                        }

                        text_resp.context_menu(|ui| {
                            if ui.button(t!("output-menu-delete")).clicked() {
                                delete_proc = Some(p_idx);
                                ui.close_menu();
                            }
                        });
                    });

                    // Child Items (if expanded)
                    if proc.is_expanded {
                        for (i_idx, item) in proc.items.iter().enumerate() {
                            let is_item_selected = match state.selected_target {
                                Some(SelectedTarget::Item { proc_idx, item_idx }) => {
                                    proc_idx == p_idx && item_idx == i_idx
                                }
                                _ => false,
                            };

                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                                ui.add_space(26.0_f32);

                                // Specific item icon
                                let (icon_rect, _) =
                                    ui.allocate_exact_size(vec2(14.0, 16.0), Sense::hover());
                                let icon_color = if is_item_selected {
                                    Tokens::ACCENT
                                } else {
                                    Tokens::TEXT_MUTED
                                };

                                match &item.kind {
                                    OutputItemKind::Log(_) => {
                                        paint_output_log_icon(ui.painter(), icon_rect, icon_color)
                                    }
                                    OutputItemKind::Notes(_) => {
                                        paint_output_notes_icon(ui.painter(), icon_rect, icon_color)
                                    }
                                    OutputItemKind::Warning(_) => paint_output_warning_icon(
                                        ui.painter(),
                                        icon_rect,
                                        icon_color,
                                    ),
                                    _ => {
                                        paint_output_table_icon(ui.painter(), icon_rect, icon_color)
                                    }
                                }

                                let item_w = (ui.available_width() - 8.0).max(40.0);
                                let (text_rect, text_resp) =
                                    ui.allocate_exact_size(vec2(item_w, 18.0), Sense::click());

                                if is_item_selected {
                                    ui.painter().rect_filled(
                                        text_rect,
                                        2.0_f32,
                                        Tokens::BG_ROW_ODD,
                                    );
                                    ui.painter().rect_stroke(
                                        text_rect,
                                        2.0_f32,
                                        Stroke::new(1.0_f32, Tokens::ACCENT),
                                    );
                                } else if text_resp.hovered() {
                                    ui.painter().rect_filled(
                                        text_rect,
                                        2.0_f32,
                                        Tokens::BG_ROW_EVEN,
                                    );
                                }

                                let text_color = if is_item_selected {
                                    Tokens::ACCENT
                                } else {
                                    Tokens::TEXT_PRIMARY
                                };

                                let title = item.title();
                                let display_title = truncate_text_to_width(
                                    ui,
                                    &title,
                                    &FontId::proportional(11.5),
                                    (text_rect.width() - 4.0).max(10.0),
                                );

                                ui.painter().with_clip_rect(text_rect).text(
                                    pos2(text_rect.min.x + 2.0, text_rect.center().y),
                                    Align2::LEFT_CENTER,
                                    display_title,
                                    FontId::proportional(11.5),
                                    text_color,
                                );

                                let text_resp = text_resp.on_hover_text(title);

                                if text_resp.clicked() || text_resp.secondary_clicked() {
                                    clicked_target = Some(SelectedTarget::Item {
                                        proc_idx: p_idx,
                                        item_idx: i_idx,
                                    });
                                }

                                let maybe_tsv = extract_item_tsv(&item.kind);
                                let item_title = item.title().to_string();
                                text_resp.context_menu(|ui| {
                                    if let Some(tsv_data) = &maybe_tsv {
                                        let is_copied = state.last_copied_id == Some(item.id);
                                        let label = if is_copied {
                                            t!("output-menu-copied")
                                        } else {
                                            t!("output-menu-copy-tsv")
                                        };
                                        if ui.button(label).clicked() {
                                            ui.ctx()
                                                .output_mut(|o| o.copied_text = tsv_data.clone());
                                            copied_item = Some(item.id);
                                            ui.close_menu();
                                        }
                                    }
                                    if ui.button(t!("output-menu-copy-text")).clicked() {
                                        ui.ctx().output_mut(|o| o.copied_text = item_title.clone());
                                        ui.close_menu();
                                    }
                                    ui.separator();
                                    if ui.button(t!("output-menu-delete")).clicked() {
                                        delete_item = Some((p_idx, i_idx));
                                        ui.close_menu();
                                    }
                                });
                            });
                        }
                    }

                    ui.add_space(2.0_f32);
                }

                if let Some(p_idx) = toggle_proc {
                    if let Some(proc) = state.procedures.get_mut(p_idx) {
                        proc.is_expanded = !proc.is_expanded;
                    }
                }

                if let Some((p_idx, i_idx)) = delete_item {
                    if let Some(proc) = state.procedures.get_mut(p_idx) {
                        if i_idx < proc.items.len() {
                            proc.items.remove(i_idx);
                        }
                        if proc.items.is_empty() {
                            delete_proc = Some(p_idx);
                        }
                    }
                    state.selected_target = None;
                }

                if let Some(p_idx) = delete_proc {
                    if p_idx < state.procedures.len() {
                        state.procedures.remove(p_idx);
                    }
                    state.selected_target = None;
                }

                if let Some(target) = clicked_target {
                    state.selected_target = Some(target);
                    state.scroll_to_target = Some(target);
                }

                if let Some(id) = copied_item {
                    state.last_copied_id = Some(id);
                }
            });
    });
}

fn truncate_text_to_width(ui: &Ui, text: &str, font: &FontId, max_width: f32) -> String {
    let full_width = ui.fonts(|f| {
        f.layout_no_wrap(text.to_string(), font.clone(), egui::Color32::WHITE)
            .size()
            .x
    });
    if full_width <= max_width {
        return text.to_string();
    }
    let ellipsis = "...";
    let ellipsis_w = ui.fonts(|f| {
        f.layout_no_wrap(ellipsis.to_string(), font.clone(), egui::Color32::WHITE)
            .size()
            .x
    });
    if ellipsis_w >= max_width {
        return String::new();
    }
    let target_w = max_width - ellipsis_w;
    let mut truncated = String::new();
    for ch in text.chars() {
        let mut test_s = truncated.clone();
        test_s.push(ch);
        let w = ui.fonts(|f| {
            f.layout_no_wrap(test_s.clone(), font.clone(), egui::Color32::WHITE)
                .size()
                .x
        });
        if w > target_w {
            break;
        }
        truncated = test_s;
    }
    truncated.push_str(ellipsis);
    truncated
}
