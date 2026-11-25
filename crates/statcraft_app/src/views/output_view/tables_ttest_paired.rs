use super::tables_common::{
    adjust_widths, bootstrap_column_headers, bootstrap_specs_to_tsv, bootstrap_table_to_tsv,
    format_bootstrap_cells, format_opt, format_sig, render_bootstrap_specifications_table,
    render_bootstrap_table, render_data_row, render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::PairedTTestResult;
use statcraft_i18n::t;

pub fn render_spss_paired_ttest_tables(
    ui: &mut Ui,
    results: &[PairedTTestResult],
    _pairs: &[(String, String)],
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
        // Table 1: Paired Samples Statistics
        ui.strong(t!("output-ttest-paired-stats-title"));
        ui.add_space(4.0_f32);

        let mut cols_p_stat: Vec<(String, f32)> = vec![
            (t!("output-col-pair"), 65.0_f32),
            (t!("output-col-variable"), 110.0_f32),
            (t!("output-col-mean"), 85.0_f32),
            (t!("output-col-n"), 55.0_f32),
            (t!("output-col-std"), 85.0_f32),
            (t!("output-col-std-error"), 85.0_f32),
        ];
        adjust_widths(ui, &mut cols_p_stat);

        let row_height = 22.0_f32;

        ScrollArea::horizontal()
            .id_salt("paired_stats_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_p_stat, row_height);

                    for (r_idx, res) in results.iter().enumerate() {
                        let bg = if r_idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let p_label = format!("Pair {}", res.pair_num);

                        let row1 = [
                            (p_label, Align2::LEFT_CENTER),
                            (res.var1_stats.var_name.clone(), Align2::LEFT_CENTER),
                            (format_opt(res.var1_stats.mean), Align2::RIGHT_CENTER),
                            (format!("{}", res.var1_stats.n), Align2::RIGHT_CENTER),
                            (format_opt(res.var1_stats.std_dev), Align2::RIGHT_CENTER),
                            (format_opt(res.var1_stats.se_mean), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_p_stat, &row1, row_height, bg);

                        let row2 = [
                            (String::new(), Align2::LEFT_CENTER),
                            (res.var2_stats.var_name.clone(), Align2::LEFT_CENTER),
                            (format_opt(res.var2_stats.mean), Align2::RIGHT_CENTER),
                            (format!("{}", res.var2_stats.n), Align2::RIGHT_CENTER),
                            (format_opt(res.var2_stats.std_dev), Align2::RIGHT_CENTER),
                            (format_opt(res.var2_stats.se_mean), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_p_stat, &row2, row_height, bg);
                    }
                });
            });

        ui.add_space(14.0_f32);

        // Table 2: Paired Samples Correlations
        ui.strong(t!("output-ttest-paired-corr-title"));
        ui.add_space(4.0_f32);

        let mut cols_corr: Vec<(String, f32)> = vec![
            (t!("output-col-pair"), 65.0_f32),
            (t!("output-col-variable"), 160.0_f32),
            (t!("output-col-n"), 55.0_f32),
            (t!("output-col-correlation"), 90.0_f32),
            (t!("output-col-sig"), 75.0_f32),
        ];
        adjust_widths(ui, &mut cols_corr);

        ScrollArea::horizontal()
            .id_salt("paired_corr_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_corr, row_height);

                    for (r_idx, res) in results.iter().enumerate() {
                        let bg = if r_idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let p_label = format!("Pair {}", res.pair_num);
                        let pair_vars =
                            format!("{} & {}", res.correlation.var1, res.correlation.var2);

                        let row = [
                            (p_label, Align2::LEFT_CENTER),
                            (pair_vars, Align2::LEFT_CENTER),
                            (format!("{}", res.correlation.n), Align2::RIGHT_CENTER),
                            (
                                format_opt(res.correlation.correlation),
                                Align2::RIGHT_CENTER,
                            ),
                            (format_sig(res.correlation.sig), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_corr, &row, row_height, bg);
                    }
                });
            });

        ui.add_space(14.0_f32);

        // Table 3: Paired Samples Test
        ui.strong(t!("output-ttest-paired-test-title"));
        ui.add_space(4.0_f32);

        let mut cols_test: Vec<(String, f32)> = vec![
            (t!("output-col-pair"), 65.0_f32),
            (t!("output-col-variable"), 150.0_f32),
            (t!("output-col-mean-diff"), 85.0_f32),
            (t!("output-col-std"), 85.0_f32),
            (t!("output-col-se-diff"), 85.0_f32),
            (t!("output-col-ci-lower"), 80.0_f32),
            (t!("output-col-ci-upper"), 80.0_f32),
            ("t".to_string(), 60.0_f32),
            ("df".to_string(), 55.0_f32),
            (t!("output-col-sig-2tailed"), 85.0_f32),
        ];
        adjust_widths(ui, &mut cols_test);

        ScrollArea::horizontal()
            .id_salt("paired_test_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_test, row_height);

                    for (r_idx, res) in results.iter().enumerate() {
                        let bg = if r_idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let p_label = format!("Pair {}", res.pair_num);
                        let pair_vars =
                            format!("{} - {}", res.correlation.var1, res.correlation.var2);
                        let diff = &res.differences;

                        let row = [
                            (p_label, Align2::LEFT_CENTER),
                            (pair_vars, Align2::LEFT_CENTER),
                            (format_opt(diff.mean_diff), Align2::RIGHT_CENTER),
                            (format_opt(diff.std_dev_diff), Align2::RIGHT_CENTER),
                            (format_opt(diff.se_diff), Align2::RIGHT_CENTER),
                            (format_opt(diff.ci_lower), Align2::RIGHT_CENTER),
                            (format_opt(diff.ci_upper), Align2::RIGHT_CENTER),
                            (format_opt(diff.t_stat), Align2::RIGHT_CENTER),
                            (
                                diff.df
                                    .map(|v| format!("{:.0}", v))
                                    .unwrap_or_else(|| "-".to_string()),
                                Align2::RIGHT_CENTER,
                            ),
                            (format_sig(diff.sig_2tailed), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_test, &row, row_height, bg);
                    }
                });
            });

        ui.add_space(16.0_f32);

        if let Some((headers, rows)) = build_paired_bootstrap_rows(results) {
            let title = format!(
                "{} ({})",
                t!("output-ttest-paired-test-title"),
                t!("output-bootstrap-group-header")
            );
            render_bootstrap_table(ui, &title, "paired_boot_table", &headers, &rows, 2);
            ui.add_space(16.0_f32);
        }
    });
}

