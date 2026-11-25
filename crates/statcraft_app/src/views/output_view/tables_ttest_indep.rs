use super::tables_common::{
    adjust_widths, bootstrap_column_headers, bootstrap_specs_to_tsv, bootstrap_table_to_tsv,
    format_bootstrap_cells, format_opt, format_opt_2dec, format_sig,
    render_bootstrap_specifications_table, render_bootstrap_table, render_data_row,
    render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::IndepTTestResult;
use statcraft_i18n::t;

pub fn render_spss_indep_ttest_tables(
    ui: &mut Ui,
    results: &[IndepTTestResult],
    _grouping_var: &str,
    group1: &str,
    group2: &str,
    _ci: f64,
) {
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
        // Table 1: Group Statistics
        ui.strong(t!("output-ttest-group-stats-title"));
        ui.add_space(4.0_f32);

        let mut cols_grp: Vec<(String, f32)> = vec![
            (t!("output-col-variable"), 110.0_f32),
            (t!("output-col-group"), 80.0_f32),
            (t!("output-col-n"), 55.0_f32),
            (t!("output-col-mean"), 80.0_f32),
            (t!("output-col-std"), 80.0_f32),
            (t!("output-col-std-error"), 80.0_f32),
        ];
        adjust_widths(ui, &mut cols_grp);

        let row_height = 22.0_f32;

        ScrollArea::horizontal()
            .id_salt("indep_grp_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_grp, row_height);

                    for (r_idx, res) in results.iter().enumerate() {
                        let bg = if r_idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };

                        let row1 = [
                            (res.variable.clone(), Align2::LEFT_CENTER),
                            (group1.to_string(), Align2::LEFT_CENTER),
                            (format!("{}", res.group1_stats.n), Align2::RIGHT_CENTER),
                            (format_opt(res.group1_stats.mean), Align2::RIGHT_CENTER),
                            (format_opt(res.group1_stats.std_dev), Align2::RIGHT_CENTER),
                            (format_opt(res.group1_stats.se_mean), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_grp, &row1, row_height, bg);

                        let row2 = [
                            (String::new(), Align2::LEFT_CENTER),
                            (group2.to_string(), Align2::LEFT_CENTER),
                            (format!("{}", res.group2_stats.n), Align2::RIGHT_CENTER),
                            (format_opt(res.group2_stats.mean), Align2::RIGHT_CENTER),
                            (format_opt(res.group2_stats.std_dev), Align2::RIGHT_CENTER),
                            (format_opt(res.group2_stats.se_mean), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_grp, &row2, row_height, bg);
                    }
                });
            });

        ui.add_space(14.0_f32);

        // Table 2: Independent Samples Test
        ui.strong(t!("output-ttest-indep-test-title"));
        ui.add_space(4.0_f32);

        let mut cols_indep: Vec<(String, f32)> = vec![
            (t!("output-col-variable"), 100.0_f32),
            (t!("output-col-variances-assumed"), 150.0_f32),
            ("F".to_string(), 55.0_f32),
            (t!("output-col-sig"), 65.0_f32),
            ("t".to_string(), 60.0_f32),
            ("df".to_string(), 60.0_f32),
            (t!("output-col-sig-2tailed"), 80.0_f32),
            (t!("output-col-mean-diff"), 80.0_f32),
            (t!("output-col-se-diff"), 80.0_f32),
            (t!("output-col-ci-lower"), 80.0_f32),
            (t!("output-col-ci-upper"), 80.0_f32),
        ];
        adjust_widths(ui, &mut cols_indep);

        ScrollArea::horizontal()
            .id_salt("indep_test_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_indep, row_height);

                    for (r_idx, res) in results.iter().enumerate() {
                        let bg = if r_idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let eq = &res.equal_variances_assumed;
                        let neq = &res.equal_variances_not_assumed;

                        let row_assumed = [
                            (res.variable.clone(), Align2::LEFT_CENTER),
                            (t!("output-row-equal-var-assumed"), Align2::LEFT_CENTER),
                            (format_opt_2dec(res.levene.f_stat), Align2::RIGHT_CENTER),
                            (format_sig(res.levene.sig), Align2::RIGHT_CENTER),
                            (format_opt(eq.t_stat), Align2::RIGHT_CENTER),
                            (format_opt_2dec(eq.df), Align2::RIGHT_CENTER),
                            (format_sig(eq.sig_2tailed), Align2::RIGHT_CENTER),
                            (format_opt(eq.mean_diff), Align2::RIGHT_CENTER),
                            (format_opt(eq.se_diff), Align2::RIGHT_CENTER),
                            (format_opt(eq.ci_lower), Align2::RIGHT_CENTER),
                            (format_opt(eq.ci_upper), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_indep, &row_assumed, row_height, bg);

                        let row_not_assumed = [
                            (String::new(), Align2::LEFT_CENTER),
                            (t!("output-row-equal-var-not-assumed"), Align2::LEFT_CENTER),
                            (String::new(), Align2::RIGHT_CENTER),
                            (String::new(), Align2::RIGHT_CENTER),
                            (format_opt(neq.t_stat), Align2::RIGHT_CENTER),
                            (format_opt_2dec(neq.df), Align2::RIGHT_CENTER),
                            (format_sig(neq.sig_2tailed), Align2::RIGHT_CENTER),
                            (format_opt(neq.mean_diff), Align2::RIGHT_CENTER),
                            (format_opt(neq.se_diff), Align2::RIGHT_CENTER),
                            (format_opt(neq.ci_lower), Align2::RIGHT_CENTER),
                            (format_opt(neq.ci_upper), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_indep, &row_not_assumed, row_height, bg);
                    }
                });
            });

        ui.add_space(16.0_f32);

        if let Some((headers, rows)) = build_indep_bootstrap_rows(results) {
            let title = format!(
                "{} ({})",
                t!("output-ttest-indep-test-title"),
                t!("output-bootstrap-group-header")
            );
            render_bootstrap_table(ui, &title, "indep_boot_table", &headers, &rows, 2);
            ui.add_space(16.0_f32);
        }
    });
}

