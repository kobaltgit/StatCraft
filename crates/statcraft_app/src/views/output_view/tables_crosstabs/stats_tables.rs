use super::super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, FontId, Ui};
use statcraft_algo::{CmhResult, CrosstabResult};
use statcraft_i18n::t;

fn render_section_title(ui: &mut Ui, title: String, selected: bool) {
    ui.label(
        egui::RichText::new(title)
            .font(FontId::proportional(12.5))
            .strong()
            .color(if selected {
                Tokens::accent()
            } else {
                Tokens::text_primary()
            }),
    );
    ui.add_space(2.0);
}

fn row_bg(idx: usize) -> egui::Color32 {
    if idx.is_multiple_of(2) {
        Tokens::bg_row_even()
    } else {
        Tokens::bg_row_odd()
    }
}

pub fn render_all_stats_tables(ui: &mut Ui, res: &CrosstabResult, selected: bool) {
    if !res.chi_square_tests.is_empty() {
        render_chi_square_table(ui, res, selected);
        ui.add_space(12.0);
    }
    if !res.directional_measures.is_empty() {
        render_directional_measures_table(ui, res, selected);
        ui.add_space(12.0);
    }
    if !res.association_measures.is_empty() {
        render_symmetric_measures_table(ui, res, selected);
        ui.add_space(12.0);
    }
    if !res.risk_estimates.is_empty() {
        render_risk_estimate_table(ui, res, selected);
        ui.add_space(12.0);
    }
    if let Some(mc_sig) = res.mcnemar_sig {
        render_mcnemar_table(ui, res.n_valid, mc_sig, selected);
        ui.add_space(12.0);
    }
    if let Some(ref cmh) = res.cmh_result {
        render_cmh_tables(ui, cmh, selected);
    }
}

fn render_chi_square_table(ui: &mut Ui, res: &CrosstabResult, selected: bool) {
    let row_height = 20.0_f32;
    render_section_title(ui, t!("output-crosstabs-chisq-title"), selected);

    let has_mc = res
        .chi_square_tests
        .iter()
        .any(|r| r.monte_carlo_sig.is_some());

    let mut chisq_cols = vec![
        (t!("output-crosstabs-chisq-title"), 190.0_f32),
        (t!("output-crosstabs-value"), 85.0_f32),
        (t!("output-crosstabs-df"), 55.0_f32),
        (t!("output-crosstabs-asymp-sig"), 150.0_f32),
        (t!("output-crosstabs-exact-sig"), 140.0_f32),
    ];
    if has_mc {
        chisq_cols.push((t!("output-crosstabs-mc-sig"), 140.0_f32));
        chisq_cols.push((t!("output-crosstabs-mc-ci-lower"), 90.0_f32));
        chisq_cols.push((t!("output-crosstabs-mc-ci-upper"), 90.0_f32));
    }
    adjust_widths(ui, &mut chisq_cols);
    render_header_row(ui, &chisq_cols, row_height);

    for (i, t_row) in res.chi_square_tests.iter().enumerate() {
        let val_str = if t_row.name == "Fisher's Exact Test" {
            String::new()
        } else if t_row.name == "N of Valid Cases" {
            format!("{:.0}", t_row.value)
        } else {
            format!("{:.3}", t_row.value)
        };
        let df_str = t_row.df.map(|d| d.to_string()).unwrap_or_default();
        let mut r_vals = vec![
            (t_row.name.clone(), Align2::LEFT_CENTER),
            (val_str, Align2::RIGHT_CENTER),
            (df_str, Align2::RIGHT_CENTER),
            (format_sig(t_row.asymp_sig), Align2::RIGHT_CENTER),
            (format_sig(t_row.exact_sig), Align2::RIGHT_CENTER),
        ];
        if has_mc {
            r_vals.push((format_sig(t_row.monte_carlo_sig), Align2::RIGHT_CENTER));
            r_vals.push((format_sig(t_row.mc_ci_lower), Align2::RIGHT_CENTER));
            r_vals.push((format_sig(t_row.mc_ci_upper), Align2::RIGHT_CENTER));
        }
        render_data_row(ui, &chisq_cols, &r_vals, row_height, row_bg(i));
    }
}

