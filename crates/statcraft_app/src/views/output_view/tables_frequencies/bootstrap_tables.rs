use crate::theme::Tokens;
pub use crate::views::output_view::tables_common::{
    bootstrap_specs_to_tsv, format_ci_level, render_bootstrap_specifications_table,
};
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Sense, Stroke, Ui};
use statcraft_algo::{BootstrapSummary, FrequencyResult};
use statcraft_i18n::t;

pub fn render_bootstrap_stats_table(ui: &mut Ui, results: &[FrequencyResult]) {
    let mut rows: Vec<(String, String, &BootstrapSummary)> = Vec::new();
    let mut ci_level = 95.0;

    for res in results {
        let Some(ref boot) = res.bootstrap else {
            continue;
        };
        ci_level = boot.options.ci_level;
        let var_name = res.display_label_or_name().to_string();
        let candidates = [
            (t!("output-stats-row-mean"), &boot.stats.mean),
            (t!("output-stats-row-se-mean"), &boot.stats.se_mean),
            (t!("output-stats-row-median"), &boot.stats.median),
            (t!("output-stats-row-std-dev"), &boot.stats.std_dev),
            (t!("output-stats-row-variance"), &boot.stats.variance),
            (t!("output-stats-row-skewness"), &boot.stats.skewness),
            (t!("output-stats-row-kurtosis"), &boot.stats.kurtosis),
        ];
        for (stat_name, summary_opt) in candidates {
            if let Some(summary) = summary_opt {
                rows.push((var_name.clone(), stat_name, summary));
            }
        }
    }

    if rows.is_empty() {
        return;
    }

    ui.strong(format!(
        "{} ({})",
        t!("output-stats-title"),
        t!("output-bootstrap-group-header")
    ));
    ui.add_space(6.0_f32);

    let level_str = format_ci_level(ci_level);
    let headers = [
        t!("output-kmeans-col-variable"),
        t!("output-bootstrap-col-statistic"),
        t!("output-km-col-estimate"),
        t!("output-bootstrap-col-bias"),
        t!("output-bootstrap-col-se"),
        t!("output-bootstrap-col-ci-lower", level = level_str.as_str()),
        t!("output-bootstrap-col-ci-upper", level = level_str.as_str()),
    ];

    let data_rows: Vec<[String; 7]> = rows
        .iter()
        .map(|(var, stat, s)| {
            [
                var.clone(),
                stat.clone(),
                format!("{:.3}", s.estimate),
                format!("{:.3}", s.bias),
                format!("{:.3}", s.std_error),
                format!("{:.3}", s.ci.lower),
                format!("{:.3}", s.ci.upper),
            ]
        })
        .collect();

    render_bootstrap_grid(ui, "freq_boot_stats", &headers, &data_rows, 2);
}

pub fn render_bootstrap_freq_table(ui: &mut Ui, res: &FrequencyResult) {
    let Some(ref boot) = res.bootstrap else {
        return;
    };
    if boot.category_percents.is_empty() || res.categories.is_empty() || res.table_suppressed {
        return;
    }

    let level_str = format_ci_level(boot.options.ci_level);
    let title = res.display_title();
    ui.strong(format!(
        "{} ({})",
        t!("output-frequencies-title", name = title.as_str()),
        t!("output-bootstrap-group-header")
    ));
    ui.add_space(6.0_f32);

    let headers = [
        t!("output-col-value"),
        t!("output-col-percent"),
        t!("output-bootstrap-col-bias"),
        t!("output-bootstrap-col-se"),
        t!("output-bootstrap-col-ci-lower", level = level_str.as_str()),
        t!("output-bootstrap-col-ci-upper", level = level_str.as_str()),
    ];

    let mut data_rows: Vec<[String; 6]> = Vec::new();
    for cat in &res.categories {
        if let Some(s) = boot.category_percents.get(&cat.value) {
            data_rows.push([
                cat.display_label().to_string(),
                format!("{:.1}", s.estimate),
                format!("{:.2}", s.bias),
                format!("{:.2}", s.std_error),
                format!("{:.1}", s.ci.lower),
                format!("{:.1}", s.ci.upper),
            ]);
        }
    }

    if !data_rows.is_empty() {
        render_bootstrap_grid(
            ui,
            &format!("freq_boot_cat_{}", res.variable),
            &headers,
            &data_rows,
            1,
        );
    }
}

