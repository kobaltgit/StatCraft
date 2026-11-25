use super::tables_common::{
    adjust_widths, bootstrap_column_headers, bootstrap_specs_to_tsv, bootstrap_table_to_tsv,
    format_bootstrap_cells, format_opt, format_sig, render_bootstrap_specifications_table,
    render_bootstrap_table, render_data_row, render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::OneSampleTTestResult;
use statcraft_i18n::t;

pub fn render_spss_onesample_ttest_tables(
    ui: &mut Ui,
    results: &[OneSampleTTestResult],
    test_value: f64,
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
        // Table 1: One-Sample Statistics
        ui.strong(t!("output-ttest-stats-title"));
        ui.add_space(4.0_f32);

        let mut cols_stat: Vec<(String, f32)> = vec![
            (t!("output-col-variable"), 120.0_f32),
            (t!("output-col-n"), 60.0_f32),
            (t!("output-col-mean"), 85.0_f32),
            (t!("output-col-std"), 85.0_f32),
            (t!("output-col-std-error"), 85.0_f32),
        ];
        adjust_widths(ui, &mut cols_stat);

        let row_height = 22.0_f32;

        ScrollArea::horizontal()
            .id_salt("onesample_stats_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_stat, row_height);

                    for (r_idx, res) in results.iter().enumerate() {
                        let bg = if r_idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let vals = [
                            (res.variable.clone(), Align2::LEFT_CENTER),
                            (format!("{}", res.n_valid), Align2::RIGHT_CENTER),
                            (format_opt(res.mean), Align2::RIGHT_CENTER),
                            (format_opt(res.std_dev), Align2::RIGHT_CENTER),
                            (format_opt(res.se_mean), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_stat, &vals, row_height, bg);
                    }
                });
            });

        ui.add_space(14.0_f32);

        // Table 2: One-Sample Test
        let val_str = format!("{:.2}", test_value);
        ui.strong(t!("output-ttest-test-title", val = val_str.as_str()));
        ui.add_space(4.0_f32);

        let mut cols_test: Vec<(String, f32)> = vec![
            (t!("output-col-variable"), 110.0_f32),
            ("t".to_string(), 65.0_f32),
            ("df".to_string(), 55.0_f32),
            (t!("output-col-sig-2tailed"), 85.0_f32),
            (t!("output-col-mean-diff"), 90.0_f32),
            (t!("output-col-ci-lower"), 80.0_f32),
            (t!("output-col-ci-upper"), 80.0_f32),
        ];
        adjust_widths(ui, &mut cols_test);

        ScrollArea::horizontal()
            .id_salt("onesample_test_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_test, row_height);

                    for (r_idx, res) in results.iter().enumerate() {
                        let bg = if r_idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let vals = [
                            (res.variable.clone(), Align2::LEFT_CENTER),
                            (format_opt(res.t_stat), Align2::RIGHT_CENTER),
                            (
                                res.df
                                    .map(|v| format!("{:.0}", v))
                                    .unwrap_or_else(|| "-".to_string()),
                                Align2::RIGHT_CENTER,
                            ),
                            (format_sig(res.sig_2tailed), Align2::RIGHT_CENTER),
                            (format_opt(res.mean_diff), Align2::RIGHT_CENTER),
                            (format_opt(res.ci_lower), Align2::RIGHT_CENTER),
                            (format_opt(res.ci_upper), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_test, &vals, row_height, bg);
                    }
                });
            });

        ui.add_space(16.0_f32);

        if let Some((headers, rows)) = build_onesample_bootstrap_rows(results) {
            let title = format!(
                "{} ({})",
                t!("output-ttest-test-title", value = test_value),
                t!("output-bootstrap-group-header")
            );
            render_bootstrap_table(ui, &title, "onesample_boot_table", &headers, &rows, 2);
            ui.add_space(16.0_f32);
        }
    });
}

fn build_onesample_bootstrap_rows(
    results: &[OneSampleTTestResult],
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let mut rows = Vec::new();
    let mut ci_level = 95.0;

    for r in results {
        let Some(ref boot) = r.bootstrap else {
            continue;
        };
        ci_level = boot.options.ci_level;
        for (stat_name, sum_opt) in [
            (t!("output-col-mean"), &boot.mean),
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

pub fn onesample_ttest_to_tsv(results: &[OneSampleTTestResult]) -> String {
    let mut out = String::new();
    if let Some(opts) = results
        .iter()
        .find_map(|r| r.bootstrap.as_ref().map(|b| &b.options))
    {
        out.push_str(&bootstrap_specs_to_tsv(opts));
        out.push_str("\n\n");
    }

    // Table 1: One-Sample Statistics
    out.push_str("One-Sample Statistics\n");
    out.push_str("Variable\tN\tMean\tStd. Deviation\tStd. Error Mean\n");
    for r in results {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            r.variable,
            r.n_valid,
            format_opt(r.mean),
            format_opt(r.std_dev),
            format_opt(r.se_mean)
        ));
    }

    out.push('\n');

    // Table 2: One-Sample Test
    out.push_str("One-Sample Test\n");
    out.push_str("Variable\tt\tdf\tSig. (2-tailed)\tMean Difference\tLower\tUpper\n");
    for r in results {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.variable,
            format_opt(r.t_stat),
            r.df.map(|v| format!("{:.0}", v)).unwrap_or_default(),
            format_sig(r.sig_2tailed),
            format_opt(r.mean_diff),
            format_opt(r.ci_lower),
            format_opt(r.ci_upper)
        ));
    }

    if let Some((headers, rows)) = build_onesample_bootstrap_rows(results) {
        out.push('\n');
        out.push_str(&bootstrap_table_to_tsv(
            &format!("One-Sample Test ({})", t!("output-bootstrap-group-header")),
            &headers,
            &rows,
        ));
        out.push('\n');
    }

    out
}
