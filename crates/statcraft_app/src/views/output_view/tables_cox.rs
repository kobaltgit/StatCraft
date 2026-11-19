use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, FontId, ScrollArea, Stroke, Ui};
use statcraft_algo::CoxRegressionResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_cox_regression(&mut self, result: CoxRegressionResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = statcraft_syntax::commands::CoxRegressionCommand::to_sps(
            &result.time_variable,
            &result.status_variable,
            1.0,
            &result.covariate_names,
            result.strata_variable.as_deref(),
            result.tie_method,
        );

        let notes = OutputNotes {
            dataset_name: self.active_dataset_name.clone(),
            timestamp: "Session".to_string(),
            active_cases: result.overall_summary.total_n,
            active_vars: result.covariate_names.len() + 2,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-cox-title",
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
                    title_key: "output-cox-table-title",
                    custom_title: None,
                    kind: OutputItemKind::CoxRegression(Box::new(result)),
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

pub fn render_spss_cox_regression(ui: &mut Ui, res: &CoxRegressionResult) {
    ui.vertical(|ui| {
        // Table 1: Case Processing Summary
        ui.label(
            egui::RichText::new(t!("output-cox-case-summary-title"))
                .font(FontId::proportional(13.0))
                .strong()
                .color(Tokens::text_primary()),
        );
        ui.add_space(4.0);
        render_cox_case_summary(ui, res);
        ui.add_space(12.0);

        // Table 2: Omnibus Tests of Model Coefficients
        ui.label(
            egui::RichText::new(t!("output-cox-omnibus-title"))
                .font(FontId::proportional(13.0))
                .strong()
                .color(Tokens::text_primary()),
        );
        ui.add_space(4.0);
        render_cox_omnibus_tests(ui, res);
        ui.add_space(12.0);

        // Table 3: Variables in the Equation
        ui.label(
            egui::RichText::new(t!("output-cox-variables-title"))
                .font(FontId::proportional(13.0))
                .strong()
                .color(Tokens::text_primary()),
        );
        ui.add_space(4.0);
        render_cox_variables(ui, res);
        ui.add_space(12.0);

        // Table 4: Survival Table at Mean of Covariates (if available)
        if !res.baseline_survival.is_empty() {
            ui.label(
                egui::RichText::new(t!("output-cox-baseline-title"))
                    .font(FontId::proportional(13.0))
                    .strong()
                    .color(Tokens::text_primary()),
            );
            ui.add_space(4.0);
            render_cox_baseline_table(ui, res);
            ui.add_space(12.0);

            // Vector Survival Step Plot
            ui.label(
                egui::RichText::new(t!("output-cox-survival-plot-title"))
                    .font(FontId::proportional(13.0))
                    .strong()
                    .color(Tokens::text_primary()),
            );
            ui.add_space(4.0);
            render_cox_survival_plot(ui, res);
            ui.add_space(8.0);
        }
    });
}

fn render_cox_case_summary(ui: &mut Ui, res: &CoxRegressionResult) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-common-stratum"), 100.0),
        (t!("output-common-total-n"), 80.0),
        (t!("output-km-events-n"), 80.0),
        (t!("output-km-censored-n"), 80.0),
        (t!("output-km-censored-pct"), 80.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("cox_case_summary_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 20.0);
                for (idx, cs) in res.case_summaries.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let st_label = cs
                        .stratum
                        .clone()
                        .unwrap_or_else(|| t!("output-common-overall"));
                    let vals = [
                        (st_label, Align2::LEFT_CENTER),
                        (format!("{}", cs.total_n), Align2::RIGHT_CENTER),
                        (format!("{}", cs.events_n), Align2::RIGHT_CENTER),
                        (format!("{}", cs.censored_n), Align2::RIGHT_CENTER),
                        (format!("{:.1}%", cs.censored_percent), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, 20.0, bg);
                }

                if res.case_summaries.len() > 1 {
                    let bg = Tokens::bg_header();
                    let ov = &res.overall_summary;
                    let vals = [
                        (t!("output-common-overall"), Align2::LEFT_CENTER),
                        (format!("{}", ov.total_n), Align2::RIGHT_CENTER),
                        (format!("{}", ov.events_n), Align2::RIGHT_CENTER),
                        (format!("{}", ov.censored_n), Align2::RIGHT_CENTER),
                        (format!("{:.1}%", ov.censored_percent), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, 20.0, bg);
                }
            });
        });
}

