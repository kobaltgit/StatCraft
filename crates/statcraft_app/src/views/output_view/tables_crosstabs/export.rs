use super::super::tables_common::{format_opt, format_sig};
use super::matrix::{format_layer_banner, format_observed_cell};
use statcraft_algo::CrosstabResult;
use statcraft_i18n::t;
use statcraft_reporting::model::{CellAlignment, ReportTable};

pub fn crosstabs_to_tsv(res: &CrosstabResult) -> String {
    let mut out = String::new();
    let layer_suffix = format_layer_banner(&res.layer_keys)
        .map(|b| format!(" ({})", b))
        .unwrap_or_default();

    // 1. Crosstabulation Matrix
    if !res.options.suppress_tables {
        out.push_str(&format!(
            "{} * {} Crosstabulation{}\n",
            res.row_var, res.col_var, layer_suffix
        ));
        out.push_str(&res.row_var);
        out.push_str("\tLayer\t");
        for col_cat in &res.col_categories {
            let col_hdr = col_cat
                .label
                .as_ref()
                .cloned()
                .unwrap_or_else(|| col_cat.value.clone());
            out.push_str(&col_hdr);
            out.push('\t');
        }
        out.push_str("Total\n");

        for row in &res.rows {
            let row_name = row
                .category
                .label
                .as_ref()
                .cloned()
                .unwrap_or_else(|| row.category.value.clone());
            if res.options.observed {
                out.push_str(&format!("{}\tCount\t", row_name));
                for cell in &row.cells {
                    out.push_str(&format!("{}\t", format_observed_cell(cell, res)));
                }
                out.push_str(&format!("{}\n", row.row_total));
            }
            if res.options.expected {
                out.push_str("\tExpected Count\t");
                for cell in &row.cells {
                    out.push_str(&format!("{}\t", format_opt(cell.expected)));
                }
                out.push_str(&format!("{:.1}\n", row.row_total as f64));
            }
            if res.options.residuals || res.options.resid_unstandardized {
                out.push_str("\tResidual\t");
                for cell in &row.cells {
                    out.push_str(&format!("{}\t", format_opt(cell.residual)));
                }
                out.push_str("0.0\n");
            }
            if res.options.resid_standardized {
                out.push_str("\tStandardized Residual\t");
                for cell in &row.cells {
                    out.push_str(&format!("{}\t", format_opt(cell.std_residual)));
                }
                out.push('\n');
            }
            if res.options.resid_adjusted {
                out.push_str("\tAdjusted Residual\t");
                for cell in &row.cells {
                    out.push_str(&format!("{}\t", format_opt(cell.adj_residual)));
                }
                out.push('\n');
            }
        }

        out.push_str("Total\tCount\t");
        for c in &res.col_totals {
            out.push_str(&format!("{}\t", c));
        }
        out.push_str(&format!("{}\n\n", res.grand_total));
    }

    // 2. Chi-Square tests
    if !res.chi_square_tests.is_empty() {
        out.push_str("Chi-Square Tests\n");
        out.push_str("Test\tValue\tdf\tAsymptotic Sig (2-sided)\tExact Sig (2-sided)\n");
        for t in &res.chi_square_tests {
            let df_str = t.df.map(|d| d.to_string()).unwrap_or_default();
            out.push_str(&format!(
                "{}\t{:.3}\t{}\t{}\t{}\n",
                t.name,
                t.value,
                df_str,
                format_sig(t.asymp_sig),
                format_sig(t.exact_sig)
            ));
        }
        out.push('\n');
    }

    // 3. Directional Measures
    if !res.directional_measures.is_empty() {
        out.push_str("Directional Measures\n");
        out.push_str("Measure\tDirection\tValue\tApproximate Sig\n");
        for m in &res.directional_measures {
            out.push_str(&format!(
                "{}\t{}\t{:.3}\t{}\n",
                m.name,
                m.direction,
                m.value,
                format_sig(m.approx_sig)
            ));
        }
        out.push('\n');
    }

    // 4. Symmetric Measures
    if !res.association_measures.is_empty() {
        out.push_str("Symmetric Measures\n");
        out.push_str("Measure\tValue\tApproximate Sig\n");
        for m in &res.association_measures {
            out.push_str(&format!(
                "{}\t{:.3}\t{}\n",
                m.name,
                m.value,
                format_sig(m.approx_sig)
            ));
        }
        out.push('\n');
    }

    // 5. Risk Estimate
    if !res.risk_estimates.is_empty() {
        out.push_str("Risk Estimate\n");
        out.push_str("Measure\tValue\t95% CI Lower\t95% CI Upper\n");
        for r in &res.risk_estimates {
            out.push_str(&format!(
                "{}\t{:.3}\t{}\t{}\n",
                r.name,
                r.value,
                format_opt(r.ci_lower),
                format_opt(r.ci_upper)
            ));
        }
        out.push('\n');
    }

    // 6. McNemar & CMH
    if let Some(mc) = res.mcnemar_sig {
        out.push_str("McNemar Test\n");
        out.push_str(&format!(
            "N\t{}\nExact Sig (2-sided)\t{}\n\n",
            res.n_valid,
            format_sig(Some(mc))
        ));
    }
    if let Some(ref cmh) = res.cmh_result {
        out.push_str("Cochran's and Mantel-Haenszel Statistics\n");
        out.push_str(&format!(
            "Breslow-Day\t{:.3}\t{}\t{}\n",
            cmh.breslow_day_stat,
            cmh.breslow_day_df,
            format_sig(Some(cmh.breslow_day_sig))
        ));
        out.push_str(&format!(
            "Tarone's\t{:.3}\t{}\t{}\n",
            cmh.tarone_stat,
            cmh.tarone_df,
            format_sig(Some(cmh.tarone_sig))
        ));
        out.push_str(&format!(
            "Cochran's\t{:.3}\t{}\t{}\n",
            cmh.cochran_stat,
            cmh.cochran_df,
            format_sig(Some(cmh.cochran_sig))
        ));
        out.push_str(&format!(
            "Mantel-Haenszel\t{:.3}\t{}\t{}\n",
            cmh.mh_stat,
            cmh.mh_df,
            format_sig(Some(cmh.mh_sig))
        ));
        out.push_str(&format!(
            "Common OR\t{:.3}\t{:.3}\t{:.3}\n",
            cmh.common_or_estimate, cmh.common_or_ci_lower, cmh.common_or_ci_upper
        ));
    }

    out
}

