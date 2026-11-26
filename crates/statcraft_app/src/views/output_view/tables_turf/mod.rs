pub mod charts;
pub mod export;

pub use export::{turf_to_report_tables, turf_to_tsv};

use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths_with_rows, render_data_row, render_header_row};
use crate::theme::Tokens;
use charts::render_reach_and_frequency_chart;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::{TurfOptions, TurfResult};
use statcraft_i18n::t;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TurfOutputItem {
    pub result: TurfResult,
    pub options: TurfOptions,
    #[serde(default)]
    pub var_labels: std::collections::HashMap<String, String>,
}

pub(super) fn display_var_name(item: &TurfOutputItem, name: &str) -> String {
    item.var_labels
        .get(name)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| name.to_string())
}

impl OutputViewerState {
    pub fn add_turf(&mut self, result: TurfResult, options: TurfOptions, sps_log: String) {
        self.add_turf_with_dataset(result, options, sps_log, None);
    }

    pub fn add_turf_with_dataset(
        &mut self,
        result: TurfResult,
        options: TurfOptions,
        sps_log: String,
        dataset: Option<&statcraft_core::DataSet>,
    ) {
        let mut var_labels = std::collections::HashMap::new();
        if let Some(ds) = dataset {
            for name in &options.variables {
                if let Some(meta) = ds.dictionary.find_by_name(name) {
                    let lbl = meta.label.trim();
                    if !lbl.is_empty() {
                        var_labels.insert(name.clone(), lbl.to_string());
                    }
                }
            }
        }

        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let notes = OutputNotes {
            dataset_name: self.active_dataset_name.clone(),
            timestamp: "Session".to_string(),
            active_cases: result.total_cases_unweighted,
            active_vars: options.variables.len(),
        };

        let item = TurfOutputItem {
            result,
            options,
            var_labels,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-turf-title",
            custom_title: None,
            dataset_name: Some(self.active_dataset_name.clone()),
            is_expanded: true,
            items: vec![
                OutputItem {
                    id: log_id,
                    title_key: "output-log",
                    custom_title: None,
                    kind: OutputItemKind::Log(sps_log),
                },
                OutputItem {
                    id: notes_id,
                    title_key: "output-notes",
                    custom_title: None,
                    kind: OutputItemKind::Notes(notes),
                },
                OutputItem {
                    id: item_id,
                    title_key: "output-turf-title",
                    custom_title: None,
                    kind: OutputItemKind::Turf(Box::new(item)),
                },
            ],
        };

        self.procedures.push(proc);
        let new_proc_idx = self.procedures.len() - 1;
        self.selected_target = Some(SelectedTarget::Item {
            proc_idx: new_proc_idx,
            item_idx: 2,
        });
        self.scroll_to_target = self.selected_target;
    }
}

pub fn render_spss_turf(ui: &mut Ui, item: &TurfOutputItem) {
    ui.vertical(|ui| {
        render_turf_summary_table(ui, item);
        ui.add_space(12.0);
        render_turf_cycles_tables(ui, item);
        render_reach_and_frequency_chart(ui, item);
    });
}