fn render_bootstrap_grid<const N: usize>(
    ui: &mut Ui,
    id_salt: &str,
    headers: &[String; N],
    rows: &[[String; N]],
    left_cols: usize,
) {
    let mut widths = [90.0_f32; N];
    for (i, h) in headers.iter().enumerate() {
        let w = ui
            .painter()
            .layout_no_wrap(
                h.clone(),
                FontId::proportional(12.0),
                Tokens::text_primary(),
            )
            .rect
            .width()
            + 16.0;
        widths[i] = widths[i].max(w);
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            let w = ui
                .painter()
                .layout_no_wrap(
                    cell.clone(),
                    FontId::proportional(11.5),
                    Tokens::text_primary(),
                )
                .rect
                .width()
                + 16.0;
            widths[i] = widths[i].max(w);
        }
    }

    let row_height = 22.0_f32;
    ScrollArea::horizontal().id_salt(id_salt).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                for (i, h) in headers.iter().enumerate() {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(widths[i], row_height), Sense::hover());
                    ui.painter().rect_filled(rect, 0.0_f32, Tokens::bg_header());
                    ui.painter()
                        .rect_stroke(rect, 0.0_f32, Stroke::new(1.0_f32, Tokens::border()));
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        h,
                        FontId::proportional(12.0),
                        Tokens::text_primary(),
                    );
                }
            });

            for (r_idx, row) in rows.iter().enumerate() {
                let bg = if r_idx % 2 == 0 {
                    Tokens::bg_row_even()
                } else {
                    Tokens::bg_row_odd()
                };
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    for (i, cell) in row.iter().enumerate() {
                        let (rect, _) =
                            ui.allocate_exact_size(vec2(widths[i], row_height), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0_f32, bg);
                        ui.painter().rect_stroke(
                            rect,
                            0.0_f32,
                            Stroke::new(0.5_f32, Tokens::border()),
                        );
                        let align = if i < left_cols {
                            Align2::LEFT_CENTER
                        } else {
                            Align2::RIGHT_CENTER
                        };
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
                                cell.as_str(),
                                FontId::proportional(11.5),
                                Tokens::text_primary(),
                            );
                    }
                });
            }
        });
    });
}

pub fn bootstrap_stats_to_tsv(results: &[FrequencyResult]) -> String {
    let mut lines = Vec::new();
    for res in results {
        let Some(ref boot) = res.bootstrap else {
            continue;
        };
        let level_str = format_ci_level(boot.options.ci_level);
        if lines.is_empty() {
            lines.push(format!(
                "{} ({})",
                t!("output-stats-title"),
                t!("output-bootstrap-group-header")
            ));
            lines.push(format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                t!("output-kmeans-col-variable"),
                t!("output-bootstrap-col-statistic"),
                t!("output-km-col-estimate"),
                t!("output-bootstrap-col-bias"),
                t!("output-bootstrap-col-se"),
                t!("output-bootstrap-col-ci-lower", level = level_str.as_str()),
                t!("output-bootstrap-col-ci-upper", level = level_str.as_str()),
            ));
        }
        let var_name = res.display_label_or_name();
        let candidates = [
            (t!("output-stats-row-mean"), &boot.stats.mean),
            (t!("output-stats-row-se-mean"), &boot.stats.se_mean),
            (t!("output-stats-row-median"), &boot.stats.median),
            (t!("output-stats-row-std-dev"), &boot.stats.std_dev),
            (t!("output-stats-row-variance"), &boot.stats.variance),
            (t!("output-stats-row-skewness"), &boot.stats.skewness),
            (t!("output-stats-row-kurtosis"), &boot.stats.kurtosis),
        ];
        for (stat_name, summary_opt) in candidates {
            if let Some(s) = summary_opt {
                lines.push(format!(
                    "{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                    var_name, stat_name, s.estimate, s.bias, s.std_error, s.ci.lower, s.ci.upper
                ));
            }
        }
    }
    if lines.len() <= 2 {
        String::new()
    } else {
        lines.join("\n")
    }
}

pub fn bootstrap_freq_to_tsv(res: &FrequencyResult) -> String {
    let Some(ref boot) = res.bootstrap else {
        return String::new();
    };
    if boot.category_percents.is_empty() || res.categories.is_empty() || res.table_suppressed {
        return String::new();
    }
    let level_str = format_ci_level(boot.options.ci_level);
    let title = res.display_title();
    let mut lines = vec![
        format!(
            "{} ({})",
            t!("output-frequencies-title", name = title.as_str()),
            t!("output-bootstrap-group-header")
        ),
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            t!("output-col-value"),
            t!("output-col-percent"),
            t!("output-bootstrap-col-bias"),
            t!("output-bootstrap-col-se"),
            t!("output-bootstrap-col-ci-lower", level = level_str.as_str()),
            t!("output-bootstrap-col-ci-upper", level = level_str.as_str()),
        ),
    ];
    for cat in &res.categories {
        if let Some(s) = boot.category_percents.get(&cat.value) {
            lines.push(format!(
                "{}\t{:.1}\t{:.2}\t{:.2}\t{:.1}\t{:.1}",
                cat.display_label(),
                s.estimate,
                s.bias,
                s.std_error,
                s.ci.lower,
                s.ci.upper
            ));
        }
    }
    if lines.len() <= 2 {
        String::new()
    } else {
        lines.join("\n")
    }
}
