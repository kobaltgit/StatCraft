use super::super::tables_common::{adjust_widths, format_opt, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, FontId, Ui};
use statcraft_algo::{CrosstabCell, CrosstabLayerKey, CrosstabResult};
use statcraft_i18n::t;

pub fn format_layer_banner(layer_keys: &[CrosstabLayerKey]) -> Option<String> {
    if layer_keys.is_empty() {
        return None;
    }
    let parts: Vec<String> = layer_keys
        .iter()
        .map(|k| {
            let val_disp = k
                .label
                .as_ref()
                .filter(|l| !l.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| k.value.clone());
            format!("{} = {}", k.var_name, val_disp)
        })
        .collect();
    Some(parts.join(" | "))
}

pub fn format_observed_cell(cell: &CrosstabCell, res: &CrosstabResult) -> String {
    let base = if res.options.hide_small_counts
        && (cell.suppressed || cell.observed < res.options.hide_small_threshold)
    {
        format!("<{}", res.options.hide_small_threshold)
    } else {
        cell.observed.to_string()
    };

    if res.options.compare_col_proportions {
        if let Some(ref grp) = cell.col_prop_group {
            if !grp.is_empty() {
                return format!("{} ({})", base, grp);
            }
        }
    }
    base
}

pub fn render_case_processing_summary(ui: &mut Ui, res: &CrosstabResult, selected: bool) {
    let row_height = 20.0_f32;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(t!("output-crosstabs-summary-title"))
                .font(FontId::proportional(12.5))
                .strong()
                .color(if selected {
                    Tokens::accent()
                } else {
                    Tokens::text_primary()
                }),
        );
        if let Some(banner) = format_layer_banner(&res.layer_keys) {
            ui.label(
                egui::RichText::new(format!("[{}: {}]", t!("output-crosstabs-layer"), banner))
                    .font(FontId::proportional(11.5))
                    .color(Tokens::text_muted()),
            );
        }
    });
    ui.add_space(2.0);

    let mut summary_cols = vec![
        (format!("{} * {}", res.row_var, res.col_var), 180.0_f32),
        (format!("{} N", t!("output-crosstabs-valid")), 80.0_f32),
        (format!("{} %", t!("output-crosstabs-valid")), 80.0_f32),
        (format!("{} N", t!("output-crosstabs-missing")), 80.0_f32),
        (format!("{} %", t!("output-crosstabs-missing")), 80.0_f32),
        (format!("{} N", t!("output-crosstabs-total")), 80.0_f32),
        (format!("{} %", t!("output-crosstabs-total")), 80.0_f32),
    ];
    adjust_widths(ui, &mut summary_cols);
    render_header_row(ui, &summary_cols, row_height);

    let n_tot = res.n_valid + res.n_missing;
    let v_pct = if n_tot > 0 {
        (res.n_valid as f64 / n_tot as f64) * 100.0
    } else {
        0.0
    };
    let m_pct = if n_tot > 0 {
        (res.n_missing as f64 / n_tot as f64) * 100.0
    } else {
        0.0
    };

    let summary_vals = vec![
        (
            format!("{} * {}", res.row_var, res.col_var),
            Align2::LEFT_CENTER,
        ),
        (res.n_valid.to_string(), Align2::RIGHT_CENTER),
        (format!("{:.1}%", v_pct), Align2::RIGHT_CENTER),
        (res.n_missing.to_string(), Align2::RIGHT_CENTER),
        (format!("{:.1}%", m_pct), Align2::RIGHT_CENTER),
        (n_tot.to_string(), Align2::RIGHT_CENTER),
        ("100.0%".to_string(), Align2::RIGHT_CENTER),
    ];
    render_data_row(
        ui,
        &summary_cols,
        &summary_vals,
        row_height,
        Tokens::bg_row_even(),
    );
}

