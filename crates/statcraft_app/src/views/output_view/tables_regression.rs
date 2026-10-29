use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{
    adjust_widths, format_opt, format_sig, render_data_row, render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::LinearRegressionResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_linear_regression(&mut self, result: LinearRegressionResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = statcraft_syntax::commands::LinearRegressionCommand::to_sps(
            &result.dep_var,
            &result.indep_vars,
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.n_valid,
            active_vars: result.indep_vars.len() + 1,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-regression-title",
            custom_title: None,
            is_expanded: true,
            items: vec![
                OutputItem {
                    id: log_id,
                    title_key: "output-log",
                    custom_title: None,
                    kind: OutputItemKind::Log(sps_log),
                },
                OutputItem {
                    id: notes_id,
                    title_key: "output-notes",
                    custom_title: None,
                    kind: OutputItemKind::Notes(notes),
                },
                OutputItem {
                    id: item_id,
                    title_key: "output-regression-table-title",
                    custom_title: None,
                    kind: OutputItemKind::LinearRegression(Box::new(result)),
                },
            ],
        };

        self.procedures.push(proc);
        let new_proc_idx = self.procedures.len() - 1;
        self.selected_target = Some(SelectedTarget::Item {
            proc_idx: new_proc_idx,
            item_idx: 2,
        });
        self.scroll_to_target = self.selected_target;
    }
}

pub fn render_spss_linear_regression(ui: &mut Ui, res: &LinearRegressionResult) {
    ui.vertical(|ui| {
        // Table 1: Variables Entered/Removed
        render_variables_entered_table(ui, res);
        ui.add_space(14.0);

        // Table 2: Model Summary
        render_model_summary_table(ui, res);
        ui.add_space(14.0);

        // Table 3: ANOVA
        render_anova_table(ui, res);
        ui.add_space(14.0);

        // Table 4: Coefficients
        render_coefficients_table(ui, res);
        ui.add_space(16.0);
    });
}

fn render_variables_entered_table(ui: &mut Ui, res: &LinearRegressionResult) {
    ui.strong(t!("output-regression-variables-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-model"), 55.0),
        (t!("output-regression-vars-entered"), 200.0),
        (t!("output-regression-vars-removed"), 120.0),
        (t!("output-regression-method"), 90.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0;
    ScrollArea::horizontal()
        .id_salt(format!("reg_vars_scroll_{}", res.dep_var))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                let vals: Vec<(String, Align2)> = vec![
                    ("1".to_string(), Align2::CENTER_CENTER),
                    (res.indep_vars.join(", "), Align2::LEFT_CENTER),
                    (t!("output-regression-none"), Align2::CENTER_CENTER),
                    (res.method.clone(), Align2::CENTER_CENTER),
                ];
                render_data_row(ui, &cols, &vals, row_height, Tokens::BG_ROW_EVEN);
            });
        });

    ui.add_space(4.0);
    ui.label(format!(
        "a. {}: {}",
        t!("output-regression-dep-var-note"),
        res.dep_var
    ));
    ui.label(format!("b. {}", t!("output-regression-all-vars-note")));
}

fn render_model_summary_table(ui: &mut Ui, res: &LinearRegressionResult) {
    ui.strong(t!("output-regression-model-summary-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-model"), 55.0),
        (t!("output-regression-col-r"), 75.0),
        (t!("output-regression-col-r-sq"), 85.0),
        (t!("output-regression-col-adj-r-sq"), 110.0),
        (t!("output-regression-col-se-est"), 120.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0;
    ScrollArea::horizontal()
        .id_salt(format!("reg_summary_scroll_{}", res.dep_var))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                let vals: Vec<(String, Align2)> = vec![
                    ("1".to_string(), Align2::CENTER_CENTER),
                    (format!("{:.3}", res.r), Align2::RIGHT_CENTER),
                    (format!("{:.3}", res.r_squared), Align2::RIGHT_CENTER),
                    (format!("{:.3}", res.adj_r_squared), Align2::RIGHT_CENTER),
                    (
                        format!("{:.4}", res.std_error_estimate),
                        Align2::RIGHT_CENTER,
                    ),
                ];
                render_data_row(ui, &cols, &vals, row_height, Tokens::BG_ROW_EVEN);
            });
        });

    ui.add_space(4.0);
    ui.label(format!(
        "a. {}: {}, {}",
        t!("output-regression-predictors-note"),
        t!("output-regression-constant"),
        res.indep_vars.join(", ")
    ));
}

