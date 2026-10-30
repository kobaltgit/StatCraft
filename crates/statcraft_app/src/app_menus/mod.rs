mod analyze_etc;
mod data_transform;
mod file_edit_view;

use crate::app::StatCraftApp;
use crate::widgets::MenuItemStatus;
use egui::Ui;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn render_single_menu(
        &mut self,
        ui: &mut Ui,
        index: usize,
        requires_dataset_status: MenuItemStatus,
    ) {
        match index {
            0..=2 => self.render_file_edit_view_menu(ui, index, requires_dataset_status),
            3..=4 => self.render_data_transform_menu(ui, index, requires_dataset_status),
            _ => self.render_analyze_etc_menu(ui, index, requires_dataset_status),
        }
    }

    pub(crate) fn render_top_menus(
        &mut self,
        ui: &mut Ui,
        controls_start_x: f32,
        requires_dataset_status: MenuItemStatus,
    ) -> f32 {
        const MENU_KEYS: [&str; 12] = [
            "menu-file",
            "menu-edit",
            "menu-view",
            "menu-data",
            "menu-transform",
            "menu-analyze",
            "menu-direct-marketing",
            "menu-graphs",
            "menu-utilities",
            "menu-extensions",
            "menu-window",
            "menu-help",
        ];

        let font_id = egui::FontId::proportional(13.0);
        let overflow_label = t!("menu-overflow");
        let overflow_w = ui.fonts(|f| {
            f.layout_no_wrap(
                overflow_label.to_string(),
                font_id.clone(),
                egui::Color32::WHITE,
            )
            .size()
            .x
        }) + 16.0;

        let mut overflow_from = None;

        for (i, &key) in MENU_KEYS.iter().enumerate() {
            let label = statcraft_i18n::tr(key);
            let text_w = ui.fonts(|f| {
                f.layout_no_wrap(label.to_string(), font_id.clone(), egui::Color32::WHITE)
                    .size()
                    .x
            });
            let item_w = text_w + 16.0;
            let current_x = ui.cursor().left();

            let room_needed = if i < 11 {
                item_w + overflow_w + 8.0
            } else {
                item_w + 8.0
            };

            if current_x + room_needed > controls_start_x {
                overflow_from = Some(i);
                break;
            }

            self.render_single_menu(ui, i, requires_dataset_status);
        }

        if let Some(start_idx) = overflow_from {
            ui.menu_button(overflow_label, |ui| {
                for rem_idx in start_idx..12 {
                    self.render_single_menu(ui, rem_idx, requires_dataset_status);
                }
            });
        }

        ui.cursor().left()
    }
}
