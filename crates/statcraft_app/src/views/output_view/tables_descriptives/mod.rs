mod style_report;

pub use style_report::{
    build_descriptives_procedure, build_descriptives_styled_table, descriptives_to_report_table,
};

use super::tables_common::{
    bootstrap_column_headers, bootstrap_specs_to_tsv, bootstrap_table_to_tsv,
    format_bootstrap_cells, render_bootstrap_specifications_table, render_bootstrap_table,
};
use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::{BootstrapSummary, DescriptivesResult};
use statcraft_i18n::t;

#[derive(Clone, Copy)]
pub(super) enum DescCol {
    Variable,
    N,
    Range,
    Min,
    Max,
    Sum,
    Mean,
    SeMean,
    StdDev,
    Variance,
    Skewness,
    SeSkewness,
    Kurtosis,
    SeKurtosis,
}

impl DescCol {
    pub(super) fn header(self) -> String {
        match self {
            Self::Variable => t!("output-col-variable"),
            Self::N => t!("output-col-n"),
            Self::Range => t!("output-col-range"),
            Self::Min => t!("output-col-min"),
            Self::Max => t!("output-col-max"),
            Self::Sum => t!("output-col-sum"),
            Self::Mean => t!("output-col-mean"),
            Self::SeMean => t!("output-col-se-mean"),
            Self::StdDev => t!("output-col-std"),
            Self::Variance => t!("output-col-var"),
            Self::Skewness => t!("output-col-skew"),
            Self::SeSkewness => t!("output-col-se-skew"),
            Self::Kurtosis => t!("output-col-kurt"),
            Self::SeKurtosis => t!("output-col-se-kurt"),
        }
    }

    fn default_width(self) -> f32 {
        match self {
            Self::Variable => 110.0_f32,
            Self::N => 50.0_f32,
            Self::Range | Self::Min | Self::Max | Self::Sum => 72.0_f32,
            Self::Mean | Self::SeMean | Self::StdDev | Self::Variance => 84.0_f32,
            Self::Skewness | Self::SeSkewness | Self::Kurtosis | Self::SeKurtosis => 82.0_f32,
        }
    }

    pub(super) fn format_cell(self, res: &DescriptivesResult, missing: &str) -> String {
        let fmt = |v: Option<f64>| {
            v.map(|x| format!("{:.4}", x))
                .unwrap_or_else(|| missing.to_string())
        };
        match self {
            Self::Variable => res.variable.clone(),
            Self::N => res.n_valid.to_string(),
            Self::Range => fmt(res.range),
            Self::Min => fmt(res.min),
            Self::Max => fmt(res.max),
            Self::Sum => fmt(res.sum),
            Self::Mean => fmt(res.mean),
            Self::SeMean => fmt(res.se_mean),
            Self::StdDev => fmt(res.std_dev),
            Self::Variance => fmt(res.variance),
            Self::Skewness => fmt(res.skewness),
            Self::SeSkewness => fmt(res.se_skewness),
            Self::Kurtosis => fmt(res.kurtosis),
            Self::SeKurtosis => fmt(res.se_kurtosis),
        }
    }
}

