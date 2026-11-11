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
                        t!("menu-file-open-syntax"),
                        MenuItemStatus::Ready { shortcut: None },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.open_syntax_dialog();
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
                    if render_menu_item(
                        ui,
                        t!("menu-file-save-syntax-as"),
                        MenuItemStatus::Ready { shortcut: None },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.save_syntax_as();
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
                    if render_menu_item(
                        ui,
                        t!("menu-edit-redo"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+Y"),
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        let _ = self.command_bus.redo(&mut self.dataset);
                    }
                    ui.separator();
                    let has_data = self.dataset.is_some();
                    let cell_selected = self.data_view_state.selected_cell.is_some();

                    if render_menu_item(
                        ui,
                        t!("menu-edit-cut"),
                        if has_data && cell_selected {
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+X"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.cut_selected_cell(ui.ctx());
                    }

                    if render_menu_item(
                        ui,
                        t!("menu-edit-copy"),
                        if has_data && cell_selected {
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+C"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.copy_selected_cell(ui.ctx());
                    }

                    if render_menu_item(
                        ui,
                        t!("menu-edit-copy-names"),
                        if has_data && cell_selected {
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+Shift+C"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.copy_with_variable_names(ui.ctx());
                    }

                    if render_menu_item(
                        ui,
                        t!("menu-edit-paste"),
                        if has_data && self.clipboard_cache.is_some() {
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+V"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.paste_selected_cell(ui.ctx());
                    }

                    if render_menu_item(
                        ui,
                        t!("menu-edit-clear"),
                        if has_data && cell_selected {
                            MenuItemStatus::Ready {
                                shortcut: Some("Delete"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.clear_selected_cell();
                    }

                    ui.separator();

                    if render_menu_item(
                        ui,
                        t!("menu-edit-find"),
                        if has_data {
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+F"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.open_find_dialog();
                    }

                    if render_menu_item(
                        ui,
                        t!("menu-edit-replace"),
                        if has_data {
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+H"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.open_replace_dialog();
                    }

                    if render_menu_item(
                        ui,
                        t!("menu-edit-goto-case"),
                        if has_data {
                            MenuItemStatus::Ready {
                                shortcut: Some("Ctrl+G"),
                            }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.open_goto_case_dialog();
                    }

                    if render_menu_item(
                        ui,
                        t!("menu-edit-goto-var"),
                        if has_data {
                            MenuItemStatus::Ready { shortcut: None }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.open_goto_var_dialog();
                    }

                    ui.separator();

                    if render_menu_item(
                        ui,
                        t!("menu-edit-insert-case"),
                        if has_data {
                            MenuItemStatus::Ready { shortcut: None }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.insert_case_action();
                    }

                    if render_menu_item(
                        ui,
                        t!("menu-edit-insert-var"),
                        if has_data {
                            MenuItemStatus::Ready { shortcut: None }
                        } else {
                            requires_dataset_status
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.insert_variable_action();
                    }

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
                        let is_light = self.theme_mode == crate::theme::ThemeMode::Light;
                        if ui
                            .selectable_label(is_light, t!("menu-view-theme-light"))
                            .clicked()
                        {
                            self.theme_mode = crate::theme::ThemeMode::Light;
                            ui.close_menu();
                        }
                        let is_dark = self.theme_mode == crate::theme::ThemeMode::Dark;
                        if ui
                            .selectable_label(is_dark, t!("menu-view-theme-dark"))
                            .clicked()
                        {
                            self.theme_mode = crate::theme::ThemeMode::Dark;
                            ui.close_menu();
                        }
                        let is_sys = self.theme_mode == crate::theme::ThemeMode::System;
                        if ui
                            .selectable_label(is_sys, t!("menu-view-theme-system"))
                            .clicked()
                        {
                            self.theme_mode = crate::theme::ThemeMode::System;
                            ui.close_menu();
                        }
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
