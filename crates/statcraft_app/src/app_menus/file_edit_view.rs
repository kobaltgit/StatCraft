use crate::app::{StatCraftApp, Tab};
use crate::app_session::PendingSessionAction;
use crate::status_message::StatusMessage;
use crate::widgets::{render_menu_item, MenuItemStatus, ToolbarPosition};
use egui::Ui;
use statcraft_i18n::{
    available_languages, current_language, language_name, reload_locales, set_language, t,
};

impl StatCraftApp {
    pub(crate) fn render_file_edit_view_menu(
        &mut self,
        ui: &mut Ui,
        index: usize,
        requires_dataset_status: MenuItemStatus,
    ) {
        match index {
            0 => {
                ui.menu_button(t!("menu-file"), |ui| {
                    ui.menu_button(t!("menu-file-new"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-file-new-project"),
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+Shift+N"),
                            },
                        )
                        .clicked()
                        {
                            ui.close_menu();
                            self.request_session_action(PendingSessionAction::NewProject);
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-file-new-dataset"),
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+N"),
                            },
                        )
                        .clicked()
                        {
                            ui.close_menu();
                            self.request_session_action(PendingSessionAction::NewDataset);
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-file-new-syntax"),
                            MenuItemStatus::Ready { shortcut: None },
                        )
                        .clicked()
                        {
                            ui.close_menu();
                            self.syntax_editor_state.reset_with_header();
                            if let Some(pos) = self.dock_state.find_tab(&Tab::SyntaxEditor) {
                                self.dock_state.set_active_tab(pos);
                            }
                            self.status_message = StatusMessage::NewSyntaxCreated;
                        }
                    });
                    if render_menu_item(
                        ui,
                        t!("menu-file-open-project"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+Shift+O"),
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.open_project_dialog();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-file-open-data"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+O"),
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.open_data_dialog();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-file-open-csv"),
                        MenuItemStatus::Ready { shortcut: None },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.open_csv_dialog();
                    }
                    ui.menu_button(t!("menu-file-import-data"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-file-import-spss"),
                            MenuItemStatus::Ready { shortcut: None },
                        )
                        .clicked()
                        {
                            ui.close_menu();
                            self.open_spss_dialog();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-file-import-csv"),
                            MenuItemStatus::Ready { shortcut: None },
                        )
                        .clicked()
                        {
                            ui.close_menu();
                            self.open_csv_dialog();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-file-import-excel"),
                            MenuItemStatus::Ready { shortcut: None },
                        )
                        .clicked()
                        {
                            ui.close_menu();
                            self.open_excel_dialog();
                        }
                    });
                    self.render_recent_files_menu(ui);
                    ui.separator();
                    if render_menu_item(
                        ui,
                        t!("menu-file-save"),
                        if self.dataset.is_some() {
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+S"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.save_project();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-file-save-as"),
                        if self.dataset.is_some() {
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+Shift+S"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.save_project_as();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-file-save-data-as"),
                        if self.dataset.is_some() {
                            MenuItemStatus::Ready { shortcut: None }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.save_data_as();
                    }
                    ui.menu_button(t!("menu-file-export"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-file-export-output"),
                            MenuItemStatus::Ready { shortcut: None },
                        )
                        .clicked()
                        {
                            ui.close_menu();
                            self.handle_export_output();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-file-export-data"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            ui.close_menu();
                            self.handle_export_data();
                        }
                    });
                    ui.separator();
                    if render_menu_item(
                        ui,
                        t!("menu-file-close-project"),
                        MenuItemStatus::Ready { shortcut: None },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.request_session_action(PendingSessionAction::CloseProject);
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-file-exit"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Alt+F4"),
                        },
                    )
                    .clicked()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            }
            1 => {
                ui.menu_button(t!("menu-edit"), |ui| {
                    if render_menu_item(
                        ui,
                        t!("menu-edit-undo"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+Z"),
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        let _ = self.command_bus.undo(&mut self.dataset);
                    }
                    render_menu_item(
                        ui,
                        t!("menu-edit-redo"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+Y"),
                        },
                    );
                    ui.separator();
                    render_menu_item(ui, t!("menu-edit-cut"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-edit-copy"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-edit-copy-names"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-edit-paste"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-edit-clear"), MenuItemStatus::InDev);
                    ui.separator();
                    render_menu_item(ui, t!("menu-edit-find"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-edit-replace"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-edit-goto-case"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-edit-goto-var"), MenuItemStatus::InDev);
                    ui.separator();
                    render_menu_item(ui, t!("menu-edit-insert-case"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-edit-insert-var"), MenuItemStatus::InDev);
                    ui.separator();
                    render_menu_item(ui, t!("menu-edit-preferences"), MenuItemStatus::InDev);
                });
            }
            2 => {
                ui.menu_button(t!("menu-view"), |ui| {
                    let mut show_labels = self.data_view_state.show_value_labels;
                    if ui
                        .checkbox(&mut show_labels, t!("menu-view-value-labels"))
                        .clicked()
                    {
                        self.data_view_state.show_value_labels = show_labels;
                    }
                    ui.separator();
                    render_menu_item(ui, t!("menu-view-status-bar"), MenuItemStatus::InDev);
                    ui.menu_button(t!("menu-view-toolbars"), |ui| {
                        if ui
                            .selectable_label(
                                self.toolbar_position == ToolbarPosition::Top,
                                t!("menu-view-toolbar-top"),
                            )
                            .clicked()
                        {
                            self.toolbar_position = ToolbarPosition::Top;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.toolbar_position == ToolbarPosition::Left,
                                t!("menu-view-toolbar-left"),
                            )
                            .clicked()
                        {
                            self.toolbar_position = ToolbarPosition::Left;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.toolbar_position == ToolbarPosition::Hidden,
                                t!("menu-view-toolbar-hidden"),
                            )
                            .clicked()
                        {
                            self.toolbar_position = ToolbarPosition::Hidden;
                            ui.close_menu();
                        }
                    });
                    render_menu_item(ui, t!("menu-view-grid-lines"), MenuItemStatus::InDev);
                    ui.separator();
                    render_menu_item(ui, t!("menu-view-zoom-in"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-view-zoom-out"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-view-zoom-reset"), MenuItemStatus::InDev);
                    ui.separator();
                    ui.menu_button(t!("menu-view-theme"), |ui| {
                        render_menu_item(ui, t!("menu-view-theme-light"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-view-theme-dark"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-view-theme-system"), MenuItemStatus::InDev);
                    });
                    ui.separator();
                    // Language Submenu
                    ui.menu_button(t!("menu-view-language"), |ui| {
                        let _ = reload_locales();
                        let cur_lang = current_language();
                        for lang in available_languages() {
                            let label = language_name(&lang);
                            let is_active = cur_lang == lang;
                            if ui.selectable_label(is_active, label).clicked() {
                                set_language(&lang);
                                ui.close_menu();
                            }
                        }
                    });
                });
            }
            _ => {}
        }
    }
}
