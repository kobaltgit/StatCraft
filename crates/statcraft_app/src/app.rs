use crate::app_session::PendingSessionAction;
use crate::app_tab_viewer::StatCraftTabViewer;
use crate::bridge_service::BridgeService;
use crate::dialogs::ConfirmSaveDialogState;
use crate::dialogs::{
    AddCasesDialogState, AddVarsDialogState, AdpDialogState, AutoRecodeDialogState,
    CodebookDialogState, ComputeDialogState, CorrelationsDialogState, CountValuesDialogState,
    CoxRegressionDialogState, CrosstabsDialogState, CurveFitDialogState, DateTimeDialogState,
    DescriptivesDialogState, DummyDialogState, ExploreDialogState, FactorDialogState,
    FindReplaceDialogState, FrequenciesDialogState, GoToDialogState,
    IndependentSamplesTTestDialogState, KMeansDialogState, KaplanMeierDialogState,
    KruskalWallisDialogState, LifeTablesDialogState, LinearRegressionDialogState,
    LogisticRegressionDialogState, McpBridgeDialogState, MeansDialogState,
    MissingValuesDialogState, MultinomialLogisticDialogState, MultivariateGlmDialogState,
    NonparamRelatedDialogState, NonparamTwoSampleDialogState, OneSampleTTestDialogState,
    OneWayAnovaDialogState, OptimalBinningDialogState, PairedSamplesTTestDialogState,
    PartialCorrDialogState, ProgrammabilityDialogState, RandomSeedDialogState,
    RankCasesDialogState, RecodeDialogState, ReliabilityDialogState, RepeatedGlmDialogState,
    ReplaceMissingDialogState, SelectCasesDialogState, ShiftValuesDialogState,
    SortCasesDialogState, SortVariablesDialogState, SplitFileDialogState, SummarizeDialogState,
    TimeSeriesDialogState, TransposeDialogState, UnivariateGlmDialogState, ValueLabelsDialogState,
    VariableTypeDialogState, VisualBinningDialogState, WeightCasesDialogState,
};
use crate::recent_files::RecentFiles;
use crate::views::{
    DataViewAction, DataViewState, DocumentTabAction, OutputViewerState, SyntaxEditorAction,
    SyntaxEditorState, VariableViewAction, VariableViewState,
};
use crate::widgets::{
    render_caption_buttons, render_resize_zones, render_toolbar, title_x, MenuItemStatus,
    ToolbarAction, ToolbarPosition, CAPTION_BAR_HEIGHT, CAPTION_BUTTON_WIDTH,
};
use egui::{CentralPanel, SidePanel, TopBottomPanel};
use egui_dock::{DockArea, DockState, NodeIndex};
use statcraft_core::{DataSet, WorkspaceManager};
use statcraft_i18n::t;
use statcraft_syntax::CommandBus;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct MemoryGuardDialogState {
    pub is_open: bool,
    pub warning: Option<crate::system_memory::MemoryWarning>,
    pub pending_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    DataView,
    VariableView,
    OutputViewer,
    SyntaxEditor,
}

pub use crate::status_message::StatusMessage;

pub const STORAGE_KEY_THEME: &str = "statcraft_theme";

pub fn is_tab_closeable(tab: Tab) -> bool {
    match tab {
        Tab::DataView | Tab::VariableView => false,
        Tab::OutputViewer | Tab::SyntaxEditor => true,
    }
}

pub fn ensure_dock_tab_open(dock_state: &mut DockState<Tab>, tab: Tab) {
    if let Some(pos) = dock_state.find_tab(&tab) {
        dock_state.set_active_tab(pos);
        return;
    }

    match tab {
        Tab::OutputViewer => {
            if dock_state.main_surface().is_empty() {
                dock_state.push_to_first_leaf(Tab::OutputViewer);
            } else {
                dock_state.main_surface_mut().split_right(
                    egui_dock::NodeIndex::root(),
                    0.35,
                    vec![Tab::OutputViewer],
                );
            }
        }
        Tab::DataView | Tab::VariableView | Tab::SyntaxEditor => {
            dock_state.push_to_first_leaf(tab);
        }
    }

    if let Some(pos) = dock_state.find_tab(&tab) {
        dock_state.set_active_tab(pos);
    }
}

