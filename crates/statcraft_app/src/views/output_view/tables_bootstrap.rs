use super::tables_common::{adjust_widths, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::{
    BootstrapCiMethod, BootstrapOptions, BootstrapSamplingMethod, BootstrapSummary,
};
use statcraft_i18n::t;

pub fn format_ci_level(level: f64) -> String {
    if (level - level.round()).abs() < 1e-9 {
        format!("{}", level.round() as i64)
    } else {
        format!("{:.1}", level)
    }
}

pub fn bootstrap_column_headers(ci_level: f64) -> [String; 4] {
    let level_str = format_ci_level(ci_level);
    [
        t!("output-bootstrap-col-bias"),
        t!("output-bootstrap-col-se"),
        t!("output-bootstrap-col-ci-lower", level = level_str.as_str()),
        t!("output-bootstrap-col-ci-upper", level = level_str.as_str()),
    ]
}

pub fn format_bootstrap_cells(summary: &BootstrapSummary, decimals: usize) -> [String; 4] {
    [
        format!("{:.*}", decimals, summary.bias),
        format!("{:.*}", decimals, summary.std_error),
        format!("{:.*}", decimals, summary.ci.lower),
        format!("{:.*}", decimals, summary.ci.upper),
    ]
}

pub fn render_bootstrap_specifications_table(ui: &mut Ui, opts: &BootstrapOptions) {
    ui.strong(t!("output-bootstrap-specs-title"));
    ui.add_space(6.0_f32);

    let sampling_val = match &opts.sampling_method {
        BootstrapSamplingMethod::Simple => t!("output-bootstrap-sampling-simple"),
        BootstrapSamplingMethod::Stratified { strata_vars } => {
            let joined = strata_vars.join(", ");
            t!(
                "output-bootstrap-sampling-stratified",
                vars = joined.as_str()
            )
        }
    };
    let ci_type_val = match opts.ci_method {
        BootstrapCiMethod::Percentile => t!("output-bootstrap-ci-percentile"),
        BootstrapCiMethod::Bca => t!("output-bootstrap-ci-bca"),
    };
    let seed_val = match opts.seed {
        Some(s) => format!("{}", s),
        None => t!("output-bootstrap-seed-random"),
    };

    let rows = [
        (t!("output-bootstrap-spec-sampling"), sampling_val),
        (
            t!("output-bootstrap-spec-samples"),
            format!("{}", opts.samples),
        ),
        (
            t!("output-bootstrap-spec-ci-level"),
            format!("{}%", format_ci_level(opts.ci_level)),
        ),
        (t!("output-bootstrap-spec-ci-type"), ci_type_val),
        (t!("output-bootstrap-spec-seed"), seed_val),
    ];

    let label_w = rows
        .iter()
        .map(|(k, _)| {
            ui.painter()
                .layout_no_wrap(
                    k.clone(),
                    FontId::proportional(11.5),
                    Tokens::text_primary(),
                )
                .rect
                .width()
                + 16.0
        })
        .fold(180.0_f32, f32::max);
    let val_w = rows
        .iter()
        .map(|(_, v)| {
            ui.painter()
                .layout_no_wrap(
                    v.clone(),
                    FontId::proportional(11.5),
                    Tokens::text_primary(),
                )
                .rect
                .width()
                + 16.0
        })
        .fold(140.0_f32, f32::max);

    let row_height = 22.0_f32;
    for (r_idx, (k, v)) in rows.iter().enumerate() {
        let bg = if r_idx % 2 == 0 {
            Tokens::bg_row_even()
        } else {
            Tokens::bg_row_odd()
        };
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for (text, w, align) in [
                (k, label_w, Align2::LEFT_CENTER),
                (v, val_w, Align2::RIGHT_CENTER),
            ] {
                let (rect, _) = ui.allocate_exact_size(vec2(w, row_height), Sense::hover());
                ui.painter().rect_filled(rect, 0.0_f32, bg);
                ui.painter()
                    .rect_stroke(rect, 0.0_f32, Stroke::new(0.5_f32, Tokens::border()));
                let pos = if align == Align2::LEFT_CENTER {
                    pos2(rect.min.x + 6.0_f32, rect.center().y)
                } else {
                    pos2(rect.max.x - 6.0_f32, rect.center().y)
                };
                ui.painter()
                    .with_clip_rect(rect.shrink2(vec2(4.0, 0.0)))
                    .text(
                        pos,
                        align,
                        text.as_str(),
                        FontId::proportional(11.5),
                        Tokens::text_primary(),
                    );
            }
        });
    }
}

