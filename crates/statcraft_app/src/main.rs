use statcraft_app::app::StatCraftApp;
use statcraft_app::APP_ICON_PNG;
use statcraft_i18n::{init_i18n, t};
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let _ = init_i18n();

    let mut viewport = egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_resizable(true)
        .with_inner_size([1200.0, 800.0])
        .with_min_inner_size([600.0, 400.0])
        .with_title(t!("app-title"));

    if let Ok(icon_data) = eframe::icon_data::from_png_bytes(APP_ICON_PNG) {
        viewport = viewport.with_icon(Arc::new(icon_data));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "ai.storyteller.statcraft",
        native_options,
        Box::new(|cc| Ok(Box::new(StatCraftApp::new(cc)))),
    )
}