fn render_turf_summary_table(ui: &mut Ui, item: &TurfOutputItem) {
    ui.strong(t!("output-turf-summary-title"));
    ui.add_space(4.0);

    let res = &item.result;
    let opts = &item.options;

    let excl_str = if res.excluded_variables.is_empty() {
        t!("output-turf-param-none")
    } else {
        res.excluded_variables
            .iter()
            .map(|(v, p)| format!("{} ({p:.1}%)", display_var_name(item, v)))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let heats_str = if opts.use_advanced_selection {
        t!(
            "output-turf-heats-detail",
            yes = t!("output-turf-param-yes"),
            size = opts.advanced_size,
            winners = opts.advanced_winners
        )
    } else {
        t!("output-turf-param-no")
    };

    let ops_str = res
        .cycles
        .first()
        .map(|c| c.estimated_operations.to_string())
        .unwrap_or_else(|| "0".to_string());

    let rows: Vec<Vec<String>> = vec![
        vec![
            t!("output-turf-param-valid-n"),
            format!("{:.0}", res.total_valid_cases),
        ],
        vec![
            t!("output-turf-param-positive-vals"),
            opts.positive_values.join(", "),
        ],
        vec![t!("output-turf-param-excluded-vars"), excl_str],
        vec![t!("output-turf-param-est-ops"), ops_str],
        vec![t!("output-turf-param-advanced-heats"), heats_str],
        vec![
            t!("output-turf-param-always-required"),
            opts.always_required.to_string(),
        ],
        vec![
            t!("output-turf-param-removal-cycles"),
            opts.removal_cycles.to_string(),
        ],
    ];

    let mut cols = vec![
        (t!("output-turf-col-param"), 200.0_f32),
        (t!("output-turf-col-value"), 260.0_f32),
    ];
    adjust_widths_with_rows(ui, &mut cols, &rows);

    ScrollArea::horizontal()
        .id_salt("turf_summary_scroll")
        .show(ui, |ui| {
            render_header_row(ui, &cols, 22.0);
            for (idx, r) in rows.iter().enumerate() {
                let bg = if idx % 2 == 0 {
                    Tokens::bg_row_even()
                } else {
                    Tokens::bg_row_odd()
                };
                let cells = vec![
                    (r.first().cloned().unwrap_or_default(), Align2::LEFT_CENTER),
                    (r.get(1).cloned().unwrap_or_default(), Align2::LEFT_CENTER),
                ];
                render_data_row(ui, &cols, &cells, 20.0, bg);
            }
        });
}

fn render_turf_cycles_tables(ui: &mut Ui, item: &TurfOutputItem) {
    let res = &item.result;
    let opts = &item.options;
    let has_weights = !opts.importance_weights.is_empty();

    for cycle in &res.cycles {
        ui.add_space(8.0);
        if cycle.cycle_index > 0 {
            let removed = cycle
                .removed_variables
                .iter()
                .map(|v| display_var_name(item, v))
                .collect::<Vec<_>>()
                .join(", ");
            ui.strong(format!(
                "{} ({}: {})",
                t!("output-turf-reach-freq-title"),
                t!("output-turf-reach-freq-cycle", cycle = cycle.cycle_index),
                removed
            ));
        } else {
            ui.strong(t!("output-turf-reach-freq-title"));
        }
        ui.add_space(4.0);

        let mut cols = vec![
            (t!("output-turf-col-combo-size"), 48.0_f32),
            (t!("output-turf-col-variables"), 200.0_f32),
            (t!("output-turf-col-reach"), 65.0_f32),
            (t!("output-turf-col-reach-pct"), 75.0_f32),
            (t!("output-turf-col-frequency"), 70.0_f32),
            (t!("output-turf-col-frequency-pct"), 85.0_f32),
            (t!("output-turf-col-mean-responses"), 90.0_f32),
        ];
        if has_weights {
            cols.push((t!("output-turf-col-score"), 85.0_f32));
        }

        let mut data_rows: Vec<Vec<String>> = Vec::new();
        for (size, rows) in &cycle.rows_by_size {
            for row in rows {
                let vars_display = row
                    .variables
                    .iter()
                    .map(|v| display_var_name(item, v))
                    .collect::<Vec<_>>()
                    .join(" + ");
                let mut r = vec![
                    size.to_string(),
                    vars_display,
                    format!("{:.0}", row.reach_count),
                    format!("{:.1}%", row.reach_pct),
                    format!("{:.0}", row.frequency_count),
                    format!("{:.1}%", row.frequency_pct),
                    format!("{:.2}", row.mean_responses_per_reached),
                ];
                if has_weights {
                    let score_str = row
                        .weighted_score
                        .map(|s| format!("{s:.4}"))
                        .unwrap_or_else(|| "-".to_string());
                    r.push(score_str);
                }
                data_rows.push(r);
            }
        }

        adjust_widths_with_rows(ui, &mut cols, &data_rows);

        ScrollArea::horizontal()
            .id_salt(format!("turf_cycle_{}_scroll", cycle.cycle_index))
            .show(ui, |ui| {
                render_header_row(ui, &cols, 22.0);
                for (idx, r) in data_rows.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };

                    let mut cells = vec![
                        (r.first().cloned().unwrap_or_default(), Align2::RIGHT_CENTER),
                        (r.get(1).cloned().unwrap_or_default(), Align2::LEFT_CENTER),
                        (r.get(2).cloned().unwrap_or_default(), Align2::RIGHT_CENTER),
                        (r.get(3).cloned().unwrap_or_default(), Align2::RIGHT_CENTER),
                        (r.get(4).cloned().unwrap_or_default(), Align2::RIGHT_CENTER),
                        (r.get(5).cloned().unwrap_or_default(), Align2::RIGHT_CENTER),
                        (r.get(6).cloned().unwrap_or_default(), Align2::RIGHT_CENTER),
                    ];
                    if has_weights {
                        cells.push((r.get(7).cloned().unwrap_or_default(), Align2::RIGHT_CENTER));
                    }

                    render_data_row(ui, &cols, &cells, 20.0, bg);
                }
            });
    }
}
