#![allow(clippy::needless_range_loop)]

pub mod app;
pub mod app_dialogs;
pub mod app_dialogs_advanced;
pub mod app_dialogs_data;
pub mod app_dialogs_edit;
pub mod app_dialogs_glm;
pub mod app_dialogs_merge;
pub mod app_dialogs_pkg3;
pub mod app_dialogs_transform;
pub mod app_menus;
pub mod app_session;
pub mod app_shortcuts;
pub mod app_tab_viewer;
pub mod app_workspace;
pub mod bridge_service;
pub mod dialogs;
pub mod export_handlers;
pub mod grid;
pub mod icon_data;
pub mod icons;
pub mod recent_files;
pub mod status_message;
pub mod syntax_runner;
pub mod system_memory;
pub mod theme;
pub mod views;
pub mod widgets;
pub mod window_frame;

pub use app::{ensure_dock_tab_open, is_tab_closeable, StatCraftApp, Tab};
pub use recent_files::RecentFiles;
pub use status_message::StatusMessage;
pub use system_memory::{
    check_memory_guard_for_files, get_system_memory, MemoryWarning, SystemMemoryInfo,
};

pub static APP_ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-256.png");
pub static APP_ICON_SVG: &[u8] = include_bytes!("../../../assets/icon.svg");