pub fn render_crosstab_matrix(ui: &mut Ui, res: &CrosstabResult, selected: bool) {
    let row_height = if res.options.apa_style {
        18.0_f32
    } else {
        20.0_f32
    };

    let base_title = t!(
        "output-crosstabs-table-title",
        row = res.row_var.as_str(),
        col = res.col_var.as_str()
    );
    let title = if let Some(banner) = format_layer_banner(&res.layer_keys) {
        format!("{} ({})", base_title, banner)
    } else {
        base_title
    };

    ui.label(
        egui::RichText::new(title)
            .font(FontId::proportional(12.5))
            .strong()
            .color(if selected {
                Tokens::accent()
            } else {
                Tokens::text_primary()
            }),
    );
    ui.add_space(2.0);

    let mut crosstab_cols = vec![
        (res.row_var.clone(), 120.0_f32),
        ("".to_string(), 150.0_f32),
    ];
    for col_cat in &res.col_categories {
        let col_hdr = col_cat
            .label
            .as_ref()
            .cloned()
            .unwrap_or_else(|| col_cat.value.clone());
        crosstab_cols.push((col_hdr, 90.0_f32));
    }
    crosstab_cols.push((t!("output-crosstabs-total"), 90.0_f32));
    adjust_widths(ui, &mut crosstab_cols);
    render_header_row(ui, &crosstab_cols, row_height);

    let mut row_flip = false;
    let mut has_ztest_letters = false;

    for row in &res.rows {
        let row_name = row
            .category
            .label
            .as_ref()
            .cloned()
            .unwrap_or_else(|| row.category.value.clone());

        let bg = if res.options.apa_style {
            Tokens::bg_panel()
        } else if row_flip {
            Tokens::bg_row_odd()
        } else {
            Tokens::bg_row_even()
        };

        // 1. Observed Count
        if res.options.observed {
            let mut c_vals = vec![
                (row_name.clone(), Align2::LEFT_CENTER),
                (t!("output-crosstabs-count"), Align2::LEFT_CENTER),
            ];
            for cell in &row.cells {
                if res.options.compare_col_proportions
                    && cell
                        .col_prop_group
                        .as_ref()
                        .map(|g| !g.is_empty())
                        .unwrap_or(false)
                {
                    has_ztest_letters = true;
                }
                c_vals.push((format_observed_cell(cell, res), Align2::RIGHT_CENTER));
            }
            c_vals.push((row.row_total.to_string(), Align2::RIGHT_CENTER));
            render_data_row(ui, &crosstab_cols, &c_vals, row_height, bg);
        }

        // 2. Expected Count
        if res.options.expected {
            let mut e_vals = vec![
                ("".to_string(), Align2::LEFT_CENTER),
                (t!("output-crosstabs-expected-count"), Align2::LEFT_CENTER),
            ];
            for cell in &row.cells {
                e_vals.push((format_opt(cell.expected), Align2::RIGHT_CENTER));
            }
            e_vals.push((format!("{:.1}", row.row_total as f64), Align2::RIGHT_CENTER));
            render_data_row(ui, &crosstab_cols, &e_vals, row_height, bg);
        }

        // 3. % within Row
        if res.options.row_pct {
            let mut r_vals = vec![
                ("".to_string(), Align2::LEFT_CENTER),
                (
                    t!(
                        "output-crosstabs-pct-within-row",
                        var = res.row_var.as_str()
                    ),
                    Align2::LEFT_CENTER,
                ),
            ];
            for cell in &row.cells {
                r_vals.push((
                    format!("{:.1}%", cell.row_pct.unwrap_or(0.0)),
                    Align2::RIGHT_CENTER,
                ));
            }
            r_vals.push(("100.0%".to_string(), Align2::RIGHT_CENTER));
            render_data_row(ui, &crosstab_cols, &r_vals, row_height, bg);
        }

        // 4. % within Column
        if res.options.col_pct {
            let mut col_vals = vec![
                ("".to_string(), Align2::LEFT_CENTER),
                (
                    t!(
                        "output-crosstabs-pct-within-col",
                        var = res.col_var.as_str()
                    ),
                    Align2::LEFT_CENTER,
                ),
            ];
            for cell in &row.cells {
                col_vals.push((
                    format!("{:.1}%", cell.col_pct.unwrap_or(0.0)),
                    Align2::RIGHT_CENTER,
                ));
            }
            let tot_pct = if res.grand_total > 0 {
                (row.row_total as f64 / res.grand_total as f64) * 100.0
            } else {
                0.0
            };
            col_vals.push((format!("{:.1}%", tot_pct), Align2::RIGHT_CENTER));
            render_data_row(ui, &crosstab_cols, &col_vals, row_height, bg);
        }

        // 5. % of Total
        if res.options.total_pct {
            let mut tot_vals = vec![
                ("".to_string(), Align2::LEFT_CENTER),
                (t!("output-crosstabs-pct-of-total"), Align2::LEFT_CENTER),
            ];
            for cell in &row.cells {
                tot_vals.push((
                    format!("{:.1}%", cell.total_pct.unwrap_or(0.0)),
                    Align2::RIGHT_CENTER,
                ));
            }
            let tot_pct = if res.grand_total > 0 {
                (row.row_total as f64 / res.grand_total as f64) * 100.0
            } else {
                0.0
            };
            tot_vals.push((format!("{:.1}%", tot_pct), Align2::RIGHT_CENTER));
            render_data_row(ui, &crosstab_cols, &tot_vals, row_height, bg);
        }

        // 6. Unstandardized Residual
        if res.options.residuals || res.options.resid_unstandardized {
            let mut res_vals = vec![
                ("".to_string(), Align2::LEFT_CENTER),
                (t!("output-crosstabs-residual"), Align2::LEFT_CENTER),
            ];
            for cell in &row.cells {
                res_vals.push((format_opt(cell.residual), Align2::RIGHT_CENTER));
            }
            res_vals.push(("0.0".to_string(), Align2::RIGHT_CENTER));
            render_data_row(ui, &crosstab_cols, &res_vals, row_height, bg);
        }

        // 7. Standardized Residual
        if res.options.resid_standardized {
            let mut std_vals = vec![
                ("".to_string(), Align2::LEFT_CENTER),
                (t!("output-crosstabs-std-residual"), Align2::LEFT_CENTER),
            ];
            for cell in &row.cells {
                std_vals.push((format_opt(cell.std_residual), Align2::RIGHT_CENTER));
            }
            std_vals.push(("".to_string(), Align2::RIGHT_CENTER));
            render_data_row(ui, &crosstab_cols, &std_vals, row_height, bg);
        }

        // 8. Adjusted Standardized Residual
        if res.options.resid_adjusted {
            let mut adj_vals = vec![
                ("".to_string(), Align2::LEFT_CENTER),
                (t!("output-crosstabs-adj-residual"), Align2::LEFT_CENTER),
            ];
            for cell in &row.cells {
                adj_vals.push((format_opt(cell.adj_residual), Align2::RIGHT_CENTER));
            }
            adj_vals.push(("".to_string(), Align2::RIGHT_CENTER));
            render_data_row(ui, &crosstab_cols, &adj_vals, row_height, bg);
        }

        row_flip = !row_flip;
    }

    // Summary Total Row
    let bg = if res.options.apa_style {
        Tokens::bg_panel()
    } else if row_flip {
        Tokens::bg_row_odd()
    } else {
        Tokens::bg_row_even()
    };
    let mut tot_vals = vec![
        (t!("output-crosstabs-total"), Align2::LEFT_CENTER),
        (t!("output-crosstabs-count"), Align2::LEFT_CENTER),
    ];
    for ct in &res.col_totals {
        tot_vals.push((ct.to_string(), Align2::RIGHT_CENTER));
    }
    tot_vals.push((res.grand_total.to_string(), Align2::RIGHT_CENTER));
    render_data_row(ui, &crosstab_cols, &tot_vals, row_height, bg);

    if has_ztest_letters {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(t!(
                "output-crosstabs-ztest-footnote",
                var = res.col_var.as_str()
            ))
            .italics()
            .font(FontId::proportional(11.0))
            .color(Tokens::text_muted()),
        );
    }
}