pub struct StatCraftApp {
    pub workspace: WorkspaceManager,
    pub dataset: Option<DataSet>,
    pub document_tab_scroll_offset: f32,
    pub pending_document_tab_action: DocumentTabAction,
    pub memory_guard_dialog_state: MemoryGuardDialogState,
    pub command_bus: CommandBus,
    pub dock_state: DockState<Tab>,
    pub data_view_state: DataViewState,
    pub variable_view_state: VariableViewState,
    pub output_view_state: OutputViewerState,
    pub descriptives_dialog_state: DescriptivesDialogState,
    pub frequencies_dialog_state: FrequenciesDialogState,
    pub explore_dialog_state: ExploreDialogState,
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
    pub adp_dialog_state: AdpDialogState,
    pub sort_cases_dialog_state: SortCasesDialogState,
    pub sort_variables_dialog_state: SortVariablesDialogState,
    pub transpose_dialog_state: TransposeDialogState,
    pub merge_cases_dialog_state: AddCasesDialogState,
    pub merge_vars_dialog_state: AddVarsDialogState,
    pub select_cases_dialog_state: SelectCasesDialogState,
    pub split_file_dialog_state: SplitFileDialogState,
    pub weight_cases_dialog_state: WeightCasesDialogState,
    pub linear_regression_dialog_state: LinearRegressionDialogState,
    pub partial_corr_dialog_state: PartialCorrDialogState,
    pub logistic_regression_dialog_state: LogisticRegressionDialogState,
    pub multinomial_logistic_dialog_state: MultinomialLogisticDialogState,
    pub curvefit_dialog_state: CurveFitDialogState,
    pub univariate_glm_dialog_state: UnivariateGlmDialogState,
    pub multivariate_glm_dialog_state: MultivariateGlmDialogState,
    pub repeated_glm_dialog_state: RepeatedGlmDialogState,
    pub kmeans_dialog_state: KMeansDialogState,
    pub codebook_dialog_state: CodebookDialogState,
    pub summarize_dialog_state: SummarizeDialogState,
    pub survival_km_dialog_state: KaplanMeierDialogState,
    pub survival_lifetables_dialog_state: LifeTablesDialogState,
    pub cox_regression_dialog_state: CoxRegressionDialogState,
    pub reliability_dialog_state: ReliabilityDialogState,
    pub factor_dialog_state: FactorDialogState,
    pub means_dialog_state: MeansDialogState,
    pub programmability_dialog_state: ProgrammabilityDialogState,
    pub random_seed_dialog_state: RandomSeedDialogState,
    pub mcp_bridge_dialog_state: McpBridgeDialogState,
    pub find_replace_dialog_state: FindReplaceDialogState,
    pub goto_dialog_state: GoToDialogState,
    pub clipboard_cache: Option<String>,
    pub bridge_service: BridgeService,
    pub pending_data_action: DataViewAction,
    pub pending_variable_action: VariableViewAction,
    pub status_message: StatusMessage,
    pub toolbar_position: ToolbarPosition,
    pub current_file_path: Option<PathBuf>,
    pub recent_files: RecentFiles,
    pub syntax_editor_state: SyntaxEditorState,
    pub pending_syntax_action: SyntaxEditorAction,
    pub confirm_save_dialog_state: ConfirmSaveDialogState,
    pub pending_session_action: Option<PendingSessionAction>,
    pub saved_checkpoint: Option<crate::app_session::SavedCheckpoint>,
    pub theme_mode: crate::theme::ThemeMode,
    applied_dark_mode: Option<bool>,
    pub window_frame_guard: Option<crate::window_frame::WindowFrameFn>,
}

pub const STORAGE_KEY_LANGUAGE: &str = "statcraft_language";
pub const STORAGE_KEY_BRIDGE_AUTOSTART: &str = "statcraft_mcp_bridge_autostart";

