use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use crate::theme::Tokens;
use egui::{vec2, Align, Button, DragValue, Layout, ScrollArea, Stroke, TextEdit, Ui};
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;
use statcraft_syntax::commands::TimeSeriesVarSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeSeriesFnType {
    #[default]
    Diff,
    SeasonalDiff,
    SimpleMovingAverage,
    CumulativeSum,
}

impl TimeSeriesFnType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeSeriesFnType::Diff => "diff",
            TimeSeriesFnType::SeasonalDiff => "sdiff",
            TimeSeriesFnType::SimpleMovingAverage => "sma",
            TimeSeriesFnType::CumulativeSum => "csum",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TimeSeriesFnType::Diff => "DIFF",
            TimeSeriesFnType::SeasonalDiff => "SDIFF",
            TimeSeriesFnType::SimpleMovingAverage => "SMA",
            TimeSeriesFnType::CumulativeSum => "CSUM",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeSeriesItem {
    pub source_var: String,
    pub target_var: String,
    pub fn_type: TimeSeriesFnType,
    pub order: usize,
    pub target_label: String,
}

impl TimeSeriesItem {
    pub fn display_text(&self) -> String {
        match self.fn_type {
            TimeSeriesFnType::CumulativeSum => {
                format!("{} = CSUM({})", self.target_var, self.source_var)
            }
            _ => format!(
                "{} = {}({}, {})",
                self.target_var,
                self.fn_type.display_name(),
                self.source_var,
                self.order
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeSeriesDialogState {
    pub is_open: bool,
    pub selected_available_idx: Option<usize>,
    pub items: Vec<TimeSeriesItem>,
    pub selected_item_idx: Option<usize>,
    pub target_name: String,
    pub target_label: String,
    pub fn_type: TimeSeriesFnType,
    pub order: usize,
}

impl Default for TimeSeriesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_available_idx: None,
            items: Vec::new(),
            selected_item_idx: None,
            target_name: String::new(),
            target_label: String::new(),
            fn_type: TimeSeriesFnType::Diff,
            order: 1,
        }
    }
}

impl TimeSeriesDialogState {
    pub fn reset(&mut self) {
        self.selected_available_idx = None;
        self.items.clear();
        self.selected_item_idx = None;
        self.target_name.clear();
        self.target_label.clear();
        self.fn_type = TimeSeriesFnType::Diff;
        self.order = 1;
    }
}

pub enum TimeSeriesDialogAction {
    Run { variables: Vec<TimeSeriesVarSpec> },
    Paste { variables: Vec<TimeSeriesVarSpec> },
    None,
}

pub fn render_time_series_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut TimeSeriesDialogState,
) -> TimeSeriesDialogAction {
    if !state.is_open {
        return TimeSeriesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return TimeSeriesDialogAction::None;
        }
    };

    let mut action = TimeSeriesDialogAction::None;
    let mut close = false;

    let available_vars: Vec<(usize, String, String, Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary
                .get(col_idx)
                .map(|meta| (col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    let mut is_open = state.is_open;
    DialogShell::standard(t!("dialog-timeseries-title")).show(ctx, &mut is_open, |ui| {
        render_content(ui, state, &available_vars);

        let can_run = !state.items.is_empty()
            && state
                .items
                .iter()
                .all(|it| !it.target_var.trim().is_empty());

        let bar_action = DialogActionBar::new(can_run).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                let variables = state
                    .items
                    .iter()
                    .map(|it| TimeSeriesVarSpec {
                        source_var: it.source_var.clone(),
                        target_var: it.target_var.clone(),
                        function: it.fn_type.as_str().to_string(),
                        order: it.order,
                        target_label: if it.target_label.trim().is_empty() {
                            None
                        } else {
                            Some(it.target_label.trim().to_string())
                        },
                    })
                    .collect();
                action = TimeSeriesDialogAction::Run { variables };
                close = true;
            }
            ActionBarAction::Paste => {
                let variables = state
                    .items
                    .iter()
                    .map(|it| TimeSeriesVarSpec {
                        source_var: it.source_var.clone(),
                        target_var: it.target_var.clone(),
                        function: it.fn_type.as_str().to_string(),
                        order: it.order,
                        target_label: if it.target_label.trim().is_empty() {
                            None
                        } else {
                            Some(it.target_label.trim().to_string())
                        },
                    })
                    .collect();
                action = TimeSeriesDialogAction::Paste { variables };
                close = true;
            }
            ActionBarAction::Reset => {
                state.reset();
            }
            ActionBarAction::Cancel => {
                close = true;
            }
            ActionBarAction::None | ActionBarAction::Help => {}
        }
    });

    state.is_open = is_open && !close;
    action
}

