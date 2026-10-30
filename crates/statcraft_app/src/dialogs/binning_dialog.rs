use super::binning_cutpoints_dialog::render_binning_cutpoints_subdialog;
use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    GroupBox, ListEvent, VariableListItem,
};
use egui::{vec2, Button, ScrollArea, TextEdit, Ui};
use statcraft_algo::{format_bound, generate_bin_labels, Cutpoint};
use statcraft_core::{DataSet, Measure};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CutpointMethod {
    #[default]
    EqualWidth,
    EqualPercentiles,
    StdDev,
}

#[derive(Debug, Clone)]
pub struct VisualBinningDialogState {
    pub is_open: bool,
    pub is_cutpoints_subdialog_open: bool,

    pub selected_var: Option<String>,
    pub selected_var_idx: Option<usize>,

    pub target_var: String,
    pub target_label: String,

    pub inclusive: bool,

    pub cutpoints: Vec<f64>,
    pub selected_cutpoint_idx: Option<usize>,
    pub custom_labels: Vec<String>,

    pub cutpoint_method: CutpointMethod,
    pub subdialog_num_intervals: String,
    pub manual_cutpoint_val: String,
}

impl Default for VisualBinningDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            is_cutpoints_subdialog_open: false,
            selected_var: None,
            selected_var_idx: None,
            target_var: String::new(),
            target_label: String::new(),
            inclusive: true,
            cutpoints: Vec::new(),
            selected_cutpoint_idx: None,
            custom_labels: vec!["All".to_string()],
            cutpoint_method: CutpointMethod::EqualWidth,
            subdialog_num_intervals: "3".to_string(),
            manual_cutpoint_val: String::new(),
        }
    }
}

impl VisualBinningDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.is_cutpoints_subdialog_open = false;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.selected_var = None;
        self.selected_var_idx = None;
        self.target_var.clear();
        self.target_label.clear();
        self.inclusive = true;
        self.cutpoints.clear();
        self.selected_cutpoint_idx = None;
        self.custom_labels = vec!["All".to_string()];
        self.cutpoint_method = CutpointMethod::EqualWidth;
        self.subdialog_num_intervals = "3".to_string();
        self.manual_cutpoint_val.clear();
    }

    pub fn select_variable(&mut self, name: &str, label: &str) {
        self.selected_var = Some(name.to_string());
        self.target_var = format!("{}_binned", name);
        let base_lbl = if label.is_empty() { name } else { label };
        self.target_label = format!("{} (Binned)", base_lbl);
        self.sync_labels();
    }

    pub fn sync_labels(&mut self) {
        self.cutpoints
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let cps: Vec<Cutpoint> = self
            .cutpoints
            .iter()
            .map(|&b| Cutpoint::new(b, self.inclusive))
            .collect();
        let auto_labels = generate_bin_labels(&cps);

        let needed = self.cutpoints.len() + 1;
        let mut new_labels = Vec::with_capacity(needed);
        for i in 0..needed {
            let default_lbl = auto_labels
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("Bin {}", i + 1));
            new_labels.push(default_lbl);
        }
        self.custom_labels = new_labels;
    }
}

pub enum VisualBinningDialogAction {
    Run {
        source_var: String,
        target_var: String,
        target_label: Option<String>,
        cutpoints: Vec<Cutpoint>,
        labels: Vec<String>,
    },
    Paste {
        source_var: String,
        target_var: String,
        target_label: Option<String>,
        cutpoints: Vec<Cutpoint>,
        labels: Vec<String>,
    },
    None,
}

