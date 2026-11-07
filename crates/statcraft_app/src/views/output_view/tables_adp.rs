use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, FontId, ScrollArea, Ui};
use statcraft_algo::{AdpObjective, AdpResult};
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_adp(&mut self, result: AdpResult, sps_log: Option<String>) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps = sps_log.unwrap_or_else(|| "ADP\n  /PREPAREINPUTS.\nEXECUTE.".to_string());

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result
                .partition_counts
                .map(|p| p.total_count)
                .unwrap_or(result.total_inputs),
            active_vars: result.total_inputs,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-adp-title",
            custom_title: None,
            is_expanded: true,
            items: vec![
                OutputItem {
                    id: log_id,
                    title_key: "output-log",
                    custom_title: None,
                    kind: OutputItemKind::Log(sps),
                },
                OutputItem {
                    id: notes_id,
                    title_key: "output-notes",
                    custom_title: None,
                    kind: OutputItemKind::Notes(notes),
                },
                OutputItem {
                    id: item_id,
                    title_key: "output-adp-summary-title",
                    custom_title: None,
                    kind: OutputItemKind::Adp(Box::new(result)),
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

pub fn render_spss_adp_tables(ui: &mut Ui, res: &AdpResult) {
    ScrollArea::horizontal()
        .id_salt("adp_tables_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Table 1: ADP Summary
                render_summary_table(ui, res);
                ui.add_space(16.0);

                // Table 2: Action Log / Processing Details
                render_action_log_table(ui, res);
                ui.add_space(16.0);

                // Table 3: Predictive Power (if calculated)
                if !res.predictive_powers.is_empty() {
                    render_predictive_power_table(ui, res);
                    ui.add_space(16.0);
                }

                // Table 4: Partition Breakdown (if partitioning enabled)
                if let Some(ref part) = res.partition_counts {
                    render_partition_table(ui, part);
                }
            });
        });
}

fn render_summary_table(ui: &mut Ui, res: &AdpResult) {
    ui.label(
        egui::RichText::new(t!("output-adp-summary-title"))
            .font(FontId::proportional(13.0))
            .strong()
            .color(Tokens::text_primary()),
    );
    ui.add_space(4.0);

    let row_height = 20.0_f32;
    let mut cols = vec![
        (t!("output-adp-summary-property"), 190.0_f32),
        (t!("output-adp-summary-value"), 240.0_f32),
    ];
    adjust_widths(ui, &mut cols);
    render_header_row(ui, &cols, 22.0);

    let obj_str = match res.objective {
        AdpObjective::BalanceSpeedAccuracy => t!("output-adp-obj-balance"),
        AdpObjective::OptimizeAccuracy => t!("output-adp-obj-accuracy"),
        AdpObjective::OptimizeSpeed => t!("output-adp-obj-speed"),
        AdpObjective::Custom => t!("output-adp-obj-custom"),
    };

    let target_str = res
        .target_variable
        .clone()
        .unwrap_or_else(|| t!("output-adp-none"));

    let summary_rows = [
        (t!("output-adp-target-label"), target_str),
        (t!("output-adp-objective-label"), obj_str),
        (
            t!("output-adp-total-inputs-label"),
            res.total_inputs.to_string(),
        ),
        (
            t!("output-adp-recommended-label"),
            res.recommended_count.to_string(),
        ),
        (
            t!("output-adp-screened-out-label"),
            res.screened_out_count.to_string(),
        ),
    ];

    for (idx, (prop, val)) in summary_rows.iter().enumerate() {
        let bg = if idx % 2 == 0 {
            Tokens::bg_row_even()
        } else {
            Tokens::bg_row_odd()
        };
        let row_vals = vec![
            (prop.clone(), Align2::LEFT_CENTER),
            (val.clone(), Align2::LEFT_CENTER),
        ];
        render_data_row(ui, &cols, &row_vals, row_height, bg);
    }
}

fn render_action_log_table(ui: &mut Ui, res: &AdpResult) {
    ui.label(
        egui::RichText::new(t!("output-adp-action-log-title"))
            .font(FontId::proportional(13.0))
            .strong()
            .color(Tokens::text_primary()),
    );
    ui.add_space(4.0);

    let row_height = 20.0_f32;
    let mut cols = vec![
        (t!("output-adp-col-field"), 130.0_f32),
        (t!("output-adp-col-status"), 110.0_f32),
        (t!("output-adp-col-transformed"), 130.0_f32),
        (t!("output-adp-col-details"), 280.0_f32),
    ];
    adjust_widths(ui, &mut cols);
    render_header_row(ui, &cols, 22.0);

    for (idx, action) in res.field_actions.iter().enumerate() {
        let bg = if idx % 2 == 0 {
            Tokens::bg_row_even()
        } else {
            Tokens::bg_row_odd()
        };

        let status_str = if action.is_screened_out {
            t!("output-adp-status-excluded")
        } else {
            t!("output-adp-status-recommended")
        };

        let transformed_name = action
            .new_field_name
            .clone()
            .unwrap_or_else(|| "-".to_string());

        let row_vals = vec![
            (action.variable_name.clone(), Align2::LEFT_CENTER),
            (status_str, Align2::CENTER_CENTER),
            (transformed_name, Align2::LEFT_CENTER),
            (action.details.clone(), Align2::LEFT_CENTER),
        ];
        render_data_row(ui, &cols, &row_vals, row_height, bg);
    }
}

