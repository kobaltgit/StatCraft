use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::LogisticRegressionResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_logistic_regression(&mut self, result: LogisticRegressionResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = statcraft_syntax::commands::LogisticRegressionCommand::to_sps(
            &result.dependent_var,
            &result.covariates,
            result.classification_table.cutoff,
            0.95,
            result.hosmer_lemeshow.is_some(),
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.n_cases,
            active_vars: result.covariates.len() + 1,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-logistic-title",
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
                    title_key: "output-logistic-table-title",
                    custom_title: None,
                    kind: OutputItemKind::LogisticRegression(Box::new(result)),
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

pub fn render_spss_logistic_tables(ui: &mut Ui, res: &LogisticRegressionResult) {
    ui.vertical(|ui| {
        // 1. Omnibus Tests
        render_omnibus_table(ui, res);
        ui.add_space(14.0);

        // 2. Model Summary
        render_model_summary_table(ui, res);
        ui.add_space(14.0);

        // 3. Hosmer-Lemeshow (if computed)
        if let Some(ref hl) = res.hosmer_lemeshow {
            render_hosmer_lemeshow_table(ui, hl);
            ui.add_space(14.0);
        }

        // 4. Classification Table
        render_classification_table(ui, res);
        ui.add_space(14.0);

        // 5. Variables in the Equation
        render_variables_in_equation_table(ui, res);
        ui.add_space(16.0);
    });
}

fn render_omnibus_table(ui: &mut Ui, res: &LogisticRegressionResult) {
    ui.strong(t!("output-logistic-omnibus-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        ("".to_string(), 80.0),
        ("".to_string(), 80.0),
        (t!("output-logistic-col-chi-square"), 110.0),
        (t!("output-logistic-col-df"), 60.0),
        (t!("output-logistic-col-sig"), 80.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 20.0;
    ScrollArea::horizontal()
        .id_salt("logistic_omnibus_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);

                let rows = [
                    t!("output-logistic-step-row"),
                    t!("output-logistic-block-row"),
                    t!("output-logistic-model-row"),
                ];

                for (idx, row_name) in rows.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let vals = [
                        (
                            if idx == 0 {
                                t!("output-logistic-step")
                            } else {
                                "".to_string()
                            },
                            Align2::LEFT_CENTER,
                        ),
                        (row_name.clone(), Align2::LEFT_CENTER),
                        (
                            format!("{:.3}", res.omnibus.chi_square),
                            Align2::RIGHT_CENTER,
                        ),
                        (format!("{}", res.omnibus.df), Align2::RIGHT_CENTER),
                        (format_sig(Some(res.omnibus.sig)), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
            });
        });
}

fn render_model_summary_table(ui: &mut Ui, res: &LogisticRegressionResult) {
    ui.strong(t!("output-logistic-model-summary-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-logistic-step"), 80.0),
        (t!("output-logistic-col-2ll"), 140.0),
        (t!("output-logistic-col-cox-snell"), 150.0),
        (t!("output-logistic-col-nagelkerke"), 150.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 20.0;
    ScrollArea::horizontal()
        .id_salt("logistic_summary_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                let vals = [
                    ("1".to_string(), Align2::CENTER_CENTER),
                    (
                        format!("{:.3}", res.model_summary.minus_2ll),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        format!("{:.3}", res.model_summary.cox_snell_r2),
                        Align2::RIGHT_CENTER,
                    ),
                    (
                        format!("{:.3}", res.model_summary.nagelkerke_r2),
                        Align2::RIGHT_CENTER,
                    ),
                ];
                render_data_row(ui, &cols, &vals, row_height, Tokens::bg_row_even());
            });
        });
}

fn render_hosmer_lemeshow_table(ui: &mut Ui, hl: &statcraft_algo::HosmerLemeshowResult) {
    ui.strong(t!("output-logistic-hosmer-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-logistic-step"), 80.0),
        (t!("output-logistic-col-chi-square"), 110.0),
        (t!("output-logistic-col-df"), 60.0),
        (t!("output-logistic-col-sig"), 80.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 20.0;
    ScrollArea::horizontal()
        .id_salt("logistic_hl_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                let vals = [
                    ("1".to_string(), Align2::CENTER_CENTER),
                    (format!("{:.3}", hl.chi_square), Align2::RIGHT_CENTER),
                    (format!("{}", hl.df), Align2::RIGHT_CENTER),
                    (format_sig(Some(hl.sig)), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &vals, row_height, Tokens::bg_row_even());
            });
        });
}