fn build_paired_bootstrap_rows(
    results: &[PairedTTestResult],
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let mut rows = Vec::new();
    let mut ci_level = 95.0;

    for r in results {
        let Some(ref boot) = r.bootstrap else {
            continue;
        };
        ci_level = boot.options.ci_level;
        let pair_lbl = format!("Pair {}", r.pair_num);
        for (stat_name, sum_opt) in [
            (
                format!("{} ({})", t!("output-col-mean"), r.var1_stats.var_name),
                &boot.var1_mean,
            ),
            (
                format!("{} ({})", t!("output-col-mean"), r.var2_stats.var_name),
                &boot.var2_mean,
            ),
            (t!("output-col-correlation"), &boot.correlation),
            (t!("output-col-mean-diff"), &boot.mean_diff),
        ] {
            if let Some(s) = sum_opt {
                let cells = format_bootstrap_cells(s, 4);
                rows.push(vec![
                    pair_lbl.clone(),
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
        t!("output-col-pair"),
        t!("output-bootstrap-col-statistic"),
        t!("output-km-col-estimate"),
        boot_cols[0].clone(),
        boot_cols[1].clone(),
        boot_cols[2].clone(),
        boot_cols[3].clone(),
    ];
    Some((headers, rows))
}

pub fn paired_ttest_to_tsv(results: &[PairedTTestResult]) -> String {
    let mut out = String::new();
    if let Some(opts) = results
        .iter()
        .find_map(|r| r.bootstrap.as_ref().map(|b| &b.options))
    {
        out.push_str(&bootstrap_specs_to_tsv(opts));
        out.push_str("\n\n");
    }

    // Table 1: Paired Samples Statistics
    out.push_str("Paired Samples Statistics\n");
    out.push_str("Pair\tVariable\tMean\tN\tStd. Deviation\tStd. Error Mean\n");
    for r in results {
        out.push_str(&format!(
            "Pair {}\t{}\t{}\t{}\t{}\t{}\n",
            r.pair_num,
            r.var1_stats.var_name,
            format_opt(r.var1_stats.mean),
            r.var1_stats.n,
            format_opt(r.var1_stats.std_dev),
            format_opt(r.var1_stats.se_mean)
        ));
        out.push_str(&format!(
            "\t{}\t{}\t{}\t{}\t{}\n",
            r.var2_stats.var_name,
            format_opt(r.var2_stats.mean),
            r.var2_stats.n,
            format_opt(r.var2_stats.std_dev),
            format_opt(r.var2_stats.se_mean)
        ));
    }

    out.push('\n');

    // Table 2: Paired Samples Correlations
    out.push_str("Paired Samples Correlations\n");
    out.push_str("Pair\tVariables\tN\tCorrelation\tSig.\n");
    for r in results {
        out.push_str(&format!(
            "Pair {}\t{} & {}\t{}\t{}\t{}\n",
            r.pair_num,
            r.correlation.var1,
            r.correlation.var2,
            r.correlation.n,
            format_opt(r.correlation.correlation),
            format_sig(r.correlation.sig)
        ));
    }

    out.push('\n');

    // Table 3: Paired Samples Test
    out.push_str("Paired Samples Test\n");
    out.push_str(
        "Pair\tPaired Differences Mean\tStd. Deviation\tStd. Error Mean\tLower\tUpper\tt\tdf\tSig. (2-tailed)\n",
    );
    for r in results {
        let diff = &r.differences;
        out.push_str(&format!(
            "Pair {}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.pair_num,
            format_opt(diff.mean_diff),
            format_opt(diff.std_dev_diff),
            format_opt(diff.se_diff),
            format_opt(diff.ci_lower),
            format_opt(diff.ci_upper),
            format_opt(diff.t_stat),
            diff.df.map(|v| format!("{:.0}", v)).unwrap_or_default(),
            format_sig(diff.sig_2tailed)
        ));
    }

    if let Some((headers, rows)) = build_paired_bootstrap_rows(results) {
        out.push('\n');
        out.push_str(&bootstrap_table_to_tsv(
            &format!(
                "Paired Samples Test ({})",
                t!("output-bootstrap-group-header")
            ),
            &headers,
            &rows,
        ));
        out.push('\n');
    }

    out
}