pub fn bootstrap_specs_to_tsv(opts: &BootstrapOptions) -> String {
    let sampling_val = match &opts.sampling_method {
        BootstrapSamplingMethod::Simple => t!("output-bootstrap-sampling-simple"),
        BootstrapSamplingMethod::Stratified { strata_vars } => {
            let joined = strata_vars.join(", ");
            t!(
                "output-bootstrap-sampling-stratified",
                vars = joined.as_str()
            )
        }
    };
    let ci_type_val = match opts.ci_method {
        BootstrapCiMethod::Percentile => t!("output-bootstrap-ci-percentile"),
        BootstrapCiMethod::Bca => t!("output-bootstrap-ci-bca"),
    };
    let seed_val = match opts.seed {
        Some(s) => format!("{}", s),
        None => t!("output-bootstrap-seed-random"),
    };

    let mut lines = vec![t!("output-bootstrap-specs-title")];
    lines.push(format!(
        "{}\t{}",
        t!("output-bootstrap-spec-sampling"),
        sampling_val
    ));
    lines.push(format!(
        "{}\t{}",
        t!("output-bootstrap-spec-samples"),
        opts.samples
    ));
    lines.push(format!(
        "{}\t{}%",
        t!("output-bootstrap-spec-ci-level"),
        format_ci_level(opts.ci_level)
    ));
    lines.push(format!(
        "{}\t{}",
        t!("output-bootstrap-spec-ci-type"),
        ci_type_val
    ));
    lines.push(format!(
        "{}\t{}",
        t!("output-bootstrap-spec-seed"),
        seed_val
    ));
    lines.join("\n")
}

pub fn render_bootstrap_table(
    ui: &mut Ui,
    title: &str,
    id_salt: &str,
    headers: &[String],
    rows: &[Vec<String>],
    left_cols: usize,
) {
    if rows.is_empty() || headers.is_empty() {
        return;
    }
    if !title.is_empty() {
        ui.strong(title);
        ui.add_space(6.0_f32);
    }

    let mut cols: Vec<(String, f32)> = headers.iter().map(|h| (h.clone(), 90.0_f32)).collect();
    adjust_widths(ui, &mut cols);
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if let Some((_, w)) = cols.get_mut(i) {
                let text_w = ui
                    .painter()
                    .layout_no_wrap(
                        cell.clone(),
                        FontId::proportional(11.5),
                        Tokens::text_primary(),
                    )
                    .rect
                    .width()
                    + 16.0;
                *w = (*w).max(text_w);
            }
        }
    }

    let row_height = 22.0_f32;
    ScrollArea::horizontal().id_salt(id_salt).show(ui, |ui| {
        ui.vertical(|ui| {
            render_header_row(ui, &cols, row_height);
            for (r_idx, row) in rows.iter().enumerate() {
                let bg = if r_idx % 2 == 0 {
                    Tokens::bg_row_even()
                } else {
                    Tokens::bg_row_odd()
                };
                let vals: Vec<(String, Align2)> = row
                    .iter()
                    .enumerate()
                    .map(|(i, cell)| {
                        let align = if i < left_cols {
                            Align2::LEFT_CENTER
                        } else {
                            Align2::RIGHT_CENTER
                        };
                        (cell.clone(), align)
                    })
                    .collect();
                render_data_row(ui, &cols, &vals, row_height, bg);
            }
        });
    });
}

pub fn bootstrap_table_to_tsv(title: &str, headers: &[String], rows: &[Vec<String>]) -> String {
    if rows.is_empty() || headers.is_empty() {
        return String::new();
    }
    let mut lines = Vec::with_capacity(rows.len() + 2);
    if !title.is_empty() {
        lines.push(title.to_string());
    }
    lines.push(headers.join("\t"));
    for row in rows {
        lines.push(row.join("\t"));
    }
    lines.join("\n")
}
