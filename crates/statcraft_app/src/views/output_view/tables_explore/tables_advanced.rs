use super::tables_summary::{fmt_opt, render_data_row, render_header_row};
use crate::theme::Tokens;
use crate::views::output_view::tables_bootstrap::{
    bootstrap_column_headers, format_bootstrap_cells, render_bootstrap_specifications_table,
    render_bootstrap_table,
};
use egui::{FontId, ScrollArea, Ui};
use statcraft_algo::{ExploreResult, LeveneBasis};
use statcraft_i18n::t;

pub(crate) fn render_m_estimators_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-m-estimators"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let cols = vec![
        (t!("output-col-variable"), 140.0_f32),
        (t!("output-explore-huber"), 115.0_f32),
        (t!("output-explore-tukey-biweight"), 115.0_f32),
        (t!("output-explore-hampel"), 115.0_f32),
        (t!("output-explore-andrews"), 115.0_f32),
    ];

    ScrollArea::horizontal()
        .id_salt("explore_m_est_scroll")
        .show(ui, |ui| {
            render_header_row(ui, &cols);
            let mut row_idx = 0;
            for var_res in &res.variables {
                for group in &var_res.groups {
                    if let Some(m) = &group.m_estimators {
                        let var_title = match &group.factor_level {
                            Some(fv) => format!("{} ({})", var_res.variable, fv),
                            None => var_res.variable.clone(),
                        };
                        let vals = vec![
                            var_title,
                            fmt_opt(m.huber),
                            fmt_opt(m.tukey),
                            fmt_opt(m.hampel),
                            fmt_opt(m.andrews),
                        ];
                        render_data_row(ui, &cols, &vals, row_idx % 2 == 1);
                        row_idx += 1;
                    }
                }
            }
        });
}

pub(crate) fn render_percentiles_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-percentiles"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let cols = vec![
        (t!("output-col-variable"), 140.0_f32),
        (t!("output-col-statistic"), 195.0_f32),
        ("5".to_string(), 65.0_f32),
        ("10".to_string(), 65.0_f32),
        ("25".to_string(), 65.0_f32),
        ("50".to_string(), 65.0_f32),
        ("75".to_string(), 65.0_f32),
        ("90".to_string(), 65.0_f32),
        ("95".to_string(), 65.0_f32),
    ];

    ScrollArea::horizontal()
        .id_salt("explore_percentiles_scroll")
        .show(ui, |ui| {
            render_header_row(ui, &cols);
            let mut row_idx = 0;
            for var_res in &res.variables {
                for group in &var_res.groups {
                    if let Some(p) = &group.percentiles {
                        let var_title = match &group.factor_level {
                            Some(fv) => format!("{} ({})", var_res.variable, fv),
                            None => var_res.variable.clone(),
                        };
                        let wavg_vals = vec![
                            var_title,
                            t!("output-explore-pct-wavg"),
                            fmt_opt(p.p5),
                            fmt_opt(p.p10),
                            fmt_opt(p.p25),
                            fmt_opt(p.p50),
                            fmt_opt(p.p75),
                            fmt_opt(p.p90),
                            fmt_opt(p.p95),
                        ];
                        render_data_row(ui, &cols, &wavg_vals, row_idx % 2 == 1);
                        row_idx += 1;

                        let tukey_vals = vec![
                            String::new(),
                            t!("output-explore-pct-tukey"),
                            String::new(),
                            String::new(),
                            fmt_opt(p.tukey_q1),
                            fmt_opt(p.tukey_q2),
                            fmt_opt(p.tukey_q3),
                            String::new(),
                            String::new(),
                        ];
                        render_data_row(ui, &cols, &tukey_vals, row_idx % 2 == 1);
                        row_idx += 1;
                    }
                }
            }
        });
}

pub(crate) fn render_normality_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-normality"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let cols = vec![
        (t!("output-col-variable"), 140.0_f32),
        (
            format!(
                "{} {}",
                t!("output-explore-kolmogorov-smirnov"),
                t!("output-explore-statistic")
            ),
            100.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-kolmogorov-smirnov"),
                t!("output-explore-df")
            ),
            65.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-kolmogorov-smirnov"),
                t!("output-explore-sig")
            ),
            70.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-shapiro-wilk"),
                t!("output-explore-statistic")
            ),
            95.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-shapiro-wilk"),
                t!("output-explore-df")
            ),
            60.0_f32,
        ),
        (
            format!(
                "{} {}",
                t!("output-explore-shapiro-wilk"),
                t!("output-explore-sig")
            ),
            65.0_f32,
        ),
    ];

    ScrollArea::horizontal()
        .id_salt("explore_normality_scroll")
        .show(ui, |ui| {
            render_header_row(ui, &cols);
            let mut row_idx = 0;
            for var_res in &res.variables {
                for group in &var_res.groups {
                    if let Some(norm) = &group.normality {
                        let var_title = match &group.factor_level {
                            Some(fv) => format!("{} ({})", var_res.variable, fv),
                            None => var_res.variable.clone(),
                        };
                        let (ks_stat, ks_df, ks_sig) = match &norm.kolmogorov_smirnov {
                            Some(ks) => {
                                let sig_str = if ks.sig >= 0.20 {
                                    ".200*".to_string()
                                } else {
                                    format!("{:.3}", ks.sig)
                                };
                                (format!("{:.3}", ks.statistic), ks.df.to_string(), sig_str)
                            }
                            None => (".".to_string(), ".".to_string(), ".".to_string()),
                        };
                        let (sw_stat, sw_df, sw_sig) = match &norm.shapiro_wilk {
                            Some(sw) => (
                                format!("{:.3}", sw.statistic),
                                sw.df.to_string(),
                                format!("{:.3}", sw.sig),
                            ),
                            None => (".".to_string(), ".".to_string(), ".".to_string()),
                        };
                        let vals = vec![var_title, ks_stat, ks_df, ks_sig, sw_stat, sw_df, sw_sig];
                        render_data_row(ui, &cols, &vals, row_idx % 2 == 1);
                        row_idx += 1;
                    }
                }
            }
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(t!("output-explore-lilliefors-footnote"))
                    .font(FontId::proportional(11.0))
                    .color(Tokens::text_muted()),
            );
            ui.label(
                egui::RichText::new(t!("output-explore-lilliefors-correction"))
                    .font(FontId::proportional(11.0))
                    .color(Tokens::text_muted()),
            );
        });
}