fn render_directional_measures_table(ui: &mut Ui, res: &CrosstabResult, selected: bool) {
    let row_height = 20.0_f32;
    render_section_title(ui, t!("output-crosstabs-directional-title"), selected);

    let mut cols = vec![
        (t!("output-crosstabs-measure"), 180.0_f32),
        (t!("output-crosstabs-direction"), 160.0_f32),
        (t!("output-crosstabs-value"), 85.0_f32),
        (t!("output-crosstabs-asymp-se"), 140.0_f32),
        (t!("output-crosstabs-approx-t"), 110.0_f32),
        (t!("output-crosstabs-approx-sig"), 140.0_f32),
    ];
    adjust_widths(ui, &mut cols);
    render_header_row(ui, &cols, row_height);

    for (i, m) in res.directional_measures.iter().enumerate() {
        let se_str = m
            .asymp_std_error
            .map(|v| format!("{:.3}", v))
            .unwrap_or_else(|| "-".to_string());
        let t_str = m
            .approx_t
            .map(|v| format!("{:.3}", v))
            .unwrap_or_else(|| "-".to_string());
        let vals = vec![
            (m.name.clone(), Align2::LEFT_CENTER),
            (m.direction.clone(), Align2::LEFT_CENTER),
            (format!("{:.3}", m.value), Align2::RIGHT_CENTER),
            (se_str, Align2::RIGHT_CENTER),
            (t_str, Align2::RIGHT_CENTER),
            (format_sig(m.approx_sig), Align2::RIGHT_CENTER),
        ];
        render_data_row(ui, &cols, &vals, row_height, row_bg(i));
    }
}

fn render_symmetric_measures_table(ui: &mut Ui, res: &CrosstabResult, selected: bool) {
    let row_height = 20.0_f32;
    render_section_title(ui, t!("output-crosstabs-symmetric-title"), selected);

    let has_se_or_t = res
        .association_measures
        .iter()
        .any(|m| m.asymp_std_error.is_some() || m.approx_t.is_some());

    let mut sym_cols = vec![
        (t!("output-crosstabs-category"), 140.0_f32),
        (t!("output-crosstabs-measure"), 180.0_f32),
        (t!("output-crosstabs-value"), 85.0_f32),
    ];
    if has_se_or_t {
        sym_cols.push((t!("output-crosstabs-asymp-se"), 130.0_f32));
        sym_cols.push((t!("output-crosstabs-approx-t"), 105.0_f32));
    }
    sym_cols.push((t!("output-crosstabs-approx-sig"), 140.0_f32));
    adjust_widths(ui, &mut sym_cols);
    render_header_row(ui, &sym_cols, row_height);

    for (i, m) in res.association_measures.iter().enumerate() {
        let mut m_vals = vec![
            (m.category.clone(), Align2::LEFT_CENTER),
            (m.name.clone(), Align2::LEFT_CENTER),
            (format!("{:.3}", m.value), Align2::RIGHT_CENTER),
        ];
        if has_se_or_t {
            let se_str = m
                .asymp_std_error
                .map(|v| format!("{:.3}", v))
                .unwrap_or_else(|| "-".to_string());
            let t_str = m
                .approx_t
                .map(|v| format!("{:.3}", v))
                .unwrap_or_else(|| "-".to_string());
            m_vals.push((se_str, Align2::RIGHT_CENTER));
            m_vals.push((t_str, Align2::RIGHT_CENTER));
        }
        m_vals.push((format_sig(m.approx_sig), Align2::RIGHT_CENTER));
        render_data_row(ui, &sym_cols, &m_vals, row_height, row_bg(i));
    }
}

fn render_risk_estimate_table(ui: &mut Ui, res: &CrosstabResult, selected: bool) {
    let row_height = 20.0_f32;
    render_section_title(ui, t!("output-crosstabs-risk-title"), selected);

    let mut cols = vec![
        (t!("output-crosstabs-measure"), 240.0_f32),
        (t!("output-crosstabs-value"), 90.0_f32),
        (t!("output-crosstabs-ci-lower"), 100.0_f32),
        (t!("output-crosstabs-ci-upper"), 100.0_f32),
    ];
    adjust_widths(ui, &mut cols);
    render_header_row(ui, &cols, row_height);

    for (i, r) in res.risk_estimates.iter().enumerate() {
        let lo = r
            .ci_lower
            .map(|v| format!("{:.3}", v))
            .unwrap_or_else(|| "-".to_string());
        let hi = r
            .ci_upper
            .map(|v| format!("{:.3}", v))
            .unwrap_or_else(|| "-".to_string());
        let vals = vec![
            (r.name.clone(), Align2::LEFT_CENTER),
            (format!("{:.3}", r.value), Align2::RIGHT_CENTER),
            (lo, Align2::RIGHT_CENTER),
            (hi, Align2::RIGHT_CENTER),
        ];
        render_data_row(ui, &cols, &vals, row_height, row_bg(i));
    }
}

fn render_mcnemar_table(ui: &mut Ui, n_valid: usize, sig: f64, selected: bool) {
    let row_height = 20.0_f32;
    render_section_title(ui, t!("output-crosstabs-mcnemar-title"), selected);

    let mut cols = vec![
        (t!("output-crosstabs-measure"), 200.0_f32),
        (t!("output-crosstabs-value"), 100.0_f32),
        (t!("output-crosstabs-exact-sig"), 150.0_f32),
    ];
    adjust_widths(ui, &mut cols);
    render_header_row(ui, &cols, row_height);

    let vals = vec![
        (t!("output-crosstabs-mcnemar-title"), Align2::LEFT_CENTER),
        (n_valid.to_string(), Align2::RIGHT_CENTER),
        (format_sig(Some(sig)), Align2::RIGHT_CENTER),
    ];
    render_data_row(ui, &cols, &vals, row_height, Tokens::bg_row_even());
}

