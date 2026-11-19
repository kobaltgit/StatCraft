use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::MultinomialLogisticResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_multinomial_logistic(&mut self, result: MultinomialLogisticResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let ref_cat_str = if result.reference_category_val.fract().abs() < 1e-6 {
            format!("{}", result.reference_category_val as i64)
        } else {
            format!("{:.2}", result.reference_category_val)
        };

        let sps_log = statcraft_syntax::commands::MultinomialLogisticCommand::to_sps(
            &result.dependent_var,
            &result.factors,
            &result.covariates,
            &ref_cat_str,
            0.95,
        );

        let notes = OutputNotes {
            dataset_name: self.active_dataset_name.clone(),
            timestamp: "Session".to_string(),
            active_cases: result.n_cases,
            active_vars: result.factors.len() + result.covariates.len() + 1,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-multinomial-title",
            custom_title: None,
            dataset_name: Some(self.active_dataset_name.clone()),
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
                    title_key: "output-multinomial-table-title",
                    custom_title: None,
                    kind: OutputItemKind::MultinomialLogistic(Box::new(result)),
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

pub fn render_spss_multinomial_logistic_tables(ui: &mut Ui, res: &MultinomialLogisticResult) {
    ui.vertical(|ui| {
        render_case_processing_table(ui, res);
        ui.add_space(14.0);

        render_model_fitting_table(ui, res);
        ui.add_space(14.0);

        if let Some(ref gof) = res.goodness_of_fit {
            render_goodness_of_fit_table(ui, gof);
            ui.add_space(14.0);
        }

        render_pseudo_r2_table(ui, res);
        ui.add_space(14.0);

        if !res.likelihood_ratio_tests.is_empty() {
            render_lrt_table(ui, res);
            ui.add_space(14.0);
        }

        render_parameter_estimates_table(ui, res);
        ui.add_space(14.0);

        if let Some(ref ct) = res.classification_table {
            render_classification_table(ui, ct);
            ui.add_space(16.0);
        }
    });
}

fn render_case_processing_table(ui: &mut Ui, res: &MultinomialLogisticResult) {
    ui.strong(t!("output-multinomial-case-processing"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        ("".to_string(), 120.0),
        ("N".to_string(), 80.0),
        (t!("output-multinomial-marginal-percent"), 140.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("multinomial_case_proc")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 24.0);
                for cat in &res.case_processing.categories {
                    let fallback = cat.category_value.to_string();
                    let label_text = cat.category_label.as_deref().unwrap_or(&fallback);
                    let row = [
                        (label_text.to_string(), Align2::LEFT_CENTER),
                        (cat.n.to_string(), Align2::RIGHT_CENTER),
                        (format!("{:.1}%", cat.percent), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &row, 20.0, Tokens::bg_panel());
                }
                let valid_row = [
                    (t!("output-common-valid"), Align2::LEFT_CENTER),
                    (
                        res.case_processing.valid_n.to_string(),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        format!("{:.1}%", res.case_processing.valid_percent),
                        Align2::RIGHT_CENTER,
                    ),
                ];
                render_data_row(ui, &cols, &valid_row, 20.0, Tokens::bg_header());

                let missing_pct = 100.0 - res.case_processing.valid_percent;
                let missing_row = [
                    (t!("output-multinomial-missing"), Align2::LEFT_CENTER),
                    (
                        res.case_processing.missing_n.to_string(),
                        Align2::RIGHT_CENTER,
                    ),
                    (format!("{:.1}%", missing_pct), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &missing_row, 20.0, Tokens::bg_panel());

                let total_row = [
                    (t!("output-common-total"), Align2::LEFT_CENTER),
                    (
                        res.case_processing.total_n.to_string(),
                        Align2::RIGHT_CENTER,
                    ),
                    ("100.0%".to_string(), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &total_row, 20.0, Tokens::bg_header());
            });
        });
}

fn render_model_fitting_table(ui: &mut Ui, res: &MultinomialLogisticResult) {
    ui.strong(t!("output-multinomial-model-fitting"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-multinomial-model"), 110.0),
        ("-2 Log Likelihood".to_string(), 120.0),
        ("Chi-Square".to_string(), 90.0),
        (t!("output-common-df"), 60.0),
        (t!("output-common-sig"), 70.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("multinomial_model_fit")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 24.0);
                let row_intercept = [
                    (
                        t!("output-multinomial-model-intercept"),
                        Align2::LEFT_CENTER,
                    ),
                    (
                        format!("{:.3}", res.model_fitting.intercept_only_minus_2ll),
                        Align2::RIGHT_CENTER,
                    ),
                    ("".to_string(), Align2::RIGHT_CENTER),
                    ("".to_string(), Align2::RIGHT_CENTER),
                    ("".to_string(), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &row_intercept, 20.0, Tokens::bg_panel());

                let row_final = [
                    (t!("output-multinomial-model-final"), Align2::LEFT_CENTER),
                    (
                        format!("{:.3}", res.model_fitting.final_minus_2ll),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        format!("{:.3}", res.model_fitting.chi_square),
                        Align2::RIGHT_CENTER,
                    ),
                    (res.model_fitting.df.to_string(), Align2::RIGHT_CENTER),
                    (
                        format_sig(Some(res.model_fitting.sig)),
                        Align2::RIGHT_CENTER,
                    ),
                ];
                render_data_row(ui, &cols, &row_final, 20.0, Tokens::bg_header());
            });
        });
}

fn render_goodness_of_fit_table(ui: &mut Ui, gof: &statcraft_algo::GoodnessOfFit) {
    ui.strong(t!("output-multinomial-goodness-of-fit"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        ("".to_string(), 90.0),
        ("Chi-Square".to_string(), 90.0),
        (t!("output-common-df"), 60.0),
        (t!("output-common-sig"), 70.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("multinomial_gof")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 24.0);
                let row_p = [
                    ("Pearson".to_string(), Align2::LEFT_CENTER),
                    (
                        format!("{:.3}", gof.pearson_chi_square),
                        Align2::RIGHT_CENTER,
                    ),
                    (gof.pearson_df.to_string(), Align2::RIGHT_CENTER),
                    (format_sig(Some(gof.pearson_sig)), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &row_p, 20.0, Tokens::bg_panel());

                let row_d = [
                    ("Deviance".to_string(), Align2::LEFT_CENTER),
                    (
                        format!("{:.3}", gof.deviance_chi_square),
                        Align2::RIGHT_CENTER,
                    ),
                    (gof.deviance_df.to_string(), Align2::RIGHT_CENTER),
                    (format_sig(Some(gof.deviance_sig)), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &row_d, 20.0, Tokens::bg_panel());
            });
        });
}

fn render_pseudo_r2_table(ui: &mut Ui, res: &MultinomialLogisticResult) {
    ui.strong(t!("output-multinomial-pseudo-r2"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-multinomial-cox-snell"), 110.0),
        (t!("output-multinomial-nagelkerke"), 110.0),
        (t!("output-multinomial-mcfadden"), 110.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("multinomial_r2")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 24.0);
                let row = [
                    (
                        format!("{:.3}", res.pseudo_r_squared.cox_and_snell),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        format!("{:.3}", res.pseudo_r_squared.nagelkerke),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        format!("{:.3}", res.pseudo_r_squared.mcfadden),
                        Align2::RIGHT_CENTER,
                    ),
                ];
                render_data_row(ui, &cols, &row, 20.0, Tokens::bg_panel());
            });
        });
}