fn render_cox_omnibus_tests(ui: &mut Ui, res: &CoxRegressionResult) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-logistic-col-2ll"), 120.0),
        (t!("output-logistic-col-chi-square"), 90.0),
        (t!("output-logistic-col-df"), 60.0),
        (t!("output-logistic-col-sig"), 70.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("cox_omnibus_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 20.0);
                let om = &res.omnibus_tests;
                let vals = [
                    (format!("{:.3}", om.final_minus_2ll), Align2::RIGHT_CENTER),
                    (format!("{:.3}", om.chi_square), Align2::RIGHT_CENTER),
                    (format!("{}", om.df), Align2::RIGHT_CENTER),
                    (format_sig(Some(om.p_value)), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &vals, 20.0, Tokens::bg_row_even());
            });
        });
}

fn render_cox_variables(ui: &mut Ui, res: &CoxRegressionResult) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-col-variable"), 110.0),
        (t!("output-logistic-col-b"), 75.0),
        (t!("output-logistic-col-se"), 75.0),
        (t!("output-logistic-col-wald"), 75.0),
        (t!("output-logistic-col-df"), 50.0),
        (t!("output-logistic-col-sig"), 75.0),
        (t!("output-logistic-col-exp-b"), 80.0),
        (t!("output-logistic-col-ci-lower"), 80.0),
        (t!("output-logistic-col-ci-upper"), 80.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("cox_vars_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 20.0);
                for (idx, v) in res.variables.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let vals = [
                        (v.name.clone(), Align2::LEFT_CENTER),
                        (format!("{:.4}", v.b), Align2::RIGHT_CENTER),
                        (format!("{:.4}", v.se), Align2::RIGHT_CENTER),
                        (format!("{:.3}", v.wald), Align2::RIGHT_CENTER),
                        (format!("{}", v.df), Align2::RIGHT_CENTER),
                        (format_sig(Some(v.p_value)), Align2::RIGHT_CENTER),
                        (format!("{:.4}", v.exp_b), Align2::RIGHT_CENTER),
                        (format!("{:.4}", v.ci_lower), Align2::RIGHT_CENTER),
                        (format!("{:.4}", v.ci_upper), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, 20.0, bg);
                }
            });
        });
}

fn render_cox_baseline_table(ui: &mut Ui, res: &CoxRegressionResult) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-common-time"), 80.0),
        (t!("output-cox-baseline-cum-hazard"), 140.0),
        (t!("output-cox-baseline-survival"), 120.0),
        (t!("output-cox-survival-at-mean"), 130.0),
    ];
    adjust_widths(ui, &mut cols);

    ScrollArea::horizontal()
        .id_salt("cox_baseline_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 20.0);
                for (idx, pt) in res.baseline_survival.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let vals = [
                        (format!("{:.2}", pt.time), Align2::RIGHT_CENTER),
                        (
                            format!("{:.4}", pt.baseline_cum_hazard),
                            Align2::RIGHT_CENTER,
                        ),
                        (format!("{:.4}", pt.baseline_survival), Align2::RIGHT_CENTER),
                        (
                            format!("{:.4}", pt.mean_covariates_survival),
                            Align2::RIGHT_CENTER,
                        ),
                    ];
                    render_data_row(ui, &cols, &vals, 20.0, bg);
                }
            });
        });
}

