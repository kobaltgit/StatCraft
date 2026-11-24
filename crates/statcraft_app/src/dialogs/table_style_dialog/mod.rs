pub mod color_palette;
pub mod condition_subdialog;
pub mod format_masks;
pub mod format_subdialog;
pub mod state;

pub use color_palette::*;
pub use condition_subdialog::*;
pub use format_masks::*;
pub use format_subdialog::*;
pub use state::*;

use crate::dialogs::common::{DialogShell, DialogSize, SubdialogAction, SubdialogActionBar};
use crate::theme::Tokens;
use egui::{vec2, Button, Context, Label, RichText, ScrollArea, Stroke, Ui};
use statcraft_i18n::t;
use statcraft_reporting::styling::{
    CellCondition, CellStyleFormat, DimensionTarget, TableTarget, TargetValue,
};

pub fn render_table_style_dialog(ctx: &Context, state: &mut TableStyleState) {
    if !state.is_open {
        return;
    }

    // Render nested modal subdialogs if active
    if state.editing_condition_idx.is_some() {
        render_condition_subdialog(ctx, state);
    }
    if state.editing_format_idx.is_some() {
        render_format_subdialog(ctx, state);
    }

    let mut open = true;
    let shell = DialogShell::new(
        t!("dialog-table-style-title"),
        DialogSize::Custom(vec2(660.0, 440.0)),
    );

    shell.show(ctx, &mut open, |ui| {
        render_table_style_content(ui, state);
    });

    if !open {
        state.cancel();
    }
}

fn render_table_style_content(ui: &mut Ui, state: &mut TableStyleState) {
    ui.label(t!("dialog-table-style-desc-1"));
    ui.label(
        RichText::new(t!("dialog-table-style-desc-2"))
            .size(11.0)
            .color(Tokens::text_muted()),
    );
    ui.add_space(8.0);

    // Main row: Rules Table on left, management buttons on right
    ui.horizontal(|ui| {
        // Table area (530x230) inside vertical block
        ui.vertical(|ui| {
            ui.label(RichText::new(t!("dialog-table-style-conditions-header")).strong());
            ui.add_space(4.0);
            let table_size = vec2(530.0, 230.0);
            render_rules_table(ui, state, table_size);
        });

        ui.add_space(8.0);

        // Sidebar buttons
        ui.vertical(|ui| {
            let btn_size = vec2(96.0, 24.0);

            if ui
                .add_sized(btn_size, Button::new(t!("dialog-table-style-btn-add")))
                .clicked()
            {
                state.add_rule();
            }

            let can_dup = state.selected_rule_idx.is_some();
            if ui
                .add_enabled_ui(can_dup, |ui| {
                    ui.add_sized(
                        btn_size,
                        Button::new(t!("dialog-table-style-btn-duplicate")),
                    )
                })
                .inner
                .clicked()
            {
                state.duplicate_rule();
            }

            let can_up = match state.selected_rule_idx {
                Some(idx) => idx > 0,
                None => false,
            };
            if ui
                .add_enabled_ui(can_up, |ui| {
                    ui.add_sized(btn_size, Button::new(t!("dialog-table-style-btn-move-up")))
                })
                .inner
                .clicked()
            {
                state.move_rule_up();
            }

            let can_down = match state.selected_rule_idx {
                Some(idx) => idx + 1 < state.rules.len(),
                None => false,
            };
            if ui
                .add_enabled_ui(can_down, |ui| {
                    ui.add_sized(
                        btn_size,
                        Button::new(t!("dialog-table-style-btn-move-down")),
                    )
                })
                .inner
                .clicked()
            {
                state.move_rule_down();
            }

            ui.add_space(76.0);

            let can_del = state.selected_rule_idx.is_some();
            if ui
                .add_enabled_ui(can_del, |ui| {
                    ui.add_sized(btn_size, Button::new(t!("dialog-table-style-btn-delete")))
                })
                .inner
                .clicked()
            {
                state.delete_rule();
            }
        });
    });

    ui.add_space(8.0);
    ui.checkbox(
        &mut state.create_style_report,
        t!("dialog-table-style-create-report"),
    );

    // Bottom Action Bar
    let action = SubdialogActionBar::default().show(ui);
    match action {
        SubdialogAction::Continue => {
            state.confirm();
        }
        SubdialogAction::Cancel => {
            state.cancel();
        }
        _ => {}
    }
}

