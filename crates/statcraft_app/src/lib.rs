pub mod app;
pub mod app_dialogs;
pub mod app_dialogs_data;
pub mod dialogs;
pub mod icon_data;
pub mod icons;
pub mod theme;
pub mod views;
pub mod widgets;

pub use app::{StatCraftApp, Tab};

pub static APP_ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-256.png");
pub static APP_ICON_SVG: &[u8] = include_bytes!("../../../assets/icon.svg");
