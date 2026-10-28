use egui::{Response, Ui, WidgetText};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItemStatus<'a> {
    Ready { shortcut: Option<&'a str> },
    ContextDisabled { reason: &'a str },
    InDev,
}

pub fn render_menu_item(
    ui: &mut Ui,
    label: impl Into<WidgetText>,
    status: MenuItemStatus<'_>,
) -> Response {
    let mut btn = egui::Button::new(label);
    match status {
        MenuItemStatus::Ready { shortcut } => {
            if let Some(sc) = shortcut {
                btn = btn.shortcut_text(sc);
            }
            ui.add(btn)
        }
        MenuItemStatus::ContextDisabled { reason } => {
            ui.add_enabled(false, btn).on_disabled_hover_text(reason)
        }
        MenuItemStatus::InDev => {
            let dev_badge = t!("menu-badge-dev");
            btn = btn.shortcut_text(dev_badge);
            let in_dev_tooltip = t!("menu-item-in-dev-tooltip");
            ui.add_enabled(false, btn)
                .on_disabled_hover_text(in_dev_tooltip)
        }
    }
}
