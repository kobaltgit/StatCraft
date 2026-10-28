use crate::dialogs::{
    CorrelationsDialogState, CrosstabsDialogState, DescriptivesDialogState, FrequenciesDialogState,
    IndependentSamplesTTestDialogState, MissingValuesDialogState, NonparamTwoSampleDialogState,
    OneSampleTTestDialogState, OneWayAnovaDialogState, PairedSamplesTTestDialogState,
    ValueLabelsDialogState, VariableTypeDialogState,
};
use crate::views::{
    render_data_view, render_output_view, render_variable_view, DataViewState, OutputViewerState,
    VariableViewAction, VariableViewState,
};
use crate::widgets::{
    render_caption_buttons, render_menu_item, render_resize_zones, render_toolbar, title_x,
    MenuItemStatus, ToolbarAction, ToolbarPosition, CAPTION_BAR_HEIGHT, CAPTION_BUTTON_WIDTH,
};
use egui::{CentralPanel, SidePanel, TopBottomPanel, Ui};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use serde_json::json;
use statcraft_core::DataSet;
use statcraft_i18n::{available_languages, current_language, language_name, set_language, t};
use statcraft_syntax::CommandBus;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    DataView,
    VariableView,
    OutputViewer,
}

#[derive(Debug, Clone)]
pub enum StatusMessage {
    Ready,
    DatasetImported {
        name: String,
        rows: usize,
        cols: usize,
    },
    DescriptivesExecuted {
        count: usize,
    },
    FrequenciesExecuted {
        count: usize,
    },
    TTestExecuted {
        count: usize,
    },
    IndepTTestExecuted {
        count: usize,
    },
    PairedTTestExecuted {
        count: usize,
    },
    AnovaExecuted {
        count: usize,
    },
    CrosstabsExecuted {
        count: usize,
    },
    CorrelationsExecuted {
        count: usize,
    },
    MannWhitneyExecuted {
        count: usize,
    },
    Error(String),
}

impl StatusMessage {
    pub fn text(&self) -> String {
        match self {
            StatusMessage::Ready => t!("status-ready"),
            StatusMessage::DatasetImported { name, rows, cols } => {
                t!(
                    "output-dataset-imported",
                    name = name.as_str(),
                    rows = *rows,
                    cols = *cols
                )
            }
            StatusMessage::DescriptivesExecuted { count } => {
                t!("output-descriptives-executed", count = *count)
            }
            StatusMessage::FrequenciesExecuted { count } => {
                t!("output-frequencies-executed", count = *count)
            }
            StatusMessage::TTestExecuted { count } => {
                t!("output-ttest-executed", count = *count)
            }
            StatusMessage::IndepTTestExecuted { count } => {
                t!("output-ttest-indep-executed", count = *count)
            }
            StatusMessage::PairedTTestExecuted { count } => {
                t!("output-ttest-paired-executed", count = *count)
            }
            StatusMessage::AnovaExecuted { count } => {
                t!("output-anova-executed", count = *count)
            }
            StatusMessage::CrosstabsExecuted { count } => {
                t!("output-crosstabs-executed", count = *count)
            }
            StatusMessage::CorrelationsExecuted { count } => {
                t!("output-correlations-executed", count = *count)
            }
            StatusMessage::MannWhitneyExecuted { count } => {
                t!("output-npar-mann-whitney-executed", count = *count)
            }
            StatusMessage::Error(e) => format!("{}: {}", t!("status-error"), e),
        }
    }
}

pub struct StatCraftApp {
    pub dataset: Option<DataSet>,
    pub command_bus: CommandBus,
    pub dock_state: DockState<Tab>,
    pub data_view_state: DataViewState,
    pub variable_view_state: VariableViewState,
    pub output_view_state: OutputViewerState,
    pub descriptives_dialog_state: DescriptivesDialogState,
    pub frequencies_dialog_state: FrequenciesDialogState,
    pub crosstabs_dialog_state: CrosstabsDialogState,
    pub correlations_dialog_state: CorrelationsDialogState,
    pub nonparam_twosample_dialog_state: NonparamTwoSampleDialogState,
    pub ttest_onesample_dialog_state: OneSampleTTestDialogState,
    pub ttest_indep_dialog_state: IndependentSamplesTTestDialogState,
    pub ttest_paired_dialog_state: PairedSamplesTTestDialogState,
    pub anova_oneway_dialog_state: OneWayAnovaDialogState,
    pub value_labels_dialog_state: ValueLabelsDialogState,
    pub missing_values_dialog_state: MissingValuesDialogState,
    pub variable_type_dialog_state: VariableTypeDialogState,
    pub pending_variable_action: VariableViewAction,
    pub status_message: StatusMessage,
    pub toolbar_position: ToolbarPosition,
}

