use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, TransferButton, VariableListItem,
};
use crate::theme::Tokens;
use egui::{vec2, Align, Button, DragValue, Layout, ScrollArea, Stroke, TextEdit, Ui};
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;
use statcraft_syntax::commands::ReplaceMissingItemSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogImputeMethod {
    SeriesMean,
    MeanOfNearby,
    MedianOfNearby,
    LinearInterpolation,
    LinearTrendAtPoint,
}

impl DialogImputeMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            DialogImputeMethod::SeriesMean => "series_mean",
            DialogImputeMethod::MeanOfNearby => "mean_nearby",
            DialogImputeMethod::MedianOfNearby => "median_nearby",
            DialogImputeMethod::LinearInterpolation => "linear",
            DialogImputeMethod::LinearTrendAtPoint => "trend",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceMissingItem {
    pub source: String,
    pub target: String,
    pub method: DialogImputeMethod,
    pub span: usize,
}

impl ReplaceMissingItem {
    pub fn display_label(&self) -> String {
        match self.method {
            DialogImputeMethod::SeriesMean => format!("{} -> SMEAN({})", self.target, self.source),
            DialogImputeMethod::MeanOfNearby => {
                format!("{} -> MEAN({}, {})", self.target, self.source, self.span)
            }
            DialogImputeMethod::MedianOfNearby => {
                format!("{} -> MEDIAN({}, {})", self.target, self.source, self.span)
            }
            DialogImputeMethod::LinearInterpolation => {
                format!("{} -> LINEAR({})", self.target, self.source)
            }
            DialogImputeMethod::LinearTrendAtPoint => {
                format!("{} -> TREND({})", self.target, self.source)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplaceMissingDialogState {
    pub is_open: bool,
    pub selected_available_idx: Option<usize>,
    pub items: Vec<ReplaceMissingItem>,
    pub selected_item_idx: Option<usize>,
    pub target_name_input: String,
    pub current_method: DialogImputeMethod,
    pub current_span: usize,
}

impl Default for ReplaceMissingDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_available_idx: None,
            items: Vec::new(),
            selected_item_idx: None,
            target_name_input: String::new(),
            current_method: DialogImputeMethod::SeriesMean,
            current_span: 2,
        }
    }
}

impl ReplaceMissingDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_available_idx = None;
        self.items.clear();
        self.selected_item_idx = None;
        self.target_name_input.clear();
        self.current_method = DialogImputeMethod::SeriesMean;
        self.current_span = 2;
    }

    pub fn reset(&mut self) {
        self.selected_available_idx = None;
        self.items.clear();
        self.selected_item_idx = None;
        self.target_name_input.clear();
        self.current_method = DialogImputeMethod::SeriesMean;
        self.current_span = 2;
    }
}

pub enum ReplaceMissingDialogAction {
    Run { specs: Vec<ReplaceMissingItemSpec> },
    Paste { specs: Vec<ReplaceMissingItemSpec> },
    None,
}