fn render_rules_table(ui: &mut Ui, state: &mut TableStyleState, size: egui::Vec2) {
    let stroke = Stroke::new(1.0_f32, Tokens::border());
    let mut clicked_idx: Option<usize> = None;
    let mut edit_cond_idx: Option<usize> = None;
    let mut edit_fmt_idx: Option<usize> = None;

    egui::Frame::none()
        .fill(Tokens::bg_panel())
        .stroke(stroke)
        .inner_margin(egui::Margin::same(2.0))
        .show(ui, |ui| {
            ui.set_min_size(size);
            ui.set_max_size(size);

            // Table Header (5 columns)
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.add_sized(
                    [130.0, 20.0],
                    Label::new(RichText::new(t!("dialog-table-style-col-table")).strong()),
                );
                ui.add_sized(
                    [120.0, 20.0],
                    Label::new(RichText::new(t!("dialog-table-style-col-value")).strong()),
                );
                ui.add_sized(
                    [90.0, 20.0],
                    Label::new(RichText::new(t!("dialog-table-style-col-dimension")).strong()),
                );
                ui.add_sized(
                    [90.0, 20.0],
                    Label::new(RichText::new(t!("dialog-table-style-col-condition")).strong()),
                );
                ui.add_sized(
                    [90.0, 20.0],
                    Label::new(RichText::new(t!("dialog-table-style-col-format")).strong()),
                );
            });
            ui.separator();

            // Table Rows
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for idx in 0..state.rules.len() {
                        let is_selected = state.selected_rule_idx == Some(idx);
                        let bg_color = if is_selected {
                            Tokens::bg_selected()
                        } else if idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };

                        let row_rect = egui::Frame::none()
                            .fill(bg_color)
                            .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let rule = &mut state.rules[idx];

                                    // 1. Таблица (ComboBox)
                                    egui::ComboBox::from_id_salt(("table_target", idx))
                                        .width(125.0)
                                        .selected_text(table_target_display(&rule.table_target))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut rule.table_target,
                                                TableTarget::AllEligible,
                                                t!("dialog-table-style-target-all"),
                                            );
                                            ui.selectable_value(
                                                &mut rule.table_target,
                                                TableTarget::Subtype("Frequencies".into()),
                                                t!("dialog-table-style-target-freq"),
                                            );
                                            ui.selectable_value(
                                                &mut rule.table_target,
                                                TableTarget::Subtype("Statistics".into()),
                                                t!("dialog-table-style-target-stats"),
                                            );
                                        });

                                    // 2. Значение (ComboBox)
                                    egui::ComboBox::from_id_salt(("value_target", idx))
                                        .width(115.0)
                                        .selected_text(value_target_display(&rule.value_target))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut rule.value_target,
                                                TargetValue::AllDataCells,
                                                t!("dialog-table-style-val-all"),
                                            );
                                            ui.selectable_value(
                                                &mut rule.value_target,
                                                TargetValue::Label("Count".into()),
                                                t!("dialog-table-style-val-count"),
                                            );
                                            ui.selectable_value(
                                                &mut rule.value_target,
                                                TargetValue::Label("Mean".into()),
                                                t!("dialog-table-style-val-mean"),
                                            );
                                            ui.selectable_value(
                                                &mut rule.value_target,
                                                TargetValue::Label("Median".into()),
                                                t!("dialog-table-style-val-median"),
                                            );
                                            ui.selectable_value(
                                                &mut rule.value_target,
                                                TargetValue::Label("Percent".into()),
                                                t!("dialog-table-style-val-percent"),
                                            );
                                        });

                                    // 3. измерение (ComboBox)
                                    egui::ComboBox::from_id_salt(("dim_target", idx))
                                        .width(85.0)
                                        .selected_text(dimension_display(rule.dimension))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut rule.dimension,
                                                DimensionTarget::Columns,
                                                t!("dialog-table-style-dim-columns"),
                                            );
                                            ui.selectable_value(
                                                &mut rule.dimension,
                                                DimensionTarget::Rows,
                                                t!("dialog-table-style-dim-rows"),
                                            );
                                            ui.selectable_value(
                                                &mut rule.dimension,
                                                DimensionTarget::Both,
                                                t!("dialog-table-style-dim-both"),
                                            );
                                        });

                                    // 4. Условие (Кликабельная ячейка)
                                    let cond_str = format_condition_summary(&rule.condition);
                                    let cond_btn = Button::new(RichText::new(cond_str).underline())
                                        .frame(false)
                                        .min_size(vec2(85.0, 20.0));
                                    if ui.add(cond_btn).clicked() {
                                        clicked_idx = Some(idx);
                                        edit_cond_idx = Some(idx);
                                    }

                                    // 5. Формат (Кликабельная ячейка)
                                    let fmt_str = format_style_summary(&rule.format);
                                    let fmt_btn = Button::new(RichText::new(fmt_str).underline())
                                        .frame(false)
                                        .min_size(vec2(85.0, 20.0));
                                    if ui.add(fmt_btn).clicked() {
                                        clicked_idx = Some(idx);
                                        edit_fmt_idx = Some(idx);
                                    }
                                });
                            })
                            .response
                            .rect;

                        // Clicking anywhere on row selects it
                        if ui.rect_contains_pointer(row_rect)
                            && ui.input(|i| i.pointer.primary_clicked())
                        {
                            clicked_idx = Some(idx);
                        }
                    }
                });
        });

    if let Some(idx) = clicked_idx {
        state.selected_rule_idx = Some(idx);
    }
    if let Some(idx) = edit_cond_idx {
        state.open_condition_editor(idx);
    }
    if let Some(idx) = edit_fmt_idx {
        state.open_format_editor(idx);
    }
}

