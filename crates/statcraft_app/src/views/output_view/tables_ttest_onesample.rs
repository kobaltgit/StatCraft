use super::tables_common::{
    adjust_widths, format_opt, format_sig, render_data_row, render_header_row,
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
                            Tokens::BG_ROW_EVEN
                        } else {
                            Tokens::BG_ROW_ODD
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
                            Tokens::BG_ROW_EVEN
                        } else {
                            Tokens::BG_ROW_ODD
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
    });
}

pub fn onesample_ttest_to_tsv(results: &[OneSampleTTestResult]) -> String {
    let mut out = String::new();

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

    out
}