fn render_anova_table(ui: &mut Ui, res: &LinearRegressionResult) {
    ui.strong(t!("output-regression-anova-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-model"), 95.0),
        (t!("output-col-sum-of-squares"), 110.0),
        (t!("output-col-df"), 55.0),
        (t!("output-col-mean-square"), 100.0),
        (t!("output-col-f"), 75.0),
        (t!("output-col-sig"), 75.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 22.0;
    ScrollArea::horizontal()
        .id_salt(format!("reg_anova_scroll_{}", res.dep_var))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);

                // Row 1: Regression
                let vals_reg: Vec<(String, Align2)> = vec![
                    (t!("output-regression-row-regression"), Align2::LEFT_CENTER),
                    (
                        format_opt(Some(res.anova.regression.sum_of_squares)),
                        Align2::RIGHT_CENTER,
                    ),
                    (res.anova.regression.df.to_string(), Align2::RIGHT_CENTER),
                    (
                        format_opt(res.anova.regression.mean_square),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        format_opt(res.anova.regression.f_stat),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        format_sig(res.anova.regression.p_value),
                        Align2::RIGHT_CENTER,
                    ),
                ];
                render_data_row(ui, &cols, &vals_reg, row_height, Tokens::BG_ROW_EVEN);

                // Row 2: Residual
                let vals_res: Vec<(String, Align2)> = vec![
                    (t!("output-regression-row-residual"), Align2::LEFT_CENTER),
                    (
                        format_opt(Some(res.anova.residual.sum_of_squares)),
                        Align2::RIGHT_CENTER,
                    ),
                    (res.anova.residual.df.to_string(), Align2::RIGHT_CENTER),
                    (
                        format_opt(res.anova.residual.mean_square),
                        Align2::RIGHT_CENTER,
                    ),
                    (String::new(), Align2::CENTER_CENTER),
                    (String::new(), Align2::CENTER_CENTER),
                ];
                render_data_row(ui, &cols, &vals_res, row_height, Tokens::BG_ROW_ODD);

                // Row 3: Total
                let vals_tot: Vec<(String, Align2)> = vec![
                    (t!("output-col-total"), Align2::LEFT_CENTER),
                    (
                        format_opt(Some(res.anova.total.sum_of_squares)),
                        Align2::RIGHT_CENTER,
                    ),
                    (res.anova.total.df.to_string(), Align2::RIGHT_CENTER),
                    (String::new(), Align2::CENTER_CENTER),
                    (String::new(), Align2::CENTER_CENTER),
                    (String::new(), Align2::CENTER_CENTER),
                ];
                render_data_row(ui, &cols, &vals_tot, row_height, Tokens::BG_ROW_EVEN);
            });
        });

    ui.add_space(4.0);
    ui.label(format!(
        "a. {}: {}",
        t!("output-regression-dep-var-note"),
        res.dep_var
    ));
    ui.label(format!(
        "b. {}: {}, {}",
        t!("output-regression-predictors-note"),
        t!("output-regression-constant"),
        res.indep_vars.join(", ")
    ));
}