pub fn table_target_display(target: &TableTarget) -> String {
    match target {
        TableTarget::AllEligible => t!("dialog-table-style-target-all"),
        TableTarget::Subtype(s) => {
            if s == "Frequencies" {
                t!("dialog-table-style-target-freq")
            } else if s == "Statistics" {
                t!("dialog-table-style-target-stats")
            } else {
                s.clone()
            }
        }
    }
}

pub fn value_target_display(target: &TargetValue) -> String {
    match target {
        TargetValue::AllDataCells => t!("dialog-table-style-val-all"),
        TargetValue::Label(s) => {
            if s == "Count" || s == "Frequency" {
                t!("dialog-table-style-val-count")
            } else if s == "Mean" {
                t!("dialog-table-style-val-mean")
            } else if s == "Median" {
                t!("dialog-table-style-val-median")
            } else if s == "Percent" {
                t!("dialog-table-style-val-percent")
            } else {
                s.clone()
            }
        }
    }
}

pub fn dimension_display(dim: DimensionTarget) -> String {
    match dim {
        DimensionTarget::Columns => t!("dialog-table-style-dim-columns"),
        DimensionTarget::Rows => t!("dialog-table-style-dim-rows"),
        DimensionTarget::Both => t!("dialog-table-style-dim-both"),
    }
}

pub fn format_condition_summary(cond: &CellCondition) -> String {
    match cond {
        CellCondition::All => t!("dialog-table-style-cond-all-values"),
        CellCondition::Equal(v) => format!("= {}", v),
        CellCondition::NotEqual(v) => format!("!= {}", v),
        CellCondition::LessThan(v) => format!("< {}", v),
        CellCondition::LessOrEqual(v) => format!("<= {}", v),
        CellCondition::GreaterThan(v) => format!("> {}", v),
        CellCondition::GreaterOrEqual(v) => format!(">= {}", v),
        CellCondition::Between(low, high) => format!("{} .. {}", low, high),
        CellCondition::NotBetween(low, high) => format!("!({} .. {})", low, high),
        CellCondition::TopN(n) => format!("Top {}", n),
        CellCondition::BottomN(n) => format!("Bottom {}", n),
        CellCondition::SysMis => t!("dialog-table-style-cond-op-sysmis"),
    }
}

pub fn format_style_summary(fmt: &CellStyleFormat) -> String {
    if fmt.reset_defaults {
        return t!("dialog-table-style-fmt-reset");
    }

    let mut parts = Vec::new();
    if fmt.bold {
        parts.push(t!("dialog-table-style-fmt-bold"));
    }
    if fmt.italic {
        parts.push(t!("dialog-table-style-fmt-italic"));
    }
    if fmt.underline {
        parts.push(t!("dialog-table-style-fmt-underline"));
    }
    if fmt.text_color.is_some() {
        parts.push(t!("dialog-table-style-format-text-color"));
    }
    if fmt.bg_color.is_some() {
        parts.push(t!("dialog-table-style-format-bg-color"));
    }

    if parts.is_empty() {
        t!("dialog-table-style-fmt-none")
    } else {
        parts.join(", ")
    }
}
