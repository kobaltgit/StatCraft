use crate::theme::Tokens;
use egui::{Color32, Response, Ui};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarPosition {
    Top,
    Left,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    OpenData,
    SaveData,
    Print,
    RecallDialogs,
    Undo,
    Redo,
    GoToCase,
    GoToVariable,
    VariablesInfo,
    Find,
    InsertCase,
    InsertVariable,
    SplitFile,
    SelectCases,
    WeightCases,
    ToggleValueLabels,
    Descriptives,
    Frequencies,
}

pub fn render_tool_button(
    ui: &mut Ui,
    icon: &str,
    tooltip: &str,
    enabled: bool,
    active: bool,
) -> Response {
    let btn_size = egui::vec2(26.0, 26.0);
    let (rect, resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());

    let (bg, tint) = if !enabled {
        (Color32::TRANSPARENT, Tokens::TEXT_MUTED)
    } else if active {
        (Tokens::BG_SELECTED, Color32::WHITE)
    } else if resp.is_pointer_button_down_on() {
        (Tokens::CAPTION_PRESSED, Color32::WHITE)
    } else if resp.hovered() {
        (Tokens::CAPTION_HOVER, Color32::WHITE)
    } else {
        (Color32::TRANSPARENT, Tokens::TEXT_PRIMARY)
    };

    if bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 3.0, bg);
        if active {
            ui.painter()
                .rect_stroke(rect, 3.0, egui::Stroke::new(1.0_f32, Tokens::ACCENT));
        }
    }

    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(16.0, 16.0));
    crate::icons::icon_image(icon, 16.0, tint).paint_at(ui, icon_rect);

    if enabled {
        resp.on_hover_text(tooltip)
    } else {
        resp.on_disabled_hover_text(format!("{} {}", tooltip, t!("menu-badge-dev")))
    }
}

pub fn render_toolbar_separator(ui: &mut Ui, is_horizontal: bool) {
    if is_horizontal {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 22.0), egui::Sense::hover());
        let center_x = rect.center().x;
        ui.painter().line_segment(
            [
                egui::pos2(center_x, rect.top() + 3.0),
                egui::pos2(center_x, rect.bottom() - 3.0),
            ],
            egui::Stroke::new(1.0_f32, Tokens::BORDER),
        );
    } else {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 8.0), egui::Sense::hover());
        let center_y = rect.center().y;
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 3.0, center_y),
                egui::pos2(rect.right() - 3.0, center_y),
            ],
            egui::Stroke::new(1.0_f32, Tokens::BORDER),
        );
    }
}

pub fn render_toolbar(
    ui: &mut Ui,
    position: ToolbarPosition,
    has_dataset: bool,
    can_undo: bool,
    can_redo: bool,
    show_value_labels: bool,
) -> ToolbarAction {
    let mut action = ToolbarAction::None;
    let is_horizontal = position == ToolbarPosition::Top;

    let mut add_buttons = |ui: &mut Ui| {
        ui.spacing_mut().item_spacing = if is_horizontal {
            egui::vec2(2.0, 0.0)
        } else {
            egui::vec2(0.0, 2.0)
        };

        // --- 1. File Ops ---
        if render_tool_button(ui, "folder-open", &t!("toolbar-open"), true, false).clicked() {
            action = ToolbarAction::OpenData;
        }
        if render_tool_button(ui, "save", &t!("toolbar-save"), has_dataset, false).clicked()
            && has_dataset
        {
            action = ToolbarAction::SaveData;
        }
        if render_tool_button(ui, "printer", &t!("toolbar-print"), has_dataset, false).clicked()
            && has_dataset
        {
            action = ToolbarAction::Print;
        }

        render_toolbar_separator(ui, is_horizontal);

        // --- 2. Recall Dialogs & Undo/Redo ---
        if render_tool_button(
            ui,
            "clock",
            &t!("toolbar-recall-dialogs"),
            has_dataset,
            false,
        )
        .clicked()
            && has_dataset
        {
            action = ToolbarAction::RecallDialogs;
        }
        if render_tool_button(ui, "undo-2", &t!("toolbar-undo"), can_undo, false).clicked()
            && can_undo
        {
            action = ToolbarAction::Undo;
        }
        if render_tool_button(ui, "redo-2", &t!("toolbar-redo"), can_redo, false).clicked()
            && can_redo
        {
            action = ToolbarAction::Redo;
        }

        render_toolbar_separator(ui, is_horizontal);

        // --- 3. Navigation & Search ---
        if render_tool_button(
            ui,
            "arrow-down-to-line",
            &t!("toolbar-goto-case"),
            has_dataset,
            false,
        )
        .clicked()
            && has_dataset
        {
            action = ToolbarAction::GoToCase;
        }
        if render_tool_button(
            ui,
            "arrow-right-to-line",
            &t!("toolbar-goto-var"),
            has_dataset,
            false,
        )
        .clicked()
            && has_dataset
        {
            action = ToolbarAction::GoToVariable;
        }
        if render_tool_button(ui, "info", &t!("toolbar-var-info"), has_dataset, false).clicked()
            && has_dataset
        {
            action = ToolbarAction::VariablesInfo;
        }
        if render_tool_button(ui, "search", &t!("toolbar-find"), has_dataset, false).clicked()
            && has_dataset
        {
            action = ToolbarAction::Find;
        }

        render_toolbar_separator(ui, is_horizontal);

        // --- 4. Data Structure (Insert) ---
        if render_tool_button(
            ui,
            "rows-plus",
            &t!("toolbar-insert-case"),
            has_dataset,
            false,
        )
        .clicked()
            && has_dataset
        {
            action = ToolbarAction::InsertCase;
        }
        if render_tool_button(ui, "columns-plus", &t!("toolbar-insert-var"), true, false).clicked()
        {
            action = ToolbarAction::InsertVariable;
        }

        render_toolbar_separator(ui, is_horizontal);

        // --- 5. Data Preparation ---
        if render_tool_button(ui, "split", &t!("toolbar-split-file"), has_dataset, false).clicked()
            && has_dataset
        {
            action = ToolbarAction::SplitFile;
        }
        if render_tool_button(
            ui,
            "filter",
            &t!("toolbar-select-cases"),
            has_dataset,
            false,
        )
        .clicked()
            && has_dataset
        {
            action = ToolbarAction::SelectCases;
        }
        if render_tool_button(ui, "scale", &t!("toolbar-weight-cases"), has_dataset, false)
            .clicked()
            && has_dataset
        {
            action = ToolbarAction::WeightCases;
        }

        render_toolbar_separator(ui, is_horizontal);

        // --- 6. View Options & Quick Analysis ---
        if render_tool_button(
            ui,
            "value-labels",
            &t!("toolbar-value-labels"),
            has_dataset,
            show_value_labels,
        )
        .clicked()
            && has_dataset
        {
            action = ToolbarAction::ToggleValueLabels;
        }

        render_toolbar_separator(ui, is_horizontal);

        if render_tool_button(
            ui,
            "calculator",
            &t!("toolbar-descriptives"),
            has_dataset,
            false,
        )
        .clicked()
            && has_dataset
        {
            action = ToolbarAction::Descriptives;
        }
        if render_tool_button(
            ui,
            "chart-bar",
            &t!("toolbar-frequencies"),
            has_dataset,
            false,
        )
        .clicked()
            && has_dataset
        {
            action = ToolbarAction::Frequencies;
        }
    };

    if is_horizontal {
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            add_buttons(ui);
        });
    } else {
        ui.vertical(|ui| {
            ui.add_space(4.0);
            add_buttons(ui);
        });
    }

    action
}
