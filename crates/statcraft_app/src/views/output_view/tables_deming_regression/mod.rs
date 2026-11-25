mod plots;

use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{
    adjust_widths_with_rows, bootstrap_column_headers, bootstrap_specs_to_tsv,
    bootstrap_table_to_tsv, format_bootstrap_cells, format_fixed_4,
    render_bootstrap_specifications_table, render_bootstrap_table, render_data_row,
    render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use plots::{render_residual_plot, render_scatter_plot};
use statcraft_algo::DemingRegressionResult;
use statcraft_i18n::t;
use statcraft_reporting::styling::TableStyleRule;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DemingRegressionOutputItem {
    pub result: DemingRegressionResult,
    #[serde(default)]
    pub dep_label: Option<String>,
    #[serde(default)]
    pub indep_label: Option<String>,
    pub include_scatter_plot: bool,
    pub include_identity_line: bool,
    pub include_residual_plot: bool,
    #[serde(default)]
    pub table_styles: Vec<TableStyleRule>,
}

impl DemingRegressionOutputItem {
    pub fn dep_display(&self) -> &str {
        self.dep_label
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&self.result.dep_var)
    }

    pub fn indep_display(&self) -> &str {
        self.indep_label
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&self.result.indep_var)
    }
}

impl OutputViewerState {
    #[allow(clippy::too_many_arguments)]
    pub fn add_deming_regression(
        &mut self,
        result: DemingRegressionResult,
        dep_label: Option<String>,
        indep_label: Option<String>,
        include_scatter_plot: bool,
        include_identity_line: bool,
        include_residual_plot: bool,
        table_styles: Vec<TableStyleRule>,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = statcraft_syntax::commands::DemingRegressionCommand::to_sps_with_bootstrap(
            &result.dep_var,
            &result.indep_var,
            result.lambda,
            result.options.ci_level,
            include_scatter_plot,
            include_identity_line,
            include_residual_plot,
            result.saved_predicted_var.is_some(),
            result.saved_residuals_var.is_some(),
            result.bootstrap.as_ref().map(|b| &b.options),
        );

        let notes = OutputNotes {
            dataset_name: self.active_dataset_name.clone(),
            timestamp: "Session".to_string(),
            active_cases: result.n_valid,
            active_vars: 2,
        };

        let item = DemingRegressionOutputItem {
            result,
            dep_label,
            indep_label,
            include_scatter_plot,
            include_identity_line,
            include_residual_plot,
            table_styles,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-deming-title",
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
                    title_key: "output-deming-table-title",
                    custom_title: None,
                    kind: OutputItemKind::DemingRegression(Box::new(item)),
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

pub fn render_spss_deming_regression(ui: &mut Ui, item: &DemingRegressionOutputItem) {
    let res = &item.result;
    let dep_display = item.dep_display();
    let indep_display = item.indep_display();
    ui.vertical(|ui| {
        if let Some(ref boot) = res.bootstrap {
            render_bootstrap_specifications_table(ui, &boot.options);
            ui.add_space(14.0);
        }

        render_model_summary_table(ui, res, dep_display, indep_display);
        ui.add_space(14.0);

        render_parameter_estimates_table(ui, res, indep_display);
        ui.add_space(14.0);

        if let Some((headers, rows)) = build_deming_bootstrap_rows(res, indep_display) {
            let title = format!(
                "{} ({})",
                t!("output-deming-params-title"),
                t!("output-bootstrap-group-header")
            );
            render_bootstrap_table(ui, &title, "deming_bootstrap_scroll", &headers, &rows, 1);
            ui.add_space(14.0);
        }

        if res.saved_predicted_var.is_some() || res.saved_residuals_var.is_some() {
            let mut saved_names = Vec::new();
            if let Some(ref p) = res.saved_predicted_var {
                saved_names.push(p.as_str());
            }
            if let Some(ref r) = res.saved_residuals_var {
                saved_names.push(r.as_str());
            }
            ui.label(format!(
                "{}: {}",
                t!("output-deming-saved-vars"),
                saved_names.join(", ")
            ));
            ui.add_space(10.0);
        }

        if item.include_scatter_plot && !res.points.is_empty() {
            render_scatter_plot(
                ui,
                res,
                dep_display,
                indep_display,
                item.include_identity_line,
            );
            ui.add_space(14.0);
        }

        if item.include_residual_plot && !res.points.is_empty() {
            render_residual_plot(ui, res);
        }
    });
}

fn render_model_summary_table(
    ui: &mut Ui,
    res: &DemingRegressionResult,
    dep_display: &str,
    indep_display: &str,
) {
    ui.strong(t!("output-deming-summary-title"));
    ui.add_space(4.0);

    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-deming-col-dep"), 90.0),
        (t!("output-deming-col-indep"), 90.0),
        (t!("output-deming-col-n-valid"), 65.0),
        (t!("output-deming-col-n-missing"), 70.0),
        (t!("output-deming-col-lambda"), 70.0),
        (t!("output-deming-col-orth-sd"), 105.0),
        (t!("output-deming-col-pearson-r"), 85.0),
    ];

    let r_str = res
        .pearson_r
        .map(format_fixed_4)
        .unwrap_or_else(|| "-".to_string());
    let vals: Vec<(String, Align2)> = vec![
        (dep_display.to_string(), Align2::LEFT_CENTER),
        (indep_display.to_string(), Align2::LEFT_CENTER),
        (res.n_valid.to_string(), Align2::RIGHT_CENTER),
        (res.n_missing.to_string(), Align2::RIGHT_CENTER),
        (format_fixed_4(res.lambda), Align2::RIGHT_CENTER),
        (
            format_fixed_4(res.orthogonal_residual_sd),
            Align2::RIGHT_CENTER,
        ),
        (r_str, Align2::RIGHT_CENTER),
    ];
    let row_strings: Vec<Vec<String>> = vec![vals.iter().map(|(s, _)| s.clone()).collect()];
    adjust_widths_with_rows(ui, &mut cols, &row_strings);

    ScrollArea::horizontal()
        .id_salt("deming_summary_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 22.0);
                render_data_row(ui, &cols, &vals, 20.0, Tokens::bg_row_even());
            });
        });
}