fn render_lrt_table(ui: &mut Ui, res: &MultinomialLogisticResult) {
    ui.strong(t!("output-multinomial-lrt"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-multinomial-effect"), 120.0),
        ("-2 Log Likelihood".to_string(), 120.0),
        ("Chi-Square".to_string(), 90.0),
        (t!("output-common-df"), 60.0),
        (t!("output-common-sig"), 70.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("multinomial_lrt")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 24.0);
                for lrt in &res.likelihood_ratio_tests {
                    let row = [
                        (lrt.effect_name.clone(), Align2::LEFT_CENTER),
                        (
                            format!("{:.3}", lrt.model_fitting_criteria_minus_2ll),
                            Align2::RIGHT_CENTER,
                        ),
                        (format!("{:.3}", lrt.chi_square), Align2::RIGHT_CENTER),
                        (lrt.df.to_string(), Align2::RIGHT_CENTER),
                        (format_sig(Some(lrt.sig)), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &row, 20.0, Tokens::bg_panel());
                }
            });
        });
}

fn render_parameter_estimates_table(ui: &mut Ui, res: &MultinomialLogisticResult) {
    ui.strong(t!("output-multinomial-param-estimates"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-multinomial-category"), 90.0),
        (t!("output-common-variable"), 110.0),
        ("B".to_string(), 70.0),
        (t!("output-multinomial-std-error"), 80.0),
        (t!("output-multinomial-wald"), 75.0),
        (t!("output-common-df"), 50.0),
        (t!("output-common-sig"), 65.0),
        (t!("output-multinomial-exp-b"), 80.0),
        (t!("output-multinomial-ci-lower"), 80.0),
        (t!("output-multinomial-ci-upper"), 80.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("multinomial_params")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 24.0);
                for p in &res.parameter_estimates {
                    let fallback = p.category_value.to_string();
                    let cat_str = p.category_label.as_deref().unwrap_or(&fallback).to_string();

                    let row = if p.is_reference_category {
                        [
                            (cat_str, Align2::LEFT_CENTER),
                            (p.param_name.clone(), Align2::LEFT_CENTER),
                            ("0".to_string(), Align2::RIGHT_CENTER),
                            (".".to_string(), Align2::RIGHT_CENTER),
                            (".".to_string(), Align2::RIGHT_CENTER),
                            ("0".to_string(), Align2::RIGHT_CENTER),
                            (".".to_string(), Align2::RIGHT_CENTER),
                            ("1".to_string(), Align2::RIGHT_CENTER),
                            (".".to_string(), Align2::RIGHT_CENTER),
                            (".".to_string(), Align2::RIGHT_CENTER),
                        ]
                    } else {
                        [
                            (cat_str, Align2::LEFT_CENTER),
                            (p.param_name.clone(), Align2::LEFT_CENTER),
                            (format!("{:.3}", p.b), Align2::RIGHT_CENTER),
                            (format!("{:.3}", p.std_err), Align2::RIGHT_CENTER),
                            (format!("{:.3}", p.wald), Align2::RIGHT_CENTER),
                            (p.df.to_string(), Align2::RIGHT_CENTER),
                            (format_sig(Some(p.sig)), Align2::RIGHT_CENTER),
                            (format!("{:.3}", p.exp_b), Align2::RIGHT_CENTER),
                            (format!("{:.3}", p.ci_lower), Align2::RIGHT_CENTER),
                            (format!("{:.3}", p.ci_upper), Align2::RIGHT_CENTER),
                        ]
                    };
                    let bg = if p.is_reference_category {
                        Tokens::bg_header()
                    } else {
                        Tokens::bg_panel()
                    };
                    render_data_row(ui, &cols, &row, 20.0, bg);
                }
            });
        });
}

