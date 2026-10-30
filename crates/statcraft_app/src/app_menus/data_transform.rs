use crate::app::StatCraftApp;
use crate::widgets::{render_menu_item, MenuItemStatus};
use egui::Ui;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn render_data_transform_menu(
        &mut self,
        ui: &mut Ui,
        index: usize,
        requires_dataset_status: MenuItemStatus,
    ) {
        match index {
            3 => {
                ui.menu_button(t!("menu-data"), |ui| {
                    if render_menu_item(ui, t!("menu-data-sort-cases"), requires_dataset_status)
                        .clicked()
                    {
                        self.sort_cases_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    render_menu_item(ui, t!("menu-data-sort-vars"), requires_dataset_status);
                    render_menu_item(ui, t!("menu-data-transpose"), requires_dataset_status);
                    ui.separator();
                    ui.menu_button(t!("menu-data-merge-files"), |ui| {
                        render_menu_item(ui, t!("menu-data-merge-cases"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-data-merge-vars"), MenuItemStatus::InDev);
                    });
                    render_menu_item(ui, t!("menu-data-restructure"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-data-aggregate"), MenuItemStatus::InDev);
                    ui.separator();
                    if render_menu_item(ui, t!("menu-data-split-file"), requires_dataset_status)
                        .clicked()
                    {
                        self.split_file_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    if render_menu_item(ui, t!("menu-data-filter"), requires_dataset_status)
                        .clicked()
                    {
                        self.select_cases_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    if render_menu_item(ui, t!("menu-data-weight-cases"), requires_dataset_status)
                        .clicked()
                    {
                        if let Some(ds) = &self.dataset {
                            self.weight_cases_dialog_state.weight_active = ds.is_weighted();
                            self.weight_cases_dialog_state.selected_var =
                                ds.weight_var().map(str::to_string);
                        }
                        self.weight_cases_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                });
            }
            4 => {
                ui.menu_button(t!("menu-transform"), |ui| {
                    if render_menu_item(ui, t!("menu-transform-compute"), requires_dataset_status)
                        .clicked()
                    {
                        self.compute_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    if render_menu_item(ui, t!("menu-transform-shift"), requires_dataset_status)
                        .clicked()
                    {
                        self.shift_values_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    ui.menu_button(t!("menu-transform-recode"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-transform-recode-diff"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.recode_dialog_state.open_diff();
                            ui.close_menu();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-transform-recode-same"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.recode_dialog_state.open_same();
                            ui.close_menu();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-transform-auto-recode"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.auto_recode_dialog_state.open();
                            ui.close_menu();
                        }
                    });
                    if render_menu_item(ui, t!("menu-transform-dummy"), requires_dataset_status)
                        .clicked()
                    {
                        self.dummy_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-transform-visual-binning"),
                        requires_dataset_status,
                    )
                    .clicked()
                    {
                        self.visual_binning_dialog_state.open();
                        ui.close_menu();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-transform-optimal-binning"),
                        requires_dataset_status,
                    )
                    .clicked()
                    {
                        self.optimal_binning_dialog_state.open();
                        ui.close_menu();
                    }
                    render_menu_item(ui, t!("menu-transform-adp"), MenuItemStatus::InDev);
                    if render_menu_item(
                        ui,
                        t!("menu-transform-rank-cases"),
                        requires_dataset_status,
                    )
                    .clicked()
                    {
                        self.rank_cases_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    if render_menu_item(ui, t!("menu-transform-date-time"), requires_dataset_status)
                        .clicked()
                    {
                        self.date_time_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-transform-time-series"),
                        MenuItemStatus::Ready { shortcut: None },
                    )
                    .clicked()
                    {
                        self.time_series_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-transform-missing-values"),
                        requires_dataset_status,
                    )
                    .clicked()
                    {
                        self.replace_missing_dialog_state.open();
                        ui.close_menu();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-transform-random-seed"),
                        MenuItemStatus::Ready { shortcut: None },
                    )
                    .clicked()
                    {
                        if let Some(seed) = statcraft_core::get_global_random_seed() {
                            self.random_seed_dialog_state.is_random = false;
                            self.random_seed_dialog_state.seed_text = seed.to_string();
                        } else {
                            self.random_seed_dialog_state.is_random = true;
                        }
                        self.random_seed_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-transform-count-values"),
                        requires_dataset_status,
                    )
                    .clicked()
                    {
                        self.count_values_dialog_state.is_open = true;
                        ui.close_menu();
                    }
                    render_menu_item(
                        ui,
                        t!("menu-transform-programmability"),
                        MenuItemStatus::InDev,
                    );
                });
            }
            _ => {}
        }
    }
}