pub fn render_visual_binning_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut VisualBinningDialogState,
) -> VisualBinningDialogAction {
    if !state.is_open {
        return VisualBinningDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return VisualBinningDialogAction::None;
        }
    };

    let mut action = VisualBinningDialogAction::None;
    let mut is_open = state.is_open;
    let mut close = false;

    let available_vars: Vec<(usize, String, String, Measure)> = (0..ds.col_count())
        .filter_map(|idx| {
            ds.dictionary
                .get(idx)
                .map(|meta| (idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    DialogShell::standard(t!("dialog-binning-title")).show(ctx, &mut is_open, |ui| {
        render_content(ui, state, &available_vars);

        ui.add_space(8.0);

        let can_run = state.selected_var.is_some() && !state.target_var.trim().is_empty();

        let bar_action = DialogActionBar::new(can_run).show(ui);
        match bar_action {
            ActionBarAction::Ok => {
                let cps: Vec<Cutpoint> = state
                    .cutpoints
                    .iter()
                    .map(|&b| Cutpoint::new(b, state.inclusive))
                    .collect();
                action = VisualBinningDialogAction::Run {
                    source_var: state.selected_var.clone().unwrap_or_default(),
                    target_var: state.target_var.clone(),
                    target_label: if state.target_label.trim().is_empty() {
                        None
                    } else {
                        Some(state.target_label.trim().to_string())
                    },
                    cutpoints: cps,
                    labels: state.custom_labels.clone(),
                };
                close = true;
            }
            ActionBarAction::Paste => {
                let cps: Vec<Cutpoint> = state
                    .cutpoints
                    .iter()
                    .map(|&b| Cutpoint::new(b, state.inclusive))
                    .collect();
                action = VisualBinningDialogAction::Paste {
                    source_var: state.selected_var.clone().unwrap_or_default(),
                    target_var: state.target_var.clone(),
                    target_label: if state.target_label.trim().is_empty() {
                        None
                    } else {
                        Some(state.target_label.trim().to_string())
                    },
                    cutpoints: cps,
                    labels: state.custom_labels.clone(),
                };
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

    render_binning_cutpoints_subdialog(ctx, dataset, state);

    if close || !is_open {
        state.is_open = false;
    }

    action
}

fn render_content(
    ui: &mut Ui,
    state: &mut VisualBinningDialogState,
    available_vars: &[(usize, String, String, Measure)],
) {
    ui.horizontal_top(|ui| {
        // Left column: Variable List
        ui.vertical(|ui| {
            ui.label(t!("dialog-binning-scanned-vars"));
            ui.add_space(2.0);

            let items: Vec<VariableListItem> = available_vars
                .iter()
                .enumerate()
                .map(|(idx, (_, name, label, measure))| VariableListItem {
                    name,
                    display_text: format_var_display(name, label),
                    measure: *measure,
                    is_selected: state.selected_var_idx == Some(idx),
                })
                .collect();

            let ev = render_variable_list_panel(
                ui,
                t!("dialog-variables-source"),
                vec2(160.0, 310.0),
                "binning_var_list",
                &items,
            );

            match ev {
                ListEvent::Clicked(idx) | ListEvent::DoubleClicked(idx) => {
                    state.selected_var_idx = Some(idx);
                    if let Some((_, name, label, _)) = available_vars.get(idx) {
                        state.select_variable(name, label);
                    }
                }
                ListEvent::None => {}
            }
        });

        ui.add_space(8.0);

        // Right column: Target info, Endpoints, Cutpoints Grid
        ui.vertical(|ui| {
            GroupBox::new(t!("dialog-binning-binned-var")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-binning-name"));
                    ui.add(TextEdit::singleline(&mut state.target_var).desired_width(120.0));

                    ui.add_space(8.0);
                    ui.label(t!("dialog-binning-label"));
                    ui.add(TextEdit::singleline(&mut state.target_label).desired_width(140.0));
                });
            });

            ui.add_space(4.0);

            GroupBox::new(t!("dialog-binning-endpoints")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .radio(state.inclusive, t!("dialog-binning-included"))
                        .clicked()
                    {
                        state.inclusive = true;
                        state.sync_labels();
                    }
                    ui.add_space(10.0);
                    if ui
                        .radio(!state.inclusive, t!("dialog-binning-excluded"))
                        .clicked()
                    {
                        state.inclusive = false;
                        state.sync_labels();
                    }
                });
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        state.selected_var.is_some(),
                        Button::new(t!("dialog-binning-make-cutpoints")),
                    )
                    .clicked()
                {
                    state.is_cutpoints_subdialog_open = true;
                }

                if ui
                    .add_enabled(
                        !state.cutpoints.is_empty(),
                        Button::new(t!("dialog-binning-clear-all")),
                    )
                    .clicked()
                {
                    state.cutpoints.clear();
                    state.sync_labels();
                }
            });

            ui.add_space(4.0);

            // Manual cutpoint adder
            ui.horizontal(|ui| {
                ui.label(t!("dialog-binning-cutpoint-val"));
                ui.add(TextEdit::singleline(&mut state.manual_cutpoint_val).desired_width(70.0));
                if ui.button(t!("dialog-binning-add-cutpoint")).clicked() {
                    if let Ok(val) = state.manual_cutpoint_val.trim().parse::<f64>() {
                        if !state.cutpoints.iter().any(|&cp| (cp - val).abs() < 1e-9) {
                            state.cutpoints.push(val);
                            state.sync_labels();
                            state.manual_cutpoint_val.clear();
                        }
                    }
                }
            });

            ui.add_space(4.0);

            // Cutpoint Grid
            GroupBox::new(t!("dialog-binning-cutpoints-table")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_width(320.0);
                    ui.label(egui::RichText::new(t!("dialog-binning-grid-bin")).strong());
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new(t!("dialog-binning-grid-high")).strong());
                    ui.add_space(40.0);
                    ui.label(egui::RichText::new(t!("dialog-binning-grid-label")).strong());
                });

                ui.separator();

                ScrollArea::vertical()
                    .id_salt("binning_grid_scroll")
                    .max_height(100.0)
                    .show(ui, |ui| {
                        let total_bins = state.cutpoints.len() + 1;
                        for bin_idx in 1..=total_bins {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}", bin_idx));
                                ui.add_space(30.0);

                                let high_text = if bin_idx <= state.cutpoints.len() {
                                    let bound = state.cutpoints[bin_idx - 1];
                                    if state.inclusive {
                                        format!("<= {}", format_bound(bound))
                                    } else {
                                        format!("< {}", format_bound(bound))
                                    }
                                } else {
                                    let high_str = t!("dialog-binning-high");
                                    if state.inclusive {
                                        format!("> {}", high_str)
                                    } else {
                                        format!(">= {}", high_str)
                                    }
                                };
                                ui.label(high_text);
                                ui.add_space(30.0);

                                if let Some(lbl) = state.custom_labels.get_mut(bin_idx - 1) {
                                    ui.add(TextEdit::singleline(lbl).desired_width(120.0));
                                }
                            });
                        }
                    });
            });
        });
    });
}