fn render_classification_table(ui: &mut Ui, ct: &statcraft_algo::MultinomialClassificationTable) {
    ui.strong(t!("output-multinomial-classification"));
    ui.add_space(4.0);

    let num_cat = ct.categories.len();
    let mut cols: Vec<(String, f32)> = Vec::with_capacity(num_cat + 2);
    cols.push((t!("output-multinomial-observed"), 100.0));
    for (j, cat) in ct.categories.iter().enumerate() {
        let fallback = cat.to_string();
        let label = ct
            .category_labels
            .get(j)
            .and_then(|l| l.as_deref())
            .unwrap_or(&fallback);
        cols.push((label.to_string(), 75.0));
    }
    cols.push((t!("output-multinomial-percent-correct"), 120.0));
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("multinomial_classtbl")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 24.0);
                for r in &ct.rows {
                    let fallback = r.observed_category.to_string();
                    let obs_label = r.observed_label.as_deref().unwrap_or(&fallback);
                    let mut row: Vec<(String, Align2)> = Vec::with_capacity(num_cat + 2);
                    row.push((obs_label.to_string(), Align2::LEFT_CENTER));
                    for cnt in &r.predicted_counts {
                        row.push((cnt.to_string(), Align2::RIGHT_CENTER));
                    }
                    row.push((format!("{:.1}%", r.percent_correct), Align2::RIGHT_CENTER));
                    render_data_row(ui, &cols, &row, 20.0, Tokens::bg_panel());
                }
                let mut overall_row: Vec<(String, Align2)> = Vec::with_capacity(num_cat + 2);
                overall_row.push((
                    t!("output-multinomial-overall-percentage"),
                    Align2::LEFT_CENTER,
                ));
                for _ in 0..num_cat {
                    overall_row.push(("".to_string(), Align2::RIGHT_CENTER));
                }
                overall_row.push((format!("{:.1}%", ct.overall_percent), Align2::RIGHT_CENTER));
                render_data_row(ui, &cols, &overall_row, 20.0, Tokens::bg_header());
            });
        });
}

pub fn multinomial_logistic_to_tsv(res: &MultinomialLogisticResult) -> String {
    let mut out = String::new();
    out.push_str("Multinomial Logistic Regression\n");
    out.push_str(&format!("Dependent Variable:\t{}\n", res.dependent_var));
    out.push_str(&format!("Valid Cases:\t{}\n\n", res.n_cases));

    out.push_str("Model Fitting Information\n");
    out.push_str("Model\t-2 Log Likelihood\tChi-Square\tdf\tSig.\n");
    out.push_str(&format!(
        "Intercept Only\t{:.3}\t\t\t\n",
        res.model_fitting.intercept_only_minus_2ll
    ));
    out.push_str(&format!(
        "Final\t{:.3}\t{:.3}\t{}\t{:.4}\n\n",
        res.model_fitting.final_minus_2ll,
        res.model_fitting.chi_square,
        res.model_fitting.df,
        res.model_fitting.sig
    ));

    out.push_str("Parameter Estimates\n");
    out.push_str("Category\tVariable\tB\tStd. Error\tWald\tdf\tSig.\tExp(B)\tLower\tUpper\n");
    for p in &res.parameter_estimates {
        let fallback = p.category_value.to_string();
        let cat_str = p.category_label.as_deref().unwrap_or(&fallback);
        if p.is_reference_category {
            out.push_str(&format!(
                "{}\t{}\t0\t.\t.\t0\t.\t1\t.\t.\n",
                cat_str, p.param_name
            ));
        } else {
            out.push_str(&format!(
                "{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{}\t{:.4}\t{:.3}\t{:.3}\t{:.3}\n",
                cat_str,
                p.param_name,
                p.b,
                p.std_err,
                p.wald,
                p.df,
                p.sig,
                p.exp_b,
                p.ci_lower,
                p.ci_upper
            ));
        }
    }

    out
}