impl StatCraftApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut dock_state = DockState::new(vec![Tab::DataView, Tab::VariableView]);
        let surface = dock_state.main_surface_mut();
        surface.split_right(NodeIndex::root(), 0.35, vec![Tab::OutputViewer]);

        Self {
            dataset: DataSet::blank(40, 0).ok(),
            command_bus: CommandBus::with_defaults(),
            dock_state,
            data_view_state: DataViewState::default(),
            variable_view_state: VariableViewState::default(),
            output_view_state: OutputViewerState::new(),
            descriptives_dialog_state: DescriptivesDialogState::default(),
            frequencies_dialog_state: FrequenciesDialogState::default(),
            crosstabs_dialog_state: CrosstabsDialogState::default(),
            correlations_dialog_state: CorrelationsDialogState::default(),
            nonparam_twosample_dialog_state: NonparamTwoSampleDialogState::default(),
            ttest_onesample_dialog_state: OneSampleTTestDialogState::default(),
            ttest_indep_dialog_state: IndependentSamplesTTestDialogState::default(),
            ttest_paired_dialog_state: PairedSamplesTTestDialogState::default(),
            anova_oneway_dialog_state: OneWayAnovaDialogState::default(),
            value_labels_dialog_state: ValueLabelsDialogState::default(),
            missing_values_dialog_state: MissingValuesDialogState::default(),
            variable_type_dialog_state: VariableTypeDialogState::default(),
            pending_variable_action: VariableViewAction::None,
            status_message: StatusMessage::Ready,
            toolbar_position: ToolbarPosition::Top,
        }
    }

    pub fn open_csv_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(t!("dialog-open-csv-title"))
            .add_filter("Tabular Data", &["csv", "tsv", "txt"])
            .pick_file()
        {
            if let Ok(content) = fs::read_to_string(&path) {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Dataset");

                let params = json!({
                    "csv_data": content,
                    "dataset_name": file_name
                });

                match self
                    .command_bus
                    .execute("data.import_csv", &mut self.dataset, params)
                {
                    Ok(res) => {
                        let rows = res["rows"].as_u64().unwrap_or(0) as usize;
                        let cols = res["cols"].as_u64().unwrap_or(0) as usize;
                        self.status_message = StatusMessage::DatasetImported {
                            name: file_name.to_string(),
                            rows,
                            cols,
                        };
                        self.output_view_state.add_dataset_imported(
                            file_name.to_string(),
                            rows,
                            cols,
                        );
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
        }
    }

    fn render_single_menu(
        &mut self,
        ui: &mut Ui,
        index: usize,
        requires_dataset_status: MenuItemStatus,
    ) {
        match index {
            0 => {
                ui.menu_button(t!("menu-file"), |ui| {
                    ui.menu_button(t!("menu-file-new"), |ui| {
                        render_menu_item(ui, t!("menu-file-new-project"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-file-new-dataset"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-file-new-syntax"), MenuItemStatus::InDev);
                    });
                    render_menu_item(ui, t!("menu-file-open-project"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-file-open-data"), MenuItemStatus::InDev);
                    if render_menu_item(
                        ui,
                        t!("menu-file-open-csv"),
                        MenuItemStatus::Ready {
                            shortcut: Some("Ctrl+O"),
                        },
                    )
                    .clicked()
                    {
                        ui.close_menu();
                        self.open_csv_dialog();
                    }
                    ui.menu_button(t!("menu-file-import-data"), |ui| {
                        render_menu_item(ui, t!("menu-file-import-spss"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-file-import-csv"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-file-import-excel"), MenuItemStatus::InDev);
                    });
                    render_menu_item(ui, t!("menu-file-recent"), MenuItemStatus::InDev);
                    ui.separator();
                    render_menu_item(ui, t!("menu-file-save"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-file-save-as"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-file-save-data-as"), MenuItemStatus::InDev);
                    ui.menu_button(t!("menu-file-export"), |ui| {
                        render_menu_item(ui, t!("menu-file-export-output"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-file-export-data"), MenuItemStatus::InDev);
                    });
                    ui.separator();
                    render_menu_item(ui, t!("menu-file-close-project"), MenuItemStatus::InDev);
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
            3 => {
                ui.menu_button(t!("menu-data"), |ui| {
                    render_menu_item(ui, t!("menu-data-sort-cases"), requires_dataset_status);
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
                    render_menu_item(ui, t!("menu-data-split-file"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-data-filter"), requires_dataset_status);
                    render_menu_item(ui, t!("menu-data-weight-cases"), MenuItemStatus::InDev);
                });
            }
            4 => {
                ui.menu_button(t!("menu-transform"), |ui| {
                    render_menu_item(ui, t!("menu-transform-compute"), requires_dataset_status);
                    render_menu_item(ui, t!("menu-transform-shift"), MenuItemStatus::InDev);
                    ui.menu_button(t!("menu-transform-recode"), |ui| {
                        render_menu_item(
                            ui,
                            t!("menu-transform-recode-diff"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-transform-recode-same"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-transform-auto-recode"),
                            MenuItemStatus::InDev,
                        );
                    });
                    render_menu_item(ui, t!("menu-transform-dummy"), MenuItemStatus::InDev);
                    render_menu_item(
                        ui,
                        t!("menu-transform-visual-binning"),
                        MenuItemStatus::InDev,
                    );
                    render_menu_item(
                        ui,
                        t!("menu-transform-optimal-binning"),
                        MenuItemStatus::InDev,
                    );
                    render_menu_item(ui, t!("menu-transform-adp"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-transform-rank-cases"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-transform-date-time"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-transform-time-series"), MenuItemStatus::InDev);
                    render_menu_item(
                        ui,
                        t!("menu-transform-missing-values"),
                        MenuItemStatus::InDev,
                    );
                    render_menu_item(ui, t!("menu-transform-random-seed"), MenuItemStatus::InDev);
                    render_menu_item(ui, t!("menu-transform-count-values"), MenuItemStatus::InDev);
                    render_menu_item(
                        ui,
                        t!("menu-transform-programmability"),
                        MenuItemStatus::InDev,
                    );
                });
            }
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
                        render_menu_item(
                            ui,
                            t!("menu-analyze-regression-linear"),
                            MenuItemStatus::InDev,
                        );
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
                        render_menu_item(
                            ui,
                            t!("menu-analyze-nonparam-independent"),
                            MenuItemStatus::InDev,
                        );
                        render_menu_item(
                            ui,
                            t!("menu-analyze-nonparam-related"),
                            MenuItemStatus::InDev,
                        );
                    });
                    ui.menu_button(t!("menu-analyze-dimension"), |ui| {
                        render_menu_item(ui, t!("menu-analyze-factor"), MenuItemStatus::InDev);
                        render_menu_item(ui, t!("menu-analyze-cluster"), MenuItemStatus::InDev);
                    });
                    ui.menu_button(t!("menu-analyze-scale"), |ui| {
                        render_menu_item(ui, t!("menu-analyze-reliability"), MenuItemStatus::InDev);
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
                    render_menu_item(ui, t!("menu-window-syntax-editor"), MenuItemStatus::InDev);
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

    fn render_top_menus(
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

struct StatCraftTabViewer<'a> {
    dataset: &'a mut Option<DataSet>,
    data_view_state: &'a mut DataViewState,
    variable_view_state: &'a mut VariableViewState,
    output_view_state: &'a mut OutputViewerState,
    pending_variable_action: &'a mut VariableViewAction,
}

impl<'a> TabViewer for StatCraftTabViewer<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::DataView => t!("menu-view-data").into(),
            Tab::VariableView => t!("menu-view-variables").into(),
            Tab::OutputViewer => t!("menu-view-output").into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::DataView => {
                render_data_view(ui, self.dataset, self.data_view_state);
            }
            Tab::VariableView => {
                let action = render_variable_view(ui, self.dataset, self.variable_view_state);
                if action != VariableViewAction::None {
                    *self.pending_variable_action = action;
                }
            }
            Tab::OutputViewer => {
                render_output_view(ui, self.output_view_state);
            }
        }
    }
}

impl eframe::App for StatCraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        render_resize_zones(ctx, is_maximized);

        // 1. Top Custom Title Bar (CSD)
        TopBottomPanel::top("title_bar")
            .exact_height(CAPTION_BAR_HEIGHT)
            .frame(egui::Frame::none().fill(crate::theme::Tokens::BG_HEADER))
            .show(ctx, |ui| {
                let bar_rect = ui.max_rect();
                let center_x = bar_rect.center().x;
                let controls_start_x = bar_rect.right() - (CAPTION_BUTTON_WIDTH * 3.0);

                // Right Side: Caption Buttons (Minimize, Maximize/Restore, Close)
                render_caption_buttons(ui, bar_rect, is_maximized);

                let has_dataset = self
                    .dataset
                    .as_ref()
                    .map(|d| d.col_count() > 0)
                    .unwrap_or(false);
                let req_ds_str = t!("menu-item-requires-dataset");
                let requires_dataset_status = if has_dataset {
                    MenuItemStatus::Ready { shortcut: None }
                } else {
                    MenuItemStatus::ContextDisabled {
                        reason: &req_ds_str,
                    }
                };

                let mut menus_end_x = bar_rect.left();
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.set_height(CAPTION_BAR_HEIGHT);
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                    ui.spacing_mut().button_padding = egui::vec2(6.0, 3.0);
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;

                    ui.add_space(6.0);

                    // App Icon (24x24 px) - vector SVG rendered cleanly via egui_extras
                    ui.add(
                        egui::Image::new(egui::include_image!("../../../assets/icon.svg"))
                            .fit_to_exact_size(egui::vec2(24.0, 24.0)),
                    );
                    ui.add_space(4.0);

                    // Menus rendering with responsive overflow
                    menus_end_x =
                        self.render_top_menus(ui, controls_start_x, requires_dataset_status);
                });

                // Center / Free Space: Window Dragging & Centered Project Title
                let drag_rect = egui::Rect::from_x_y_ranges(
                    (menus_end_x + 4.0)..=(controls_start_x - 4.0),
                    bar_rect.y_range(),
                );

                if drag_rect.width() > 10.0 {
                    let drag_resp = ui.interact(
                        drag_rect,
                        egui::Id::new("titlebar_drag_area"),
                        egui::Sense::click_and_drag(),
                    );
                    if drag_resp.drag_started_by(egui::PointerButton::Primary) {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    if drag_resp.double_clicked() {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                    }
                }

                let project_title = match &self.status_message {
                    StatusMessage::DatasetImported { name, .. } => format!("{} •", name),
                    _ => "Untitled-1 •".to_string(),
                };

                let font_id = egui::FontId::proportional(12.5);
                let galley = ui.painter().layout_no_wrap(
                    project_title.clone(),
                    font_id.clone(),
                    crate::theme::Tokens::TEXT_MUTED,
                );
                let text_w = galley.size().x;

                if let Some(x) = title_x(center_x, menus_end_x, controls_start_x, text_w) {
                    let text_pos = egui::pos2(x, bar_rect.center().y - galley.size().y / 2.0);
                    ui.painter()
                        .galley(text_pos, galley, crate::theme::Tokens::TEXT_MUTED);
                } else if controls_start_x - menus_end_x > 80.0 {
                    let avail = controls_start_x - menus_end_x - 24.0;
                    let mut job = egui::text::LayoutJob::simple_singleline(
                        project_title,
                        font_id,
                        crate::theme::Tokens::TEXT_MUTED,
                    );
                    job.wrap = egui::text::TextWrapping::truncate_at_width(avail);
                    let g = ui.painter().layout_job(job);
                    let text_pos =
                        egui::pos2(menus_end_x + 12.0, bar_rect.center().y - g.size().y / 2.0);
                    ui.painter()
                        .galley(text_pos, g, crate::theme::Tokens::TEXT_MUTED);
                }
            });

        let has_dataset = self
            .dataset
            .as_ref()
            .map(|d| d.col_count() > 0)
            .unwrap_or(false);
        let can_undo = !self.command_bus.history().is_empty();
        let can_redo = false;
        let mut toolbar_action = ToolbarAction::None;

        // 2. Main Toolbar (Top)
        if self.toolbar_position == ToolbarPosition::Top {
            TopBottomPanel::top("main_toolbar_top")
                .exact_height(32.0)
                .frame(
                    egui::Frame::none()
                        .fill(crate::theme::Tokens::BG_PANEL)
                        .inner_margin(egui::Margin::symmetric(4.0, 3.0)),
                )
                .show(ctx, |ui| {
                    toolbar_action = render_toolbar(
                        ui,
                        ToolbarPosition::Top,
                        has_dataset,
                        can_undo,
                        can_redo,
                        self.data_view_state.show_value_labels,
                    );
                });
        }

        // 3. Bottom Status Bar
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(self.status_message.text());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ds) = &self.dataset {
                        ui.label(t!("status-cols-count", count = ds.col_count()));
                        ui.separator();
                        ui.label(t!("status-rows-count", count = ds.row_count()));
                    } else {
                        ui.label(t!("status-ready"));
                    }
                });
            });
        });

        // 4. Main Toolbar (Left)
        if self.toolbar_position == ToolbarPosition::Left {
            SidePanel::left("main_toolbar_left")
                .exact_width(34.0)
                .resizable(false)
                .frame(
                    egui::Frame::none()
                        .fill(crate::theme::Tokens::BG_PANEL)
                        .inner_margin(egui::Margin::symmetric(4.0, 4.0)),
                )
                .show(ctx, |ui| {
                    toolbar_action = render_toolbar(
                        ui,
                        ToolbarPosition::Left,
                        has_dataset,
                        can_undo,
                        can_redo,
                        self.data_view_state.show_value_labels,
                    );
                });
        }

        // Dispatch toolbar actions
        match toolbar_action {
            ToolbarAction::OpenData => {
                self.open_csv_dialog();
            }
            ToolbarAction::Undo => {
                let _ = self.command_bus.undo(&mut self.dataset);
            }
            ToolbarAction::InsertCase => {
                let _ = self.command_bus.execute(
                    "data.add_row",
                    &mut self.dataset,
                    serde_json::json!({}),
                );
            }
            ToolbarAction::InsertVariable => {
                let _ =
                    self.command_bus
                        .execute("var.add", &mut self.dataset, serde_json::json!({}));
            }
            ToolbarAction::ToggleValueLabels => {
                self.data_view_state.show_value_labels = !self.data_view_state.show_value_labels;
            }
            ToolbarAction::Descriptives => {
                self.descriptives_dialog_state.is_open = true;
            }
            ToolbarAction::Frequencies => {
                self.frequencies_dialog_state.is_open = true;
            }
            _ => {}
        }

        // 3. Central Dock Area
        CentralPanel::default().show(ctx, |ui| {
            let mut tab_viewer = StatCraftTabViewer {
                dataset: &mut self.dataset,
                data_view_state: &mut self.data_view_state,
                variable_view_state: &mut self.variable_view_state,
                output_view_state: &mut self.output_view_state,
                pending_variable_action: &mut self.pending_variable_action,
            };

            let style = egui_dock::Style::from_egui(ui.style());

            DockArea::new(&mut self.dock_state)
                .style(style)
                .show_inside(ui, &mut tab_viewer);
        });

        // 4. Handle Variable View actions (opening modal dialogs)
        match self.pending_variable_action {
            VariableViewAction::OpenValueLabels(col_idx) => {
                if let Some(ds) = &self.dataset {
                    if let Some(var) = ds.dictionary.get(col_idx) {
                        self.value_labels_dialog_state.open_for_variable(
                            col_idx,
                            &var.name,
                            &var.values.mapping,
                        );
                    }
                }
                self.pending_variable_action = VariableViewAction::None;
            }
            VariableViewAction::OpenMissingValues(col_idx) => {
                if let Some(ds) = &self.dataset {
                    if let Some(var) = ds.dictionary.get(col_idx) {
                        self.missing_values_dialog_state.open_for_variable(
                            col_idx,
                            &var.name,
                            &var.missing,
                        );
                    }
                }
                self.pending_variable_action = VariableViewAction::None;
            }
            VariableViewAction::OpenVariableType(col_idx) => {
                if let Some(ds) = &self.dataset {
                    if let Some(var) = ds.dictionary.get(col_idx) {
                        self.variable_type_dialog_state = VariableTypeDialogState::open(
                            col_idx,
                            &var.name,
                            &var.var_type,
                            var.width,
                            var.decimals,
                        );
                    }
                }
                self.pending_variable_action = VariableViewAction::None;
            }
            VariableViewAction::None => {}
        }

        // 5. Modal Dialogs
        self.handle_dialogs(ctx);
    }
}