fn render_coefficients_table(ui: &mut Ui, res: &LinearRegressionResult) {
    ui.strong(t!("output-regression-coeffs-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-model"), 110.0),
        (t!("output-regression-col-b"), 80.0),
        (t!("output-regression-col-se"), 80.0),
        (t!("output-regression-col-beta"), 80.0),
        (t!("output-col-t"), 70.0),
        (t!("output-col-sig"), 70.0),
        (t!("output-col-ci-lower"), 85.0),
        (t!("output-col-ci-upper"), 85.0),
    ];
    if res.options.collinearity {
        cols.push((t!("output-regression-col-tol"), 80.0));
        cols.push((t!("output-regression-col-vif"), 80.0));
    }
    adjust_widths(ui, &mut cols);

    let row_height = 22.0;
    ScrollArea::horizontal()
        .id_salt(format!("reg_coeff_scroll_{}", res.dep_var))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);

                for (idx, coeff) in res.coefficients.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::BG_ROW_EVEN
                    } else {
                        Tokens::BG_ROW_ODD
                    };

                    let var_display = if coeff.var_name == "(Constant)" {
                        t!("output-regression-constant")
                    } else {
                        coeff.var_name.clone()
                    };

                    let mut vals: Vec<(String, Align2)> = vec![
                        (var_display, Align2::LEFT_CENTER),
                        (format!("{:.4}", coeff.b), Align2::RIGHT_CENTER),
                        (format!("{:.4}", coeff.std_error), Align2::RIGHT_CENTER),
                        (
                            coeff.beta.map(|b| format!("{:.3}", b)).unwrap_or_default(),
                            Align2::RIGHT_CENTER,
                        ),
                        (format!("{:.3}", coeff.t_stat), Align2::RIGHT_CENTER),
                        (format_sig(Some(coeff.p_value)), Align2::RIGHT_CENTER),
                        (format!("{:.4}", coeff.ci_lower), Align2::RIGHT_CENTER),
                        (format!("{:.4}", coeff.ci_upper), Align2::RIGHT_CENTER),
                    ];

                    if res.options.collinearity {
                        vals.push((
                            coeff
                                .tolerance
                                .map(|t| format!("{:.3}", t))
                                .unwrap_or_default(),
                            Align2::RIGHT_CENTER,
                        ));
                        vals.push((
                            coeff.vif.map(|v| format!("{:.3}", v)).unwrap_or_default(),
                            Align2::RIGHT_CENTER,
                        ));
                    }

                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
            });
        });

    ui.add_space(4.0);
    ui.label(format!(
        "a. {}: {}",
        t!("output-regression-dep-var-note"),
        res.dep_var
    ));
}

pub fn regression_to_tsv(res: &LinearRegressionResult) -> String {
    let mut out = String::new();

    // 1. Model Summary
    out.push_str(&format!(
        "{}\n",
        t!("output-regression-model-summary-title")
    ));
    out.push_str(&format!(
        "{}\t{}\t{}\t{}\t{}\n",
        t!("output-col-model"),
        t!("output-regression-col-r"),
        t!("output-regression-col-r-sq"),
        t!("output-regression-col-adj-r-sq"),
        t!("output-regression-col-se-est")
    ));
    out.push_str(&format!(
        "1\t{:.4}\t{:.4}\t{:.4}\t{:.4}\n\n",
        res.r, res.r_squared, res.adj_r_squared, res.std_error_estimate
    ));

    // 2. ANOVA
    out.push_str(&format!("{}\n", t!("output-regression-anova-title")));
    out.push_str(&format!(
        "{}\t{}\t{}\t{}\t{}\t{}\n",
        t!("output-col-model"),
        t!("output-col-sum-of-squares"),
        t!("output-col-df"),
        t!("output-col-mean-square"),
        t!("output-col-f"),
        t!("output-col-sig")
    ));
    out.push_str(&format!(
        "{}\t{:.4}\t{}\t{:.4}\t{:.4}\t{:.4}\n",
        t!("output-regression-row-regression"),
        res.anova.regression.sum_of_squares,
        res.anova.regression.df,
        res.anova.regression.mean_square.unwrap_or(0.0),
        res.anova.regression.f_stat.unwrap_or(0.0),
        res.anova.regression.p_value.unwrap_or(0.0)
    ));
    out.push_str(&format!(
        "{}\t{:.4}\t{}\t{:.4}\t\t\n",
        t!("output-regression-row-residual"),
        res.anova.residual.sum_of_squares,
        res.anova.residual.df,
        res.anova.residual.mean_square.unwrap_or(0.0)
    ));
    out.push_str(&format!(
        "{}\t{:.4}\t{}\t\t\t\n\n",
        t!("output-col-total"),
        res.anova.total.sum_of_squares,
        res.anova.total.df
    ));

    // 3. Coefficients
    out.push_str(&format!("{}\n", t!("output-regression-coeffs-title")));
    out.push_str(&format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        t!("output-col-model"),
        t!("output-regression-col-b"),
        t!("output-regression-col-se"),
        t!("output-regression-col-beta"),
        t!("output-col-t"),
        t!("output-col-sig"),
        t!("output-col-ci-lower"),
        t!("output-col-ci-upper")
    ));
    for c in &res.coefficients {
        out.push_str(&format!(
            "{}\t{:.4}\t{:.4}\t{}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\n",
            c.var_name,
            c.b,
            c.std_error,
            c.beta.map(|b| format!("{:.4}", b)).unwrap_or_default(),
            c.t_stat,
            c.p_value,
            c.ci_lower,
            c.ci_upper
        ));
    }

    out
}
