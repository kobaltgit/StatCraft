use super::tables_summary::fmt_opt;
use crate::views::output_view::tables_bootstrap::{
    bootstrap_column_headers, format_bootstrap_cells,
};
use statcraft_algo::{ExploreOutputDisplay, ExploreResult, LeveneBasis};
use statcraft_i18n::t;
use statcraft_reporting::{CellAlignment, ReportTable};

pub fn explore_to_report_tables(res: &ExploreResult) -> Vec<ReportTable> {
    let mut tables = Vec::new();
    let show_stats = res.display_mode != ExploreOutputDisplay::Plots;
    let show_plots = res.display_mode != ExploreOutputDisplay::Statistics;

    tables.push(build_case_processing_report_table(res));

    if show_stats {
        if let Some(t) = build_descriptives_report_table(res) {
            tables.push(t);
        }
        if let Some(t) = build_m_estimators_report_table(res) {
            tables.push(t);
        }
        if let Some(t) = build_percentiles_report_table(res) {
            tables.push(t);
        }
        if let Some(t) = build_extremes_report_table(res) {
            tables.push(t);
        }
        if let Some(t) = build_bootstrap_report_table(res) {
            tables.push(t);
        }
    }
    if show_plots {
        if let Some(t) = build_normality_report_table(res) {
            tables.push(t);
        }
        if let Some(t) = build_levene_report_table(res) {
            tables.push(t);
        }
    }
    tables
}

fn var_group_label(var: &str, factor: &Option<String>) -> String {
    match factor {
        Some(f) => format!("{} ({})", var, f),
        None => var.to_string(),
    }
}