fn render_classification_table(ui: &mut Ui, res: &LogisticRegressionResult) {
    ui.strong(t!("output-logistic-classification-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-logistic-observed"), 120.0),
        (format!("{} 0", t!("output-logistic-predicted")), 90.0),
        (format!("{} 1", t!("output-logistic-predicted")), 90.0),
        (t!("output-logistic-pct-correct"), 140.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 20.0;
    ScrollArea::horizontal()
        .id_salt("logistic_class_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);

                for (idx, row) in res.classification_table.rows.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let vals = [
                        (row.observed_label.clone(), Align2::LEFT_CENTER),
                        (format!("{}", row.pred_zero), Align2::RIGHT_CENTER),
                        (format!("{}", row.pred_one), Align2::RIGHT_CENTER),
                        (format!("{:.1}", row.percent_correct), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }

                // Overall percentage row
                let overall_vals = [
                    (t!("output-logistic-overall-pct"), Align2::LEFT_CENTER),
                    ("".to_string(), Align2::CENTER_CENTER),
                    ("".to_string(), Align2::CENTER_CENTER),
                    (
                        format!("{:.1}", res.classification_table.overall_percent_correct),
                        Align2::RIGHT_CENTER,
                    ),
                ];
                render_data_row(ui, &cols, &overall_vals, row_height, Tokens::bg_row_odd());
            });
        });
}

fn render_variables_in_equation_table(ui: &mut Ui, res: &LogisticRegressionResult) {
    ui.strong(t!("output-logistic-variables-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-variable"), 120.0),
        (t!("output-logistic-col-b"), 80.0),
        (t!("output-logistic-col-se"), 80.0),
        (t!("output-logistic-col-wald"), 85.0),
        (t!("output-logistic-col-df"), 50.0),
        (t!("output-logistic-col-sig"), 75.0),
        (t!("output-logistic-col-exp-b"), 85.0),
        (t!("output-logistic-col-ci-lower"), 80.0),
        (t!("output-logistic-col-ci-upper"), 80.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 20.0;
    ScrollArea::horizontal()
        .id_salt("logistic_vars_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);

                for (idx, coeff) in res.coefficients.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let name = if coeff.param_name == "Constant" {
                        t!("output-logistic-constant")
                    } else {
                        coeff.param_name.clone()
                    };

                    let vals = [
                        (name, Align2::LEFT_CENTER),
                        (format!("{:.3}", coeff.b), Align2::RIGHT_CENTER),
                        (format!("{:.3}", coeff.se), Align2::RIGHT_CENTER),
                        (format!("{:.3}", coeff.wald), Align2::RIGHT_CENTER),
                        (format!("{}", coeff.df), Align2::RIGHT_CENTER),
                        (format_sig(Some(coeff.sig)), Align2::RIGHT_CENTER),
                        (format!("{:.3}", coeff.exp_b), Align2::RIGHT_CENTER),
                        (format!("{:.3}", coeff.lower_ci), Align2::RIGHT_CENTER),
                        (format!("{:.3}", coeff.upper_ci), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
            });
        });
}

pub fn logistic_to_tsv(res: &LogisticRegressionResult) -> String {
    let mut out = String::new();
    out.push_str("Variable\tB\tS.E.\tWald\tdf\tSig.\tExp(B)\t95% C.I. Lower\t95% C.I. Upper\n");

    for coeff in &res.coefficients {
        out.push_str(&format!(
            "{}\t{:.4}\t{:.4}\t{:.4}\t{}\t{}\t{:.4}\t{:.4}\t{:.4}\n",
            coeff.param_name,
            coeff.b,
            coeff.se,
            coeff.wald,
            coeff.df,
            format_sig(Some(coeff.sig)),
            coeff.exp_b,
            coeff.lower_ci,
            coeff.upper_ci,
        ));
    }

    out
}
