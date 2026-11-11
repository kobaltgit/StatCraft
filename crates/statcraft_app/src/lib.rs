pub mod app;
pub mod app_dialogs;
pub mod app_dialogs_data;
pub mod app_dialogs_edit;
pub mod app_dialogs_transform;
pub mod app_menus;
pub mod app_session;
pub mod bridge_service;
pub mod dialogs;
pub mod export_handlers;
pub mod icon_data;
pub mod icons;
pub mod recent_files;
pub mod status_message;
pub mod syntax_runner;
pub mod theme;
pub mod views;
pub mod widgets;

pub use app::{StatCraftApp, Tab};
pub use recent_files::RecentFiles;
pub use status_message::StatusMessage;

pub static APP_ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-256.png");
pub static APP_ICON_SVG: &[u8] = include_bytes!("../../../assets/icon.svg");
