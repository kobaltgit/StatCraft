use super::model::{
    OutputItem, OutputItemKind, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{vec2, Align2, FontId, Ui};
use serde_json::Value;

pub fn render_spss_generic_report(
    ui: &mut Ui,
    command_id: &str,
    status: &str,
    summary: &str,
    key_values: &[(String, String)],
    table_headers: &[String],
    table_rows: &[Vec<String>],
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 4.0);

        // Header Title / Status Bar
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            ui.label(
                egui::RichText::new(command_id)
                    .strong()
                    .font(FontId::proportional(13.0))
                    .color(Tokens::text_primary()),
            );

            if !status.is_empty() {
                let badge_color = if status.eq_ignore_ascii_case("success")
                    || status.eq_ignore_ascii_case("ok")
                {
                    Tokens::accent()
                } else {
                    Tokens::caption_close_hover()
                };
                ui.label(
                    egui::RichText::new(format!("[{}]", status))
                        .font(FontId::proportional(11.0))
                        .color(badge_color),
                );
            }
        });

        if !summary.is_empty() {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(summary)
                    .font(FontId::proportional(12.0))
                    .color(Tokens::text_muted()),
            );
        }

        // Key-Value Metadata Summary Table (if available)
        if !key_values.is_empty() {
            ui.add_space(4.0);
            let mut cols = vec![
                ("Parameter / Metric".to_string(), 140.0_f32),
                ("Value".to_string(), 180.0_f32),
            ];
            adjust_widths(ui, &mut cols);

            render_header_row(ui, &cols, 20.0);
            for (idx, (k, v)) in key_values.iter().enumerate() {
                let bg = if idx % 2 == 0 {
                    Tokens::bg_row_even()
                } else {
                    Tokens::bg_row_odd()
                };
                let row_vals = vec![
                    (k.clone(), Align2::LEFT_CENTER),
                    (v.clone(), Align2::LEFT_CENTER),
                ];
                render_data_row(ui, &cols, &row_vals, 19.0, bg);
            }
        }

        // Tabular Data Table (if available)
        if !table_headers.is_empty() && !table_rows.is_empty() {
            ui.add_space(6.0);
            let mut cols: Vec<(String, f32)> = table_headers
                .iter()
                .map(|h| (h.clone(), 90.0_f32))
                .collect();
            adjust_widths(ui, &mut cols);

            render_header_row(ui, &cols, 22.0);
            for (idx, row) in table_rows.iter().enumerate() {
                let bg = if idx % 2 == 0 {
                    Tokens::bg_row_even()
                } else {
                    Tokens::bg_row_odd()
                };
                let row_vals: Vec<(String, Align2)> = row
                    .iter()
                    .enumerate()
                    .map(|(c_idx, val)| {
                        let align = if c_idx == 0 {
                            Align2::LEFT_CENTER
                        } else {
                            Align2::RIGHT_CENTER
                        };
                        (val.clone(), align)
                    })
                    .collect();
                render_data_row(ui, &cols, &row_vals, 20.0, bg);
            }
        }
    });
}

pub fn generic_report_to_tsv(
    command_id: &str,
    status: &str,
    summary: &str,
    key_values: &[(String, String)],
    table_headers: &[String],
    table_rows: &[Vec<String>],
) -> String {
    let mut out = String::new();

    if !table_headers.is_empty() && !table_rows.is_empty() {
        out.push_str(&table_headers.join("\t"));
        out.push('\n');
        for row in table_rows {
            out.push_str(&row.join("\t"));
            out.push('\n');
        }
        return out;
    }

    if !key_values.is_empty() {
        out.push_str("Key\tValue\n");
        for (k, v) in key_values {
            out.push_str(&format!("{}\t{}\n", k, v));
        }
        return out;
    }

    format!(
        "Command\t{}\nStatus\t{}\nSummary\t{}\n",
        command_id, status, summary
    )
}

impl OutputViewerState {
    pub fn add_generic_result(&mut self, command_id: &str, raw_log: &str, val: &Value) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let item_id = self.alloc_id();

        let status = val
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("success")
            .to_string();

        let summary = val
            .get("message")
            .or_else(|| val.get("summary"))
            .or_else(|| val.get("output"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut key_values = Vec::new();
        let mut table_headers = Vec::new();
        let mut table_rows = Vec::new();

        if let Value::Object(map) = val {
            for (k, v) in map {
                if k == "status" || k == "message" || k == "summary" || k == "output" {
                    continue;
                }
                match v {
                    Value::String(s) => key_values.push((k.clone(), s.clone())),
                    Value::Number(n) => key_values.push((k.clone(), n.to_string())),
                    Value::Bool(b) => key_values.push((k.clone(), b.to_string())),
                    Value::Array(arr) if table_headers.is_empty() => {
                        // Check if it's an array of objects -> render as tabular data
                        if let Some(Value::Object(first_obj)) = arr.first() {
                            table_headers = first_obj.keys().cloned().collect();
                            for item in arr {
                                if let Value::Object(row_obj) = item {
                                    let row = table_headers
                                        .iter()
                                        .map(|h| {
                                            row_obj
                                                .get(h)
                                                .map(format_json_value)
                                                .unwrap_or_default()
                                        })
                                        .collect();
                                    table_rows.push(row);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else if let Value::Array(arr) = val {
            if let Some(Value::Object(first_obj)) = arr.first() {
                table_headers = first_obj.keys().cloned().collect();
                for item in arr {
                    if let Value::Object(row_obj) = item {
                        let row = table_headers
                            .iter()
                            .map(|h| row_obj.get(h).map(format_json_value).unwrap_or_default())
                            .collect();
                        table_rows.push(row);
                    }
                }
            }
        }

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-generic-title",
            custom_title: Some(command_id.to_string()),
            is_expanded: true,
            items: vec![
                OutputItem {
                    id: log_id,
                    title_key: "output-log",
                    custom_title: None,
                    kind: OutputItemKind::Log(raw_log.to_string()),
                },
                OutputItem {
                    id: item_id,
                    title_key: "output-table",
                    custom_title: Some(format!("Report: {}", command_id)),
                    kind: OutputItemKind::GenericReport {
                        command_id: command_id.to_string(),
                        status,
                        summary,
                        key_values,
                        table_headers,
                        table_rows,
                    },
                },
            ],
        };

        self.procedures.push(proc);
        let new_proc_idx = self.procedures.len() - 1;
        self.selected_target = Some(SelectedTarget::Item {
            proc_idx: new_proc_idx,
            item_idx: 1,
        });
        self.scroll_to_target = self.selected_target;
    }
}

fn format_json_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                format!("{:.4}", f)
            } else {
                n.to_string()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => "-".to_string(),
        other => other.to_string(),
    }
}