pub(super) fn active_columns(results: &[DescriptivesResult]) -> Vec<DescCol> {
    let mut cols = vec![DescCol::Variable, DescCol::N];
    let opts = results.iter().find_map(|r| r.applied_options.as_ref());

    let include = |flag: bool, any_some: bool| opts.map(|_| flag).unwrap_or(any_some);

    if include(
        opts.is_some_and(|o| o.range),
        results.iter().any(|r| r.range.is_some()),
    ) {
        cols.push(DescCol::Range);
    }
    if include(
        opts.is_some_and(|o| o.min),
        results.iter().any(|r| r.min.is_some()),
    ) {
        cols.push(DescCol::Min);
    }
    if include(
        opts.is_some_and(|o| o.max),
        results.iter().any(|r| r.max.is_some()),
    ) {
        cols.push(DescCol::Max);
    }
    if include(
        opts.is_some_and(|o| o.sum),
        results.iter().any(|r| r.sum.is_some()),
    ) {
        cols.push(DescCol::Sum);
    }
    if include(
        opts.is_some_and(|o| o.mean),
        results.iter().any(|r| r.mean.is_some()),
    ) {
        cols.push(DescCol::Mean);
    }
    if include(
        opts.is_some_and(|o| o.se_mean),
        results.iter().any(|r| r.se_mean.is_some()),
    ) {
        cols.push(DescCol::SeMean);
    }
    if include(
        opts.is_some_and(|o| o.std_dev),
        results.iter().any(|r| r.std_dev.is_some()),
    ) {
        cols.push(DescCol::StdDev);
    }
    if include(
        opts.is_some_and(|o| o.variance),
        results.iter().any(|r| r.variance.is_some()),
    ) {
        cols.push(DescCol::Variance);
    }
    if include(
        opts.is_some_and(|o| o.skewness),
        results.iter().any(|r| r.skewness.is_some()),
    ) {
        cols.push(DescCol::Skewness);
        if include(
            opts.is_some_and(|o| o.skewness),
            results.iter().any(|r| r.se_skewness.is_some()),
        ) {
            cols.push(DescCol::SeSkewness);
        }
    }
    if include(
        opts.is_some_and(|o| o.kurtosis),
        results.iter().any(|r| r.kurtosis.is_some()),
    ) {
        cols.push(DescCol::Kurtosis);
        if include(
            opts.is_some_and(|o| o.kurtosis),
            results.iter().any(|r| r.se_kurtosis.is_some()),
        ) {
            cols.push(DescCol::SeKurtosis);
        }
    }

    cols
}

pub(super) fn compute_listwise_n(results: &[DescriptivesResult]) -> usize {
    results
        .iter()
        .find_map(|r| r.listwise_n)
        .unwrap_or_else(|| results.iter().map(|r| r.n_valid).min().unwrap_or(0))
}

fn render_pivot_row(
    ui: &mut Ui,
    cols: &[(String, f32)],
    vals: &[String],
    row_height: f32,
    bg: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (c_idx, val) in vals.iter().enumerate() {
            let width = cols[c_idx].1;
            let (rect, _) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
            ui.painter().rect_filled(rect, 0.0_f32, bg);
            ui.painter()
                .rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::border()));

            let (align, pos) = if c_idx == 0 {
                (Align2::LEFT_CENTER, pos2(rect.min.x + 6.0, rect.center().y))
            } else {
                (
                    Align2::RIGHT_CENTER,
                    pos2(rect.max.x - 6.0, rect.center().y),
                )
            };

            ui.painter().text(
                pos,
                align,
                val,
                FontId::proportional(11.5),
                Tokens::text_primary(),
            );
        }
    });
}

pub fn render_spss_descriptives_pivot_table(ui: &mut Ui, results: &[DescriptivesResult]) {
    if results.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        if let Some(opts) = results
            .iter()
            .find_map(|r| r.bootstrap.as_ref().map(|b| &b.options))
        {
            render_bootstrap_specifications_table(ui, opts);
            ui.add_space(14.0_f32);
        }

        let active = active_columns(results);
        let mut cols: Vec<(String, f32)> = active
            .iter()
            .map(|c| (c.header(), c.default_width()))
            .collect();

        // Dynamic width adjustment (i18n Shield)
        for (col_name, width) in &mut cols {
            let text_w = ui
                .painter()
                .layout_no_wrap(
                    col_name.clone(),
                    FontId::proportional(12.0),
                    Tokens::text_primary(),
                )
                .rect
                .width();
            *width = (*width).max(text_w + 16.0);
        }

        let listwise_label = t!("output-valid-n-listwise");
        let listwise_w = ui
            .painter()
            .layout_no_wrap(
                listwise_label.clone(),
                FontId::proportional(11.5),
                Tokens::text_primary(),
            )
            .rect
            .width();
        if let Some(first) = cols.first_mut() {
            first.1 = first.1.max(listwise_w + 16.0);
        }

        let row_height = 22.0_f32;

        ScrollArea::horizontal()
            .id_salt("descriptives_table_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // 1. Table Header Row
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                        for (col_name, width) in &cols {
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(*width, row_height), Sense::hover());
                            ui.painter().rect_filled(rect, 0.0_f32, Tokens::bg_header());
                            ui.painter().rect_stroke(
                                rect,
                                0.0_f32,
                                Stroke::new(1.0_f32, Tokens::border()),
                            );
                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                col_name,
                                FontId::proportional(12.0),
                                Tokens::text_primary(),
                            );
                        }
                    });

                    // 2. Data Rows
                    for (row_idx, res) in results.iter().enumerate() {
                        let bg = if row_idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let row_vals: Vec<String> =
                            active.iter().map(|c| c.format_cell(res, ".")).collect();
                        render_pivot_row(ui, &cols, &row_vals, row_height, bg);
                    }

                    // 3. Valid N (listwise) Summary Row
                    let listwise_n = compute_listwise_n(results);
                    let mut summary_vals = vec![String::new(); cols.len()];
                    if !summary_vals.is_empty() {
                        summary_vals[0] = listwise_label;
                    }
                    if summary_vals.len() > 1 {
                        summary_vals[1] = listwise_n.to_string();
                    }
                    let summary_bg = if results.len().is_multiple_of(2) {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    render_pivot_row(ui, &cols, &summary_vals, row_height, summary_bg);
                });
            });

        if let Some((headers, rows)) = build_descriptives_bootstrap_rows(results) {
            ui.add_space(14.0_f32);
            let title = format!(
                "{} ({})",
                t!("output-descriptives-title"),
                t!("output-bootstrap-group-header")
            );
            render_bootstrap_table(ui, &title, "desc_boot_table", &headers, &rows, 2);
        }
    });
}

