use crate::app::{StatCraftApp, Tab};
use crate::widgets::{render_menu_item, MenuItemStatus};
use egui::Ui;
use egui_dock::{DockState, NodeIndex};
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn render_analyze_etc_menu(
        &mut self,
        ui: &mut Ui,
        index: usize,
        requires_dataset_status: MenuItemStatus,
    ) {
        match index {
            5 => {
                ui.menu_button(t!("menu-analyze"), |ui| {
                    ui.menu_button(t!("menu-analyze-reports"), |ui| {
                        render_menu_item(
                            ui,
                            t!("menu-analyze-reports-codebook"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-analyze-reports-case-summaries"),
                            MenuItemStatus::InDev,
                        );
                    });
                    ui.menu_button(t!("menu-analyze-descriptive-stats"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-descriptives"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.descriptives_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-frequencies"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.frequencies_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        render_menu_item(ui, t!("menu-analyze-explore"), MenuItemStatus::InDev);
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-crosstabs"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.crosstabs_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(t!("menu-analyze-compare-means"), |ui| {
                        if render_menu_item(ui, t!("menu-analyze-means"), requires_dataset_status)
                            .clicked()
                        {
                            self.means_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-ttest-one-sample"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.ttest_onesample_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-ttest-indep"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.ttest_indep_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-ttest-paired"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.ttest_paired_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-anova-oneway"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.anova_oneway_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(t!("menu-analyze-glm"), |ui| {
                        render_menu_item(
                            ui,
                            t!("menu-analyze-glm-univariate"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-analyze-glm-multivariate"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-analyze-glm-repeated"),
                            MenuItemStatus::InDev,
                        );
                    });
                    ui.menu_button(t!("menu-analyze-correlate"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-correlate-bivariate"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.correlations_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        render_menu_item(
                            ui,
                            t!("menu-analyze-correlate-partial"),
                            MenuItemStatus::InDev,
                        );
                    });
                    ui.menu_button(t!("menu-analyze-regression"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-regression-linear"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.linear_regression_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        render_menu_item(
                            ui,
                            t!("menu-analyze-regression-binary"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-analyze-regression-multinomial"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-analyze-regression-curve"),
                            MenuItemStatus::InDev,
                        );
                    });
                    ui.menu_button(t!("menu-analyze-nonparametric"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-nonparam-two-indep"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.nonparam_twosample_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-nonparam-k-indep"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.kruskal_wallis_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-nonparam-related"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.nonparam_related_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(t!("menu-analyze-dimension"), |ui| {
                        if render_menu_item(ui, t!("menu-analyze-factor"), requires_dataset_status)
                            .clicked()
                        {
                            self.factor_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                        render_menu_item(ui, t!("menu-analyze-cluster"), MenuItemStatus::InDev);
                    });
                    ui.menu_button(t!("menu-analyze-scale"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-analyze-reliability"),
                            requires_dataset_status,
                        )
                        .clicked()
                        {
                            self.reliability_dialog_state.is_open = true;
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(t!("menu-analyze-survival"), |ui| {
                        render_menu_item(
                            ui,
                            t!("menu-analyze-survival-life-tables"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-analyze-survival-kaplan-meier"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-analyze-survival-cox"),
                            MenuItemStatus::InDev,
                        );
                    });
                });
            }
            6 => {
                ui.menu_button(t!("menu-direct-marketing"), |ui| {
                    render_menu_item(ui, t!("menu-dm-rfm"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-dm-cluster"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-dm-profile"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-dm-control-package"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-dm-response-rate"), MenuItemStatus::InDev);
                });
            }
            7 => {
                ui.menu_button(t!("menu-graphs"), |ui| {
                    render_menu_item(ui, t!("menu-graphs-chart-builder"), MenuItemStatus::InDev);
                    ui.menu_button(t!("menu-graphs-legacy"), |ui| {
                        render_menu_item(ui, t!("menu-graphs-bar"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-graphs-line"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-graphs-pie"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-graphs-boxplot"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-graphs-error-bar"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-graphs-scatter"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-graphs-histogram"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-graphs-pyramid"), MenuItemStatus::InDev);
                    });
                });
            }
            8 => {
                ui.menu_button(t!("menu-utilities"), |ui| {
                    render_menu_item(ui, t!("menu-utilities-variables"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-utilities-oms"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-utilities-comments"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-utilities-codebook"), MenuItemStatus::InDev);
                });
            }
            9 => {
                ui.menu_button(t!("menu-extensions"), |ui| {
                    render_menu_item(ui, t!("menu-extensions-hub"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-extensions-install"), MenuItemStatus::InDev);
                    render_menu_item(
                        ui,
                        t!("menu-extensions-dialog-builder"),
                        MenuItemStatus::InDev,
                    );
                    render_menu_item(ui, t!("menu-extensions-mcp-bridge"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-extensions-scripting"), MenuItemStatus::InDev);
                });
            }
            10 => {
                ui.menu_button(t!("menu-window"), |ui| {
                    if render_menu_item(
                        ui,
                        t!("menu-window-reset-layout"),
                        MenuItemStatus::Ready { shortcut: None },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        let mut dock_state = DockState::new(vec![Tab::DataView, Tab::VariableView]);
                        let surface = dock_state.main_surface_mut();
                        surface.split_right(NodeIndex::root(), 0.35, vec![Tab::OutputViewer]);
                        self.dock_state = dock_state;
                    }
                    ui.menu_button(t!("menu-window-presets"), |ui| {
                        if render_menu_item(
                            ui,
                            t!("menu-window-preset-split"),
                            MenuItemStatus::Ready { shortcut: None },
                        )
                        .clicked()
                        {
                            ui.close_menu();
                            let mut dock_state =
                                DockState::new(vec![Tab::DataView, Tab::VariableView]);
                            let surface = dock_state.main_surface_mut();
                            surface.split_right(NodeIndex::root(), 0.35, vec![Tab::OutputViewer]);
                            self.dock_state = dock_state;
                        }
                        render_menu_item(
                            ui,
                            t!("menu-window-preset-classic"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-window-preset-syntax"),
                            MenuItemStatus::InDev,
                        );
                    });
                    ui.separator();
                    if render_menu_item(
                        ui,
                        t!("menu-window-data-editor"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+1"),
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        if let Some(pos) = self.dock_state.find_tab(&Tab::DataView) {
                            self.dock_state.set_active_tab(pos);
                        }
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-window-variable-view"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+2"),
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        if let Some(pos) = self.dock_state.find_tab(&Tab::VariableView) {
                            self.dock_state.set_active_tab(pos);
                        }
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-window-output-viewer"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+3"),
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        if let Some(pos) = self.dock_state.find_tab(&Tab::OutputViewer) {
                            self.dock_state.set_active_tab(pos);
                        }
                    }
                    if render_menu_item(
                        ui,
                        t!("menu-window-syntax-editor"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+4"),
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        if let Some(pos) = self.dock_state.find_tab(&Tab::SyntaxEditor) {
                            self.dock_state.set_active_tab(pos);
                        }
                    }
                    render_menu_item(ui, t!("menu-window-ai-copilot"), MenuItemStatus::InDev);
                    ui.separator();
                    render_menu_item(ui, t!("menu-window-minimize-all"), MenuItemStatus::InDev);
                });
            }
            11 => {
                ui.menu_button(t!("menu-help"), |ui| {
                    render_menu_item(ui, t!("menu-help-coach"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-help-algorithms"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-help-syntax"), MenuItemStatus::InDev);
                    ui.menu_button(t!("menu-help-sample-data"), |ui| {
                        render_menu_item(ui, t!("menu-help-sample-iris"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-help-sample-trial"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-help-sample-survey"), MenuItemStatus::InDev);
                    });
                    ui.separator();
                    render_menu_item(ui, t!("menu-help-check-updates"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-help-about"), MenuItemStatus::InDev);
                });
            }
            _ => {}
        }
    }
}