fn render_cmh_tables(ui: &mut Ui, cmh: &CmhResult, selected: bool) {
    let row_height = 20.0_f32;

    // 1. Homogeneity of Odds Ratio (Breslow-Day & Tarone's)
    render_section_title(ui, t!("output-crosstabs-homogeneity-title"), selected);
    let mut hom_cols = vec![
        (t!("output-crosstabs-measure"), 180.0_f32),
        (t!("output-crosstabs-stat"), 100.0_f32),
        (t!("output-crosstabs-df"), 60.0_f32),
        (t!("output-crosstabs-asymp-sig"), 150.0_f32),
    ];
    adjust_widths(ui, &mut hom_cols);
    render_header_row(ui, &hom_cols, row_height);

    let hom_rows = [
        (
            "Breslow-Day",
            cmh.breslow_day_stat,
            cmh.breslow_day_df,
            cmh.breslow_day_sig,
        ),
        ("Tarone's", cmh.tarone_stat, cmh.tarone_df, cmh.tarone_sig),
    ];
    for (i, (name, stat, df, sig)) in hom_rows.iter().enumerate() {
        let vals = vec![
            ((*name).to_string(), Align2::LEFT_CENTER),
            (format!("{:.3}", stat), Align2::RIGHT_CENTER),
            (df.to_string(), Align2::RIGHT_CENTER),
            (format_sig(Some(*sig)), Align2::RIGHT_CENTER),
        ];
        render_data_row(ui, &hom_cols, &vals, row_height, row_bg(i));
    }

    ui.add_space(12.0);

    // 2. Conditional Independence (Cochran's & Mantel-Haenszel)
    render_section_title(ui, t!("output-crosstabs-cmh-title"), selected);
    let mut cmh_cols = vec![
        (t!("output-crosstabs-measure"), 180.0_f32),
        (t!("output-crosstabs-stat"), 100.0_f32),
        (t!("output-crosstabs-df"), 60.0_f32),
        (t!("output-crosstabs-asymp-sig"), 150.0_f32),
    ];
    adjust_widths(ui, &mut cmh_cols);
    render_header_row(ui, &cmh_cols, row_height);

    let cmh_rows = [
        (
            "Cochran's",
            cmh.cochran_stat,
            cmh.cochran_df,
            cmh.cochran_sig,
        ),
        ("Mantel-Haenszel", cmh.mh_stat, cmh.mh_df, cmh.mh_sig),
    ];
    for (i, (name, stat, df, sig)) in cmh_rows.iter().enumerate() {
        let vals = vec![
            ((*name).to_string(), Align2::LEFT_CENTER),
            (format!("{:.3}", stat), Align2::RIGHT_CENTER),
            (df.to_string(), Align2::RIGHT_CENTER),
            (format_sig(Some(*sig)), Align2::RIGHT_CENTER),
        ];
        render_data_row(ui, &cmh_cols, &vals, row_height, row_bg(i));
    }

    ui.add_space(12.0);

    // 3. Common Odds Ratio Estimate
    render_section_title(ui, t!("output-crosstabs-common-or-title"), selected);
    let mut or_cols = vec![
        (t!("output-crosstabs-value"), 90.0_f32),
        (t!("output-crosstabs-ln-se"), 120.0_f32),
        (t!("output-crosstabs-ci-lower"), 100.0_f32),
        (t!("output-crosstabs-ci-upper"), 100.0_f32),
        (t!("output-crosstabs-stat"), 90.0_f32),
        (t!("output-crosstabs-asymp-sig"), 140.0_f32),
    ];
    adjust_widths(ui, &mut or_cols);
    render_header_row(ui, &or_cols, row_height);

    let or_vals = vec![
        (
            format!("{:.3}", cmh.common_or_estimate),
            Align2::RIGHT_CENTER,
        ),
        (format!("{:.3}", cmh.common_or_ln_se), Align2::RIGHT_CENTER),
        (
            format!("{:.3}", cmh.common_or_ci_lower),
            Align2::RIGHT_CENTER,
        ),
        (
            format!("{:.3}", cmh.common_or_ci_upper),
            Align2::RIGHT_CENTER,
        ),
        (
            format!("{:.3}", cmh.common_or_test_val),
            Align2::RIGHT_CENTER,
        ),
        (format_sig(Some(cmh.common_or_sig)), Align2::RIGHT_CENTER),
    ];
    render_data_row(ui, &or_cols, &or_vals, row_height, Tokens::bg_row_even());
}