impl StatCraftApp {
    pub fn new_headless() -> Self {
        let mut dock_state =
            DockState::new(vec![Tab::DataView, Tab::VariableView, Tab::SyntaxEditor]);
        let surface = dock_state.main_surface_mut();
        surface.split_right(NodeIndex::root(), 0.35, vec![Tab::OutputViewer]);

        let workspace = WorkspaceManager::new_blank();
        let dataset = workspace.active_dataset().cloned();

        Self {
            workspace,
            dataset,
            document_tab_scroll_offset: 0.0,
            pending_document_tab_action: DocumentTabAction::None,
            memory_guard_dialog_state: MemoryGuardDialogState::default(),
            command_bus: CommandBus::with_defaults(),
            dock_state,
            data_view_state: DataViewState::default(),
            variable_view_state: VariableViewState::default(),
            output_view_state: OutputViewerState::new(),
            descriptives_dialog_state: DescriptivesDialogState::default(),
            frequencies_dialog_state: FrequenciesDialogState::default(),
            explore_dialog_state: ExploreDialogState::default(),
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
            adp_dialog_state: AdpDialogState::default(),
            sort_cases_dialog_state: SortCasesDialogState::default(),
            sort_variables_dialog_state: SortVariablesDialogState::default(),
            transpose_dialog_state: TransposeDialogState::default(),
            merge_cases_dialog_state: AddCasesDialogState::default(),
            merge_vars_dialog_state: AddVarsDialogState::default(),
            select_cases_dialog_state: SelectCasesDialogState::default(),
            split_file_dialog_state: SplitFileDialogState::default(),
            weight_cases_dialog_state: WeightCasesDialogState::default(),
            linear_regression_dialog_state: LinearRegressionDialogState::default(),
            partial_corr_dialog_state: PartialCorrDialogState::default(),
            logistic_regression_dialog_state: LogisticRegressionDialogState::default(),
            multinomial_logistic_dialog_state: MultinomialLogisticDialogState::default(),
            curvefit_dialog_state: CurveFitDialogState::default(),
            univariate_glm_dialog_state: UnivariateGlmDialogState::default(),
            multivariate_glm_dialog_state: MultivariateGlmDialogState::default(),
            repeated_glm_dialog_state: RepeatedGlmDialogState::default(),
            kmeans_dialog_state: KMeansDialogState::default(),
            codebook_dialog_state: CodebookDialogState::default(),
            summarize_dialog_state: SummarizeDialogState::default(),
            survival_km_dialog_state: KaplanMeierDialogState::default(),
            survival_lifetables_dialog_state: LifeTablesDialogState::default(),
            cox_regression_dialog_state: CoxRegressionDialogState::default(),
            reliability_dialog_state: ReliabilityDialogState::default(),
            factor_dialog_state: FactorDialogState::default(),
            means_dialog_state: MeansDialogState::default(),
            programmability_dialog_state: ProgrammabilityDialogState::default(),
            random_seed_dialog_state: RandomSeedDialogState::default(),
            mcp_bridge_dialog_state: McpBridgeDialogState::default(),
            find_replace_dialog_state: FindReplaceDialogState::default(),
            goto_dialog_state: GoToDialogState::default(),
            clipboard_cache: None,
            bridge_service: BridgeService::new(false),
            pending_data_action: DataViewAction::None,
            pending_variable_action: VariableViewAction::None,
            status_message: StatusMessage::Ready,
            toolbar_position: ToolbarPosition::Top,
            current_file_path: None,
            recent_files: RecentFiles::default(),
            syntax_editor_state: SyntaxEditorState::default(),
            pending_syntax_action: SyntaxEditorAction::None,
            confirm_save_dialog_state: ConfirmSaveDialogState::default(),
            pending_session_action: None,
            saved_checkpoint: None,
            theme_mode: crate::theme::ThemeMode::Dark,
            applied_dark_mode: None,
            window_frame_guard: None,
        }
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut app = Self::new_headless();

        if let Some(saved_lang) = cc.storage.and_then(|s| s.get_string(STORAGE_KEY_LANGUAGE)) {
            statcraft_i18n::set_language(&saved_lang);
        }

        if let Some(saved_theme) = cc
            .storage
            .and_then(|s| s.get_string(STORAGE_KEY_THEME))
            .and_then(|raw| serde_json::from_str::<crate::theme::ThemeMode>(&raw).ok())
        {
            app.theme_mode = saved_theme;
        }

        let is_dark = match app.theme_mode {
            crate::theme::ThemeMode::Dark => true,
            crate::theme::ThemeMode::Light => false,
            crate::theme::ThemeMode::System => cc
                .egui_ctx
                .system_theme()
                .map(|t| t == egui::Theme::Dark)
                .unwrap_or(true),
        };
        crate::theme::set_dark_mode(is_dark);
        cc.egui_ctx.set_visuals(if is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        app.applied_dark_mode = Some(is_dark);

        app.recent_files = cc
            .storage
            .and_then(|s| s.get_string(RecentFiles::STORAGE_KEY))
            .and_then(|s| serde_json::from_str::<RecentFiles>(&s).ok())
            .unwrap_or_else(RecentFiles::load_default);

        let bridge_autostart = cc
            .storage
            .and_then(|s| s.get_string(STORAGE_KEY_BRIDGE_AUTOSTART))
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);

        app.bridge_service.set_autostart(bridge_autostart);
        if bridge_autostart {
            app.bridge_service.start();
        }

        app.window_frame_guard = crate::window_frame::service(cc);

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

    pub fn ensure_tab_open(&mut self, tab: Tab) {
        ensure_dock_tab_open(&mut self.dock_state, tab);
    }
}

impl eframe::App for StatCraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(guard) = self.window_frame_guard.as_mut() {
            guard();
        }