fn render_parameter_estimates_table(
    ui: &mut Ui,
    res: &DemingRegressionResult,
    indep_display: &str,
) {
    ui.strong(t!("output-deming-params-title"));
    ui.add_space(4.0);

    let ci_pct = (res.options.ci_level * 100.0).round() as i32;
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-deming-col-param"), 120.0),
        (t!("output-deming-col-estimate"), 90.0),
        (t!("output-deming-col-se"), 95.0),
        (
            format!("{} {}%", t!("output-deming-col-ci-low"), ci_pct),
            110.0,
        ),
        (
            format!("{} {}%", t!("output-deming-col-ci-high"), ci_pct),
            110.0,
        ),
    ];

    let rows = [
        [
            (t!("output-deming-constant"), Align2::LEFT_CENTER),
            (format_fixed_4(res.intercept), Align2::RIGHT_CENTER),
            (format_fixed_4(res.intercept_se), Align2::RIGHT_CENTER),
            (format_fixed_4(res.intercept_ci_lower), Align2::RIGHT_CENTER),
            (format_fixed_4(res.intercept_ci_upper), Align2::RIGHT_CENTER),
        ],
        [
            (indep_display.to_string(), Align2::LEFT_CENTER),
            (format_fixed_4(res.slope), Align2::RIGHT_CENTER),
            (format_fixed_4(res.slope_se), Align2::RIGHT_CENTER),
            (format_fixed_4(res.slope_ci_lower), Align2::RIGHT_CENTER),
            (format_fixed_4(res.slope_ci_upper), Align2::RIGHT_CENTER),
        ],
    ];
    let row_strings: Vec<Vec<String>> = rows
        .iter()
        .map(|r| r.iter().map(|(s, _)| s.clone()).collect())
        .collect();
    adjust_widths_with_rows(ui, &mut cols, &row_strings);

    ScrollArea::horizontal()
        .id_salt("deming_params_scroll")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, 22.0);
                for (idx, row) in rows.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    render_data_row(ui, &cols, row, 20.0, bg);
                }
            });
        });
}

fn build_deming_bootstrap_rows(
    res: &DemingRegressionResult,
    indep_display: &str,
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let boot = res.bootstrap.as_ref()?;
    let mut headers = vec![t!("output-deming-col-param")];
    headers.extend(bootstrap_column_headers(boot.options.ci_level));
    let mut rows = Vec::new();
    if let Some(ref b_int) = boot.intercept {
        let mut row = vec![t!("output-deming-constant")];
        row.extend(format_bootstrap_cells(b_int, 4));
        rows.push(row);
    }
    if let Some(ref b_slope) = boot.slope {
        let mut row = vec![indep_display.to_string()];
        row.extend(format_bootstrap_cells(b_slope, 4));
        rows.push(row);
    }
    if rows.is_empty() {
        None
    } else {
        Some((headers, rows))
    }
}

pub fn deming_regression_to_tsv(item: &DemingRegressionOutputItem) -> String {
    let res = &item.result;
    let dep_display = item.dep_display();
    let indep_display = item.indep_display();
    let mut out = String::new();
    if let Some(ref boot) = res.bootstrap {
        out.push_str(&bootstrap_specs_to_tsv(&boot.options));
        out.push('\n');
    }

    out.push_str(
        "Dependent\tIndependent\tValid N\tMissing N\tLambda\tOrthogonal Residual SD\tPearson r\n",
    );
    let r_str = res
        .pearson_r
        .map(format_fixed_4)
        .unwrap_or_else(|| "-".to_string());
    out.push_str(&format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\n\n",
        dep_display,
        indep_display,
        res.n_valid,
        res.n_missing,
        format_fixed_4(res.lambda),
        format_fixed_4(res.orthogonal_residual_sd),
        r_str
    ));

    let ci_pct = (res.options.ci_level * 100.0).round() as i32;
    out.push_str(&format!(
        "Parameter\tEstimate\tJackknife SE\tLower {}% CI\tUpper {}% CI\n",
        ci_pct, ci_pct
    ));
    out.push_str(&format!(
        "(Constant)\t{}\t{}\t{}\t{}\n",
        format_fixed_4(res.intercept),
        format_fixed_4(res.intercept_se),
        format_fixed_4(res.intercept_ci_lower),
        format_fixed_4(res.intercept_ci_upper)
    ));
    out.push_str(&format!(
        "{}\t{}\t{}\t{}\t{}\n",
        indep_display,
        format_fixed_4(res.slope),
        format_fixed_4(res.slope_se),
        format_fixed_4(res.slope_ci_lower),
        format_fixed_4(res.slope_ci_upper)
    ));

    if let Some((headers, rows)) = build_deming_bootstrap_rows(res, indep_display) {
        out.push('\n');
        out.push_str(&bootstrap_table_to_tsv("Bootstrap", &headers, &rows));
    }

    out
}
