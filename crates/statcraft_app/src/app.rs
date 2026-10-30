use crate::app_session::PendingSessionAction;
use crate::dialogs::ConfirmSaveDialogState;
use crate::dialogs::{
    AutoRecodeDialogState, ComputeDialogState, CorrelationsDialogState, CountValuesDialogState,
    CrosstabsDialogState, DateTimeDialogState, DescriptivesDialogState, DummyDialogState,
    FactorDialogState, FrequenciesDialogState, IndependentSamplesTTestDialogState,
    KruskalWallisDialogState, LinearRegressionDialogState, MeansDialogState,
    MissingValuesDialogState, NonparamRelatedDialogState, NonparamTwoSampleDialogState,
    OneSampleTTestDialogState, OneWayAnovaDialogState, OptimalBinningDialogState,
    PairedSamplesTTestDialogState, RandomSeedDialogState, RankCasesDialogState, RecodeDialogState,
    ReliabilityDialogState, ReplaceMissingDialogState, SelectCasesDialogState,
    ShiftValuesDialogState, SortCasesDialogState, SplitFileDialogState, TimeSeriesDialogState,
    ValueLabelsDialogState, VariableTypeDialogState, VisualBinningDialogState,
    WeightCasesDialogState,
};
use crate::recent_files::RecentFiles;
use crate::views::{
    render_data_view, render_output_view, render_syntax_editor, render_variable_view,
    DataViewState, OutputViewerState, SyntaxEditorState, VariableViewAction, VariableViewState,
};
use crate::widgets::{
    render_caption_buttons, render_resize_zones, render_toolbar, title_x, MenuItemStatus,
    ToolbarAction, ToolbarPosition, CAPTION_BAR_HEIGHT, CAPTION_BUTTON_WIDTH,
};
use egui::{CentralPanel, SidePanel, TopBottomPanel, Ui};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use statcraft_core::DataSet;
use statcraft_i18n::t;
use statcraft_syntax::CommandBus;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    DataView,
    VariableView,
    OutputViewer,
    SyntaxEditor,
}

pub use crate::status_message::StatusMessage;

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
    pub kruskal_wallis_dialog_state: KruskalWallisDialogState,
    pub nonparam_related_dialog_state: NonparamRelatedDialogState,
    pub ttest_onesample_dialog_state: OneSampleTTestDialogState,
    pub ttest_indep_dialog_state: IndependentSamplesTTestDialogState,
    pub ttest_paired_dialog_state: PairedSamplesTTestDialogState,
    pub anova_oneway_dialog_state: OneWayAnovaDialogState,
    pub value_labels_dialog_state: ValueLabelsDialogState,
    pub missing_values_dialog_state: MissingValuesDialogState,
    pub variable_type_dialog_state: VariableTypeDialogState,
    pub compute_dialog_state: ComputeDialogState,
    pub count_values_dialog_state: CountValuesDialogState,
    pub dummy_dialog_state: DummyDialogState,
    pub shift_values_dialog_state: ShiftValuesDialogState,
    pub date_time_dialog_state: DateTimeDialogState,
    pub rank_cases_dialog_state: RankCasesDialogState,
    pub time_series_dialog_state: TimeSeriesDialogState,
    pub replace_missing_dialog_state: ReplaceMissingDialogState,
    pub recode_dialog_state: RecodeDialogState,
    pub auto_recode_dialog_state: AutoRecodeDialogState,
    pub visual_binning_dialog_state: VisualBinningDialogState,
    pub optimal_binning_dialog_state: OptimalBinningDialogState,
    pub sort_cases_dialog_state: SortCasesDialogState,
    pub select_cases_dialog_state: SelectCasesDialogState,
    pub split_file_dialog_state: SplitFileDialogState,
    pub weight_cases_dialog_state: WeightCasesDialogState,
    pub linear_regression_dialog_state: LinearRegressionDialogState,
    pub reliability_dialog_state: ReliabilityDialogState,
    pub factor_dialog_state: FactorDialogState,
    pub means_dialog_state: MeansDialogState,
    pub random_seed_dialog_state: RandomSeedDialogState,
    pub pending_variable_action: VariableViewAction,
    pub status_message: StatusMessage,
    pub toolbar_position: ToolbarPosition,
    pub current_file_path: Option<PathBuf>,
    pub recent_files: RecentFiles,
    pub syntax_editor_state: SyntaxEditorState,
    pub confirm_save_dialog_state: ConfirmSaveDialogState,
    pub pending_session_action: Option<PendingSessionAction>,
    pub saved_checkpoint: Option<crate::app_session::SavedCheckpoint>,
}