fn render_content(
    ui: &mut Ui,
    state: &mut TimeSeriesDialogState,
    available_vars: &[(usize, String, String, Measure)],
) {
    ui.horizontal(|ui| {
        // Col 1: Available variables list (170 px)
        let source_items: Vec<VariableListItem> = available_vars
            .iter()
            .enumerate()
            .map(|(idx, (_col, name, label, measure))| VariableListItem {
                name,
                display_text: format_var_display(name, label),
                measure: *measure,
                is_selected: state.selected_available_idx == Some(idx),
            })
            .collect();

        let source_ev = render_variable_list_panel(
            ui,
            t!("dialog-variables-source"),
            vec2(170.0, 310.0),
            "timeseries_source_vars",
            &source_items,
        );

        match source_ev {
            ListEvent::Clicked(idx) => {
                state.selected_available_idx = Some(idx);
                state.selected_item_idx = None;
            }
            ListEvent::DoubleClicked(idx) => {
                if let Some((_, name, _, _)) = available_vars.get(idx) {
                    add_default_item(state, name);
                }
            }
            ListEvent::None => {}
        }

        // Col 2: Transfer button (36 px)
        let can_move_right = state.selected_available_idx.is_some();
        let can_move_left = state.selected_item_idx.is_some();

        let transfer_btn = if state.selected_item_idx.is_some() {
            TransferButton::to_source(can_move_left)
        } else {
            TransferButton::to_target(can_move_right)
        };

        ui.allocate_ui_with_layout(vec2(36.0, 310.0), Layout::top_down(Align::Center), |ui| {
            ui.add_space(60.0);
            if transfer_btn.show(ui).clicked() {
                if state.selected_item_idx.is_some() {
                    if let Some(idx) = state.selected_item_idx {
                        if idx < state.items.len() {
                            state.items.remove(idx);
                            state.selected_item_idx = None;
                            state.target_name.clear();
                            state.target_label.clear();
                        }
                    }
                } else if let Some(idx) = state.selected_available_idx {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        add_default_item(state, name);
                    }
                }
            }
        });

        // Col 3: Items list + Function & Settings
        ui.allocate_ui_with_layout(vec2(320.0, 310.0), Layout::top_down(Align::Min), |ui| {
            ui.label(t!("dialog-timeseries-new-vars"));
            egui::Frame::canvas(ui.style())
                .fill(Tokens::BG_PANEL)
                .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
                .show(ui, |ui| {
                    let list_sz = vec2(316.0, 95.0);
                    ui.set_min_size(list_sz);
                    ui.set_max_size(list_sz);
                    ScrollArea::vertical().show(ui, |ui| {
                        let mut clicked_idx = None;
                        for (idx, item) in state.items.iter().enumerate() {
                            let is_sel = state.selected_item_idx == Some(idx);
                            if ui.selectable_label(is_sel, item.display_text()).clicked() {
                                clicked_idx = Some(idx);
                            }
                        }
                        if let Some(idx) = clicked_idx {
                            state.selected_item_idx = Some(idx);
                            state.selected_available_idx = None;
                            if let Some(it) = state.items.get(idx) {
                                state.target_name = it.target_var.clone();
                                state.target_label = it.target_label.clone();
                                state.fn_type = it.fn_type;
                                state.order = it.order;
                            }
                        }
                    });
                });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(t!("dialog-timeseries-name"));
                ui.add(TextEdit::singleline(&mut state.target_name).desired_width(90.0));
                ui.add_space(8.0);
                ui.label(t!("dialog-timeseries-label"));
                ui.add(TextEdit::singleline(&mut state.target_label).desired_width(120.0));
            });

            ui.add_space(4.0);

            GroupBox::new(t!("dialog-timeseries-function-group")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.radio_value(
                            &mut state.fn_type,
                            TimeSeriesFnType::Diff,
                            t!("dialog-timeseries-fn-diff"),
                        );
                        ui.radio_value(
                            &mut state.fn_type,
                            TimeSeriesFnType::SeasonalDiff,
                            t!("dialog-timeseries-fn-sdiff"),
                        );
                    });
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.radio_value(
                            &mut state.fn_type,
                            TimeSeriesFnType::SimpleMovingAverage,
                            t!("dialog-timeseries-fn-sma"),
                        );
                        ui.radio_value(
                            &mut state.fn_type,
                            TimeSeriesFnType::CumulativeSum,
                            t!("dialog-timeseries-fn-csum"),
                        );
                    });
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let order_enabled = state.fn_type != TimeSeriesFnType::CumulativeSum;
                    ui.add_enabled_ui(order_enabled, |ui| {
                        ui.label(t!("dialog-timeseries-order"));
                        ui.add(DragValue::new(&mut state.order).speed(1.0).range(1..=1000));
                    });

                    ui.add_space(12.0);

                    let can_change =
                        state.selected_item_idx.is_some() && !state.target_name.trim().is_empty();
                    if ui
                        .add_enabled(can_change, Button::new(t!("dialog-timeseries-change")))
                        .clicked()
                    {
                        if let Some(idx) = state.selected_item_idx {
                            if let Some(it) = state.items.get_mut(idx) {
                                it.target_var = state.target_name.trim().to_string();
                                it.target_label = state.target_label.trim().to_string();
                                it.fn_type = state.fn_type;
                                it.order = state.order.max(1);
                            }
                        }
                    }
                });
            });
        });
    });
}

fn add_default_item(state: &mut TimeSeriesDialogState, source_name: &str) {
    let suffix = match state.fn_type {
        TimeSeriesFnType::Diff => format!("_diff{}", state.order),
        TimeSeriesFnType::SeasonalDiff => format!("_sdiff{}", state.order),
        TimeSeriesFnType::SimpleMovingAverage => format!("_sma{}", state.order),
        TimeSeriesFnType::CumulativeSum => "_csum".to_string(),
    };
    let target = format!("{}{}", source_name, suffix);
    state.items.push(TimeSeriesItem {
        source_var: source_name.to_string(),
        target_var: target.clone(),
        fn_type: state.fn_type,
        order: state.order.max(1),
        target_label: String::new(),
    });
    state.selected_item_idx = Some(state.items.len() - 1);
    state.target_name = target;
    state.target_label.clear();
    state.selected_available_idx = None;
}
