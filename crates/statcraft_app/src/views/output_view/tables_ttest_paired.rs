use super::tables_common::{
    adjust_widths, format_opt, format_sig, render_data_row, render_header_row,
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
                            Tokens::BG_ROW_EVEN
                        } else {
                            Tokens::BG_ROW_ODD
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
                            Tokens::BG_ROW_EVEN
                        } else {
                            Tokens::BG_ROW_ODD
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
                            Tokens::BG_ROW_EVEN
                        } else {
                            Tokens::BG_ROW_ODD
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
    });
}

pub fn paired_ttest_to_tsv(results: &[PairedTTestResult]) -> String {
    let mut out = String::new();

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

    out
}
