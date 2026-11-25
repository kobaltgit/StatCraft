pub use super::report_tables::explore_to_report_tables;
use crate::views::output_view::tables_bootstrap::{
    bootstrap_column_headers, bootstrap_specs_to_tsv, bootstrap_table_to_tsv,
    format_bootstrap_cells,
};
use statcraft_algo::{ExploreOutputDisplay, ExploreResult};

pub fn explore_to_tsv(res: &ExploreResult) -> String {
    let mut out = String::from("Explore / EXAMINE Summary\n");
    let show_stats = res.display_mode != ExploreOutputDisplay::Plots;

    for table in explore_to_report_tables(res) {
        out.push_str(&table.title);
        out.push('\n');
        out.push_str(&table.headers.join("\t"));
        out.push('\n');
        for row in &table.rows {
            out.push_str(&row.join("\t"));
            out.push('\n');
        }
        for note in &table.footnotes {
            out.push_str(note);
            out.push('\n');
        }
        out.push('\n');
    }

    if show_stats {
        if let Some(ref boot) = res.bootstrap {
            out.push_str(&bootstrap_specs_to_tsv(&boot.options));
            let b_cols = bootstrap_column_headers(boot.options.ci_level);
            let headers = vec![
                "Variable".to_string(),
                "Statistic".to_string(),
                b_cols[0].clone(),
                b_cols[1].clone(),
                b_cols[2].clone(),
                b_cols[3].clone(),
            ];
            let mut rows = Vec::new();
            for g in &boot.groups {
                let name = match &g.factor_level {
                    Some(f) => format!("{} ({})", g.variable, f),
                    None => g.variable.clone(),
                };
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
                        rows.push(vec![
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
            if !rows.is_empty() {
                out.push_str(&bootstrap_table_to_tsv(
                    "Bootstrap for Descriptives",
                    &headers,
                    &rows,
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_algo::{ExploreCaseProcessing, ExploreGroupResult, ExploreVariableResult};

    #[test]
    fn test_explore_to_tsv_output() {
        let res = ExploreResult {
            variables: vec![ExploreVariableResult {
                variable: "salary".to_string(),
                factor_var: None,
                groups: vec![ExploreGroupResult {
                    factor_level: None,
                    case_processing: ExploreCaseProcessing {
                        variable: "salary".to_string(),
                        factor_level: None,
                        n_valid: 10,
                        pct_valid: 100.0,
                        n_missing: 0,
                        pct_missing: 0.0,
                        n_total: 10,
                        pct_total: 100.0,
                    },
                    descriptives: None,
                    m_estimators: None,
                    percentiles: None,
                    normality: None,
                    extremes: None,
                    plot_data: None,
                }],
                levene_test: None,
                spread_level: None,
            }],
            display_mode: ExploreOutputDisplay::Both,
            boxplot_mode: statcraft_algo::ExploreBoxplotDisplay::FactorLevelsTogether,
            plot_stem_leaf: true,
            plot_histogram: false,
            bootstrap: None,
        };

        let tsv = explore_to_tsv(&res);
        assert!(tsv.contains("Case Processing Summary"));
        assert!(tsv.contains("salary\t10\t100.0%\t0\t0.0%\t10\t100.0%"));
    }
}