fn build_case_processing_report_table(res: &ExploreResult) -> ReportTable {
    let mut t = ReportTable::new(t!("output-explore-case-processing"));
    t.headers = vec![
        "Variable",
        "Valid N",
        "Valid %",
        "Missing N",
        "Missing %",
        "Total N",
        "Total %",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    t.alignments = vec![
        CellAlignment::Left,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
    ];
    for var in &res.variables {
        for g in &var.groups {
            let cp = &g.case_processing;
            t.rows.push(vec![
                var_group_label(&var.variable, &g.factor_level),
                cp.n_valid.to_string(),
                format!("{:.1}%", cp.pct_valid),
                cp.n_missing.to_string(),
                format!("{:.1}%", cp.pct_missing),
                cp.n_total.to_string(),
                format!("{:.1}%", cp.pct_total),
            ]);
        }
    }
    t
}

fn build_descriptives_report_table(res: &ExploreResult) -> Option<ReportTable> {
    let mut t = ReportTable::new(t!("output-explore-descriptives"));
    t.headers = vec![
        t!("output-explore-variable"),
        t!("output-explore-statistic"),
        t!("output-explore-value"),
        t!("output-explore-std-error"),
    ];
    t.alignments = vec![
        CellAlignment::Left,
        CellAlignment::Left,
        CellAlignment::Right,
        CellAlignment::Right,
    ];
    for var in &res.variables {
        for g in &var.groups {
            if let Some(d) = &g.descriptives {
                let name = var_group_label(&var.variable, &g.factor_level);
                let rows = [
                    ("Mean", fmt_opt(d.mean), fmt_opt(d.mean_se)),
                    (
                        "95% CI Lower Bound",
                        fmt_opt(d.mean_ci_lower),
                        String::new(),
                    ),
                    (
                        "95% CI Upper Bound",
                        fmt_opt(d.mean_ci_upper),
                        String::new(),
                    ),
                    (
                        "5% Trimmed Mean",
                        fmt_opt(d.trimmed_mean_5pct),
                        String::new(),
                    ),
                    ("Median", fmt_opt(d.median), String::new()),
                    ("Variance", fmt_opt(d.variance), String::new()),
                    ("Std. Deviation", fmt_opt(d.std_dev), String::new()),
                    ("Minimum", fmt_opt(d.min), String::new()),
                    ("Maximum", fmt_opt(d.max), String::new()),
                    ("Range", fmt_opt(d.range), String::new()),
                    ("Interquartile Range", fmt_opt(d.iqr), String::new()),
                    ("Skewness", fmt_opt(d.skewness), fmt_opt(d.skewness_se)),
                    ("Kurtosis", fmt_opt(d.kurtosis), fmt_opt(d.kurtosis_se)),
                ];
                for (stat, val, se) in rows {
                    t.rows.push(vec![name.clone(), stat.to_string(), val, se]);
                }
            }
        }
    }
    (!t.rows.is_empty()).then_some(t)
}

fn build_m_estimators_report_table(res: &ExploreResult) -> Option<ReportTable> {
    let mut t = ReportTable::new(t!("output-explore-m-estimators"));
    t.headers = vec![
        "Variable",
        "Huber's M-Estimator",
        "Tukey's Biweight",
        "Hampel's M-Estimator",
        "Andrews' Wave",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    t.alignments = vec![
        CellAlignment::Left,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
    ];
    for var in &res.variables {
        for g in &var.groups {
            if let Some(m) = &g.m_estimators {
                t.rows.push(vec![
                    var_group_label(&var.variable, &g.factor_level),
                    fmt_opt(m.huber),
                    fmt_opt(m.tukey),
                    fmt_opt(m.hampel),
                    fmt_opt(m.andrews),
                ]);
            }
        }
    }
    (!t.rows.is_empty()).then_some(t)
}

fn build_percentiles_report_table(res: &ExploreResult) -> Option<ReportTable> {
    let mut t = ReportTable::new(t!("output-explore-percentiles"));
    t.headers = vec![
        "Variable",
        "Statistic",
        "5",
        "10",
        "25",
        "50",
        "75",
        "90",
        "95",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    t.alignments = vec![
        CellAlignment::Left,
        CellAlignment::Left,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
    ];
    for var in &res.variables {
        for g in &var.groups {
            if let Some(p) = &g.percentiles {
                let name = var_group_label(&var.variable, &g.factor_level);
                t.rows.push(vec![
                    name.clone(),
                    "Weighted Average (Definition 1)".to_string(),
                    fmt_opt(p.p5),
                    fmt_opt(p.p10),
                    fmt_opt(p.p25),
                    fmt_opt(p.p50),
                    fmt_opt(p.p75),
                    fmt_opt(p.p90),
                    fmt_opt(p.p95),
                ]);
                t.rows.push(vec![
                    name,
                    "Tukey's Hinges".to_string(),
                    String::new(),
                    String::new(),
                    fmt_opt(p.tukey_q1),
                    fmt_opt(p.tukey_q2),
                    fmt_opt(p.tukey_q3),
                    String::new(),
                    String::new(),
                ]);
            }
        }
    }
    (!t.rows.is_empty()).then_some(t)
}

fn build_normality_report_table(res: &ExploreResult) -> Option<ReportTable> {
    let mut t = ReportTable::new(t!("output-explore-normality"));
    t.headers = vec![
        "Variable",
        "KS Statistic",
        "KS df",
        "KS Sig",
        "SW Statistic",
        "SW df",
        "SW Sig",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    t.alignments = vec![
        CellAlignment::Left,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
    ];
    for var in &res.variables {
        for g in &var.groups {
            if let Some(n) = &g.normality {
                let (ks_s, ks_d, ks_p) = match &n.kolmogorov_smirnov {
                    Some(k) => (
                        format!("{:.3}", k.statistic),
                        k.df.to_string(),
                        format!("{:.3}", k.sig),
                    ),
                    None => (".".into(), ".".into(), ".".into()),
                };
                let (sw_s, sw_d, sw_p) = match &n.shapiro_wilk {
                    Some(s) => (
                        format!("{:.3}", s.statistic),
                        s.df.to_string(),
                        format!("{:.3}", s.sig),
                    ),
                    None => (".".into(), ".".into(), ".".into()),
                };
                t.rows.push(vec![
                    var_group_label(&var.variable, &g.factor_level),
                    ks_s,
                    ks_d,
                    ks_p,
                    sw_s,
                    sw_d,
                    sw_p,
                ]);
            }
        }
    }
    t.footnotes
        .push("a. Lilliefors Significance Correction".to_string());
    (!t.rows.is_empty()).then_some(t)
}

fn build_extremes_report_table(res: &ExploreResult) -> Option<ReportTable> {
    let has_labels = res.variables.iter().any(|v| {
        v.groups.iter().any(|g| {
            g.extremes.as_ref().is_some_and(|e| {
                e.highest.iter().any(|i| i.case_label.is_some())
                    || e.lowest.iter().any(|i| i.case_label.is_some())
            })
        })
    });
    let mut t = ReportTable::new(t!("output-explore-extremes"));
    t.headers = vec![
        t!("output-explore-variable"),
        t!("output-explore-order"),
        t!("output-explore-case-number"),
    ];
    t.alignments = vec![
        CellAlignment::Left,
        CellAlignment::Left,
        CellAlignment::Right,
    ];
    if has_labels {
        t.headers.push(t!("output-explore-case-label"));
        t.alignments.push(CellAlignment::Left);
    }
    t.headers.push(t!("output-explore-value"));
    t.alignments.push(CellAlignment::Right);

    for var in &res.variables {
        for g in &var.groups {
            if let Some(ext) = &g.extremes {
                let name = var_group_label(&var.variable, &g.factor_level);
                for item in &ext.highest {
                    let mut r = vec![
                        name.clone(),
                        format!("Highest {}", item.order),
                        item.case_number.to_string(),
                    ];
                    if has_labels {
                        r.push(item.case_label.clone().unwrap_or_default());
                    }
                    r.push(format!("{:.4}", item.value));
                    t.rows.push(r);
                }
                for item in &ext.lowest {
                    let mut r = vec![
                        name.clone(),
                        format!("Lowest {}", item.order),
                        item.case_number.to_string(),
                    ];
                    if has_labels {
                        r.push(item.case_label.clone().unwrap_or_default());
                    }
                    r.push(format!("{:.4}", item.value));
                    t.rows.push(r);
                }
            }
        }
    }
    (!t.rows.is_empty()).then_some(t)
}

fn build_levene_report_table(res: &ExploreResult) -> Option<ReportTable> {
    let mut t = ReportTable::new(t!("output-explore-homogeneity-title"));
    t.headers = vec![
        "Variable",
        "Based on",
        "Levene Statistic",
        "df1",
        "df2",
        "Sig.",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    t.alignments = vec![
        CellAlignment::Left,
        CellAlignment::Left,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
    ];
    for var in &res.variables {
        if let Some(lev) = &var.levene_test {
            for row in &lev.rows {
                let basis = match row.basis {
                    LeveneBasis::BasedOnMean => "Based on Mean",
                    LeveneBasis::BasedOnMedian => "Based on Median",
                    LeveneBasis::BasedOnMedianAdjustedDf => "Based on Median and with adjusted df",
                    LeveneBasis::BasedOnTrimmedMean => "Based on trimmed mean",
                };
                t.rows.push(vec![
                    lev.variable.clone(),
                    basis.to_string(),
                    format!("{:.3}", row.levene_statistic),
                    format!("{:.0}", row.df1),
                    format!("{:.3}", row.df2),
                    format!("{:.3}", row.sig),
                ]);
            }
        }
    }
    (!t.rows.is_empty()).then_some(t)
}

fn build_bootstrap_report_table(res: &ExploreResult) -> Option<ReportTable> {
    let boot = res.bootstrap.as_ref()?;
    let b_cols = bootstrap_column_headers(boot.options.ci_level);
    let mut t = ReportTable::new(t!("output-explore-bootstrap-title"));
    t.headers = vec![
        "Variable".to_string(),
        "Statistic".to_string(),
        b_cols[0].clone(),
        b_cols[1].clone(),
        b_cols[2].clone(),
        b_cols[3].clone(),
    ];
    t.alignments = vec![
        CellAlignment::Left,
        CellAlignment::Left,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
    ];
    for g in &boot.groups {
        let name = var_group_label(&g.variable, &g.factor_level);
        for (lbl, opt) in [
            ("Mean", &g.mean),
            ("5% Trimmed Mean", &g.trimmed_mean_5pct),
            ("Median", &g.median),
            ("Variance", &g.variance),
            ("Std. Deviation", &g.std_dev),
            ("Interquartile Range", &g.iqr),
        ] {
            if let Some(s) = opt {
                let c = format_bootstrap_cells(s, 4);
                t.rows.push(vec![
                    name.clone(),
                    lbl.to_string(),
                    c[0].clone(),
                    c[1].clone(),
                    c[2].clone(),
                    c[3].clone(),
                ]);
            }
        }
    }
    (!t.rows.is_empty()).then_some(t)
}