pub(crate) fn render_levene_table(ui: &mut Ui, res: &ExploreResult) {
    ui.label(
        egui::RichText::new(t!("output-explore-homogeneity-title"))
            .font(FontId::proportional(13.0))
            .color(Tokens::text_primary()),
    );

    let cols = vec![
        (t!("output-col-variable"), 120.0_f32),
        (t!("output-explore-levene-basis"), 210.0_f32),
        (t!("output-explore-levene-stat"), 105.0_f32),
        (t!("output-explore-df1"), 55.0_f32),
        (t!("output-explore-df2"), 65.0_f32),
        (t!("output-explore-sig"), 65.0_f32),
    ];

    ScrollArea::horizontal()
        .id_salt("explore_levene_scroll")
        .show(ui, |ui| {
            render_header_row(ui, &cols);
            let mut row_idx = 0;
            for var_res in &res.variables {
                if let Some(lev) = &var_res.levene_test {
                    for (i, row) in lev.rows.iter().enumerate() {
                        let lead = if i == 0 {
                            lev.variable.clone()
                        } else {
                            String::new()
                        };
                        let basis_lbl = match row.basis {
                            LeveneBasis::BasedOnMean => t!("output-explore-levene-mean"),
                            LeveneBasis::BasedOnMedian => t!("output-explore-levene-median"),
                            LeveneBasis::BasedOnMedianAdjustedDf => {
                                t!("output-explore-levene-median-adj")
                            }
                            LeveneBasis::BasedOnTrimmedMean => t!("output-explore-levene-trimmed"),
                        };
                        let vals = vec![
                            lead,
                            basis_lbl,
                            format!("{:.3}", row.levene_statistic),
                            format!("{:.0}", row.df1),
                            format!("{:.3}", row.df2),
                            format!("{:.3}", row.sig),
                        ];
                        render_data_row(ui, &cols, &vals, row_idx % 2 == 1);
                        row_idx += 1;
                    }
                }
            }
        });
}

pub(crate) fn render_bootstrap_table_explore(ui: &mut Ui, res: &ExploreResult) {
    let Some(ref boot) = res.bootstrap else {
        return;
    };
    render_bootstrap_specifications_table(ui, &boot.options);
    ui.add_space(10.0);

    let b_cols = bootstrap_column_headers(boot.options.ci_level);
    let headers = vec![
        t!("output-col-variable"),
        t!("output-col-statistic"),
        b_cols[0].clone(),
        b_cols[1].clone(),
        b_cols[2].clone(),
        b_cols[3].clone(),
    ];

    let mut rows = Vec::new();
    for g in &boot.groups {
        let label = match &g.factor_level {
            Some(fv) => format!("{} ({})", g.variable, fv),
            None => g.variable.clone(),
        };
        let stats: [(&str, &Option<statcraft_algo::BootstrapSummary>); 8] = [
            ("output-explore-mean", &g.mean),
            ("output-explore-5-trimmed-mean", &g.trimmed_mean_5pct),
            ("output-explore-median", &g.median),
            ("output-explore-variance", &g.variance),
            ("output-explore-std-deviation", &g.std_dev),
            ("output-explore-iqr", &g.iqr),
            ("output-explore-skewness", &g.skewness),
            ("output-explore-kurtosis", &g.kurtosis),
        ];
        let mut first = true;
        for (key, summary_opt) in stats {
            if let Some(s) = summary_opt {
                let cells = format_bootstrap_cells(s, 4);
                let lead = if first {
                    first = false;
                    label.clone()
                } else {
                    String::new()
                };
                rows.push(vec![
                    lead,
                    t!(key),
                    cells[0].clone(),
                    cells[1].clone(),
                    cells[2].clone(),
                    cells[3].clone(),
                ]);
            }
        }
    }

    if !rows.is_empty() {
        render_bootstrap_table(
            ui,
            "explore_bootstrap_desc",
            &t!("output-explore-bootstrap-title"),
            &headers,
            &rows,
            2,
        );
    }
}