        let is_dark = match self.theme_mode {
            crate::theme::ThemeMode::Dark => true,
            crate::theme::ThemeMode::Light => false,
            crate::theme::ThemeMode::System => ctx
                .system_theme()
                .map(|t| t == egui::Theme::Dark)
                .unwrap_or(true),
        };
        if self.applied_dark_mode != Some(is_dark) {
            self.applied_dark_mode = Some(is_dark);
            crate::theme::set_dark_mode(is_dark);
            ctx.set_visuals(if is_dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
        }

        let active_name = self.active_dataset_display_name();
        self.output_view_state.set_active_dataset(active_name);

        self.bridge_service.process_requests(
            &mut self.dataset,
            &mut self.command_bus,
            &mut self.output_view_state,
            &mut self.dock_state,
        );

        if self.bridge_service.is_active() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        render_resize_zones(ctx, is_maximized);

        self.handle_keyboard_shortcuts(ctx);

        // 1. Top Custom Title Bar (CSD)
        TopBottomPanel::top("title_bar")
            .exact_height(CAPTION_BAR_HEIGHT)
            .frame(egui::Frame::none().fill(crate::theme::Tokens::bg_header()))
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
                    .workspace
                    .active_session()
                    .map(|s| s.file_name().to_string())
                    .or_else(|| {
                        self.current_file_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| match &self.status_message {
                        StatusMessage::DatasetImported { name, .. } => name.clone(),
                        StatusMessage::ProjectOpened { name, .. } => name.clone(),
                        _ => "DataSet1".to_string(),
                    });
                let asterisk = if self.has_unsaved_changes() { " *" } else { "" };
                let project_title = format!("{}{}", file_name, asterisk);

                let font_id = egui::FontId::proportional(12.5);
                let galley = ui.painter().layout_no_wrap(
                    project_title.clone(),
                    font_id.clone(),
                    crate::theme::Tokens::text_muted(),
                );
                let text_w = galley.size().x;

                if let Some(x) = title_x(center_x, menus_end_x, controls_start_x, text_w) {
                    let text_pos = egui::pos2(x, bar_rect.center().y - galley.size().y / 2.0);
                    ui.painter()
                        .galley(text_pos, galley, crate::theme::Tokens::text_muted());
                } else if controls_start_x - menus_end_x > 80.0 {
                    let avail = controls_start_x - menus_end_x - 24.0;
                    let mut job = egui::text::LayoutJob::simple_singleline(
                        project_title,
                        font_id,
                        crate::theme::Tokens::text_muted(),
                    );
                    job.wrap = egui::text::TextWrapping::truncate_at_width(avail);
                    let g = ui.painter().layout_job(job);
                    let text_pos =
                        egui::pos2(menus_end_x + 12.0, bar_rect.center().y - g.size().y / 2.0);
                    ui.painter()
                        .galley(text_pos, g, crate::theme::Tokens::text_muted());
                }
            });