fn render_cox_survival_plot(ui: &mut Ui, res: &CoxRegressionResult) {
    if res.baseline_survival.is_empty() {
        return;
    }

    let (rect, _) = ui.allocate_exact_size(vec2(460.0, 220.0), egui::Sense::hover());
    let painter = ui.painter();

    // Plot background & border
    painter.rect_filled(rect, 3.0, Tokens::bg_panel());
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, Tokens::border()));

    let margin = 35.0_f32;
    let plot_x = rect.min.x + margin;
    let plot_y = rect.min.y + 15.0;
    let plot_w = rect.width() - margin - 20.0;
    let plot_h = rect.height() - margin - 25.0;

    // Axis lines
    let origin = pos2(plot_x, plot_y + plot_h);
    let x_end = pos2(plot_x + plot_w, plot_y + plot_h);
    let y_end = pos2(plot_x, plot_y);

    let axis_stroke = Stroke::new(1.0_f32, Tokens::text_muted());
    painter.line_segment([origin, x_end], axis_stroke);
    painter.line_segment([origin, y_end], axis_stroke);

    // Y ticks (0.0 to 1.0)
    for i in 0..=4 {
        let frac = (i as f32) / 4.0;
        let y_val = plot_y + plot_h * (1.0 - frac);
        let tick_start = pos2(plot_x - 4.0, y_val);
        let tick_end = pos2(plot_x, y_val);
        painter.line_segment([tick_start, tick_end], axis_stroke);
        painter.text(
            pos2(plot_x - 8.0, y_val),
            Align2::RIGHT_CENTER,
            format!("{:.1}", frac),
            FontId::proportional(10.0),
            Tokens::text_muted(),
        );
    }

    let max_time = res
        .baseline_survival
        .last()
        .map(|p| p.time)
        .unwrap_or(1.0)
        .max(1.0) as f32;

    // Step function curve
    let curve_stroke = Stroke::new(2.0_f32, Tokens::accent());
    let mut current_pt = origin;
    let mut prev_time = 0.0_f32;
    let mut prev_surv = 1.0_f32;

    for pt in &res.baseline_survival {
        let t_val = pt.time as f32;
        let s_val = pt.mean_covariates_survival.clamp(0.0, 1.0) as f32;

        let x1 = plot_x + (prev_time / max_time) * plot_w;
        let x2 = plot_x + (t_val / max_time) * plot_w;
        let y_level = plot_y + plot_h * (1.0 - prev_surv);
        let y_next = plot_y + plot_h * (1.0 - s_val);

        // Horizontal line
        painter.line_segment([pos2(x1, y_level), pos2(x2, y_level)], curve_stroke);
        // Vertical step
        painter.line_segment([pos2(x2, y_level), pos2(x2, y_next)], curve_stroke);

        prev_time = t_val;
        prev_surv = s_val;
        current_pt = pos2(x2, y_next);
    }

    // Connect to max_time horizontally
    let final_x = plot_x + plot_w;
    painter.line_segment([current_pt, pos2(final_x, current_pt.y)], curve_stroke);
}

pub fn cox_to_tsv(res: &CoxRegressionResult) -> String {
    let mut tsv = String::new();
    tsv.push_str("=== Case Processing Summary ===\n");
    tsv.push_str("Stratum\tTotal N\tEvents N\tCensored N\tCensored Percent\n");
    for cs in &res.case_summaries {
        let st = cs.stratum.as_deref().unwrap_or("Overall");
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{:.1}%\n",
            st, cs.total_n, cs.events_n, cs.censored_n, cs.censored_percent
        ));
    }

    tsv.push_str("\n=== Omnibus Tests of Model Coefficients ===\n");
    tsv.push_str("-2 Log Likelihood\tChi-square\tdf\tSig.\n");
    tsv.push_str(&format!(
        "{:.3}\t{:.3}\t{}\t{:.4}\n",
        res.omnibus_tests.final_minus_2ll,
        res.omnibus_tests.chi_square,
        res.omnibus_tests.df,
        res.omnibus_tests.p_value
    ));

    tsv.push_str("\n=== Variables in the Equation ===\n");
    tsv.push_str("Variable\tB\tS.E.\tWald\tdf\tSig.\tExp(B)\t95% CI Lower\t95% CI Upper\n");
    for v in &res.variables {
        tsv.push_str(&format!(
            "{}\t{:.4}\t{:.4}\t{:.3}\t{}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\n",
            v.name, v.b, v.se, v.wald, v.df, v.p_value, v.exp_b, v.ci_lower, v.ci_upper
        ));
    }

    tsv
}