pub fn render_replace_missing_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut ReplaceMissingDialogState,
) -> ReplaceMissingDialogAction {
    if !state.is_open {
        return ReplaceMissingDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return ReplaceMissingDialogAction::None;
        }
    };

    let mut action = ReplaceMissingDialogAction::None;
    let mut is_open = state.is_open;
    let mut close = false;

    let available_vars: Vec<(usize, String, String, Measure)> = (0..ds.col_count())
        .filter_map(|idx| {
            ds.dictionary
                .get(idx)
                .map(|meta| (idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    DialogShell::standard(t!("dialog-missing-title")).show(ctx, &mut is_open, |ui| {
        render_content(ui, state, &available_vars);

        ui.add_space(8.0);

        let can_run =
            !state.items.is_empty() && state.items.iter().all(|it| !it.target.trim().is_empty());

        let bar_action = DialogActionBar::new(can_run).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                let specs = state
                    .items
                    .iter()
                    .map(|it| ReplaceMissingItemSpec {
                        source_var: it.source.clone(),
                        target_var: it.target.clone(),
                        method: it.method.as_str().to_string(),
                        span: it.span,
                    })
                    .collect();
                action = ReplaceMissingDialogAction::Run { specs };
                close = true;
            }
            ActionBarAction::Paste => {
                let specs = state
                    .items
                    .iter()
                    .map(|it| ReplaceMissingItemSpec {
                        source_var: it.source.clone(),
                        target_var: it.target.clone(),
                        method: it.method.as_str().to_string(),
                        span: it.span,
                    })
                    .collect();
                action = ReplaceMissingDialogAction::Paste { specs };
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
    state: &mut ReplaceMissingDialogState,
    available_vars: &[(usize, String, String, Measure)],
) {
    ui.horizontal(|ui| {
        // Col 1: Available variables list (180 px)
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
            vec2(180.0, 310.0),
            "replace_missing_source_vars",
            &source_items,
        );

        match source_ev {
            ListEvent::Clicked(idx) => {
                state.selected_available_idx = Some(idx);
                state.selected_item_idx = None;
            }
            ListEvent::DoubleClicked(idx) => {
                if let Some((_, name, _, _)) = available_vars.get(idx) {
                    let default_tgt = format!("{}_1", name);
                    state.items.push(ReplaceMissingItem {
                        source: name.clone(),
                        target: default_tgt.clone(),
                        method: state.current_method,
                        span: state.current_span,
                    });
                    state.selected_item_idx = Some(state.items.len() - 1);
                    state.target_name_input = default_tgt;
                    state.selected_available_idx = None;
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
                            state.target_name_input.clear();
                        }
                    }
                } else if let Some(idx) = state.selected_available_idx {
                    if let Some((_, name, _, _)) = available_vars.get(idx) {
                        let default_tgt = format!("{}_1", name);
                        state.items.push(ReplaceMissingItem {
                            source: name.clone(),
                            target: default_tgt.clone(),
                            method: state.current_method,
                            span: state.current_span,
                        });
                        state.selected_item_idx = Some(state.items.len() - 1);
                        state.target_name_input = default_tgt;
                    }
                }
            }
        });

        // Col 3: Items list + Name/Change + Method GroupBox (290 px)
        ui.allocate_ui_with_layout(vec2(290.0, 310.0), Layout::top_down(Align::Min), |ui| {
            ui.label(t!("dialog-missing-new-vars"));
            egui::Frame::canvas(ui.style())
                .fill(Tokens::BG_PANEL)
                .stroke(Stroke::new(1.0_f32, Tokens::BORDER))
                .show(ui, |ui| {
                    let list_sz = vec2(286.0, 95.0);
                    ui.set_min_size(list_sz);
                    ui.set_max_size(list_sz);
                    ScrollArea::vertical().show(ui, |ui| {
                        let mut clicked_idx = None;
                        for (idx, item) in state.items.iter().enumerate() {
                            let is_sel = state.selected_item_idx == Some(idx);
                            let text = item.display_label();
                            if ui.selectable_label(is_sel, text).clicked() {
                                clicked_idx = Some(idx);
                            }
                        }
                        if let Some(idx) = clicked_idx {
                            state.selected_item_idx = Some(idx);
                            state.selected_available_idx = None;
                            if let Some(it) = state.items.get(idx) {
                                state.target_name_input = it.target.clone();
                                state.current_method = it.method;
                                state.current_span = it.span;
                            }
                        }
                    });
                });

            ui.add_space(4.0);

            // Name and Change button
            ui.horizontal(|ui| {
                ui.label(t!("dialog-missing-name"));
                ui.add(TextEdit::singleline(&mut state.target_name_input).desired_width(140.0));
                let can_change =
                    state.selected_item_idx.is_some() && !state.target_name_input.trim().is_empty();
                if ui
                    .add_enabled(can_change, Button::new(t!("dialog-missing-change")))
                    .clicked()
                {
                    if let Some(idx) = state.selected_item_idx {
                        if let Some(it) = state.items.get_mut(idx) {
                            it.target = state.target_name_input.trim().to_string();
                            it.method = state.current_method;
                            it.span = state.current_span;
                        }
                    }
                }
            });

            ui.add_space(4.0);

            // Method selection GroupBox
            GroupBox::new(t!("dialog-missing-method-group")).show(ui, |ui| {
                ui.radio_value(
                    &mut state.current_method,
                    DialogImputeMethod::SeriesMean,
                    t!("dialog-missing-method-smean"),
                );
                ui.radio_value(
                    &mut state.current_method,
                    DialogImputeMethod::MeanOfNearby,
                    t!("dialog-missing-method-nearby-mean"),
                );
                ui.radio_value(
                    &mut state.current_method,
                    DialogImputeMethod::MedianOfNearby,
                    t!("dialog-missing-method-nearby-median"),
                );
                ui.radio_value(
                    &mut state.current_method,
                    DialogImputeMethod::LinearInterpolation,
                    t!("dialog-missing-method-linear"),
                );
                ui.radio_value(
                    &mut state.current_method,
                    DialogImputeMethod::LinearTrendAtPoint,
                    t!("dialog-missing-method-trend"),
                );

                if state.current_method == DialogImputeMethod::MeanOfNearby
                    || state.current_method == DialogImputeMethod::MedianOfNearby
                {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-missing-span"));
                        ui.add(
                            DragValue::new(&mut state.current_span)
                                .range(1..=100)
                                .speed(1.0),
                        );
                    });
                }
            });
        });
    });
}