fn build_indep_bootstrap_rows(
    results: &[IndepTTestResult],
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let mut rows = Vec::new();
    let mut ci_level = 95.0;

    for r in results {
        let Some(ref boot) = r.bootstrap else {
            continue;
        };
        ci_level = boot.options.ci_level;
        for (stat_name, sum_opt) in [
            (
                format!("{} ({})", t!("output-col-mean"), r.group1_stats.group_val),
                &boot.group1_mean,
            ),
            (
                format!("{} ({})", t!("output-col-mean"), r.group2_stats.group_val),
                &boot.group2_mean,
            ),
            (t!("output-col-mean-diff"), &boot.mean_diff),
        ] {
            if let Some(s) = sum_opt {
                let cells = format_bootstrap_cells(s, 4);
                rows.push(vec![
                    r.variable.clone(),
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

pub fn indep_ttest_to_tsv(results: &[IndepTTestResult]) -> String {
    let mut out = String::new();
    if let Some(opts) = results
        .iter()
        .find_map(|r| r.bootstrap.as_ref().map(|b| &b.options))
    {
        out.push_str(&bootstrap_specs_to_tsv(opts));
        out.push_str("\n\n");
    }

    // Table 1: Group Statistics
    out.push_str("Group Statistics\n");
    out.push_str("Variable\tGroup\tN\tMean\tStd. Deviation\tStd. Error Mean\n");
    for r in results {
        let g1 = &r.group1_stats;
        let g2 = &r.group2_stats;
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            r.variable,
            g1.group_val,
            g1.n,
            format_opt(g1.mean),
            format_opt(g1.std_dev),
            format_opt(g1.se_mean)
        ));
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            r.variable,
            g2.group_val,
            g2.n,
            format_opt(g2.mean),
            format_opt(g2.std_dev),
            format_opt(g2.se_mean)
        ));
    }

    out.push('\n');

    // Table 2: Independent Samples Test
    out.push_str("Independent Samples Test\n");
    out.push_str(
        "Variable\tEqual variances assumed / not assumed\tLevene F\tLevene Sig.\tt\tdf\tSig. (2-tailed)\tMean Difference\tStd. Error Difference\tLower\tUpper\n",
    );
    for r in results {
        let eq = &r.equal_variances_assumed;
        let neq = &r.equal_variances_not_assumed;
        out.push_str(&format!(
            "{}\tEqual variances assumed\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.variable,
            format_opt_2dec(r.levene.f_stat),
            format_sig(r.levene.sig),
            format_opt(eq.t_stat),
            format_opt_2dec(eq.df),
            format_sig(eq.sig_2tailed),
            format_opt(eq.mean_diff),
            format_opt(eq.se_diff),
            format_opt(eq.ci_lower),
            format_opt(eq.ci_upper)
        ));
        out.push_str(&format!(
            "{}\tEqual variances not assumed\t\t\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.variable,
            format_opt(neq.t_stat),
            format_opt_2dec(neq.df),
            format_sig(neq.sig_2tailed),
            format_opt(neq.mean_diff),
            format_opt(neq.se_diff),
            format_opt(neq.ci_lower),
            format_opt(neq.ci_upper)
        ));
    }

    if let Some((headers, rows)) = build_indep_bootstrap_rows(results) {
        out.push('\n');
        out.push_str(&bootstrap_table_to_tsv(
            &format!(
                "Independent Samples Test ({})",
                t!("output-bootstrap-group-header")
            ),
            &headers,
            &rows,
        ));
        out.push('\n');
    }

    out
}