pub fn crosstabs_to_report_tables(res: &CrosstabResult) -> Vec<ReportTable> {
    let mut tables = Vec::new();
    let layer_suffix = format_layer_banner(&res.layer_keys)
        .map(|b| format!(" ({})", b))
        .unwrap_or_default();

    // 1. Crosstabulation Matrix
    if !res.options.suppress_tables {
        let mut matrix = ReportTable::new(format!(
            "{}{}",
            t!(
                "output-crosstabs-table-title",
                row = res.row_var.as_str(),
                col = res.col_var.as_str()
            ),
            layer_suffix
        ));
        let mut headers = vec![res.row_var.clone(), t!("output-crosstabs-layer")];
        let mut alignments = vec![CellAlignment::Left, CellAlignment::Left];
        for col_cat in &res.col_categories {
            headers.push(
                col_cat
                    .label
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| col_cat.value.clone()),
            );
            alignments.push(CellAlignment::Right);
        }
        headers.push(t!("output-crosstabs-total"));
        alignments.push(CellAlignment::Right);
        matrix.headers = headers;
        matrix.alignments = alignments;

        for row in &res.rows {
            let row_name = row
                .category
                .label
                .as_ref()
                .cloned()
                .unwrap_or_else(|| row.category.value.clone());
            if res.options.observed {
                let mut r = vec![row_name, t!("output-crosstabs-count")];
                for cell in &row.cells {
                    r.push(format_observed_cell(cell, res));
                }
                r.push(row.row_total.to_string());
                matrix.rows.push(r);
            }
        }
        let mut tot = vec![t!("output-crosstabs-total"), t!("output-crosstabs-count")];
        for ct in &res.col_totals {
            tot.push(ct.to_string());
        }
        tot.push(res.grand_total.to_string());
        matrix.rows.push(tot);
        tables.push(matrix);
    }

    // 2. Chi-Square Tests
    if !res.chi_square_tests.is_empty() {
        let mut chisq = ReportTable::new(t!("output-crosstabs-chisq-title"));
        chisq.headers = vec![
            t!("output-crosstabs-chisq-title"),
            t!("output-crosstabs-value"),
            t!("output-crosstabs-df"),
            t!("output-crosstabs-asymp-sig"),
            t!("output-crosstabs-exact-sig"),
        ];
        chisq.alignments = vec![
            CellAlignment::Left,
            CellAlignment::Right,
            CellAlignment::Right,
            CellAlignment::Right,
            CellAlignment::Right,
        ];
        for tr in &res.chi_square_tests {
            chisq.rows.push(vec![
                tr.name.clone(),
                format!("{:.3}", tr.value),
                tr.df.map(|d| d.to_string()).unwrap_or_default(),
                format_sig(tr.asymp_sig),
                format_sig(tr.exact_sig),
            ]);
        }
        tables.push(chisq);
    }

    tables
}