impl StatCraftApp {
    pub fn new_headless() -> Self {
        let mut dock_state =
            DockState::new(vec![Tab::DataView, Tab::VariableView, Tab::SyntaxEditor]);
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
            kruskal_wallis_dialog_state: KruskalWallisDialogState::default(),
            nonparam_related_dialog_state: NonparamRelatedDialogState::default(),
            ttest_onesample_dialog_state: OneSampleTTestDialogState::default(),
            ttest_indep_dialog_state: IndependentSamplesTTestDialogState::default(),
            ttest_paired_dialog_state: PairedSamplesTTestDialogState::default(),
            anova_oneway_dialog_state: OneWayAnovaDialogState::default(),
            value_labels_dialog_state: ValueLabelsDialogState::default(),
            missing_values_dialog_state: MissingValuesDialogState::default(),
            variable_type_dialog_state: VariableTypeDialogState::default(),
            compute_dialog_state: ComputeDialogState::default(),
            count_values_dialog_state: CountValuesDialogState::default(),
            dummy_dialog_state: DummyDialogState::default(),
            shift_values_dialog_state: ShiftValuesDialogState::default(),
            date_time_dialog_state: DateTimeDialogState::default(),
            rank_cases_dialog_state: RankCasesDialogState::default(),
            time_series_dialog_state: TimeSeriesDialogState::default(),
            replace_missing_dialog_state: ReplaceMissingDialogState::default(),
            recode_dialog_state: RecodeDialogState::default(),
            auto_recode_dialog_state: AutoRecodeDialogState::default(),
            visual_binning_dialog_state: VisualBinningDialogState::default(),
            optimal_binning_dialog_state: OptimalBinningDialogState::default(),
            sort_cases_dialog_state: SortCasesDialogState::default(),
            select_cases_dialog_state: SelectCasesDialogState::default(),
            split_file_dialog_state: SplitFileDialogState::default(),
            weight_cases_dialog_state: WeightCasesDialogState::default(),
            linear_regression_dialog_state: LinearRegressionDialogState::default(),
            reliability_dialog_state: ReliabilityDialogState::default(),
            factor_dialog_state: FactorDialogState::default(),
            means_dialog_state: MeansDialogState::default(),
            random_seed_dialog_state: RandomSeedDialogState::default(),
            pending_variable_action: VariableViewAction::None,
            status_message: StatusMessage::Ready,
            toolbar_position: ToolbarPosition::Top,
            current_file_path: None,
            recent_files: RecentFiles::default(),
            syntax_editor_state: SyntaxEditorState::default(),
            confirm_save_dialog_state: ConfirmSaveDialogState::default(),
            pending_session_action: None,
            saved_checkpoint: None,
        }
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut app = Self::new_headless();
        app.recent_files = cc
            .storage
            .and_then(|s| s.get_string(RecentFiles::STORAGE_KEY))
            .and_then(|s| serde_json::from_str::<RecentFiles>(&s).ok())
            .unwrap_or_else(RecentFiles::load_default);
        app
    }

    pub fn open_csv_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(t!("dialog-open-csv-title"))
            .add_filter("Tabular Data", &["csv", "tsv", "txt"])
            .pick_file()
        {
            self.request_session_action(PendingSessionAction::OpenPath(path));
        }
    }
}

struct StatCraftTabViewer<'a> {
    dataset: &'a mut Option<DataSet>,
    data_view_state: &'a mut DataViewState,
    variable_view_state: &'a mut VariableViewState,
    output_view_state: &'a mut OutputViewerState,
    syntax_editor_state: &'a mut SyntaxEditorState,
    pending_variable_action: &'a mut VariableViewAction,
}

impl<'a> TabViewer for StatCraftTabViewer<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::DataView => t!("menu-view-data").into(),
            Tab::VariableView => t!("menu-view-variables").into(),
            Tab::OutputViewer => t!("menu-view-output").into(),
            Tab::SyntaxEditor => t!("menu-file-new-syntax").into(),
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
            Tab::SyntaxEditor => {
                render_syntax_editor(ui, self.syntax_editor_state);
            }
        }
    }
}

impl eframe::App for StatCraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        render_resize_zones(ctx, is_maximized);

        if ctx.input(|i| i.viewport().close_requested()) && self.has_unsaved_changes() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.request_session_action(PendingSessionAction::CloseProject);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::N,
            ))
        }) {
            self.request_session_action(PendingSessionAction::NewProject);
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::N,
            ))
        }) {
            self.request_session_action(PendingSessionAction::NewDataset);
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::O,
            ))
        }) {
            self.open_data_dialog();
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::S,
            ))
        }) {
            self.save_project();
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::S,
            ))
        }) {
            self.save_project_as();
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::O,
            ))
        }) {
            self.open_project_dialog();
        }

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

                let file_name = self
                    .current_file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| match &self.status_message {
                        StatusMessage::DatasetImported { name, .. } => name.clone(),
                        StatusMessage::ProjectOpened { name, .. } => name.clone(),
                        _ => "Untitled-1".to_string(),
                    });
                let asterisk = if self.has_unsaved_changes() { "*" } else { "" };
                let project_title = format!("{}{} •", file_name, asterisk);

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
        let can_redo = self
            .dataset
            .as_ref()
            .map(|d| !d.delta.redo_stack.is_empty())
            .unwrap_or(false);
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
                ui.colored_label(self.status_message.color(), self.status_message.text());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ds) = &self.dataset {
                        ui.label(t!("status-cols-count", count = ds.col_count()));
                        ui.separator();
                        ui.label(t!("status-rows-count", count = ds.row_count()));
                        if let Some(filter_var) = ds.active_filter_var() {
                            ui.separator();
                            ui.label(t!("status-filter-on", var = filter_var));
                        }
                        if let Some(split) = ds.split_file() {
                            if split.is_active() {
                                ui.separator();
                                let vars_str = split.variables.join(", ");
                                ui.label(t!("status-split-file-on", vars = vars_str.as_str()));
                            }
                        }
                        if let Some(weight_var) = ds.weight_var() {
                            ui.separator();
                            ui.label(t!("status-weight-on", var = weight_var));
                        }
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
            ToolbarAction::SaveData => {
                self.save_project();
            }
            ToolbarAction::Undo => {
                let _ = self.command_bus.undo(&mut self.dataset);
            }
            ToolbarAction::Redo => {
                if let Some(ds) = &mut self.dataset {
                    let _ = ds.redo();
                }
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
                syntax_editor_state: &mut self.syntax_editor_state,
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
        self.handle_confirm_save_dialog(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(json_str) = serde_json::to_string(&self.recent_files) {
            storage.set_string(RecentFiles::STORAGE_KEY, json_str);
        }
    }
}