        let has_dataset = self
            .dataset
            .as_ref()
            .map(|d| d.col_count() > 0)
            .unwrap_or(false);
        let can_undo = self.command_bus.can_undo();
        let can_redo = self.command_bus.can_redo();
        let mut toolbar_action = ToolbarAction::None;

        // 2. Main Toolbar (Top)
        if self.toolbar_position == ToolbarPosition::Top {
            TopBottomPanel::top("main_toolbar_top")
                .exact_height(32.0)
                .frame(
                    egui::Frame::none()
                        .fill(crate::theme::Tokens::bg_panel())
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
                        .fill(crate::theme::Tokens::bg_panel())
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
                let _ = self.command_bus.redo(&mut self.dataset);
            }
            ToolbarAction::InsertCase => {
                let _ = self.command_bus.execute(
                    "data.add_row",
                    &mut self.dataset,
                    serde_json::json!({}),
                );
            }
            ToolbarAction::InsertVariable => {
                let col_count = self.dataset.as_ref().map(|d| d.col_count()).unwrap_or(0);
                let mut var_num = col_count + 1;
                let mut var_name = format!("VAR{:05}", var_num);
                if let Some(ds) = &self.dataset {
                    while ds
                        .dictionary
                        .variables
                        .iter()
                        .any(|v| v.name.eq_ignore_ascii_case(&var_name))
                    {
                        var_num += 1;
                        var_name = format!("VAR{:05}", var_num);
                    }
                }
                let _ = self.command_bus.execute(
                    "var.add",
                    &mut self.dataset,
                    serde_json::json!({ "name": var_name }),
                );
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
                workspace: &self.workspace,
                document_tab_scroll_offset: &mut self.document_tab_scroll_offset,
                pending_document_tab_action: &mut self.pending_document_tab_action,
                dataset: &mut self.dataset,
                data_view_state: &mut self.data_view_state,
                variable_view_state: &mut self.variable_view_state,
                output_view_state: &mut self.output_view_state,
                syntax_editor_state: &mut self.syntax_editor_state,
                pending_data_action: &mut self.pending_data_action,
                pending_variable_action: &mut self.pending_variable_action,
                pending_syntax_action: &mut self.pending_syntax_action,
            };

            let style = egui_dock::Style::from_egui(ui.style());

            DockArea::new(&mut self.dock_state)
                .style(style)
                .show_inside(ui, &mut tab_viewer);
        });

        match self.pending_document_tab_action {
            DocumentTabAction::Select(id) => {
                self.switch_to_dataset(id);
                self.pending_document_tab_action = DocumentTabAction::None;
            }
            DocumentTabAction::Close(id) => {
                self.request_close_dataset(id);
                self.pending_document_tab_action = DocumentTabAction::None;
            }
            DocumentTabAction::NewBlank => {
                self.add_new_blank_dataset();
                self.pending_document_tab_action = DocumentTabAction::None;
            }
            DocumentTabAction::None => {}
        }

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

        // 4c. Handle Data View actions (sorting, deleting cases, inserting cases/vars)
        match std::mem::replace(&mut self.pending_data_action, DataViewAction::None) {
            DataViewAction::SortCases {
                column_name,
                ascending,
            } => {
                let dir_str = if ascending { "Ascending" } else { "Descending" };
                let params = serde_json::json!({
                    "keys": [{
                        "column_name": column_name,
                        "direction": dir_str,
                    }]
                });
                match self
                    .command_bus
                    .execute("data.sort_cases", &mut self.dataset, params)
                {
                    Ok(_) => {
                        let keys = vec![statcraft_algo::SortKeySpec::new(
                            column_name,
                            if ascending {
                                statcraft_algo::SortDirection::Ascending
                            } else {
                                statcraft_algo::SortDirection::Descending
                            },
                        )];
                        let sps = statcraft_syntax::commands::SortCasesCommand::to_sps(&keys);
                        self.output_view_state.add_entry(t!("output-log"), sps);
                        self.status_message = StatusMessage::SortCasesExecuted { count: 1 };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            DataViewAction::DeleteCases { mut rows } => {
                rows.sort_unstable();
                rows.dedup();
                for row_idx in rows.into_iter().rev() {
                    let params = serde_json::json!({
                        "row_index": row_idx,
                    });
                    let _ = self
                        .command_bus
                        .execute("data.delete_row", &mut self.dataset, params);
                }
            }
            DataViewAction::InsertCaseAbove { row } => {
                let params = serde_json::json!({
                    "row_index": row,
                });
                let _ = self
                    .command_bus
                    .execute("data.add_row", &mut self.dataset, params);
            }
            DataViewAction::InsertVariable { col } => {
                let col_count = self.dataset.as_ref().map(|d| d.col_count()).unwrap_or(0);
                let mut var_num = col_count + 1;
                let mut var_name = format!("VAR{:05}", var_num);
                if let Some(ds) = &self.dataset {
                    while ds
                        .dictionary
                        .variables
                        .iter()
                        .any(|v| v.name.eq_ignore_ascii_case(&var_name))
                    {
                        var_num += 1;
                        var_name = format!("VAR{:05}", var_num);
                    }
                }
                let params = serde_json::json!({
                    "col_index": col,
                    "name": var_name,
                });
                let _ = self
                    .command_bus
                    .execute("var.add", &mut self.dataset, params);
            }
            DataViewAction::None => {}
        }

        // 4b. Handle Syntax Editor actions (running batch or selection)
        match std::mem::replace(&mut self.pending_syntax_action, SyntaxEditorAction::None) {
            SyntaxEditorAction::RunAll => {
                let script = self.syntax_editor_state.text.clone();
                self.ensure_tab_open(Tab::OutputViewer);
                match crate::syntax_runner::run_syntax_batch(
                    &script,
                    &mut self.dataset,
                    &mut self.command_bus,
                    &mut self.output_view_state,
                ) {
                    Ok(outcome) => {
                        self.status_message = StatusMessage::SyntaxExecuted {
                            count: outcome.count,
                        };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            SyntaxEditorAction::RunSelection(script) => {
                self.ensure_tab_open(Tab::OutputViewer);
                match crate::syntax_runner::run_syntax_batch(
                    &script,
                    &mut self.dataset,
                    &mut self.command_bus,
                    &mut self.output_view_state,
                ) {
                    Ok(outcome) => {
                        self.status_message = StatusMessage::SyntaxExecuted {
                            count: outcome.count,
                        };
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            SyntaxEditorAction::Clear => {
                self.syntax_editor_state.clear();
            }
            SyntaxEditorAction::Open => {
                self.open_syntax_dialog();
            }
            SyntaxEditorAction::Save => {
                self.save_syntax();
            }
            SyntaxEditorAction::None => {}
        }

        // 5. Modal Dialogs
        self.handle_dialogs(ctx);
        self.handle_edit_dialogs(ctx);
        self.handle_confirm_save_dialog(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(json_str) = serde_json::to_string(&self.recent_files) {
            storage.set_string(RecentFiles::STORAGE_KEY, json_str);
        }
        storage.set_string(STORAGE_KEY_LANGUAGE, statcraft_i18n::current_language());
        if let Ok(theme_str) = serde_json::to_string(&self.theme_mode) {
            storage.set_string(STORAGE_KEY_THEME, theme_str);
        }
        storage.set_string(
            STORAGE_KEY_BRIDGE_AUTOSTART,
            self.bridge_service.autostart().to_string(),
        );
    }
}
