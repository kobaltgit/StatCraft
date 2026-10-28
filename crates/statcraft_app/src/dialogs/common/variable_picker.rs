use crate::theme::Tokens;
use crate::widgets::paint_measure_icon;
use egui::{vec2, Align, Layout, ScrollArea, Sense, Stroke, Ui, Vec2};
use statcraft_core::Measure;

pub fn format_var_display(name: &str, label: &str) -> String {
    let trimmed = label.trim();
    if !trimmed.is_empty() {
        format!("{} [{}]", trimmed, name)
    } else {
        name.to_string()
    }
}

pub struct VariableListItem<'a> {
    pub name: &'a str,
    pub display_text: String,
    pub measure: Measure,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListEvent {
    None,
    Clicked(usize),
    DoubleClicked(usize),
}

pub fn render_variable_list_panel(
    ui: &mut Ui,
    title: impl AsRef<str>,
    panel_size: Vec2,
    list_id: &str,
    items: &[VariableListItem],
) -> ListEvent {
    let mut event = ListEvent::None;

    ui.allocate_ui_with_layout(panel_size, Layout::top_down(Align::Min), |ui| {
        ui.label(title.as_ref());
        egui::Frame::canvas(ui.style())
            .fill(Tokens::BG_PANEL)
            .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
            .show(ui, |ui| {
                let inner_size = vec2(panel_size.x - 4.0, panel_size.y - 24.0);
                ui.set_min_size(inner_size);
                ui.set_max_size(inner_size);

                ScrollArea::vertical()
                    .id_salt(list_id)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (idx, item) in items.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let icon_size = vec2(14.0, 14.0);
                                let (rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
                                paint_measure_icon(ui.painter(), rect, item.measure);

                                let res = ui.selectable_label(item.is_selected, &item.display_text);
                                if res.clicked() {
                                    event = ListEvent::Clicked(idx);
                                }
                                if res.double_clicked() {
                                    event = ListEvent::DoubleClicked(idx);
                                }
                            });
                        }
                    });
            });
    });

    event
}