fn build_descriptives_bootstrap_rows(
    results: &[DescriptivesResult],
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let mut rows = Vec::new();
    let mut ci_level = 95.0;

    for res in results {
        let Some(ref boot) = res.bootstrap else {
            continue;
        };
        ci_level = boot.options.ci_level;
        let candidates: [(String, &Option<BootstrapSummary>); 5] = [
            (t!("output-col-mean"), &boot.mean),
            (t!("output-col-std"), &boot.std_dev),
            (t!("output-col-var"), &boot.variance),
            (t!("output-col-skew"), &boot.skewness),
            (t!("output-col-kurt"), &boot.kurtosis),
        ];
        for (stat_name, sum_opt) in candidates {
            if let Some(s) = sum_opt {
                let cells = format_bootstrap_cells(s, 4);
                rows.push(vec![
                    res.variable.clone(),
                    stat_name,
                    format!("{:.4}", s.estimate),
                    cells[0].clone(),
                    cells[1].clone(),
                    cells[2].clone(),
                    cells[3].clone(),
                ]);
            }
        }
    }

    if rows.is_empty() {
        return None;
    }

    let boot_cols = bootstrap_column_headers(ci_level);
    let headers = vec![
        t!("output-col-variable"),
        t!("output-bootstrap-col-statistic"),
        t!("output-km-col-estimate"),
        boot_cols[0].clone(),
        boot_cols[1].clone(),
        boot_cols[2].clone(),
        boot_cols[3].clone(),
    ];
    Some((headers, rows))
}

pub fn descriptives_to_tsv(results: &[DescriptivesResult]) -> String {
    let mut out = String::new();
    if let Some(opts) = results
        .iter()
        .find_map(|r| r.bootstrap.as_ref().map(|b| &b.options))
    {
        out.push_str(&bootstrap_specs_to_tsv(opts));
        out.push('\n');
    }

    let active = active_columns(results);
    let headers: Vec<String> = active.iter().map(|c| c.header()).collect();

    out.push_str(&headers.join("\t"));
    out.push('\n');

    for res in results {
        let row: Vec<String> = active.iter().map(|c| c.format_cell(res, "")).collect();
        out.push_str(&row.join("\t"));
        out.push('\n');
    }

    let listwise_n = compute_listwise_n(results);
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-valid-n-listwise"),
        listwise_n
    ));

    if let Some((headers, rows)) = build_descriptives_bootstrap_rows(results) {
        let title = format!(
            "{} ({})",
            t!("output-descriptives-title"),
            t!("output-bootstrap-group-header")
        );
        out.push_str(&bootstrap_table_to_tsv(&title, &headers, &rows));
        out.push('\n');
    }

    out
}