fn render_predictive_power_table(ui: &mut Ui, res: &AdpResult) {
    ui.label(
        egui::RichText::new(t!("output-adp-predictive-power-title"))
            .font(FontId::proportional(13.0))
            .strong()
            .color(Tokens::text_primary()),
    );
    ui.add_space(4.0);

    let row_height = 20.0_f32;
    let mut cols = vec![
        (t!("output-adp-col-predictor"), 130.0_f32),
        (t!("output-adp-col-method"), 110.0_f32),
        (t!("output-adp-col-importance"), 150.0_f32),
        (t!("output-adp-col-sig"), 90.0_f32),
    ];
    adjust_widths(ui, &mut cols);
    render_header_row(ui, &cols, 22.0);

    for (idx, pwr) in res.predictive_powers.iter().enumerate() {
        let bg = if idx % 2 == 0 {
            Tokens::bg_row_even()
        } else {
            Tokens::bg_row_odd()
        };

        let imp_text = format!("{:.3}", pwr.score);
        let sig_text = format_sig(pwr.p_value);

        let row_vals = vec![
            (pwr.variable_name.clone(), Align2::LEFT_CENTER),
            (pwr.method.clone(), Align2::CENTER_CENTER),
            (imp_text, Align2::RIGHT_CENTER),
            (sig_text, Align2::RIGHT_CENTER),
        ];
        render_data_row(ui, &cols, &row_vals, row_height, bg);
    }
}

fn render_partition_table(ui: &mut Ui, part: &statcraft_algo::PartitionCounts) {
    ui.label(
        egui::RichText::new(t!("output-adp-partition-title"))
            .font(FontId::proportional(13.0))
            .strong()
            .color(Tokens::text_primary()),
    );
    ui.add_space(4.0);

    let row_height = 20.0_f32;
    let mut cols = vec![
        (t!("output-adp-col-partition"), 150.0_f32),
        (t!("output-adp-col-n"), 90.0_f32),
        (t!("output-adp-col-pct"), 90.0_f32),
    ];
    adjust_widths(ui, &mut cols);
    render_header_row(ui, &cols, 22.0);

    let tot = part.total_count as f64;
    let rows = [
        (
            t!("output-adp-partition-train"),
            part.train_count,
            if tot > 0.0 {
                (part.train_count as f64 / tot) * 100.0
            } else {
                0.0
            },
        ),
        (
            t!("output-adp-partition-test"),
            part.test_count,
            if tot > 0.0 {
                (part.test_count as f64 / tot) * 100.0
            } else {
                0.0
            },
        ),
        (
            t!("output-adp-partition-validate"),
            part.validate_count,
            if tot > 0.0 {
                (part.validate_count as f64 / tot) * 100.0
            } else {
                0.0
            },
        ),
        (t!("output-adp-partition-total"), part.total_count, 100.0),
    ];

    for (idx, (name, n, pct)) in rows.iter().enumerate() {
        let bg = if idx % 2 == 0 {
            Tokens::bg_row_even()
        } else {
            Tokens::bg_row_odd()
        };
        let row_vals = vec![
            (name.clone(), Align2::LEFT_CENTER),
            (n.to_string(), Align2::RIGHT_CENTER),
            (format!("{:.1}%", pct), Align2::RIGHT_CENTER),
        ];
        render_data_row(ui, &cols, &row_vals, row_height, bg);
    }
}

pub fn adp_to_tsv(res: &AdpResult) -> String {
    let mut out = String::new();

    // Summary table
    out.push_str("Property\tValue\n");
    out.push_str(&format!(
        "Target Variable\t{}\n",
        res.target_variable.as_deref().unwrap_or("None")
    ));
    out.push_str(&format!("Total Inputs\t{}\n", res.total_inputs));
    out.push_str(&format!("Recommended\t{}\n", res.recommended_count));
    out.push_str(&format!("Screened Out\t{}\n", res.screened_out_count));
    out.push('\n');

    // Action log table
    out.push_str("Field\tStatus\tTransformed Field\tDetails\n");
    for action in &res.field_actions {
        let status = if action.is_screened_out {
            "Excluded"
        } else {
            "Recommended"
        };
        let trans = action.new_field_name.as_deref().unwrap_or("-");
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            action.variable_name, status, trans, action.details
        ));
    }

    // Predictive power
    if !res.predictive_powers.is_empty() {
        out.push('\n');
        out.push_str("Predictor\tMethod\tImportance\tSig.\n");
        for pwr in &res.predictive_powers {
            out.push_str(&format!(
                "{}\t{}\t{:.4}\t{}\n",
                pwr.variable_name,
                pwr.method,
                pwr.score,
                format_sig(pwr.p_value)
            ));
        }
    }

    out
}
